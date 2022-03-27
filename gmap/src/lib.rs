#[cfg(feature = "serde")]
pub mod format;
pub mod grids;

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::ops::Index;

use itertools::{EitherOrBoth, Itertools};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GMapError {
  #[error("Invalid alpha maps given: {0}")]
  InvalidAlpha(String),
  #[error("Cannot decrease dimension")]
  CannotDecreaseDimension,
  #[error("Darts are not sewable")]
  Unsewable,
  #[error("Darts are not unsewable")]
  Ununsewable,
  #[error("Dart to be linked is not free")]
  NotFree,
  #[error("Dart is already free")]
  AlreadyFree,
  #[error("Dimensions larger than {} are not supported", MAX_DIMENSION)]
  DimensionTooLarge,
  #[error("Dart is deleted")]
  Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Dart(pub usize);

impl fmt::Display for Dart {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// Bitfield where bit i is 1 if alpha_i should be included as a generator.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Alphas(pub u32);

impl Alphas {
  pub const VERTEX: Self = Self(!1);
  pub const EDGE: Self = Self(!2);
  pub const HALF_EDGE: Self = Self(!3);
  pub const FACE: Self = Self(!4);
  pub const ANGLE: Self = Self(!5);
  pub const SIDE: Self = Self(!6);

  #[inline(always)]
  pub fn cell(i: usize) -> Self {
    Self(!(1 << i))
  }

  #[inline(always)]
  pub fn has(self, i: usize) -> bool {
    (self.0 >> i) & 1 == 1
  }

  pub fn from_indices(i: impl IntoIterator<Item = usize>) -> Self {
    Self(i.into_iter().fold(0, |x, y| x | (1 << y)))
  }

  pub fn to_indices(&self) -> impl Iterator<Item = usize> + '_ {
    let mut i = 0;
    let mut b = self.0;
    std::iter::from_fn(move || {
      if b == 0 {
        return None;
      }
      while b & 1 == 0 {
        b = b >> 1;
        i += 1;
      }
      i += 1;
      b = b >> 1;
      return Some(i - 1);
    })
  }
}

/// Maximum dimension allowed.  The memory requirement goes up exponentially with dimension, so 31 should be more than enough.
pub const MAX_DIMENSION: usize = 31;

pub struct GMap {
  // This is a usize because we need to index by dimensions so often it's unwieldy to store it as something smaller.
  dimension: usize,
  /// 2-dimensional vector indexed as dart * (dimension + 1) + alpha_index
  alpha: Vec<Dart>,
  deleted: Vec<bool>,
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
    let ndarts = alpha.len() / (dimension + 1);
    let deleted = vec![false; ndarts];
    let g = GMap {
      dimension,
      alpha,
      deleted,
    };
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
    if self.deleted.len() != n {
      return Err(GMapError::InvalidAlpha(format!(
        "Wrong size of deleted {}",
        self.deleted.len()
      )));
    }
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
        if !self.deleted[d] && self.deleted[self[(Dart(d), i)].0] {
          return Err(GMapError::InvalidAlpha(format!(
            "pointer from undeleted dart {} to deleted dart",
            d
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

  /// Number of darts (including deleted)
  #[inline(always)]
  pub fn ndarts(&self) -> usize {
    self.alpha.len() / (self.dimension + 1)
  }

  pub fn darts(&self) -> impl Iterator<Item = Dart> + '_ {
    (0..self.ndarts())
      .map(Dart)
      .filter(|&d| !self.is_deleted(d))
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
    self.deleted.push(false);
    d
  }

  pub fn delete(&mut self, d: Dart) {
    // all reachable darts
    let to_delete: Vec<Dart> = self.orbit(d, Alphas(!0)).collect();
    for d1 in to_delete {
      self.deleted[d1.0] = true;
    }
  }

  pub fn is_deleted(&self, d: Dart) -> bool {
    self.deleted[d.0]
  }

  pub fn is_free(&self, d: Dart, i: usize) -> bool {
    self[(d, i)] == d
  }

  fn link(&mut self, i: usize, d0: Dart, d1: Dart) -> Result<(), GMapError> {
    if !self.is_free(d0, i) || !self.is_free(d1, i) {
      return Err(GMapError::NotFree);
    }
    if self.is_deleted(d0) || self.is_deleted(d1) {
      return Err(GMapError::Deleted);
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

  /// Add a cycle of 2n darts related alternately by i and j links.
  /// Returns the lowest-numbered dart.
  /// The lower-numbered dart in each i-link has the same parity
  /// in the cycle as the lowest.
  pub fn add_cycle(&mut self, i: usize, j: usize, n: usize) -> Dart {
    if n < 1 {
      panic!("n must be positive");
    }
    let start = self.add_dart();
    let start1 = self.add_dart();
    self.link(i, start, start1).unwrap();
    let mut prev = start1;
    for _ in 0..(n - 1) {
      let d0 = self.add_dart();
      self.link(j, prev, d0).unwrap();
      let d1 = self.add_dart();
      self.link(i, d0, d1).unwrap();
      prev = d1;
    }
    self.link(j, prev, start).unwrap();
    start
  }

  pub fn add_polygon(&mut self, n: usize) -> Dart {
    self.add_cycle(1, 0, n)
  }

  fn plane_orbit_indices(&self, d: Dart, a: Alphas) -> Option<OrbitImpl<'_>> {
    use OrbitImpl::*;
    fn plus_one(g: &GMap, d: Dart, i: usize) -> OrbitImpl<'static> {
      let d1 = g[(d, i)];
      if d == d1 {
        Array1([(None, d)].into_iter())
      } else {
        Array2([(None, d), (Some(i), d1)].into_iter())
      }
    }

    match (!a.0) & 7 {
      0 => None,
      // vertex
      1 => Some(Path(PathOrbit {
        g: self,
        i: 1,
        j: 2,
        start: d,
        current: d,
        state: PathOrbitState::Initial,
      })),
      // edge
      2 => {
        let d0 = self[(d, 0)];
        let d2 = self[(d, 2)];
        if d == d0 {
          Some(plus_one(self, d, 2))
        } else {
          if d == d2 {
            Some(Array2([(None, d), (Some(0), d0)].into_iter()))
          } else {
            Some(Array4(
              [
                (None, d),
                (Some(0), d0),
                (Some(2), d2),
                (Some(2), self[(d0, 2)]),
              ]
              .into_iter(),
            ))
          }
        }
      }
      // half-edge
      3 => Some(plus_one(self, d, 2)),
      // face
      4 => Some(Path(PathOrbit {
        g: self,
        i: 0,
        j: 1,
        start: d,
        current: d,
        state: PathOrbitState::Initial,
      })),
      // angle
      5 => Some(plus_one(self, d, 1)),
      // side
      6 => Some(plus_one(self, d, 0)),
      // dart
      7 => Some(Array1([(None, d)].into_iter())),
      _ => unreachable!(),
    }
  }

  /// Enumerate the a-orbit of d.
  /// Returns an iterator returning darts together with the
  /// index via which each dart was first reached.
  /// The order of darts returned is deterministic based on the local topology.
  pub fn orbit_indices(
    &self,
    d: Dart,
    a: Alphas,
  ) -> impl Iterator<Item = (Option<usize>, Dart)> + '_ {
    if self.dimension == 2 {
      if let Some(x) = self.plane_orbit_indices(d, a) {
        return x;
      }
    }

    let mut frontier = VecDeque::with_capacity(1);
    frontier.push_back((None, d));
    OrbitImpl::BFS(Orbit {
      g: self,
      a,
      seen: HashSet::new(),
      frontier,
    })
  }

  /// Iterate over the cycle obtained by repeatedly applying the alpha indices to d until it reaches d again.
  pub fn cycle<'a>(&'a self, d: Dart, indices: &'a [usize]) -> impl Iterator<Item = Dart> + 'a {
    std::iter::successors(Some(d), move |&x| {
      let x = self.al(x, indices.iter().cloned());
      if x == d {
        None
      } else {
        Some(x)
      }
    })
  }

  pub fn orbit(&self, d: Dart, a: Alphas) -> impl Iterator<Item = Dart> + '_ {
    self.orbit_indices(d, a).into_iter().map(|(_, d)| d)
  }

  pub fn cell(&self, d: Dart, i: usize) -> impl Iterator<Item = Dart> + '_ {
    self.orbit(d, Alphas::cell(i))
  }

  /// Sew the i-cell at d0 to the i-cell at d1,
  /// so that they share an incident (i - 1)-cell.
  /// Returns a mapping of pairs of darts which were sewn.
  pub fn sew(&mut self, i: usize, d0: Dart, d1: Dart) -> Result<HashMap<Dart, Dart>, GMapError> {
    // Only include indices with distance >1 from i.
    let a = Alphas(!(1 << i) & !((1 << i) >> 1) & !((1 << i) << 1));
    let mut m01: HashMap<Dart, Dart> = HashMap::new();
    let mut s1: HashSet<Dart> = HashSet::new();

    for z in self
      .orbit_indices(d0, a)
      .zip_longest(self.orbit_indices(d1, a))
    {
      let (j, d0, d1) = match z {
        EitherOrBoth::Both((j0, d0), (j1, d1)) if j0 == j1 => (j0, d0, d1),
        _ => {
          return Err(GMapError::Unsewable);
        }
      };
      if m01.contains_key(&d0) || m01.contains_key(&d1) || s1.contains(&d0) || s1.contains(&d1) {
        return Err(GMapError::Unsewable);
      }
      if let Some(j) = j {
        if m01.get(&self[(d0, j)]).map(|&x| self[(x, j)]) != Some(d1) {
          return Err(GMapError::Unsewable);
        }
      }
      if !self.is_free(d0, i) || !self.is_free(d1, i) {
        return Err(GMapError::Unsewable);
      }
      m01.insert(d0, d1);
      s1.insert(d1);
    }

    for (&d0, &d1) in m01.iter() {
      self.link(i, d0, d1).unwrap();
    }
    Ok(m01)
  }

  /// Unsew the pair of i-cells at d.
  /// Returns the list of pairs of darts which were unsewn.
  pub fn unsew(&mut self, d: Dart, i: usize) -> Result<HashMap<Dart, Dart>, GMapError> {
    let indices = Alphas(!(1 << i) & !((1 << i) >> 1) & !((1 << i) << 1));
    let to_unsew: HashMap<Dart, Dart> = self.orbit(d, indices).map(|x| (x, self[(x, i)])).collect();
    if to_unsew.values().any(|d| to_unsew.contains_key(d)) {
      return Err(GMapError::Ununsewable);
    }
    for d0 in self.orbit(d, indices).collect::<Vec<_>>() {
      self.unlink(i, d0).unwrap();
    }
    Ok(to_unsew)
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

  /// one dart per a-orbit.
  /// returned darts are lowest-numbered in their a-orbit.
  pub fn one_dart_per_orbit<'a>(&'a self, a: Alphas) -> impl Iterator<Item = Dart> + 'a {
    self.unique_by_orbit(self.darts(), a)
  }

  /// one dart per i-cell.
  /// returned darts are lowest-numbered in their i-cell.
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

impl fmt::Debug for GMap {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let width = (self.ndarts() - 1).to_string().len();
    write!(f, "GMap {{\n")?;
    write!(f, "  dimension: {}\n", self.dimension)?;
    write!(f, "  alphas: {{\n")?;
    for i in 0..self.ndarts() {
      write!(
        f,
        "    {:width$}: [{}],\n",
        i,
        (0..=self.dimension)
          .map(|j| format!("{:width$}", self[(Dart(i), j)].0, width = width))
          .join(", "),
        width = width,
      )?;
    }
    write!(f, "}}}}")
  }
}

struct Orbit<'a> {
  g: &'a GMap,
  a: Alphas,
  seen: HashSet<Dart>,
  frontier: VecDeque<(Option<usize>, Dart)>,
}

impl Iterator for Orbit<'_> {
  type Item = (Option<usize>, Dart);

  fn next(&mut self) -> Option<Self::Item> {
    let (from, dart) = loop {
      match self.frontier.pop_front() {
        Some((from, dart)) if !self.seen.contains(&dart) => {
          break (from, dart);
        }
        None => {
          return None;
        }
        _ => {
          continue;
        }
      }
    };
    self.seen.insert(dart);
    for i in 0..=self.g.dimension() {
      if !self.a.has(i) {
        continue;
      }
      let neighbor = self.g[(dart, i)];
      self.frontier.push_back((Some(i), neighbor));
    }
    return Some((from, dart));
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathOrbitState {
  Initial,
  ForwardI,
  ForwardJ,
  BackwardI,
  BackwardJ,
  Done,
}

struct PathOrbit<'a> {
  g: &'a GMap,
  i: usize,
  j: usize,
  start: Dart,
  current: Dart,
  state: PathOrbitState,
}

impl Iterator for PathOrbit<'_> {
  type Item = (Option<usize>, Dart);

  fn next(&mut self) -> Option<Self::Item> {
    use PathOrbitState::*;

    if self.state == Initial {
      self.current = self.g[(self.start, self.i)];
      self.state = ForwardJ;
      if self.current == self.start {
        self.current = self.g[(self.start, self.j)];
        self.state = BackwardI;
      }
      return Some((None, self.start));
    }
    if self.state == Done {
      return None;
    }

    let old = self.current;
    if old == self.start {
      return None;
    }

    let (current_index, prev_index) = match self.state {
      ForwardI | BackwardI => (self.i, self.j),
      ForwardJ | BackwardJ => (self.j, self.i),
      _ => unreachable!(),
    };
    let result = Some((Some(prev_index), old));

    self.current = self.g[(old, current_index)];
    self.state = match self.state {
      ForwardI => ForwardJ,
      ForwardJ => ForwardI,
      BackwardI => BackwardJ,
      BackwardJ => BackwardI,
      _ => unreachable!(),
    };

    if self.current == old {
      match self.state {
        ForwardI | ForwardJ => {
          self.current = self.g[(self.start, self.j)];
          self.state = BackwardI;
        }
        _ => {
          self.state = Done;
        }
      }
    }

    return result;
  }
}

enum OrbitImpl<'a> {
  BFS(Orbit<'a>),
  Path(PathOrbit<'a>),
  Array1(std::array::IntoIter<(Option<usize>, Dart), 1>),
  Array2(std::array::IntoIter<(Option<usize>, Dart), 2>),
  Array4(std::array::IntoIter<(Option<usize>, Dart), 4>),
}

impl Iterator for OrbitImpl<'_> {
  type Item = (Option<usize>, Dart);

  fn next(&mut self) -> Option<Self::Item> {
    use OrbitImpl::*;
    match self {
      BFS(x) => x.next(),
      Path(x) => x.next(),
      Array1(x) => x.next(),
      Array2(x) => x.next(),
      Array4(x) => x.next(),
    }
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
#[derive(Debug)]
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
    v.resize(g.ndarts(), Dart(!0));
    let mut seen = HashSet::new();
    for d in (0..g.ndarts()).map(Dart) {
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

  /// for each dart, return the a-orbit representative.  repeated a-orbits are filtered out
  pub fn unique_orbit_repr<'a>(
    &'a self,
    g: &'a GMap,
    l: impl IntoIterator<Item = Dart> + 'a,
    a: Alphas,
  ) -> impl Iterator<Item = Dart> + 'a {
    let reprs = self.get_all(a);
    let mut seen = HashSet::new();
    l.into_iter().filter_map(move |dart| {
      if let Some(reprs) = reprs {
        let r = reprs[dart.0];
        if seen.contains(&r) {
          return None;
        }
        seen.insert(r);
        Some(r)
      } else {
        if seen.contains(&dart) {
          return None;
        }
        let mut r = dart;
        for d in g.orbit(dart, a) {
          if d.0 < r.0 {
            r = d;
          }
          seen.insert(d);
        }
        Some(r)
      }
    })
  }

  pub fn orbit_repr_per_incident_orbit<'a>(
    &'a self,
    g: &'a GMap,
    d: Dart,
    a: Alphas,
    b: Alphas,
  ) -> impl Iterator<Item = Dart> + 'a {
    self.unique_orbit_repr(g, g.orbit(d, b), a)
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
    #[rustfmt::skip]
    let alpha = darts([
      1, 5, 7,
      0, 2, 6,
      3, 1, 2,
      2, 4, 3,
      5, 3, 4,
      4, 0, 5,
      7, 11, 1,
      6, 8, 0,
      9, 7, 8,
      8, 10, 9,
      11, 9, 10,
      10, 6, 11,
    ]);
    GMap::from_alpha(2, alpha).unwrap()
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
      let side: Vec<Dart> = g.orbit(Dart(0), Alphas::SIDE).sorted().collect();
      let dart: Vec<Dart> = g.orbit(Dart(0), Alphas(0)).sorted().collect();
      let all: Vec<Dart> = g.orbit(Dart(0), Alphas(!0)).sorted().collect();
      assert_eq!(vertex, darts([0, 5, 7, 8]));
      assert_eq!(edge, darts([0, 1, 6, 7]));
      assert_eq!(face, darts(0..6));
      assert_eq!(halfedge, darts([0, 7]));
      assert_eq!(angle, darts([0, 5]));
      assert_eq!(side, darts([0, 1]));
      assert_eq!(dart, darts([0]));
      assert_eq!(all, darts(0..12));

      let faces: Vec<Dart> = g.one_dart_per_cell(2).collect();
      let halfedges: Vec<Dart> = g.one_dart_per_orbit(Alphas::HALF_EDGE).collect();
      assert_eq!(faces.len(), 2);
      assert_eq!(halfedges.len(), 10);

      let face0_cycle: Vec<Dart> = g.cycle(Dart(0), &[1, 0]).collect();
      let face1_cycle: Vec<Dart> = g.cycle(Dart(1), &[1, 0]).collect();
      let edge_cycle: Vec<Dart> = g.cycle(Dart(0), &[0, 2]).collect();
      assert_eq!(face0_cycle, darts([0, 4, 2]));
      assert_eq!(face1_cycle, darts([1, 3, 5]));
      assert_eq!(edge_cycle, darts([0, 6]));
    }

    let mut g = diagonal_cp_example();
    f(&g);
    g.increase_dimension(4).unwrap();
    f(&g);
  }

  #[test]
  fn test_sew() {
    let mut g = diagonal_cp_example();
    g.sew(1, Dart(2), Dart(4)).unwrap_err();
    // Currently there's no way to 2-link darts 2 and 3
    // via the public interface, since this isn't allowed by sewing.
    // Probably it isn't needed.
    g.sew(2, Dart(2), Dart(3)).unwrap_err();
    assert!(g.is_free(Dart(2), 2));

    let result = g.sew(2, Dart(2), Dart(10)).unwrap();
    assert_eq!(g[(Dart(2), 2)], Dart(10));
    assert_eq!(g[(Dart(3), 2)], Dart(11));
    assert_eq!(result.len(), 2);
    assert_eq!(result[&Dart(2)], Dart(10));
    assert_eq!(result[&Dart(3)], Dart(11));

    // test case where sewn regions have the same size but different shape
    g = GMap::empty(3).unwrap();
    let x = g.add_edge();
    let y = g.add_dart();
    let z = g.add_dart();
    g.sew(1, y, z).unwrap();
    g.sew(3, x, y).unwrap_err();
    assert!(g.is_free(x, 3));

    let w = g.add_dart();
    g.sew(1, x, w).unwrap();
    let v = g.add_dart();
    g.sew(0, y, v).unwrap();
    let result = g.sew(3, x, y).unwrap();
    assert_eq!(result.len(), 3);
    assert!(!g.is_free(x, 3));
  }

  #[test]
  fn test_unsew() {
    let mut g = diagonal_cp_example();
    g.unsew(Dart(2), 2).unwrap_err();
    g.unsew(Dart(2), 1).unwrap();
    assert!(g.is_free(Dart(2), 1));
    assert!(g.is_free(Dart(1), 1));

    g = diagonal_cp_example();
    g.unsew(Dart(0), 0).unwrap();
    assert!(g.is_free(Dart(0), 0));
    assert!(g.is_free(Dart(1), 0));
    assert!(g.is_free(Dart(6), 0));
    assert!(g.is_free(Dart(7), 0));
    assert!(!g.is_free(Dart(0), 2));
    assert!(!g.is_free(Dart(1), 2));
    assert!(!g.is_free(Dart(6), 2));
    assert!(!g.is_free(Dart(7), 2));

    g = diagonal_cp_example();
    g.unsew(Dart(0), 2).unwrap();
    assert!(!g.is_free(Dart(0), 0));
    assert!(!g.is_free(Dart(1), 0));
    assert!(!g.is_free(Dart(6), 0));
    assert!(!g.is_free(Dart(7), 0));
    assert!(g.is_free(Dart(0), 2));
    assert!(g.is_free(Dart(1), 2));
    assert!(g.is_free(Dart(6), 2));
    assert!(g.is_free(Dart(7), 2));

    #[rustfmt::skip]
    {
      g = GMap::from_alpha(2, darts([
        1, 0, 1,
        0, 1, 0,
      ])).unwrap()
    };
    g.unsew(Dart(0), 2).unwrap_err();
    assert!(!g.is_free(Dart(0), 2));
  }

  #[test]
  fn test_orbit_reprs() {
    let g = diagonal_cp_example();
    let mut or = OrbitReprs::new();
    or.build(&g, Alphas::FACE);

    assert_eq!(or[(Alphas::FACE, Dart(4))], Dart(0));
    assert_eq!(
      or.get_all(Alphas::FACE).unwrap(),
      darts([0, 0, 0, 0, 0, 0, 6, 6, 6, 6, 6, 6])
    );

    assert_eq!(
      or.ensure_all(&g, Alphas::EDGE),
      darts([0, 0, 2, 2, 4, 4, 0, 0, 8, 8, 10, 10])
    );
  }

  #[test]
  fn test_orbit_maps() {
    let g = diagonal_cp_example();
    let mut m: OrbitMap<usize> = OrbitMap::over_cells(0);
    m.insert(&g, Dart(0), 1);
    m.insert(&g, Dart(4), 2);
    m.insert(&g, Dart(7), 3);

    let m: Vec<(Dart, usize)> = m.into_map().into_iter().sorted().collect();
    let expected: Vec<(Dart, usize)> = [(0, 3), (3, 2), (4, 2), (5, 3), (7, 3), (8, 3)]
      .into_iter()
      .map(|(x, y)| (Dart(x), y))
      .collect();
    assert_eq!(m, expected);
  }
}
