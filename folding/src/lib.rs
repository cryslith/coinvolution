mod convex;
mod format;

use std::collections::{HashMap, VecDeque};
use std::f64::consts::PI;

use gmap::{Dart, GMap, OrbitMap};
use na::{Isometry3, Point2, Point3, Rotation3, Unit, UnitQuaternion};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
  #[error(transparent)]
  GMap(#[from] gmap::GMapError),
  #[error("FOLD field is required: {0}")]
  FoldMissingField(String),
  #[error("FOLD input must be a manifold")]
  FoldNonManifold,
  #[error("FOLD input contains invalid coordinates")]
  FoldBadCoordinates,
  #[error("FOLD input contains invalid angle assignment")]
  FoldBadAngle,
  #[error("FOLD file references nonexistent {0} at index {1}")]
  FoldInvalidReference(String, usize),
}

pub struct CreasePattern {
  /// 2-dimensional planar graph
  g: GMap,
  /// True if the dart points counterclockwise within its face, false if it points clockwise.  This determines the direction of the face normal according to the right hand rule.
  orientation: HashMap<Dart, bool>,
  /// Location of each vertex in the unfolded crease pattern.
  vertices_coords: OrbitMap<Point2<f64>>,
  /// Angle of each edge in the range [-180, 180] degrees.
  /// Positive angles point the face normals towards each other.
  fold_angle: OrbitMap<f64>,
}

/// information connecting the crease pattern to a FOLD file.
pub struct FoldTracking {
  face_to_dart: HashMap<usize, Dart>,
  edge_to_dart: HashMap<usize, Dart>,
  vertex_to_dart: HashMap<usize, Dart>,
}

impl CreasePattern {
  pub fn from(fold: &format::Fold) -> Result<(Self, FoldTracking), Error> {
    let f = &fold.key_frame;
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
      // let faces: Vec<usize> = faces.iter().map(|x| x.unwrap()).collect();
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
    let mut vertices_coords: OrbitMap<Point2<f64>> = OrbitMap::over_cells(1);
    for (vertex, coords) in f.vertices_coords.iter().enumerate() {
      let &d = vertex_to_dart
        .get(&vertex)
        .ok_or_else(|| Error::FoldInvalidReference("vertex".to_string(), vertex))?;
      if coords.len() < 2 || coords.len() > 3 || (coords.len() == 3 && coords[2] != 0f64) {
        return Err(Error::FoldBadCoordinates);
      }
      let p = Point2::new(coords[0], coords[1]);
      vertices_coords.insert(&g, d, p);
    }

    // and fold angle
    fn interpret_assignment(a: &String) -> Result<Option<f64>, Error> {
      Ok(Some(match &a[..] {
        "B" => return Ok(None),
        "M" => -180f64,
        "V" => 180f64,
        "F" => 0f64,
        _ => return Err(Error::FoldBadAngle),
      }))
    }
    // check that there are no unassigned or malformed angles
    f.edges_assignment
      .iter()
      .map(|x| interpret_assignment(x).map(|_| ()))
      .fold(Ok(()), |r, a| r.and(a))?;
    let mut angles_from_assignment = f
      .edges_assignment
      .iter()
      .enumerate()
      .filter_map(|(i, a)| interpret_assignment(a).unwrap().map(|x| (i, x)));
    let mut angles_specified = f.edges_fold_angle.iter().copied().enumerate();
    let edges_fold_angle: &mut dyn Iterator<Item = (usize, f64)> = if f.edges_fold_angle.is_empty()
    {
      &mut angles_from_assignment
    } else {
      &mut angles_specified
    };

    let mut fold_angle: OrbitMap<f64> = OrbitMap::over_cells(1);
    for (edge, angle) in edges_fold_angle {
      let &d = edge_to_dart
        .get(&edge)
        .ok_or_else(|| Error::FoldInvalidReference("edge".to_string(), edge))?;
      if angle < -180f64 || angle > 180f64 {
        return Err(Error::FoldBadAngle);
      }
      fold_angle.insert(&g, d, angle);
    }

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

pub struct FoldedState {
  cp: CreasePattern,
  /// Locations of vertices in folded state
  folded_coords: OrbitMap<Point3<f64>>,
  face_isometries: OrbitMap<Isometry3<f64>>,
  // layers: ?
}

impl FoldedState {
  pub fn from(cp: CreasePattern) -> Self {
    let g = &cp.g;
    let mut folded_coords: OrbitMap<Point3<f64>> = OrbitMap::over_cells(0);
    let mut seen_edges: OrbitMap<()> = OrbitMap::over_cells(1);
    let mut isometries: OrbitMap<Isometry3<f64>> = OrbitMap::over_cells(2);

    let mut frontier: VecDeque<Dart> = VecDeque::new();
    let first_face = if cp.orientation[&Dart(0)] {
      Dart(0)
    } else {
      g.al(Dart(0), [0])
    };
    frontier.push_back(first_face);
    isometries.insert(g, first_face, Isometry3::identity());

    while !frontier.is_empty() {
      let my_face = frontier.pop_front().unwrap();
      let my_isometry = isometries.map()[&my_face];

      // loop over counterclockwise darts of my_face
      let mut edge = my_face;
      loop {
        if !seen_edges.map().contains_key(&edge) && !g.is_free(edge, 2) {
          let other_face = g.al(edge, [2, 0]);
          let other_isometry: Isometry3<f64> = {
            let p = cp.vertices_coords.map()[&edge];
            let q = cp.vertices_coords.map()[&other_face];
            let p = Point3::new(p.x, p.y, 0.0);
            let q = Point3::new(q.x, q.y, 0.0);
            // here we use the fact that edge is counterclockwise
            // to get the correct sign on the angle
            let fold_angle_rad = cp.fold_angle.map()[&edge] * PI / 180.0;
            let axis = Unit::new_normalize(q - p);
            let rotation = Rotation3::from_axis_angle(&axis, fold_angle_rad);
            let r1 = Isometry3::rotation_wrt_point(UnitQuaternion::from(rotation), p);
            my_isometry * r1
          };
          if let Some(old_other_isometry) = isometries.map().get(&other_face) {
            todo!("check old isometry matches new one")
          } else {
            isometries.insert(g, other_face, other_isometry);
            for vertex in g.one_dart_per_incident_cell(other_face, 0, 2) {
              if !folded_coords.map().contains_key(&vertex) {
                let p_v = cp.vertices_coords.map()[&vertex];
                let p_v = Point3::new(p_v.x, p_v.y, 0.0);
                folded_coords.insert(g, vertex, p_v);
              }
            }
            frontier.push_back(other_face);
          }

          seen_edges.insert(g, edge, ());
        }

        edge = g.al(edge, [1, 0]);
        if edge == my_face {
          break;
        }
      }
    }

    Self {
      cp,
      folded_coords,
      face_isometries: isometries,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use format::tests::load_example;

  // todo: these are integration tests not unit tests; move them somewhere else and add an actual unit test

  #[test]
  fn parse_diagonal_cp() {
    let f = load_example("diagonal-cp.fold");
    let (cp, ft) = CreasePattern::from(&f).unwrap();
    let nverts = cp.g.one_dart_per_cell(0).count();
    let nedges = cp.g.one_dart_per_cell(1).count();
    let nfaces = cp.g.one_dart_per_cell(2).count();
    assert_eq!(nverts, 4);
    assert_eq!(nedges, 5);
    assert_eq!(nfaces, 2);

    for i in 0..4 {
      let v = ft.vertex_to_dart[&i];
      assert_eq!(cp.orientation[&v], true);
      assert_eq!(cp.orientation[&cp.g.al(v, [0])], false);

      let expected = match i {
        0 => Point2::new(0.0, 0.0),
        1 => Point2::new(1.0, 0.0),
        2 => Point2::new(1.0, 1.0),
        3 => Point2::new(0.0, 1.0),
        _ => panic!(),
      };
      assert_eq!(cp.vertices_coords.map()[&v], expected);
    }

    let e = ft.edge_to_dart[&4];
    assert_eq!(cp.fold_angle.map()[&e], 180f64);
  }

  #[test]
  fn fold_diagonal_cp_unchecked() {
    let f = load_example("diagonal-cp.fold");
    let (cp, ft) = CreasePattern::from(&f).unwrap();
    let fs = FoldedState::from(cp);
  }
}
