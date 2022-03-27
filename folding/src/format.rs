/// Implementation of the FOLD spec.
/// See https://github.com/edemaine/fold/blob/main/doc/spec.md

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Fold {
  pub file_spec: f64,
  pub file_creator: String,
  pub file_author: String,
  pub file_title: String,
  pub file_description: String,
  pub file_classes: Vec<String>,
  #[serde(flatten)]
  pub key_frame: Frame,
  pub file_frames: Vec<Frame>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
pub struct Frame {
  pub frame_author: String,
  pub frame_title: String,
  pub frame_description: String,
  pub frame_classes: Vec<String>,
  pub frame_attributes: Vec<String>,
  pub frame_unit: String,
  pub vertices_coords: Vec<Vec<f64>>,
  pub vertices_vertices: Vec<Vec<usize>>,
  pub vertices_faces: Vec<Vec<usize>>,
  pub edges_vertices: Vec<(usize, usize)>,
  pub edges_faces: Vec<Vec<usize>>,
  pub edges_assignment: Vec<String>,
  #[serde(rename = "edges_fold_angle")]
  pub edges_fold_angle: Vec<f64>,
  pub edges_length: Vec<f64>,
  pub faces_vertices: Vec<Vec<usize>>,
  pub faces_edges: Vec<Vec<usize>>,
  #[serde(rename = "faceOrders")]
  pub face_orders: Vec<(usize, usize, i8)>,
  #[serde(rename = "edgeOrders")]
  pub edge_orders: Vec<(usize, usize, i8)>,
  pub frame_parent: Option<usize>,
  pub frame_inherit: bool,
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::fs::File;
  use std::io::BufReader;
  use std::path::PathBuf;

  #[test]
  fn load_diagonal_cp() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/fold-examples/diagonal-cp.fold");
    let file = File::open(d).unwrap();
    let reader = BufReader::new(file);
    let f: Fold = serde_json::from_reader(reader).unwrap();
    assert_eq!(f.file_creator, "Crease Pattern Editor");
  }
}
