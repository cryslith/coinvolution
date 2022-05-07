use std::collections::HashMap;

use gmap::{Dart, GMap, OrbitReprs};

#[derive(Debug)]
enum Error {
  BadFlatConstraints,
  KawasakiViolation,
}

enum Angle {
  /// 0 degrees
  Valley,
  /// 180 degrees
  Flat,
  /// 360 degrees
  Mountain,
}

struct Problem {
  // planar graph with exterior face included
  g: GMap,
  or: OrbitReprs,
  edge_lengths: HashMap<Dart, u32>,
  angle_constraints: HashMap<Dart, Angle>,
}

impl Problem {
  fn preprocess(&mut self) -> Result<(), Error> {
    // for every vertex, inspect the angle constraints around that vertex

    // for every face, compute kawasaki's theorem around that face
    todo!()
  }
}
