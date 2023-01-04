use crate::puzzle::{Layer};

use gmap::{GMap, OrbitMap};
use serde::{Deserialize , Serialize};

#[derive(Serialize)]
pub(crate) struct SolveRequest {
  pub graph: GMap,
  pub layers: Vec<Layer>,
}

#[derive(Deserialize)]
pub(crate) struct SolveResponse {
  pub layers: Vec<Layer>,
}
