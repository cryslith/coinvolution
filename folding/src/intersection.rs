use crate::{FACE_SHRINK_EPSILON, PLANE_DISTANCE_EPSILON};

use gmap::{Alphas, Dart, GMap, OrbitMap};
use na::{Point3, Vector3};

/// Shrink each face by an epsilon value.  Maps each angle to a location.
fn shrunk_faces_coords(g: &GMap, coords: &OrbitMap<Point3<f64>>) -> OrbitMap<Point3<f64>> {
  let mut shrunk_coords = OrbitMap::new(Alphas::ANGLE);
  for face in g.one_dart_per_cell(2) {
    let center = {
      let (p, n) = g
        .one_dart_per_incident_orbit(face, Alphas::VERTEX, Alphas::FACE)
        .fold((Vector3::zeros(), 0f64), |(p, n), d| {
          let p1 = coords
            .map()
            .get(&d)
            .expect("missing vertex in layout")
            .coords;
          (p + p1, n + 1f64)
        });
      p / n
    };
    for d in g.one_dart_per_incident_orbit(face, Alphas::VERTEX, Alphas::FACE) {
      let p_old = coords
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

/// Check if any point of a face is in the plane specified by a point and a normal vector
pub(crate) fn is_face_in_plane(
  g: &GMap,
  coords: &OrbitMap<Point3<f64>>,
  face: Dart,
  normal: Vector3<f64>,
  plane_point: Point3<f64>,
) -> bool {
  let mut v = face;
  loop {
    if normal.dot(&(coords.map()[&v] - plane_point)).abs() < PLANE_DISTANCE_EPSILON {
      break true;
    }
    v = g.al(v, [0, 1]);
    if v == face {
      break false;
    }
  }
}

/// Find all edges of a face lying in the specified plane.
fn edges_in_plane(
  g: &GMap,
  coords: &OrbitMap<Point3<f64>>,
  face: Dart,
  normal: Vector3<f64>,
  plane_point: Point3<f64>,
) -> Vec<Dart> {
  let mut result = vec![];
  let mut v = face;
  let mut v_in = normal.dot(&(coords.map()[&v] - plane_point)).abs() < PLANE_DISTANCE_EPSILON;
  loop {
    let v1 = g.al(v, [0, 1]);
    let v1_in = normal.dot(&(coords.map()[&v1] - plane_point)).abs() < PLANE_DISTANCE_EPSILON;
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
  g: &GMap,
  coords: &OrbitMap<Point3<f64>>,
  face: Dart,
  normal: Vector3<f64>,
  plane_point: Point3<f64>,
) -> Option<(Dart, Dart)> {
  let mut pos = None;
  let mut neg = None;

  let mut v = face;
  let mut d = normal.dot(&(coords.map()[&v] - plane_point));
  loop {
    let v1 = g.al(v, [0, 1]);
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
  g: &GMap,
  coords: &OrbitMap<Point3<f64>>,
  edge: Dart,
  normal: Vector3<f64>,
  plane_point: Point3<f64>,
) -> Point3<f64> {
  let p0 = coords.map()[&edge];
  let p1 = coords.map()[&g.al(edge, [0])];
  let d0 = normal.dot(&(p0 - plane_point));
  let d1 = normal.dot(&(p1 - plane_point));
  let x = d1 / (d1 - d0);
  if x.is_nan() || !(0f64 <= x && x <= 1f64) {
    // should never happen because (d0, d1) should have opposite signs
    panic!("edge does not cross plane");
  }
  Point3::from(p1.coords.lerp(&p0.coords, x))
}
