#[macro_use]
pub mod utils;
pub mod svg;

use std::cell::RefCell;
use std::rc::Rc;

use gmap::{grid, GMap, OrbitMap};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct State {
  p: Rc<RefCell<Puzzle>>,
}

#[wasm_bindgen]
impl State {
  #[wasm_bindgen(constructor)]
  pub fn new() -> Self {
    Self {
      p: Rc::new(RefCell::new(Puzzle::new())),
    }
  }

  pub fn display(&self, svg_: JsValue) {
    let svg_ = svg::SVG(svg_);
    let p = &self.p.borrow();
    for face in p.g.one_dart_per_cell(2, None) {
      let clicker = svg_.path();
      
      // make_face_clicker(face, &vertex_locations[..]);
    }
  }
}

pub struct Puzzle {
  g: GMap,
  layout: OrbitMap<(f64, f64)>,     // positions of every vertex
  face_clickers: OrbitMap<svg::Object>, // SVG path for each face
}

fn trace<T>(x: T) -> T
where
  T: std::fmt::Debug,
{
  log!("[rust] {:?}", x);
  x
}

impl Puzzle {
  pub fn new() -> Self {
    let (g, squares) = grid::new(10, 10);
    let mut layout = OrbitMap::over_cells(0, 2);
    for (i, row) in grid::vertex_grid(&g, &squares).iter().enumerate() {
      for (j, &v) in row.iter().enumerate() {
        layout.insert(&g, v, (j as f64, i as f64))
      }
    }

    Puzzle {
      g,
      layout,
      face_clickers: OrbitMap::over_cells(2, 2),
    }
  }
}

pub fn make_face_clickers(state: &JsValue, p: &Puzzle) {
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

fn center(p: &Puzzle, d: usize, i: usize) -> (f64, f64) {
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

pub fn on_face_click(p: &Puzzle, face: usize) {
  log!("clicked on face {}", face);
}
