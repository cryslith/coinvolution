mod convex;
mod format;

use std::collections::{HashMap, HashSet, VecDeque};
use std::f64::consts::PI;

use gmap::{Alphas, Dart, GMap, OrbitMap};
use na::{Isometry3, Point2, Point3, Rotation3, Unit, UnitQuaternion, Vector3};
use thiserror::Error;

const ISO_ANGLE_EPSILON: f64 = 0.001;
const ISO_LENGTH_EPSILON: f64 = 0.001;

const COPLANAR_ANGLE_EPSILON: f64 = 0.001;
const PLANE_DISTANCE_EPSILON: f64 = 0.001;

const FACE_SHRINK_EPSILON: f64 = 0.001;

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
  #[error("FOLD file has incompatible face data for face {0}")]
  FoldBadFace(usize),
  #[error("Folding found distinct isometries for face dart {0}")]
  DistinctIsometries(Dart),
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
  // XXX shouldn't these be vectors?
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
      ("faces_edges", f.faces_edges.is_empty()),
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

    for (face, (vertices, edges)) in f
      .faces_vertices
      .iter()
      .zip(f.faces_edges.iter())
      .enumerate()
    {
      if vertices.len() != edges.len() {
        return Err(Error::FoldBadFace(face));
      }

      // make polygon for each face
      let mut d = g.add_polygon(vertices.len());
      face_to_dart.insert(face, d);

      // assign vertices and assign+sew edges.
      // recall from the FOLD spec that vertices are listed in counterclockwise order
      // and edges[i] connects vertices[i] to vertices[i+1].
      for (&vertex, &edge) in vertices.iter().zip(edges.iter()) {
        // d is the counterclockwise dart for vertices[i]
        if !vertex_to_dart.contains_key(&vertex) {
          vertex_to_dart.insert(vertex, d);
        }
        orientation.insert(d, true);
        orientation.insert(g.al(d, [1]), false);

        d = g.al(d, [1, 0]);

        // edges[i] connects vertices[i] to vertices[i+1], so now d = al(vertices[i], [1, 0]) is
        // the counterclockwise dart for both edges[i] and vertices[i+1]

        if let Some(&other_edge) = edge_to_dart.get(&edge) {
          // sew our edge to the other edge
          // reverse before sewing because both darts are counterclockwise
          g.sew(2, d, g.al(other_edge, [0]))?;
        } else {
          edge_to_dart.insert(edge, d);
        }
      }
    }

    // extract vertex coordinates
    let mut vertices_coords: OrbitMap<Point2<f64>> = OrbitMap::over_cells(0);
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
  /// Locations of vertices in folded state
  folded_coords: OrbitMap<Point3<f64>>,
  face_isometries: OrbitMap<Isometry3<f64>>,
  // layers: ?
}

impl FoldedState {
  pub fn from(cp: &CreasePattern, fixed_face: Dart) -> Result<Self, Error> {
    let g = &cp.g;
    let mut folded_coords: OrbitMap<Point3<f64>> = OrbitMap::over_cells(0);
    let mut seen_edges: OrbitMap<()> = OrbitMap::over_cells(1);
    let mut isometries: OrbitMap<Isometry3<f64>> = OrbitMap::over_cells(2);

    let mut frontier: VecDeque<Dart> = VecDeque::new();
    let fixed_face = if cp.orientation[&fixed_face] {
      fixed_face
    } else {
      g.al(fixed_face, [0])
    };
    frontier.push_back(fixed_face);
    isometries.insert(g, fixed_face, Isometry3::identity());
    for vertex in g.one_dart_per_incident_cell(fixed_face, 0, 2) {
      let p_v = cp.vertices_coords.map()[&vertex];
      let p_v = Point3::new(p_v.x, p_v.y, 0.0);
      folded_coords.insert(g, vertex, p_v);
    }

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
            // check the old isometry matches the new one.
            let i = other_isometry.inv_mul(old_other_isometry);
            let angle = i.rotation.angle();
            let length = i.translation.vector.magnitude_squared();
            if angle.abs() > ISO_ANGLE_EPSILON || length > ISO_LENGTH_EPSILON {
              return Err(Error::DistinctIsometries(my_face));
            }
          } else {
            isometries.insert(g, other_face, other_isometry);
            for vertex in g.one_dart_per_incident_cell(other_face, 0, 2) {
              if !folded_coords.map().contains_key(&vertex) {
                let p_v = cp.vertices_coords.map()[&vertex];
                let p_v = Point3::new(p_v.x, p_v.y, 0.0);
                folded_coords.insert(g, vertex, other_isometry * p_v);
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

    Ok(Self {
      folded_coords,
      face_isometries: isometries,
    })
  }

  /// Check if any point of a face is in the plane specified by a point and a normal vector
  fn is_face_in_plane(
    &self,
    cp: &CreasePattern,
    face: Dart,
    normal: Vector3<f64>,
    plane_point: Point3<f64>,
  ) -> bool {
    let mut v = face;
    loop {
      if normal
        .dot(&(self.folded_coords.map()[&v] - plane_point))
        .abs()
        < PLANE_DISTANCE_EPSILON
      {
        break true;
      }
      v = cp.g.al(v, [0, 1]);
      if v == face {
        break false;
      }
    }
  }

  /// Find all edges of a face lying in the specified plane.
  fn edges_in_plane(
    &self,
    cp: &CreasePattern,
    face: Dart,
    normal: Vector3<f64>,
    plane_point: Point3<f64>,
  ) -> Vec<Dart> {
    let mut result = vec![];
    let face = if cp.orientation[&face] {
      face
    } else {
      cp.g.al(face, [0])
    };
    let mut v = face;
    let mut v_in = normal
      .dot(&(self.folded_coords.map()[&v] - plane_point))
      .abs()
      < PLANE_DISTANCE_EPSILON;
    loop {
      let v1 = cp.g.al(v, [0, 1]);
      let v1_in = normal
        .dot(&(self.folded_coords.map()[&v1] - plane_point))
        .abs()
        < PLANE_DISTANCE_EPSILON;
      if v_in && v1_in {
        result.push(v);
      }

      v = v1;
      v_in = v1_in;
      if v == face {
        break;
      }
    }
    result
  }

  /// Find the two edges where a face crosses the specified plane.
  fn face_plane_crossing(
    &self,
    cp: &CreasePattern,
    face: Dart,
    coords: &OrbitMap<Point3<f64>>,
    normal: Vector3<f64>,
    plane_point: Point3<f64>,
  ) -> Option<(Dart, Dart)> {
    let mut pos = None;
    let mut neg = None;

    let mut v = face;
    let mut d = normal.dot(&(coords.map()[&v] - plane_point));
    loop {
      let v1 = cp.g.al(v, [0, 1]);
      let d1 = normal.dot(&(coords.map()[&v1] - plane_point));
      if d < 0.0 && d1 >= 0.0 {
        pos = Some(v);
      }
      if d >= 0.0 && d1 < 0.0 {
        neg = Some(v);
      }

      v = v1;
      d = d1;
      if v == face {
        break;
      }
    }
    pos.zip(neg)
  }

  /// Given an edge and a plane, find the point where the edge crosses the plane
  fn edge_crossing_point(
    &self,
    cp: &CreasePattern,
    edge: Dart,
    coords: &OrbitMap<Point3<f64>>,
    normal: Vector3<f64>,
    plane_point: Point3<f64>,
  ) -> Point3<f64> {
    let p0 = coords.map()[&edge];
    let p1 = coords.map()[&cp.g.al(edge, [0])];
    let d0 = normal.dot(&(p0 - plane_point));
    let d1 = normal.dot(&(p1 - plane_point));
    let x = d1 / (d1 - d0);
    if x.is_nan() || !(0 <= x <= 1) {
      // should never happen because (d0, d1) should have opposite signs
      panic!("edge does not cross plane");
    }
    Point3::from(p1.coords.lerp(&p0.coords, x))
  }

  /// Shrink each face by an epsilon value.  Maps each angle to a location
  fn shrunk_faces_coords(&self, cp: &CreasePattern) -> OrbitMap<Point3<f64>> {
    let g = &cp.g;
    let mut shrunk_coords = OrbitMap::new(Alphas::ANGLE);
    for face in g.one_dart_per_cell(2) {
      let center = {
        let (p, n) = g
          .one_dart_per_incident_orbit(face, Alphas::VERTEX, Alphas::FACE)
          .fold((Vector3::zeros(), 0f64), |(p, n), d| {
            let p1 = self
              .folded_coords
              .map()
              .get(&d)
              .expect("missing vertex in layout")
              .coords;
            (p + p1, n + 1f64)
          });
        p / n
      };
      for d in g.one_dart_per_incident_orbit(face, Alphas::VERTEX, Alphas::FACE) {
        let p_old = self
          .folded_coords
          .map()
          .get(&d)
          .expect("missing vertex in layout")
          .coords;
        let p_new = Point3::from(p_old + (center - p_old).normalize() * FACE_SHRINK_EPSILON);
        shrunk_coords.insert(&g, d, p_new);
      }
    }
    shrunk_coords
  }

  fn check_polygon_intersections(&self, cp: &CreasePattern) -> Result<(), Error> {
    let g = &cp.g;

    // loop over all pairs of faces
    let faces: Vec<Dart> = g.one_dart_per_cell(2).collect();
    for face_index1 in 0..faces.len() {
      'faces: for face_index2 in (face_index1 + 1)..faces.len() {
        let face1 = faces[face_index1];
        let face2 = faces[face_index2];

        // skip faces which share a crease
        let face1_darts: HashSet<Dart> = g.cell(face1, 2).collect();
        for d in g.cell(face2, 2) {
          if face1_darts.contains(&g.al(d, [2])) {
            continue 'faces;
          }
        }

        let i1 = self.face_isometries.map()[&face1];
        let i2 = self.face_isometries.map()[&face2];
        let angle_diff = (i1.rotation / i2.rotation).angle();

        // normal vectors
        let n1 = i1.rotation.transform_vector(&Vector3::new(0.0, 0.0, 1.0));
        let n2 = i2.rotation.transform_vector(&Vector3::new(0.0, 0.0, 1.0));
        // choose arbitrary points of each face to define the planes
        let p1 = self.folded_coords.map()[&face1];
        let p2 = self.folded_coords.map()[&face2];

        // check if angle between planes is small
        if angle_diff.abs() < COPLANAR_ANGLE_EPSILON
          || (angle_diff - PI).abs() < COPLANAR_ANGLE_EPSILON
        {
          // The faces lie in near-parallel planes.  We need to check if they're coplanar.

          let coplanar =
            self.is_face_in_plane(cp, face1, n2, p2) && self.is_face_in_plane(cp, face2, n1, p1);
          if !coplanar {
            continue;
          }

          // track intersections of coplanar faces
          todo!("coplanar");
        } else {
          // check for intersections
          todo!("nonparallel");
        }
      }
    }
    todo!()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use format::tests::load_example;
  use gmap::Alphas;

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
    assert_eq!(cp.vertices_coords.indices(), Alphas::VERTEX);
    assert_eq!(cp.fold_angle.indices(), Alphas::EDGE);

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
    let (mut cp, ft) = CreasePattern::from(&f).unwrap();
    let FoldTracking {
      vertex_to_dart: vertices,
      edge_to_dart: edges,
      face_to_dart: faces,
    } = ft;

    let fs = FoldedState::from(&cp, faces[&0]).unwrap();
    assert_eq!(
      fs.folded_coords.map()[&vertices[&0]],
      Point3::new(0.0, 0.0, 0.0)
    );
    assert_eq!(
      fs.folded_coords.map()[&vertices[&1]],
      Point3::new(1.0, 0.0, 0.0)
    );
    assert_eq!(
      fs.folded_coords.map()[&vertices[&3]],
      Point3::new(0.0, 1.0, 0.0)
    );

    assert!(
      na::distance(
        &fs.folded_coords.map()[&vertices[&2]],
        &Point3::new(0.0, 0.0, 0.0)
      ) < 0.001
    );

    // now fold at a different angle
    cp.fold_angle.insert(&cp.g, edges[&4], -30.0);
    let fs = FoldedState::from(&cp, faces[&0]).unwrap();
    assert_eq!(
      fs.folded_coords.map()[&vertices[&0]],
      Point3::new(0.0, 0.0, 0.0)
    );
    assert_eq!(
      fs.folded_coords.map()[&vertices[&1]],
      Point3::new(1.0, 0.0, 0.0)
    );
    assert_eq!(
      fs.folded_coords.map()[&vertices[&3]],
      Point3::new(0.0, 1.0, 0.0)
    );

    assert!(
      na::distance(
        &fs.folded_coords.map()[&vertices[&2]],
        &Point3::new(0.933, 0.933, -0.353553391)
      ) < 0.001,
      "left = {}, right = {}",
      &fs.folded_coords.map()[&vertices[&2]],
      &Point3::new(0.933, 0.933, -0.353553391),
    );
  }

  #[test]
  fn parse_triangle_cp() {
    let f = load_example("triangle.fold");
    let (cp, ft) = CreasePattern::from(&f).unwrap();
    let nverts = cp.g.one_dart_per_cell(0).count();
    let nedges = cp.g.one_dart_per_cell(1).count();
    let nfaces = cp.g.one_dart_per_cell(2).count();
    assert_eq!(nverts, 5);
    assert_eq!(nedges, 8);
    assert_eq!(nfaces, 4);
  }

  #[test]
  fn fold_triangle_unchecked() {
    let f = load_example("triangle.fold");
    let (mut cp, ft) = CreasePattern::from(&f).unwrap();
    let FoldTracking {
      vertex_to_dart: vertices,
      edge_to_dart: edges,
      face_to_dart: faces,
    } = ft;

    let fs = FoldedState::from(&cp, faces[&0]).unwrap();
    for (i, &(x, y)) in [(0., 0.), (2., 2.), (4., 0.), (4., 0.), (0., 0.)]
      .iter()
      .enumerate()
    {
      let p1 = fs.folded_coords.map()[&vertices[&i]];
      let p2 = Point3::new(x, y, 0.);
      assert!(
        na::distance(&p1, &p2) < 0.001,
        "point: {}, left = {}, right = {}",
        i,
        p1,
        p2,
      );
    }

    // now fold at a different angle
    cp.fold_angle.insert(&cp.g, edges[&0], -90.0);
    match FoldedState::from(&cp, faces[&0]) {
      Err(Error::DistinctIsometries(_)) => (),
      Err(x) => panic!("wrong error {}", x),
      _ => panic!("ought to fail"),
    }

    cp.fold_angle.insert(&cp.g, edges[&6], 90.0);
    let fs = FoldedState::from(&cp, faces[&0]).unwrap();
    for (i, &(x, y, z)) in [
      (0., 0., 0.),
      (2., 2., 0.),
      (2., 2., -2.8284271),
      (4., 0., 0.),
      (0., 0., 0.),
    ]
    .iter()
    .enumerate()
    {
      let p1 = fs.folded_coords.map()[&vertices[&i]];
      let p2 = Point3::new(x, y, z);
      assert!(
        na::distance(&p1, &p2) < 0.001,
        "point: {}, left = {}, right = {}",
        i,
        p1,
        p2,
      );
    }
  }
}
