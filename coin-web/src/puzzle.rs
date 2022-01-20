use crate::svg::{self, SVG};
use crate::JState;

use gmap::{grids::square, GMap, OrbitMap};

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use wasm_bindgen::prelude::Closure;

struct FaceClicker {
  path: svg::Object,
  click: Option<Closure<dyn FnMut()>>,
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
  FaceClicked { face: usize },
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
      let jstate_onclick = jstate.clone();
      let onclick = Closure::new(move || {
        jstate_onclick.handle(crate::Event::Puzzle(Event::FaceClicked { face }));
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

  pub(crate) fn handle(&mut self, e: Event, events: &mut VecDeque<crate::Event>, jstate: &JState) {
    match e {
      Event::FaceClicked { face } => {
        log!("event: face {} clicked", face);
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
