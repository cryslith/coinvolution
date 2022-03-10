use crate::{Dart, GMap};

use itertools::Itertools;

pub fn new(n: usize, m: usize) -> (GMap, Vec<Vec<Dart>>) {
  let mut g = GMap::empty(2).unwrap();
  let rows: Vec<Vec<Dart>> = (0..n)
    .map(|_| (0..m).map(|_| g.add_polygon(4)).collect())
    .collect();
  // Each square is the dart on the square's north edge, northwest vertex
  for r in &rows {
    for (&s0, &s1) in r.iter().tuple_windows() {
      g.sew(2, g.al(s0, [0, 1]), g.al(s1, [1])).unwrap();
    }
  }

  for (r0, r1) in rows.iter().tuple_windows() {
    for (&s0, &s1) in r0.iter().zip(r1.iter()) {
      g.sew(2, g.al(s0, [1, 0, 1]), s1).unwrap();
    }
  }

  (g, rows)
}

pub fn vertex_grid(g: &GMap, squares: &[Vec<Dart>]) -> Vec<Vec<Dart>> {
  squares
    .iter()
    .map(|row| {
      row
        .iter()
        .cloned()
        .chain(row.last().map(|&d| g.al(d, [0])))
        .collect()
    })
    .chain(squares.last().map(|row| {
      row
        .iter()
        .map(|&d| g.al(d, [1, 0, 1]))
        .chain(row.last().map(|&d| g.al(d, [1, 0, 1, 0])))
        .collect()
    }))
    .collect()
}
