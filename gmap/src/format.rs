pub use crate::Alphas;

use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Dart(pub usize);

#[derive(Serialize, Deserialize)]
pub struct GMap {
  dimension: usize,
  alpha: HashMap<Dart, Vec<Dart>>,
}

impl GMap {
  fn to_gmap(
    &self,
  ) -> Result<(crate::GMap, Vec<Dart>, HashMap<Dart, crate::Dart>), crate::GMapError> {
    let output_input: Vec<Dart> = self.alpha.keys().cloned().collect();
    let input_output: HashMap<Dart, crate::Dart> = output_input
      .iter()
      .enumerate()
      .map(|(x, &y)| (y, crate::Dart(x)))
      .collect();
    let alpha: Vec<crate::Dart> = output_input
      .iter()
      .flat_map(|y| self.alpha[y].iter().map(|x| input_output[x]))
      .collect();
    crate::GMap::from_alpha(self.dimension, alpha).map(|g| (g, output_input, input_output))
  }

  fn with_translation<F>(g: &crate::GMap, translation: F) -> Self
  where
    F: Fn(crate::Dart) -> Dart,
  {
    let alpha: HashMap<Dart, Vec<Dart>> = g
      .alpha()
      .iter()
      .chunks(g.dimension + 1)
      .into_iter()
      .enumerate()
      .filter_map(|(i, c)| {
        if g.is_deleted(crate::Dart(i)) {
          None
        } else {
          Some((
            translation(crate::Dart(i)),
            c.map(|&d| translation(d)).collect(),
          ))
        }
      })
      .collect();
    Self {
      dimension: g.dimension,
      alpha,
    }
  }
}

impl From<&crate::GMap> for GMap {
  fn from(g: &crate::GMap) -> Self {
    Self::with_translation(g, |crate::Dart(i)| Dart(i))
  }
}

#[derive(Serialize, Deserialize)]
pub struct OrbitMap<A> {
  map: HashMap<Dart, A>,
  indices: Vec<usize>,
}

impl<A> OrbitMap<A> {
  fn into_orbitmap<F>(self, translation: F) -> crate::OrbitMap<A>
  where
    F: Fn(Dart) -> crate::Dart,
  {
    let map = self.map.into_iter().map(|(k, v)| (translation(k), v));
    crate::OrbitMap {
      map,
      indices: Alphas::from_indices(self.indices),
    }
  }

  fn with_translation<F>(o: crate::OrbitMap<A>, translation: F) -> Self
  where
    F: Fn(crate::Dart) -> Dart,
  {
    let map = o.into_map().into_iter().map(|(k, v)| (translation(k), v));
    Self {
      map,
      indices: o.indices.to_indices(),
    }
  }
}

impl<A> From<crate::OrbitMap<A>> for OrbitMap<A> {
  fn from(o: crate::OrbitMap<A>) -> Self {
    self.with_translation(o, |crate::Dart(i)| Dart(i))
  }
}
