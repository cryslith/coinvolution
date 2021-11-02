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
pub fn count_darts(p: &PuzzleState) -> usize {
  trace(p.g.alpha().len())
}

#[wasm_bindgen]
pub fn make_face_clickers(state: &JsValue, p: &PuzzleState) {
  let g = &p.g;
  for face in g.one_dart_per_cell(2, None) {
    let mut vertex_locations = vec![];
    let mut v = face;
    loop {
      let &(x, y) = p.layout.map().get(&v).expect("missing vertex in layout");
      vertex_locations.push(x);
      vertex_locations.push(y);
      v = g.al(v, [0, 1]);
      if v == face {
        break;
      }
    }

    make_face_clicker(state, face, &vertex_locations[..]);
  }
}

fn center(p: &PuzzleState, d: usize, i: usize) -> (f64, f64) {
  let ((x, y), n) =
    p.g
      .one_dart_per_incident_cell(d, 0, i, None)
      .fold(((0f64, 0f64), 0f64), |((x, y), n), d| {
        let &(x1, y1) = p.layout.map().get(&d).expect("missing vertex in layout");
        ((x + x1, y + y1), n + 1f64)
      });
  (x / n, y / n)
}

#[wasm_bindgen(raw_module = "../www/graph.js")]
extern "C" {
  fn make_face_clicker(state: &JsValue, face: usize, vertex_locations: &[f64]);
}

#[wasm_bindgen]
pub fn on_face_click(p: &PuzzleState, face: usize) {
  log!("clicked on face {}", face);
}
