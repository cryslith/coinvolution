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

pub enum Error {
  MissingTranslation,
}

impl GMap {
  fn to_gmap(&self) -> Result<(crate::GMap, Vec<Dart>), crate::GMapError> {
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
    crate::GMap::from_alpha(self.dimension, alpha).map(|g| (g, output_input))
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
  indices: Alphas,
}
