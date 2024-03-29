mod circular;

use circular::{Circular, Data, Node};

use std::collections::HashMap;

use gmap::{Alphas, Dart, GMap, OrbitReprs};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
  #[error("Given angle constraints are inconsistent")]
  BadAngleConstraints,
  #[error("Violation of Kawasaki-Justin theorem")]
  KawasakiViolation,
  #[error("Graph must be planar")]
  Nonplanar,
  #[error(transparent)]
  GMap(#[from] gmap::GMapError),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Angle {
  /// 0 degrees
  Valley,
  /// 180 degrees
  Flat,
  /// 360 degrees
  Mountain,
}

pub type Length = i32;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
  Red,
  Blue,
}

#[derive(Debug)]
pub struct Problem {
  // planar graph with exterior face included.
  // must be oriented in the sense that the lower-numbered
  // dart in every angle points counterclockwise in its face.
  g: GMap,
  or: OrbitReprs,
  edge_lengths: HashMap<Dart, Length>,
  angle_constraints: HashMap<Dart, Angle>,
  exterior_face: Dart,
}

#[derive(Debug)]
pub struct Constraints {
  cg: GMap,
  clause_sizes: HashMap<Dart, usize>,
  clause_colors: HashMap<Dart, Color>,
  /// correspondence map from Problem.g angles to cg darts
  angle_to_cg: HashMap<Dart, Dart>,
}

impl Problem {
  pub fn with_exterior(
    g: GMap,
    edge_lengths: HashMap<Dart, Length>,
    angle_constraints: HashMap<Dart, Angle>,
    exterior_face: Dart,
  ) -> Result<Self, Error> {
    if g.dimension() != 2 {
      // should implement more robust planarity checking
      return Err(Error::Nonplanar);
    }
    let mut or = OrbitReprs::new();
    or.ensure_all(&g, Alphas::ANGLE);
    or.ensure_all(&g, Alphas::EDGE);
    let mut x = Self {
      g,
      or,
      edge_lengths,
      angle_constraints,
      exterior_face,
    };
    x.preprocess_angle_constraints()?;
    Ok(x)
  }

  pub fn g(&self) -> &GMap {
    &self.g
  }

  pub fn edge_lengths(&self) -> &HashMap<Dart, Length> {
    &self.edge_lengths
  }

  pub fn angle_constraints(&self) -> &HashMap<Dart, Angle> {
    &self.angle_constraints
  }

  pub fn exterior_face(&self) -> Dart {
    self.exterior_face
  }

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

  /// compute the constraint graph
  pub fn constraint_graph(&self) -> Result<Constraints, Error> {
    let mut constraints = Constraints {
      cg: GMap::empty(2)?,
      clause_sizes: HashMap::new(),
      clause_colors: HashMap::new(),
      angle_to_cg: HashMap::new(),
    };
    for vertex in self.g.one_dart_per_cell(0) {
      self.process_vertex(vertex, &mut constraints)?;
    }
    for face in self.g.one_dart_per_cell(2) {
      self.process_face(face, &mut constraints, face == self.exterior_face)?;
    }
    Ok(constraints)
  }

  /// compute the constraint on angles around the vertex
  fn process_vertex(&self, vertex: Dart, constraints: &mut Constraints) -> Result<(), Error> {
    let Constraints {
      cg,
      clause_sizes,
      clause_colors,
      angle_to_cg,
    } = constraints;

    let mut nonflat: Vec<Dart> = vec![];
    let mut a = vertex;
    loop {
      if self.angle_constraints.get(&a) != Some(&Angle::Flat) {
        nonflat.push(a);
      }
      // counterclockwise
      a = self.g.al(a, [2, 1]);
      if a == vertex {
        break;
      }
    }

    if nonflat.len() == 0 {
      return Ok(());
    }
    let mut cv = add_vertex(cg, nonflat.len());
    clause_sizes.insert(cv, 1);
    clause_colors.insert(cv, Color::Blue);
    for a in nonflat {
      angle_to_cg.insert(a, cv);
      // counterclockwise
      cv = cg.al(cv, [2, 1]);
    }
    Ok(())
  }

  fn process_face(
    &self,
    face: Dart,
    constraints: &mut Constraints,
    exterior: bool,
  ) -> Result<(), Error> {
    // XXX during this function we need to account for already-constrained angles

    let Constraints {
      cg,
      clause_sizes,
      clause_colors,
      angle_to_cg,
    } = constraints;

    // circular list in counterclockwise order of (cg counterclockwise dart for angle, length of edge immediately counterclockwise)
    let mut tracking: Circular<(Dart, Length)> = Circular::new();
    let mut prev_node = None;
    for &(a, length) in self.nonflat_lengths(face)?.iter() {
      let cga = angle_to_cg[&a];
      let new_node = tracking.add_node((cga, length));
      if let Some(prev_node) = prev_node {
        tracking.splice(prev_node, new_node);
      }
      prev_node = Some(new_node);
    }
    let mut head = prev_node.unwrap();

    self.check_kawasaki(&tracking, head)?;

    // during this loop make sure head is any pointer into the correct list
    loop {
      #[allow(non_snake_case)]
      let (start, end, n_S, length) = if let Some(x) = find_enclosed_edge_sequence(&tracking, head)
      {
        x
      } else {
        // All remaining edges have the same length
        break;
      };
      if n_S % 2 == 0 {
        // |S| is even

        let start_prev = tracking.split(start, end);
        // add new clause
        let mountains = n_S / 2;
        let even_clause = add_vertex(cg, n_S);
        clause_sizes.insert(even_clause, mountains);
        clause_colors.insert(even_clause, Color::Red);
        glue_clause(cg, &tracking, start, even_clause);
        // resize edge
        let (_, length_a) = tracking[start_prev].data;
        let (_, length_b) = tracking[end].data;
        let new_length = length_a - length + length_b;
        tracking.mut_data(start_prev).1 = new_length;

        // make sure head remains valid
        head = start_prev;
      } else {
        // |S| is odd

        let start_prev = tracking.split(start, end);
        // add first new clause
        let mountains = (n_S + 1) / 2;
        let odd_clause = add_vertex(cg, n_S + 1);
        clause_sizes.insert(odd_clause, mountains);
        clause_colors.insert(odd_clause, Color::Red);
        glue_clause(cg, &tracking, start, odd_clause);
        // add second new clause
        let odd_clause_2 = add_vertex(cg, 2);
        clause_sizes.insert(odd_clause_2, 1);
        clause_colors.insert(odd_clause_2, Color::Blue);
        cg.sew(0, cg[(odd_clause, 1)], odd_clause_2).unwrap();

        // replace old angles with new angle
        let (_, end_length) = tracking[end].data;
        let new_node = tracking.add_node((cg.al(odd_clause_2, [2, 1]), end_length));
        tracking.splice(start_prev, new_node);

        // make sure head remains valid
        head = start_prev;
      }
    }

    let n_remaining = tracking.iter(head).count();
    let mountains = if exterior { n_remaining / 2 + 1 } else { n_remaining / 2 - 1};
    let equal_clause = add_vertex(cg, n_remaining);
    clause_sizes.insert(equal_clause, mountains);
    clause_colors.insert(equal_clause, Color::Red);
    glue_clause(cg, &tracking, head, equal_clause);
    Ok(())
  }

  /// Returns a counterclockwise-ordered vector of (angle, flat length immediately counterclockwise)
  fn nonflat_lengths(&self, face: Dart) -> Result<Vec<(Dart, Length)>, Error> {
    let g = &self.g;
    let mut nonflat: Vec<(Dart, Length)> = vec![];
    let mut a = face;
    // find nonflat angle to start at
    loop {
      if self.angle_constraints.get(&a) != Some(&Angle::Flat) {
        break;
      }
      a = g.al(a, [1, 0]);
      if a == face {
        // no nonflat angles
        return Err(Error::BadAngleConstraints);
      }
    }
    let start = a;
    loop {
      let current_nonflat = a;
      let mut length_counter = 0;
      a = g.al(a, [1, 0]);
      loop {
        length_counter += self.edge_lengths[&self.or[(Alphas::EDGE, a)]];
        if self.angle_constraints.get(&a) != Some(&Angle::Flat) {
          break;
        }
        a = g.al(a, [1, 0]);
      }
      nonflat.push((current_nonflat, length_counter));
      if a == start {
        break;
      }
    }
    if nonflat.len() % 2 != 0 {
      return Err(Error::BadAngleConstraints);
    }
    Ok(nonflat)
  }

  /// Verify the Kawasaki-Justin theorem holds for the provided lengths
  fn check_kawasaki(&self, tracking: &Circular<(Dart, Length)>, head: Node) -> Result<(), Error> {
    let mut length_total = 0;
    let mut b = true;
    for n in tracking.iter(head) {
      let Data {
        data: (_, length), ..
      } = tracking[n];
      if b {
        length_total += length;
      } else {
        length_total -= length;
      }
      b = !b;
    }
    if !b {
      // odd number of edges
      return Err(Error::KawasakiViolation);
    }
    if length_total == 0 {
      Ok(())
    } else {
      Err(Error::KawasakiViolation)
    }
  }
}

fn add_vertex(g: &mut GMap, n: usize) -> Dart {
  g.add_cycle(1, 2, n)
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
  let mut length = loop {
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
    if l < length {
      // start over
      start = end;
      length = l;
      n = 1;
    }
    end = next;
    n += 1;
  }
  return Some((start, end, n, length));
}

fn glue_clause(cg: &mut GMap, tracking: &Circular<(Dart, Length)>, head: Node, clause: Dart) {
  let mut clause_edge = clause;
  for n in tracking.iter(head) {
    let Data {
      data: (angle_edge, _),
      ..
    } = tracking[n];
    // sew the two halves of the edges together
    cg.sew(0, cg[(angle_edge, 2)], clause_edge).unwrap();
    // move around the clause counterclockwise
    clause_edge = cg.al(clause_edge, [2, 1]);
  }
}

impl Constraints {
  pub fn cg(&self) -> &GMap {
    &self.cg
  }

  pub fn clause_sizes(&self) -> &HashMap<Dart, usize> {
    &self.clause_sizes
  }

  pub fn clause_colors(&self) -> &HashMap<Dart, Color> {
    &self.clause_colors
  }

  pub fn angle_to_cg(&self) -> &HashMap<Dart, Dart> {
    &self.angle_to_cg
  }
}

pub mod examples {
  use super::Problem;

  use std::collections::HashMap;

  use gmap::{Alphas, Dart, GMap, OrbitReprs, grids::square};

  // start should be a counterclockwise dart
  pub fn wrap_exterior(g: &mut GMap, start: Dart) -> Dart {
    let mut interior_boundary = vec![];
    // start with a clockwise dart
    let start = g[(start, 0)];
    let mut d = start;
    loop {
      // traverse clockwise
      interior_boundary.push(d);
      d = if let Some(n) = g.cell(d, 0).find(|&n| n != d && g.is_free(n, 2)) {
        n
      } else {
        break;
      };
      d = g[(d, 0)];
      if d == start {
        break;
      }
    }
    let ext = g.add_polygon(interior_boundary.len());
    let mut d2 = ext;
    for d in interior_boundary {
      g.sew(2, d, d2).unwrap();
      // clockwise for the interior is counterclockwise around the exterior
      d2 = g.al(d2, [1, 0]);
    }
    ext
  }

  // TODO abstract single-polygon examples into a single function

  pub fn kite() -> Problem {
    let mut g = GMap::empty(2).unwrap();
    let mut edge_lengths = HashMap::new();
    let angle_constraints = HashMap::new();

    let f = g.add_polygon(4);
    let ext = wrap_exterior(&mut g, f);

    let mut or = OrbitReprs::new();
    or.ensure_all(&g, Alphas::EDGE);

    for (edge, length) in [
      (f, 1),
      (g.al(f, [1, 0]), 1),
      (g.al(f, [1, 0, 1, 0]), 2),
      (g.al(f, [1, 0, 1, 0, 1, 0]), 2),
    ] {
      edge_lengths.insert(or[(Alphas::EDGE, edge)], length);
    }

    Problem::with_exterior(g, edge_lengths, angle_constraints, ext).unwrap()
  }

  pub fn trapezoid() -> Problem {
    let mut g = GMap::empty(2).unwrap();
    let mut edge_lengths = HashMap::new();
    let angle_constraints = HashMap::new();

    let f = g.add_polygon(4);
    let ext = wrap_exterior(&mut g, f);

    let mut or = OrbitReprs::new();
    or.ensure_all(&g, Alphas::EDGE);

    for (edge, length) in [
      (f, 1),
      (g.al(f, [1, 0]), 2),
      (g.al(f, [1, 0, 1, 0]), 3),
      (g.al(f, [1, 0, 1, 0, 1, 0]), 2),
    ] {
      edge_lengths.insert(or[(Alphas::EDGE, edge)], length);
    }

    Problem::with_exterior(g, edge_lengths, angle_constraints, ext).unwrap()
  }

  pub fn square_grid(n: usize) -> Problem {
    let (mut g, squares) = square::new(n, n);
    let mut edge_lengths = HashMap::new();
    let angle_constraints = HashMap::new();

    let ext = wrap_exterior(&mut g, squares[0][0]);

    // let mut or = OrbitReprs::new();
    // or.ensure_all(&g, Alphas::EDGE);
    for edge in g.one_dart_per_cell(1) {
      edge_lengths.insert(edge, 1);
    }

    Problem::with_exterior(g, edge_lengths, angle_constraints, ext).unwrap()
  }

  pub fn big_kite(n: usize) -> Problem {
    let mut g = GMap::empty(2).unwrap();
    let mut edge_lengths = HashMap::new();
    let angle_constraints = HashMap::new();

    let f = g.add_polygon(2 * n);
    let ext = wrap_exterior(&mut g, f);

    let mut or = OrbitReprs::new();
    or.ensure_all(&g, Alphas::EDGE);

    let mut d = f;
    for i in 0..(2*n) {
      let length = if i + 1 <= n {
        i + 1
      } else {
        2*n - i
      } as i32;
      edge_lengths.insert(or[(Alphas::EDGE, d)], length);
      
      d = g.al(d, [1, 0]);
    }

    Problem::with_exterior(g, edge_lengths, angle_constraints, ext).unwrap()
  }

  pub fn regular(n: usize) -> Problem {
    let mut g = GMap::empty(2).unwrap();
    let mut edge_lengths = HashMap::new();
    let angle_constraints = HashMap::new();

    let f = g.add_polygon(2 * n);
    let ext = wrap_exterior(&mut g, f);

    let mut or = OrbitReprs::new();
    or.ensure_all(&g, Alphas::EDGE);

    let mut d = f;
    for _ in 0..(2*n) {
      edge_lengths.insert(or[(Alphas::EDGE, d)], 1);
      
      d = g.al(d, [1, 0]);
    }

    Problem::with_exterior(g, edge_lengths, angle_constraints, ext).unwrap()
  }

  pub fn big_arc(n: usize) -> Problem {
    let mut g = GMap::empty(2).unwrap();
    let mut edge_lengths = HashMap::new();
    let angle_constraints = HashMap::new();

    let f = g.add_polygon(2 * n);
    let ext = wrap_exterior(&mut g, f);

    let mut or = OrbitReprs::new();
    or.ensure_all(&g, Alphas::EDGE);

    let mut d = f;
    for i in 0..(2*n) {
      let length = if i + 1 <= 2*n - 1 {
        i + 1
      } else {
        n
      } as i32;
      edge_lengths.insert(or[(Alphas::EDGE, d)], length);
      
      d = g.al(d, [1, 0]);
    }

    Problem::with_exterior(g, edge_lengths, angle_constraints, ext).unwrap()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  pub fn fold_kite() {
    let problem = examples::kite();
    let constraints = problem.constraint_graph().unwrap();
  }

  #[test]
  pub fn fold_trapezoid() {
    let problem = examples::trapezoid();
    let constraints = problem.constraint_graph().unwrap();
  }

  #[test]
  pub fn fold_grid() {
    let problem = examples::square_grid(4);
    let constraints = problem.constraint_graph().unwrap();
  }

  #[test]
  pub fn fold_big_kite() {
    let problem = examples::big_kite(4);
    let constraints = problem.constraint_graph().unwrap();
  }

  #[test]
  pub fn fold_regular() {
    let problem = examples::regular(4);
    let constraints = problem.constraint_graph().unwrap();
  }
  #[test]
  pub fn fold_big_arc() {
    let problem = examples::big_arc(4);
    let constraints = problem.constraint_graph().unwrap();
  }
}
