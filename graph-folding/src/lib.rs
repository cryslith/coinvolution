mod circular;

use std::collections::HashMap;

use circular::{Circular, Node, Data};
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

type Length = i32;

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

    // circular list in counterclockwise order of (cg counterclockwise dart for angle, length of edge immediately counterclockwise)
    let mut tracking: Circular<(Dart, Length)> = Circular::new();
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

    let mut prev_node = None;
    for (i, a) in nonflat.iter().enumerate() {
      let cga = angle_to_cg[&a];
      let length = todo!("compute length in counterclockwise direction");
      let new_node = tracking.add_node((cga, length));
      if let Some(prev_node) = prev_node {
        tracking.splice(prev_node, new_node);
      }
      prev_node = Some(new_node);
    }
    let head = prev_node.unwrap();

    todo!("verify kawasaki's theorem here to avoid any issues later");

    loop {
      let (start, end, n, length) = if let Some(x) = find_enclosed_edge_sequence(&tracking, head) {
        x
      } else {
        // all edges are the same length
        break;
      };
      if n % 2 == 0 {
        // |S| is even
      } else {
        // |S| is odd
      }
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

// start, end, number of angles in S, length of enclosed edges.
// note that the "end" node will have a larger length (as will start.prev)
fn find_enclosed_edge_sequence(
  tracking: &Circular<(Dart, Length)>,
  head: Node,
) -> Option<(Node, Node, usize, Length)> {
  // this is a theoretically inefficient implementation,
  // but it's more practical than the one in the paper for now.
  // a middle ground would be to track information about contiguous
  // runs of edges on each edge (rather than constructing a new linked list).
  // this information would also include whether the run has larger or smaller edges than the adjacent runs.

  let mut prev_length = None;
  let mut start = head;
  // find where the length decreases
  let length = loop {
    let Data {
      next, data: (_, l), ..
    } = tracking[start];
    if let Some(prev_length) = prev_length {
      if l < prev_length {
        break l;
      }
    }
    if start == head && prev_length.is_some() {
      // all edges have the same length
      return None;
    }
    prev_length = Some(l);
    start = next;
  };
  // find where length increases
  let mut end = start;
  let mut n = 1;
  loop {
    let Data {
      next, data: (_, l), ..
    } = tracking[end];
    if l > length {
      break;
    }
    end = next;
    n += 1;
  }
  return Some((start, end, n, length));
}
