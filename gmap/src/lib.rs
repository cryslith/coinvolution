pub mod grids;

use std::collections::{HashMap, HashSet};
use std::fmt;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Dart(pub usize);

impl fmt::Display for Dart {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

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
  /// 2-dimensional vector indexed as dart * (dimension + 1) + alpha_index
  alpha: Vec<Dart>,
}

impl Index<(Dart, usize)> for GMap {
  type Output = Dart;

  #[inline(always)]
  fn index(&self, (d, i): (Dart, usize)) -> &Self::Output {
    &self.alpha[d.0 * (self.dimension + 1) + i]
  }
}

impl GMap {
  pub fn empty(dimension: usize) -> Result<Self, GMapError> {
    Self::from_alpha(dimension, vec![])
  }

  pub fn from_alpha(dimension: usize, alpha: Vec<Dart>) -> Result<Self, GMapError> {
    if dimension > MAX_DIMENSION {
      return Err(GMapError::DimensionTooLarge);
    }
    let g = GMap { dimension, alpha };
    g.check_valid()?;
    Ok(g)
  }

  fn check_valid(&self) -> Result<(), GMapError> {
    if self.alpha.len() % (self.dimension + 1) != 0 {
      return Err(GMapError::InvalidAlpha(format!(
        "Wrong number of elements {} in alpha",
        self.alpha.len()
      )));
    }
    let n = self.ndarts();
    for d in 0..n {
      for i in 0..=self.dimension {
        if self[(Dart(d), i)].0 >= n {
          return Err(GMapError::InvalidAlpha(format!(
            "dart {} index {} out of range",
            d, i
          )));
        }
      }
    }

    for i in 0..=self.dimension {
      for d in 0..n {
        if self[(self[(Dart(d), i)], i)] != Dart(d) {
          return Err(GMapError::InvalidAlpha(format!(
            "alpha_{} is not an involution",
            i
          )));
        }
      }
    }

    for i in 0..(self.dimension - 1) {
      for j in (i + 2)..=self.dimension {
        for d in 0..n {
          if self.al(Dart(d), [i, j]) != self.al(Dart(d), [j, i]) {
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

  #[inline(always)]
  pub fn dimension(&self) -> usize {
    self.dimension
  }

  /// Number of darts
  #[inline(always)]
  pub fn ndarts(&self) -> usize {
    self.alpha.len() / (self.dimension + 1)
  }

  pub fn alpha(&self) -> &[Dart] {
    &self.alpha
  }

  #[inline(always)]
  fn al1(&mut self, d: Dart, i: usize) -> &mut Dart {
    &mut self.alpha[d.0 * (self.dimension + 1) + i]
  }

  pub fn al(&self, d: Dart, indices: impl IntoIterator<Item = usize>) -> Dart {
    indices
      .into_iter()
      .fold(d, |d, a| self.alpha[d.0 * (self.dimension + 1) + a])
  }

  pub fn increase_dimension(&mut self, dim: usize) -> Result<(), GMapError> {
    if dim < self.dimension {
      return Err(GMapError::CannotDecreaseDimension);
    }
    if dim > MAX_DIMENSION {
      return Err(GMapError::DimensionTooLarge);
    }
    let n = self.ndarts();
    let mut new_al = vec![Dart(!0); n * (dim + 1)];
    for d in 0..n {
      for i in 0..(self.dimension + 1) {
        new_al[d * (dim + 1) + i] = *self.al1(Dart(d), i);
      }
      for i in (self.dimension + 1)..(dim + 1) {
        new_al[d * (dim + 1) + i] = Dart(d);
      }
    }
    self.alpha = new_al;
    self.dimension = dim;
    Ok(())
  }

  pub fn add_dart(&mut self) -> Dart {
    let d = Dart(self.ndarts());
    self.alpha.resize(self.alpha.len() + self.dimension + 1, d);
    d
  }

  pub fn is_free(&self, d: Dart, i: usize) -> bool {
    self[(d, i)] == d
  }

  fn link(&mut self, i: usize, d0: Dart, d1: Dart) -> Result<(), GMapError> {
    if !self.is_free(d0, i) || !self.is_free(d1, i) {
      return Err(GMapError::NotFree);
    }
    *self.al1(d0, i) = d1;
    *self.al1(d1, i) = d0;
    Ok(())
  }

  fn unlink(&mut self, i: usize, d0: Dart) -> Result<Dart, GMapError> {
    let d1 = self[(d0, i)];
    if d0 == d1 {
      return Err(GMapError::AlreadyFree);
    }
    *self.al1(d0, i) = d0;
    *self.al1(d1, i) = d1;
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
    let mut prev = self[(start, 0)];
    for _ in 0..(n - 1) {
      let c = self.add_edge();
      self.link(1, prev, c).unwrap();
      prev = self[(c, 0)];
    }
    self.link(1, start, prev).unwrap();
    start
  }

  /// Enumerate the a-orbit of d.
  /// Returns a list of darts d1 together with their paths,
  /// which is the sequence of indices (taken from a) from d to d1.
  pub fn orbit_paths(&self, d: Dart, a: Alphas) -> Vec<(Vec<usize>, Dart)> {
    let mut seen = HashSet::new();
    // XXX should be vecdeque?1
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
        let neighbor = self.al(dart, [i]);
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

/// Maintains a single representative for each orbit, defined as the lowest-numbered dart for that orbit.
/// Can be potentially more efficient (but less convenient) than OrbitMap,
/// especially when many maps over the same orbits are used.
pub struct OrbitReprs(HashMap<Alphas, Vec<Dart>>);

impl OrbitReprs {
  pub fn new() -> Self {
    Self(HashMap::new())
  }

  pub fn get(&self, a: Alphas, d: Dart) -> Option<Dart> {
    self.0.get(&a).map(|v| v[d.0])
  }

  pub fn get_or_search(&self, g: &GMap, a: Alphas, d: Dart) -> Dart {
    if let Some(r) = self.get(a, d) {
      return r;
    }
    g.orbit(d, a).min().unwrap()
  }

  pub fn get_all(&self, a: Alphas) -> Option<&[Dart]> {
    self.0.get(&a).map(Vec::as_slice)
  }

  pub fn ensure_all(&mut self, g: &GMap, a: Alphas) -> &[Dart] {
    if !self.0.contains_key(&a) {
      self.build(g, a)
    }
    self.0.get(&a).unwrap()
  }

  /// (Re)build the orbit representative list for a-orbits.
  pub fn build(&mut self, g: &GMap, a: Alphas) {
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
}

impl Index<(Alphas, Dart)> for OrbitReprs {
  type Output = Dart;

  fn index(&self, (a, d): (Alphas, Dart)) -> &Self::Output {
    &self.get_all(a).unwrap()[d.0]
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use itertools::Itertools;

  fn darts(x: impl IntoIterator<Item = usize>) -> Vec<Dart> {
    x.into_iter().map(Dart).collect()
  }

  fn diagonal_cp_example() -> GMap {
    GMap::from_alpha(
      2,
      darts([
        1, 5, 7, 0, 2, 6, 3, 1, 2, 2, 4, 3, 5, 3, 4, 4, 0, 5, 7, 11, 1, 6, 8, 0, 9, 7, 8, 8, 10, 9,
        11, 9, 10, 10, 6, 11,
      ]),
    )
    .unwrap()
  }

  #[test]
  fn test_load_diagonal_cp() {
    let g = diagonal_cp_example();
    assert_eq!(g.ndarts(), 12);
    assert_eq!(g[(Dart(0), 2)], Dart(7));
    assert_eq!(g.al(Dart(0), [0, 2, 1, 0]), Dart(10));
    assert!(g.is_free(Dart(4), 2));
    assert!(!g.is_free(Dart(4), 0));
  }

  #[test]
  fn test_orbits() {
    fn f(g: &GMap) {
      let vertex: Vec<Dart> = g.cell(Dart(0), 0).sorted().collect();
      let edge: Vec<Dart> = g.cell(Dart(0), 1).sorted().collect();
      let face: Vec<Dart> = g.cell(Dart(0), 2).sorted().collect();
      let halfedge: Vec<Dart> = g.orbit(Dart(0), Alphas::HALF_EDGE).sorted().collect();
      let angle: Vec<Dart> = g.orbit(Dart(0), Alphas::ANGLE).sorted().collect();
      let dart: Vec<Dart> = g.orbit(Dart(0), Alphas(0)).sorted().collect();
      let all: Vec<Dart> = g.orbit(Dart(0), Alphas(!0)).sorted().collect();

      assert_eq!(vertex, darts([0, 5, 7, 8]));
      assert_eq!(edge, darts([0, 1, 6, 7]));
      assert_eq!(face, darts(0..6));
      assert_eq!(halfedge, darts([0, 7]));
      assert_eq!(angle, darts([0, 5]));
      assert_eq!(dart, darts([0]));
      assert_eq!(all, darts(0..12));
    }

    let mut g = diagonal_cp_example();
    f(&g);
    g.increase_dimension(4).unwrap();
    f(&g);
  }
}
