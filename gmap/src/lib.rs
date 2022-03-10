pub mod grids;

use std::collections::{HashMap, HashSet};
use std::ops::Index;

#[derive(Debug)]
pub enum GMapError {
  InvalidAlpha(String),
  CannotDecreaseDimension,
  Unsewable,
  NotFree,
  AlreadyFree,
  DimensionTooLarge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Dart(pub usize);

/// Bitfield where bit i is 1 if alpha_i should be included as a generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Alphas(pub u32);

impl Alphas {
  pub const VERTEX: Self = Self(!1);
  pub const EDGE: Self = Self(!2);
  pub const HALF_EDGE: Self = Self(!3);
  pub const FACE: Self = Self(!4);
  pub const ANGLE: Self = Self(!5);

  #[inline(always)]
  pub fn cell(i: usize) -> Self {
    Self(!(1 << i))
  }

  #[inline(always)]
  pub fn has(self, i: usize) -> bool {
    (self.0 >> i) & 1 == 1
  }
}

/// Maximum dimension allowed.  The memory requirement goes up exponentially with dimension, so 31 should be more than enough.
pub const MAX_DIMENSION: usize = 31;

pub struct GMap {
  // This is a usize because we need to index by dimensions so often it's unwieldy to store it as something smaller.
  dimension: usize,
  alpha: Vec<Vec<Dart>>,
}

impl GMap {
  pub fn empty(dimension: usize) -> Result<Self, GMapError> {
    Self::from_alpha(dimension, vec![])
  }

  pub fn from_alpha(dimension: usize, alpha: Vec<Vec<Dart>>) -> Result<Self, GMapError> {
    if dimension > MAX_DIMENSION {
      return Err(GMapError::DimensionTooLarge);
    }
    let g = GMap { dimension, alpha };
    g.check_valid()?;
    Ok(g)
  }

  fn check_valid(&self) -> Result<(), GMapError> {
    for (d, al) in self.alpha.iter().enumerate() {
      if al.len() - 1 != self.dimension {
        return Err(GMapError::InvalidAlpha(format!(
          "dart {} has dimension {}, expected {}",
          d,
          al.len() - 1,
          self.dimension
        )));
      }
      for (i, x) in al.iter().cloned().enumerate() {
        if x.0 >= self.alpha.len() {
          return Err(GMapError::InvalidAlpha(format!(
            "dart {} index {} out of range",
            d, i
          )));
        }
      }
    }

    for i in 0..=self.dimension {
      for (d, al) in self.alpha.iter().cloned().enumerate() {
        if self.alpha[al[i].0][i] != Dart(d) {
          return Err(GMapError::InvalidAlpha(format!(
            "alpha_{} is not an involution",
            i
          )));
        }
      }
    }

    for i in 0..(self.dimension - 1) {
      for j in (i + 2)..=self.dimension {
        for al in self.alpha.iter() {
          if self.alpha[al[i].0][j] != self.alpha[al[j].0][i] {
            return Err(GMapError::InvalidAlpha(format!(
              "alpha_{} alpha_{} is not an involution",
              i, j
            )));
          }
        }
      }
    }

    Ok(())
  }

  pub fn dimension(&self) -> usize {
    self.dimension
  }

  pub fn alpha(&self) -> &[Vec<Dart>] {
    &self.alpha
  }

  pub fn al(&self, d: Dart, indices: impl IntoIterator<Item = usize>) -> Dart {
    indices.into_iter().fold(d, |d, a| self.alpha[d.0][a])
  }

  pub fn increase_dimension(&mut self, dim: usize) -> Result<(), GMapError> {
    if dim < self.dimension {
      return Err(GMapError::CannotDecreaseDimension);
    }
    if dim > MAX_DIMENSION {
      return Err(GMapError::DimensionTooLarge);
    }
    self.dimension = dim;
    for (d, al) in self.alpha.iter_mut().enumerate() {
      al.resize(dim, Dart(d));
    }
    Ok(())
  }

  pub fn add_dart(&mut self) -> Dart {
    let d = Dart(self.alpha.len());
    self.alpha.push(vec![d; self.dimension + 1]);
    d
  }

  fn link(&mut self, i: usize, d0: Dart, d1: Dart) -> Result<(), GMapError> {
    if self.alpha[d0.0][i] != d0 {
      return Err(GMapError::NotFree);
    }
    self.alpha[d0.0][i] = d1;
    self.alpha[d1.0][i] = d0;
    Ok(())
  }

  fn unlink(&mut self, i: usize, d0: Dart) -> Result<Dart, GMapError> {
    let d1 = self.alpha[d0.0][i];
    if d0 == d1 {
      return Err(GMapError::AlreadyFree);
    }
    self.alpha[d0.0][i] = d0;
    self.alpha[d1.0][i] = d1;
    Ok(d1)
  }

  pub fn add_edge(&mut self) -> Dart {
    let d0 = self.add_dart();
    let d1 = self.add_dart();
    self.link(0, d0, d1).unwrap();
    d0
  }

  pub fn add_polygon(&mut self, n: usize) -> Dart {
    let start = self.add_edge();
    let mut prev = self.alpha[start.0][0];
    for _ in 0..(n - 1) {
      let c = self.add_edge();
      self.link(1, prev, c).unwrap();
      prev = self.alpha[c.0][0];
    }
    self.link(1, start, prev).unwrap();
    start
  }

  /// Enumerate the a-orbit of d.
  /// Returns a list of darts d1 together with their paths,
  /// which is the sequence of indices (taken from a) from d to d1.
  pub fn orbit_paths(&self, d: Dart, a: Alphas) -> Vec<(Vec<usize>, Dart)> {
    let mut seen = HashSet::new();
    let mut frontier: Vec<(Vec<usize>, Dart)> = vec![(vec![], d)];
    let mut orbit = Vec::new();
    while !frontier.is_empty() {
      let (path, dart) = frontier.remove(0);
      if seen.contains(&dart) {
        continue;
      }
      seen.insert(dart);
      orbit.push((path.clone(), dart));
      for i in 0..=self.dimension {
        if !a.has(i) {
          continue;
        }
        let neighbor = self.alpha[dart.0][i];
        let mut new_path = path.clone();
        new_path.push(i);
        frontier.push((new_path, neighbor));
      }
    }
    orbit
  }

  pub fn orbit(&self, d: Dart, a: Alphas) -> impl Iterator<Item = Dart> {
    self.orbit_paths(d, a).into_iter().map(|(_, d)| d)
  }

  pub fn cell(&self, d: Dart, i: usize) -> impl Iterator<Item = Dart> {
    self.orbit(d, Alphas::cell(i))
  }

  /// Sew the i-cell at d0 to the i-cell at d1.
  /// Returns the list of pairs of darts which were sewn.
  pub fn sew(&mut self, i: usize, d0: Dart, d1: Dart) -> Result<Vec<(Dart, Dart)>, GMapError> {
    // Only include indices with distance >1 from i.
    let indices = Alphas(!(1 << i) & !((1 << i) >> 1) & !((1 << i) << 1));
    let m0: HashMap<_, _> = self.orbit_paths(d0, indices).into_iter().collect();
    let m1: HashMap<_, _> = self.orbit_paths(d1, indices).into_iter().collect();
    if m0.len() != m1.len() || m0.iter().any(|(x, _)| !m1.contains_key(x)) {
      return Err(GMapError::Unsewable);
    }
    let mut output = Vec::new();
    for (k, d0) in m0.into_iter() {
      let d1 = *m1.get(&k).ok_or(GMapError::Unsewable)?;
      self.link(i, d0, d1)?;
      output.push((d0, d1));
    }
    Ok(output)
  }

  /// Unsew the pair of i-cells at d.
  /// Returns the list of pairs of darts which were unsewn.
  pub fn unsew(&mut self, d: Dart, i: usize) -> Result<Vec<(Dart, Dart)>, GMapError> {
    let indices = Alphas(!(1 << i) & !((1 << i) >> 1) & !((1 << i) << 1));
    let mut output = Vec::new();
    for d0 in self.orbit(d, indices) {
      let d1 = self.unlink(i, d0)?;
      output.push((d0, d1));
    }
    Ok(output)
  }

  /// filter out darts which are in the same a-orbit as a previous dart
  pub fn unique_by_orbit<'a>(
    &'a self,
    l: impl IntoIterator<Item = Dart> + 'a,
    a: Alphas,
  ) -> impl Iterator<Item = Dart> + 'a {
    let mut seen = HashSet::new();
    l.into_iter().filter_map(move |dart| {
      if seen.contains(&dart) {
        return None;
      }
      for n in self.orbit(dart, a) {
        seen.insert(n);
      }
      Some(dart)
    })
  }

  /// one dart per a-orbit
  pub fn one_dart_per_orbit<'a>(&'a self, a: Alphas) -> impl Iterator<Item = Dart> + 'a {
    self.unique_by_orbit((0..self.alpha.len()).map(Dart), a)
  }

  /// one dart per i-cell
  pub fn one_dart_per_cell<'a>(&'a self, i: usize) -> impl Iterator<Item = Dart> + 'a {
    self.one_dart_per_orbit(Alphas::cell(i))
  }

  /// one dart per a-orbit incident to d's b-orbit.
  /// darts are guaranteed to be in both orbits.
  pub fn one_dart_per_incident_orbit<'a>(
    &'a self,
    d: Dart,
    a: Alphas,
    b: Alphas,
  ) -> impl Iterator<Item = Dart> + 'a {
    self.unique_by_orbit(self.orbit(d, b), a)
  }

  /// one dart per i-cell (in dim) incident to d's j-cell (in dim).
  /// darts are guaranteed to be in both cells.
  pub fn one_dart_per_incident_cell<'a>(
    &'a self,
    d: Dart,
    i: usize,
    j: usize,
  ) -> impl Iterator<Item = Dart> + 'a {
    self.one_dart_per_incident_orbit(d, Alphas::cell(i), Alphas::cell(j))
  }
}

/// Map from orbits to A.  Duplicates its values once for each dart in the orbit.
pub struct OrbitMap<A> {
  map: HashMap<Dart, A>,
  indices: Alphas,
}

impl<A> OrbitMap<A>
where
  A: Clone,
{
  pub fn new(indices: Alphas) -> Self {
    Self {
      map: HashMap::new(),
      indices,
    }
  }

  pub fn over_cells(i: usize) -> Self {
    Self::new(Alphas::cell(i))
  }

  pub fn map(&self) -> &HashMap<Dart, A> {
    &self.map
  }

  pub fn into_map(self) -> HashMap<Dart, A> {
    self.map
  }

  pub fn indices(&self) -> Alphas {
    self.indices
  }

  pub fn insert(&mut self, g: &GMap, k: Dart, v: A) {
    for n in g.orbit(k, self.indices) {
      self.map.insert(n, v.clone());
    }
  }

  pub fn remove(&mut self, g: &GMap, k: Dart) -> Option<A> {
    g.orbit(k, self.indices)
      .fold(None, |v, n| v.or(self.map.remove(&n)))
  }
}

/// Maintains a single representative for each orbit.
/// Can be potentially more efficient (but less convenient) than OrbitMap,
/// especially when many maps over the same orbits are used.
pub struct OrbitReprs(HashMap<Alphas, Vec<Dart>>);

impl OrbitReprs {
  pub fn new() -> Self {
    Self(HashMap::new())
  }

  pub fn orbit_repr(&self, a: Alphas) -> Option<&[Dart]> {
    self.0.get(&a).map(Vec::as_slice)
  }

  pub fn ensure_orbit_repr(&mut self, g: &GMap, a: Alphas) -> &[Dart] {
    if !self.0.contains_key(&a) {
      self.build_orbit_repr(g, a)
    }
    self.0.get(&a).unwrap()
  }

  /// (Re)build the orbit representative list for a-orbits.
  fn build_orbit_repr(&mut self, g: &GMap, a: Alphas) {
    let mut v = Vec::new();
    let mut seen = HashSet::new();
    for d in (0..g.alpha.len()).map(Dart) {
      if seen.contains(&d) {
        continue;
      }
      for n in g.orbit(d, a) {
        seen.insert(n);
        v[n.0] = d;
      }
    }
    self.0.insert(a, v);
  }

  /// Set the orbit representative list for the a-cell at d to be d.
  pub fn set_orbit_repr(&mut self, g: &GMap, a: Alphas, d: Dart) {
    if !self.0.contains_key(&a) {
      self.build_orbit_repr(g, a)
    }
    let v = self.0.get_mut(&a).unwrap();
    for d1 in g.orbit(d, a) {
      v[d1.0] = d;
    }
  }
}

impl Index<(Alphas, Dart)> for OrbitReprs {
  type Output = Dart;

  fn index(&self, (a, d): (Alphas, Dart)) -> &Self::Output {
    &self.orbit_repr(a).unwrap()[d.0]
  }
}
