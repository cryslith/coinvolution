use std::collections::HashMap;

use gmap::{Alphas, Dart, GMap, OrbitReprs};

#[derive(Debug)]
enum Error {
  BadAngleConstraints,
  KawasakiViolation,
}

#[derive(PartialEq, Eq)]
enum Angle {
  /// 0 degrees
  Valley,
  /// 180 degrees
  Flat,
  /// 360 degrees
  Mountain,
}

type Length = u32;

enum Color {
  Red,
  Blue,
}

struct Problem {
  // planar graph with exterior face included.
  // must be oriented in the sense that the lower-numbered
  // dart in every angle points counterclockwise in its face.
  g: GMap,
  or: OrbitReprs,
  edge_lengths: HashMap<Dart, Length>,
  angle_constraints: HashMap<Dart, Angle>,
}

impl Problem {
  fn preprocess_angle_constraints(&mut self) -> Result<(), Error> {
    for vertex in self.g.one_dart_per_cell(0) {
      let angles: Vec<Dart> = self
        .or
        .orbit_repr_per_incident_orbit(&self.g, vertex, Alphas::ANGLE, Alphas::VERTEX)
        .collect();
      let flats = angles
        .iter()
        .filter(|d| self.angle_constraints.get(d) == Some(&Angle::Flat))
        .count();
      let mountains = angles
        .iter()
        .filter(|d| self.angle_constraints.get(d) == Some(&Angle::Mountain))
        .count();
      if (flats != 0 && flats != 2) || mountains > 1 || (flats == 2 && mountains == 1) {
        return Err(Error::BadAngleConstraints);
      }
      if flats == 2 || mountains == 1 {
        for &a in &angles {
          if !self.angle_constraints.contains_key(&a) {
            self.angle_constraints.insert(a, Angle::Flat);
          }
        }
      }
    }
    Ok(())
  }

  /// compute the constraint on angles around the vertex
  fn vertex_constraint(
    &mut self,
    vertex: Dart,
    cg: &mut GMap,
    clause_sizes: &mut HashMap<Dart, usize>,
    clause_colors: &mut HashMap<Dart, Color>,
  ) -> Result<(), Error> {
    let angles =
      self
        .or
        .orbit_repr_per_incident_orbit(&self.g, vertex, Alphas::ANGLE, Alphas::VERTEX);
    let nonflat: Vec<Dart> = angles
      .filter(|d| self.angle_constraints.get(d) != Some(&Angle::Flat))
      .collect();
    if nonflat.len() == 0 {
      return Ok(());
    }
    let cv = add_vertex(cg, nonflat.len());
    clause_sizes.insert(cv, 1);
    clause_colors.insert(cv, Color::Blue);
    Ok(())
  }

  /// compute the constraints and additional variables on the faces,
  /// producing the planar constraint graph
  fn face_constraints(&self) -> Result<(), Error> {
    todo!()
  }

  fn process_face(
    &self,
    face: Dart,
    angle_to_cg: HashMap<Dart, Dart>,
    cg: &mut GMap,
  ) -> Result<(), Error> {
    let g = &self.g;

    // map from cg counterclockwise darts to tracking information:
    // prev, next, clockwise edge length
    let mut tracking: HashMap<Dart, (Dart, Dart, Length)> = HashMap::new();
    let mut nonflat: Vec<Dart> = vec![];
    let mut a = face;
    loop {
      if self.angle_constraints.get(&a) != Some(&Angle::Flat) {
        nonflat.push(a);
      }
      a = g.al(a, [1, 0]);
      if a == face {
        break;
      }
    }
    if nonflat.len() == 0 || nonflat.len() % 2 != 0 {
      return Err(Error::BadAngleConstraints);
    }

    for (i, a) in nonflat.iter().enumerate() {
      let cga = angle_to_cg[&a];
      let length = self.edge_lengths[&a];
      let prev_index = if i == 0 { nonflat.len() - 1 } else { i - 1 };
      let next_index = if i == nonflat.len() - 1 { 0 } else { i + 1 };
      tracking.insert(cga, (nonflat[prev_index], nonflat[next_index], length));
    }

    todo!()
  }
}

fn add_vertex(g: &mut GMap, n: usize) -> Dart {
  if n < 1 {
    panic!("vertex must have at least one edge");
  }
  let start = g.add_dart();
  let mut prev = g.add_dart();
  g.sew(1, start, prev).unwrap();
  for _ in 0..(n - 1) {
    let d0 = g.add_dart();
    let d1 = g.add_dart();
    g.sew(1, d0, d1).unwrap();
    g.sew(2, d0, prev).unwrap();
    prev = d1;
  }
  g.sew(2, start, prev).unwrap();
  start
}
