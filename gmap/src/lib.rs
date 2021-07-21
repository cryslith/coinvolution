use itertools::Itertools;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub enum GMapError {
  InvalidAlpha(String),
  CannotDecreaseDimension,
  Unsewable,
  NotFree,
  AlreadyFree,
}

pub fn cell_alphas(i: usize, dim: usize) -> Vec<usize> {
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

  pub fn grid(n: usize, m: usize) -> (Self, Vec<Vec<usize>>) {
    let mut g = Self::empty(2);
    let rows: Vec<Vec<usize>> = (0..n)
      .map(|_| (0..m).map(|_| g.add_polygon(4)).collect())
      .collect();
    // Each square is the dart on the square's north edge, northwest vertex
    for r in &rows {
      for (&s0, &s1) in r.iter().tuple_windows() {
        g.sew(2, g.al(s0, [0, 1]), g.al(s1, [1])).unwrap();
      }
    }

    for (r0, r1) in rows.iter().tuple_windows() {
      for (&s0, &s1) in r0.iter().zip(r1.iter()) {
        g.sew(2, g.al(s0, [1, 0, 1]), s1).unwrap();
      }
    }

    (g, rows)
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

  pub fn al(&self, d: usize, alphas: impl IntoIterator<Item = usize>) -> usize {
    alphas.into_iter().fold(d, |d, a| self.alpha[d][a])
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
    self.orbit(d, &cell_alphas(i, dim.unwrap_or(self.dimension)))
  }

  pub fn sew(&mut self, i: usize, d0: usize, d1: usize) -> Result<Vec<(usize, usize)>, GMapError> {
    let alphas: Vec<usize> = (0..=self.dimension)
      .filter(|x| (x.wrapping_sub(i) as isize).abs() > 1)
      .collect();
    let m0: HashMap<_, _> = self.orbit_paths(d0, &alphas).into_iter().collect();
    let mut m1: HashMap<_, _> = self.orbit_paths(d1, &alphas).into_iter().collect();
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
    let alphas: Vec<usize> = (0..=self.dimension)
      .filter(|x| (x.wrapping_sub(i) as isize).abs() > 1)
      .collect();
    let mut output = Vec::new();
    for d0 in self.orbit(d, &alphas) {
      let d1 = self.unlink(i, d0)?;
      output.push((d0, d1));
    }
    Ok(output)
  }
}
