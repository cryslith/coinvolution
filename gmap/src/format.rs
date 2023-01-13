use crate::Dart;

use std::collections::HashMap;

use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GMap {
  dimension: usize,
  alpha: HashMap<Dart, Vec<Dart>>,
}

impl From<crate::GMap> for GMap {
  fn from(g: crate::GMap) -> Self {
    let alpha: HashMap<Dart, Vec<Dart>> = g
      .alpha()
      .iter()
      .cloned()
      .chunks(g.dimension + 1)
      .into_iter()
      .enumerate()
      .filter_map(|(i, c)| {
        if g.is_deleted(Dart(i)) {
          None
        } else {
          Some((Dart(i), c.collect()))
        }
      })
      .collect();
    Self {
      dimension: g.dimension,
      alpha,
    }
  }
}

impl TryFrom<GMap> for crate::GMap {
  type Error = crate::GMapError;
  fn try_from(o: GMap) -> Result<Self, Self::Error> {
    Self::from_alpha(o.dimension, o.alpha)
  }
}

#[derive(Serialize, Deserialize)]
pub struct Alphas(Vec<usize>);

impl From<crate::Alphas> for Alphas {
  fn from(a: crate::Alphas) -> Self {
    Self(a.to_indices().collect())
  }
}

impl From<Alphas> for crate::Alphas {
  fn from(o: Alphas) -> Self {
    Self::from_indices(o.0)
  }
}
