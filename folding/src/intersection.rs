use crate::{FACE_SHRINK_EPSILON, PLANE_DISTANCE_EPSILON};

use gmap::{Alphas, Dart, GMap, OrbitMap};
use na::{Isometry3, Point2, Point3, Vector3};
use parry2d_f64::transformation::convex_polygons_intersection_points;

/// Shrink each face by an epsilon value.  Maps each angle to a location.
pub(crate) fn shrunk_faces_coords(
  g: &GMap,
  coords: &OrbitMap<Point3<f64>>,
) -> OrbitMap<Point3<f64>> {
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

/// Given nonparallel faces (and normal vectors), do they intersect?
pub(crate) fn do_faces_intersect(
  g: &GMap,
  coords: &OrbitMap<Point3<f64>>,
  face1: Dart,
  normal1: Vector3<f64>,
  face2: Dart,
  normal2: Vector3<f64>,
) -> bool {
  let p1 = coords.map()[&face1];
  let p2 = coords.map()[&face2];
  let (ea1, eb1) = if let Some(crossings) = face_plane_crossing(g, coords, face1, normal2, p2) {
    crossings
  } else {
    return false;
  };
  let (ea2, eb2) = if let Some(crossings) = face_plane_crossing(g, coords, face2, normal1, p1) {
    crossings
  } else {
    return false;
  };
  let pa1 = edge_crossing_point(g, coords, ea1, normal2, p2);
  let pb1 = edge_crossing_point(g, coords, eb1, normal2, p2);
  let pa2 = edge_crossing_point(g, coords, ea2, normal1, p1);
  let pb2 = edge_crossing_point(g, coords, eb2, normal1, p1);

  let proj = normal1.cross(&normal2);
  let a1 = proj.dot(&pa1.coords);
  let a2 = proj.dot(&pa2.coords);
  let b1 = proj.dot(&pb1.coords);
  let b2 = proj.dot(&pb2.coords);

  let c1 = a1 < b1;
  let c2 = a1 < b2;
  let c3 = a2 < b1;
  let c4 = a2 < b2;

  !(c1 == c2 && c2 == c3 && c3 == c4)
}

/// Compute the intersection of two parallel faces.  i must map both faces to a z-plane.
pub fn face_overlap(
  g: &GMap,
  coords: &OrbitMap<Point3<f64>>,
  face1: Dart,
  i: &Isometry3<f64>,
  face2: Dart,
) -> Option<Vec<Point2<f64>>> {
  let points1: Vec<Point2<f64>> = g
    .cycle(face1, &[0, 1])
    .map(|v| {
      let p = i * coords.map()[&v];
      Point2::new(p.x, p.y)
    })
    .collect();
  let points2: Vec<Point2<f64>> = g
    .cycle(face2, &[0, 1])
    .map(|v| {
      let p = i * coords.map()[&v];
      Point2::new(p.x, p.y)
    })
    .collect();
  let mut output = vec![];
  convex_polygons_intersection_points(&points1[..], &points2[..], &mut output);
  if output.is_empty() {
    None
  } else {
    Some(output)
  }
}
