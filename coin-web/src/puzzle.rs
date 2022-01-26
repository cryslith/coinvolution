use crate::svg::{self, get_location, JsEvent, SVG};
use crate::JState;

use gmap::{grids::square, GMap, OrbitMap};

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use wasm_bindgen::prelude::Closure;

struct FaceClicker {
  path: svg::Object,
  click: Option<Closure<dyn FnMut(&JsEvent)>>,
}

pub enum Marker {
  Dot,
  Cross,
  Fill,
  CrossLine,
  Arrow,
}

pub type Color = String;

pub enum DataType {
  String(Color),
  Enum(Vec<(Marker, Color)>),
}

#[derive(Clone)]
pub enum Data {
  String(String),
  Enum(usize),
}

pub struct Layer {
  datatype: DataType,
  data: OrbitMap<Data>,
}

pub struct Puzzle {
  g: GMap,
  svg: SVG,
  layout: OrbitMap<(f64, f64)>, // positions of every vertex
  face_clickers: OrbitMap<Rc<RefCell<FaceClicker>>>,
  layers: Vec<Layer>,
  active_layer: Option<usize>,
}

pub enum Event {
  FaceClicked { face: usize, x: f64, y: f64 },
}

impl Puzzle {
  pub fn new(svg: svg::SVG) -> Self {
    let (g, squares) = square::new(10, 10);
    let mut layout = OrbitMap::over_cells(0, 2);
    for (i, row) in square::vertex_grid(&g, &squares).iter().enumerate() {
      for (j, &v) in row.iter().enumerate() {
        layout.insert(&g, v, (j as f64, i as f64))
      }
    }

    Puzzle {
      g,
      svg,
      layout,
      face_clickers: OrbitMap::over_cells(2, 2),
      layers: vec![],
      active_layer: None,
    }
  }

  // fn face_click(

  pub fn display(&mut self, jstate: &JState) {
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
      let svg_onclick = self.svg.clone();
      let jstate_onclick = jstate.clone();
      let onclick = Closure::new(move |e: &JsEvent| {
        let p = get_location(&svg_onclick, &e);
        jstate_onclick.handle(crate::Event::Puzzle(Event::FaceClicked {
          face,
          x: p.x(),
          y: p.y(),
        }));
      });
      clicker.click(&onclick);
      self.face_clickers.insert(
        &g,
        face,
        Rc::new(RefCell::new(FaceClicker {
          path: clicker,
          click: Some(onclick),
        })),
      );
    }
  }

  pub fn identify_dart(&self, face: usize, x: f64, y: f64) -> usize {
    let g = &self.g;
    let mut best_vertex = None;
    let mut best_distance = 0f64;
    let dist = |v: usize| {
      let &(vx, vy) = self.layout.map().get(&v).expect("missing vertex in layout");
      let dx = vx - x;
      let dy = vy - y;
      return dx * dx + dy * dy;
    };
    for v in g.one_dart_per_incident_cell(face, 0, 2, None) {
      let d = dist(v);
      if best_vertex == None || d < best_distance {
        best_vertex = Some(v);
        best_distance = d;
      }
    }
    let best_vertex = best_vertex.expect("no vertices");
    let a1 = g.al(best_vertex, [0]);
    let a2 = g.al(best_vertex, [1, 0]);
    if dist(a1) < dist(a2) {
      return best_vertex;
    } else {
      return g.al(best_vertex, [1]);
    }
  }

  pub(crate) fn handle(&mut self, e: Event, events: &mut VecDeque<crate::Event>, jstate: &JState) {
    match e {
      Event::FaceClicked { face, x, y } => {
        let dart = self.identify_dart(face, x, y);
        log!(
          "event: face {} clicked at ({}, {}).  dart: {}",
          face,
          x,
          y,
          dart
        );
      }
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
