mod format;

use std::collections::HashMap;

use gmap::{Dart, GMap, OrbitMap};
use na::Point2;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  GMap(#[from] gmap::GMapError),
  #[error("FOLD field is required: {0}")]
  FoldMissingField(String),
  #[error("FOLD input must be a manifold")]
  FoldNonManifold,
  #[error("FOLD file references nonexistent {0} at index {1}")]
  FoldInvalidReference(String, usize),
}

pub struct CreasePattern {
  /// 2-dimensional planar graph
  g: GMap,
  /// True if the dart points counterclockwise within its face, false if it points clockwise.  This determines the direction of the face normal according to the right hand rule.
  orientation: HashMap<Dart, bool>,
  /// Location of each vertex in the unfolded crease pattern.
  vertices_coords: HashMap<Dart, Point2<f64>>,
  /// Angle of each edge in the range [-180, 180] degrees.
  /// Positive angles point the face normals towards each other.
  fold_angle: OrbitMap<f64>,
}

/// information connecting the crease pattern to a FOLD file
pub struct FoldTracking {
  face_to_dart: HashMap<usize, Dart>,
  edge_to_dart: HashMap<usize, Dart>,
  vertex_to_dart: HashMap<usize, Dart>,
}

impl CreasePattern {
  pub fn from(fold: format::Fold) -> Result<(Self, FoldTracking), Error> {
    let f = fold.key_frame;
    let mut g = GMap::empty(2)?;
    // these contain only counterclockwise darts
    let mut face_to_dart: HashMap<usize, Dart> = HashMap::new();
    let mut edge_to_dart: HashMap<usize, Dart> = HashMap::new();
    let mut vertex_to_dart: HashMap<usize, Dart> = HashMap::new();

    let mut orientation: HashMap<Dart, bool> = HashMap::new();

    for (n, v) in [
      ("faces_vertices", f.faces_vertices.is_empty()),
      ("edges_faces", f.edges_faces.is_empty()),
      ("vertices_coords", f.vertices_coords.is_empty()),
      (
        "edges_assignment or edges_foldAngle",
        f.edges_assignment.is_empty() && f.edges_fold_angle.is_empty(),
      ),
    ] {
      if v {
        return Err(Error::FoldMissingField(n.to_string()));
      }
    }

    // first make polygons for all the faces
    for (face, vertices) in f.faces_vertices.iter().enumerate() {
      let mut d = g.add_polygon(vertices.len());
      face_to_dart.insert(face, d);
      for &vertex in vertices.iter() {
        if !vertex_to_dart.contains_key(&vertex) {
          vertex_to_dart.insert(vertex, d);
        }
        orientation.insert(d, true);
        orientation.insert(g.al(d, [1]), false);
        d = g.al(d, [1, 0]);
      }
    }

    // now glue the polygons together along their edges
    for (edge, faces) in f.edges_faces.iter().enumerate() {
      if faces.len() < 1 || faces.len() > 2 {
        return Err(Error::FoldNonManifold);
      }
      let &d = face_to_dart
        .get(&faces[0])
        .ok_or_else(|| Error::FoldInvalidReference("face".to_string(), faces[0]))?;
      edge_to_dart.insert(edge, d);
      if faces.len() == 1 {
        continue;
      }
      let &d2 = face_to_dart
        .get(&faces[1])
        .ok_or_else(|| Error::FoldInvalidReference("face".to_string(), faces[1]))?;
      // reverse d2 before sewing to keep the orientation correct,
      // since both of them are counterclockwise
      g.sew(2, d, g.al(d2, [0]))?;
    }

    // extract vertex coordinates
    let vertices_coords: HashMap<Dart, Point2<f64>> = HashMap::new();
    todo!();

    // and fold angle
    let fold_angle: OrbitMap<f64> = OrbitMap::over_cells(1);
    todo!();

    let cp = CreasePattern {
      g,
      orientation,
      vertices_coords,
      fold_angle,
    };

    let ft = FoldTracking {
      face_to_dart,
      edge_to_dart,
      vertex_to_dart,
    };

    Ok((cp, ft))
  }
}
