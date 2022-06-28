use crate::{Alphas, Dart, GMap, OrbitMap};

use itertools::Itertools;

/// See https://www.redblobgames.com/grids/hexagons/#coordinates-axial.
/// This returns hexes with (0 <= r < n, 0 <= q < m)
pub fn new(n: usize, m: usize) -> (GMap, Vec<Vec<Dart>>) {
  let mut g = GMap::empty(2).unwrap();
  let rows: Vec<Vec<Dart>> = (0..n)
    .map(|_| (0..m).map(|_| g.add_polygon(6)).collect())
    .collect();
  // Each hex is the dart on the hex's northeast edge, north vertex
  for r in &rows {
    for (&s0, &s1) in r.iter().tuple_windows() {
      g.sew(2, g.al(s0, [0, 1]), g.al(s1, [1, 0, 1])).unwrap();
    }
  }

  for (r0, r1) in rows.iter().tuple_windows() {
    for (&s0, &s1) in r0.iter().zip(r1.iter()) {
      g.sew(2, g.al(s0, [0, 1, 0, 1]), g.al(s1, [1])).unwrap();
    }
    for (&s0, &s1) in r0.iter().skip(1).zip(r1.iter()) {
      g.sew(2, g.al(s0, [1, 0, 1, 0, 1]), s1).unwrap();
    }
  }

  (g, rows)
}

/// Returns coordinates along basis vectors (a, b) where a + b = (0, 1),
/// 2a - b = (1, 0)
/// That is, a and b are 15 degrees rotated from the r- and q- axes,
/// and their length is the distance from the center of a hex to a vertex
pub fn vertex_coords(g: &GMap, rows: &[Vec<Dart>]) -> OrbitMap<(isize, isize)> {
  let mut coords = OrbitMap::new(Alphas::VERTEX);
  for (r, row) in rows.iter().enumerate() {
    for (q, &h) in row.iter().enumerate() {
      let r = r as isize;
      let q = q as isize;
      let a = r + 2 * q;
      let b = r - q;
      coords.insert(&g, h, (a, b - 1));
      coords.insert(&g, g.al(h, [0]), (a + 1, b - 1));
      coords.insert(&g, g.al(h, [0, 1, 0]), (a + 1, b));
      coords.insert(&g, g.al(h, [0, 1, 0, 1, 0]), (a, b + 1));
      coords.insert(&g, g.al(h, [0, 1, 0, 1, 0, 1, 0]), (a - 1, b + 1));
      coords.insert(&g, g.al(h, [0, 1, 0, 1, 0, 1, 0, 1, 0]), (a - 1, b));
    }
  }
  coords
}
