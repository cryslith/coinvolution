pub mod grids;

use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum GMapError {
  InvalidAlpha(String),
  CannotDecreaseDimension,
  Unsewable,
  NotFree,
  AlreadyFree,
}

pub fn cell_indices(i: usize, dim: usize) -> Vec<usize> {
  (0..=dim).filter(|&x| x != i).collect()
}

pub struct GMap {
  dimension: usize,
  alpha: Vec<Vec<usize>>,
}

impl GMap {
  pub fn empty(dimension: usize) -> Self {
    Self::from_alpha(dimension, vec![]).unwrap()
  }

  pub fn from_alpha(dimension: usize, alpha: Vec<Vec<usize>>) -> Result<Self, GMapError> {
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
        if x >= self.alpha.len() {
          return Err(GMapError::InvalidAlpha(format!(
            "dart {} index {} out of range",
            d, i
          )));
        }
      }
    }

    for i in 0..=self.dimension {
      for (d, al) in self.alpha.iter().cloned().enumerate() {
        if self.alpha[al[i]][i] != d {
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
          if self.alpha[al[i]][j] != self.alpha[al[j]][i] {
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

  pub fn alpha(&self) -> &[Vec<usize>] {
    &self.alpha
  }

  pub fn al(&self, d: usize, indices: impl IntoIterator<Item = usize>) -> usize {
    indices.into_iter().fold(d, |d, a| self.alpha[d][a])
  }

  pub fn increase_dimension(&mut self, dim: usize) -> Result<(), GMapError> {
    if dim < self.dimension {
      return Err(GMapError::CannotDecreaseDimension);
    }
    self.dimension = dim;
    for (d, al) in self.alpha.iter_mut().enumerate() {
      al.resize(dim, d);
    }
    Ok(())
  }

  pub fn add_dart(&mut self) -> usize {
    self.alpha.push(vec![self.alpha.len(); self.dimension + 1]);
    self.alpha.len() - 1
  }

  fn link(&mut self, i: usize, d0: usize, d1: usize) -> Result<(), GMapError> {
    if self.alpha[d0][i] != d0 {
      return Err(GMapError::NotFree);
    }
    self.alpha[d0][i] = d1;
    self.alpha[d1][i] = d0;
    Ok(())
  }

  fn unlink(&mut self, i: usize, d0: usize) -> Result<usize, GMapError> {
    let d1 = self.alpha[d0][i];
    if d0 == d1 {
      return Err(GMapError::AlreadyFree);
    }
    self.alpha[d0][i] = d0;
    self.alpha[d1][i] = d1;
    Ok(d1)
  }

  pub fn add_edge(&mut self) -> usize {
    let d0 = self.add_dart();
    let d1 = self.add_dart();
    self.link(0, d0, d1).unwrap();
    d0
  }

  pub fn add_polygon(&mut self, n: usize) -> usize {
    let start = self.add_edge();
    let mut prev = self.alpha[start][0];
    for _ in 0..(n - 1) {
      let c = self.add_edge();
      self.link(1, prev, c).unwrap();
      prev = self.alpha[c][0];
    }
    self.link(1, start, prev).unwrap();
    start
  }

  pub fn orbit_paths(&self, d: usize, a: &[usize]) -> Vec<(Vec<usize>, usize)> {
    let mut seen = HashSet::new();
    let mut frontier: Vec<(Vec<usize>, usize)> = vec![(vec![], d)];
    let mut orbit = Vec::new();
    while !frontier.is_empty() {
      let (path, dart) = frontier.remove(0);
      if seen.contains(&dart) {
        continue;
      }
      seen.insert(dart);
      orbit.push((path.clone(), dart));
      for &i in a {
        let neighbor = self.alpha[dart][i];
        let mut new_path = path.clone();
        new_path.push(i);
        frontier.push((new_path, neighbor));
      }
    }
    orbit
  }

  pub fn orbit(&self, d: usize, a: &[usize]) -> impl Iterator<Item = usize> {
    self.orbit_paths(d, a).into_iter().map(|(_, d)| d)
  }

  pub fn cell(&self, d: usize, i: usize, dim: Option<usize>) -> impl Iterator<Item = usize> {
    self.orbit(d, &cell_indices(i, dim.unwrap_or(self.dimension)))
  }

  pub fn sew(&mut self, i: usize, d0: usize, d1: usize) -> Result<Vec<(usize, usize)>, GMapError> {
    let indices: Vec<usize> = (0..=self.dimension)
      .filter(|x| (x.wrapping_sub(i) as isize).abs() > 1)
      .collect();
    let m0: HashMap<_, _> = self.orbit_paths(d0, &indices).into_iter().collect();
    let mut m1: HashMap<_, _> = self.orbit_paths(d1, &indices).into_iter().collect();
    if m0.len() != m1.len() || m0.iter().any(|(x, _)| !m1.contains_key(x)) {
      return Err(GMapError::Unsewable);
    }
    let mut output = Vec::new();
    for (k, d0) in m0.into_iter() {
      let d1 = m1.remove(&k).ok_or(GMapError::Unsewable)?;
      self.link(i, d0, d1)?;
      output.push((d0, d1));
    }
    Ok(output)
  }

  pub fn unsew(&mut self, d: usize, i: usize) -> Result<Vec<(usize, usize)>, GMapError> {
    let indices: Vec<usize> = (0..=self.dimension)
      .filter(|x| (x.wrapping_sub(i) as isize).abs() > 1)
      .collect();
    let mut output = Vec::new();
    for d0 in self.orbit(d, &indices) {
      let d1 = self.unlink(i, d0)?;
      output.push((d0, d1));
    }
    Ok(output)
  }

  /// filter out darts which are in the same a-orbit as a previous dart
  pub fn unique_by_orbit<'a>(
    &'a self,
    l: impl IntoIterator<Item = usize> + 'a,
    a: Vec<usize>,
  ) -> impl Iterator<Item = usize> + 'a {
    let mut seen = HashSet::new();
    l.into_iter().filter_map(move |dart| {
      if seen.contains(&dart) {
        return None;
      }
      for n in self.orbit(dart, &a) {
        seen.insert(n);
      }
      Some(dart)
    })
  }

  /// one dart per a-orbit
  pub fn one_dart_per_orbit<'a>(&'a self, a: Vec<usize>) -> impl Iterator<Item = usize> + 'a {
    self.unique_by_orbit(0..self.alpha.len(), a)
  }

  /// one dart per i-cell (in dim)
  pub fn one_dart_per_cell<'a>(
    &'a self,
    i: usize,
    dim: Option<usize>,
  ) -> impl Iterator<Item = usize> + 'a {
    self.one_dart_per_orbit(cell_indices(i, dim.unwrap_or(self.dimension)))
  }

  /// one dart per a-orbit incident to d's b-orbit.
  /// darts are guaranteed to be in both orbits.
  pub fn one_dart_per_incident_orbit<'a>(
    &'a self,
    d: usize,
    a: Vec<usize>,
    b: &[usize],
  ) -> impl Iterator<Item = usize> + 'a {
    self.unique_by_orbit(self.orbit(d, &b), a)
  }

  /// one dart per i-cell (in dim) incident to d's j-cell (in dim).
  /// darts are guaranteed to be in both cells.
  pub fn one_dart_per_incident_cell<'a>(
    &'a self,
    d: usize,
    i: usize,
    j: usize,
    dim: Option<usize>,
  ) -> impl Iterator<Item = usize> + 'a {
    let dim = dim.unwrap_or(self.dimension);
    self.one_dart_per_incident_orbit(d, cell_indices(i, dim), &cell_indices(j, dim))
  }
}

pub struct OrbitMap<A> {
  map: HashMap<usize, A>,
  indices: Vec<usize>,
}

impl<A> OrbitMap<A>
where
  A: Clone,
{
  pub fn new(indices: Vec<usize>) -> Self {
    Self {
      map: HashMap::new(),
      indices,
    }
  }

  pub fn over_cells(i: usize, dim: usize) -> Self {
    Self::new(cell_indices(i, dim))
  }

  pub fn map(&self) -> &HashMap<usize, A> {
    &self.map
  }

  pub fn into_map(self) -> HashMap<usize, A> {
    self.map
  }

  pub fn indices(&self) -> &[usize] {
    &self.indices
  }

  pub fn insert(&mut self, g: &GMap, k: usize, v: A) {
    for n in g.orbit(k, &self.indices) {
      self.map.insert(n, v.clone());
    }
  }

  pub fn remove(&mut self, g: &GMap, k: usize) -> Option<A> {
    g.orbit(k, &self.indices)
      .fold(None, |v, n| v.or(self.map.remove(&n)))
  }
}
