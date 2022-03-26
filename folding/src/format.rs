/// Implementation of the FOLD spec.
/// See https://github.com/edemaine/fold/blob/main/doc/spec.md

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct Fold {
  file_spec: f64,
  file_creator: String,
  file_author: String,
  file_title: String,
  file_description: String,
  file_classes: Vec<String>,
  #[serde(flatten)]
  key_frame: Frame,
  file_frames: Vec<Frame>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(default)]
struct Frame {
  frame_author: String,
  frame_title: String,
  frame_description: String,
  frame_classes: Vec<String>,
  frame_attributes: Vec<String>,
  frame_unit: String,
  vertices_coords: Vec<Vec<f64>>,
  vertices_vertices: Vec<Vec<usize>>,
  vertices_faces: Vec<Vec<usize>>,
  edges_vertices: Vec<(usize, usize)>,
  edges_faces: Vec<Vec<usize>>,
  edges_assignment: Vec<String>,
  #[serde(rename = "edges_fold_angle")]
  edges_fold_angle: Vec<f64>,
  edges_length: Vec<f64>,
  faces_vertices: Vec<Vec<usize>>,
  faces_edges: Vec<Vec<usize>>,
  #[serde(rename = "faceOrders")]
  face_orders: Vec<(usize, usize, i8)>,
  #[serde(rename = "edgeOrders")]
  edge_orders: Vec<(usize, usize, i8)>,
  frame_parent: Option<usize>,
  frame_inherit: bool,
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
