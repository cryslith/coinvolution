#[macro_use]
pub mod utils;

use gmap::{grid, GMap, OrbitMap};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct PuzzleState {
  g: GMap,
  layout: OrbitMap<(f64, f64)>, // positions of every vertex
}

fn trace<T>(x: T) -> T
where
  T: std::fmt::Debug,
{
  log!("[rust] {:?}", x);
  x
}

#[wasm_bindgen]
pub fn initialize_puzzle() -> PuzzleState {
  let (g, squares) = grid::new(10, 10);
  let mut layout = OrbitMap::over_cells(0, 2);
  for (i, row) in grid::vertex_grid(&g, &squares).iter().enumerate() {
    for (j, &v) in row.iter().enumerate() {
      layout.insert(&g, v, (j as f64, i as f64))
    }
  }

  PuzzleState { g, layout }
}

#[wasm_bindgen]
pub fn count_darts(g: &PuzzleState) -> usize {
  trace(g.g.alpha().len())
}
