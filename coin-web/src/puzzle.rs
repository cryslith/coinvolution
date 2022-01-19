use crate::svg::{self, SVG};

use gmap::{grid, GMap, OrbitMap};

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::Closure;

struct FaceClicker {
  path: svg::Object,
  click: Option<Closure<dyn FnMut()>>,
}  

pub struct Puzzle {
  g: GMap,
  svg: SVG,
  layout: OrbitMap<(f64, f64)>,         // positions of every vertex
  face_clickers: OrbitMap<Rc<RefCell<FaceClicker>>>,
}

impl Puzzle {
  pub fn new(svg: svg::SVG) -> Self {
    let (g, squares) = grid::new(10, 10);
    let mut layout = OrbitMap::over_cells(0, 2);
    for (i, row) in grid::vertex_grid(&g, &squares).iter().enumerate() {
      for (j, &v) in row.iter().enumerate() {
        layout.insert(&g, v, (j as f64, i as f64))
      }
    }

    Puzzle {
      g,
      svg,
      layout,
      face_clickers: OrbitMap::over_cells(2, 2),
    }
  }

  pub fn display(&mut self) {
    let g = &self.g;
    for face in g.one_dart_per_cell(2, None) {
      let mut segments = vec![];
      let mut v = face;
      loop {
        let &(x, y) = self.layout.map().get(&v).expect("missing vertex in layout");
        segments.push(format!("{} {} {}", if v == face { "M" } else { "L" }, x, y));

        v = g.al(v, [0, 1]);
        if v == face {
          break;
        }
      }

      segments.push(format!("Z"));

      let clicker = self.svg.path();
      clicker.plot(&segments.join(" "));
      clicker.attr("stroke", "gray");
      clicker.attr("stroke-width", "0.05");
      clicker.attr("fill", "transparent");
      let onclick = Closure::new(move || log!("face {} clicked", face));
      clicker.click(&onclick);
      self.face_clickers.insert(&g, face, Rc::new(RefCell::new(FaceClicker {
        path: clicker,
        click: Some(onclick),
      })));
    }
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
