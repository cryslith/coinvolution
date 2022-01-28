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

#[derive(Clone, Debug)]
pub enum Data {
  String(String),
  Enum(usize),
}

pub struct Layer {
  // todo combine datatype and data into a single enum
  datatype: DataType,
  data: OrbitMap<Data>,
  active_dart: Option<usize>,
  markers: OrbitMap<svg::Object>,
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
  ClickReceived { face: usize, x: f64, y: f64 },
  DartClicked(usize),
  LayerData(usize),
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
      layers: vec![Layer {
        datatype: DataType::Enum(vec![
          (Marker::Dot, "black".to_string()),
          (Marker::Dot, "red".to_string()),
        ]),
        data: OrbitMap::over_cells(1, 2),
        active_dart: None,
        markers: OrbitMap::over_cells(1, 2),
      }],
      active_layer: Some(0),
    }
  }

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
        jstate_onclick.handle(crate::Event::Puzzle(Event::ClickReceived {
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

  pub(crate) fn handle(&mut self, e: Event, events: &mut VecDeque<crate::Event>, _jstate: &JState) {
    match e {
      Event::ClickReceived { face, x, y } => {
        let dart = self.identify_dart(face, x, y);
        log!(
          "event: face {} clicked at ({}, {}).  dart: {}",
          face,
          x,
          y,
          dart
        );
        events.push_back(crate::Event::Puzzle(Event::DartClicked(dart)));
      }
      Event::DartClicked(dart) => {
        let layer = if let Some(layer) = self.active_layer {
          &mut self.layers[layer]
        } else {
          return;
        };
        match layer.datatype {
          DataType::String(_) => {
            layer.active_dart = Some(dart);
          }
          DataType::Enum(ref v) => {
            let i = match layer.data.map().get(&dart) {
              None => 0,
              Some(Data::Enum(i)) => i + 1,
              _ => panic!("incorrect data for datatype"),
            };
            if i < v.len() {
              layer.data.insert(&self.g, dart, Data::Enum(i));
            } else {
              layer.data.remove(&self.g, dart);
            }
            events.push_back(crate::Event::Puzzle(Event::LayerData(dart)));
          }
        }
      }
      Event::LayerData(dart) => {
        let layer = if let Some(layer) = self.active_layer {
          &mut self.layers[layer]
        } else {
          return;
        };
        let data = layer.data.map().get(&dart);
        log!("dart {} updated, value: {:?}", dart, data);
        match &layer.datatype {
          DataType::String(color) => todo!(),
          DataType::Enum(spec) => {
            if let Some(old_marker) = layer.markers.map().get(&dart) {
              old_marker.remove();
            }

            match data {
              None => {}
              Some(Data::Enum(i)) => {
                let (marker_type, color) = &spec[*i];
                match marker_type {
                  Marker::Dot => {
                    let (cx, cy) = center(&self.g, &self.layout, dart, 1); // todo detect cell type
                    let new_marker = self.svg.path();
                    new_marker.plot(&format!(
                      "M {} {} \
                                             m 0.1 0 \
                                             a 0.1 0.1 0 0 0 -0.2 0 \
                                             a 0.1 0.1 0 0 0 +0.2 0",
                      cx, cy
                    )); // todo abstract magic numbers
                    new_marker.attr("stroke", "none");
                    new_marker.attr("fill", color);
                    new_marker.attr("pointer-events", "none");
                    layer.markers.insert(&self.g, dart, new_marker);
                  }
                  _ => todo!(),
                }
              }
              _ => panic!("incorrect data for datatype"),
            }
          }
          _ => panic!("incorrect data for datatype"),
        }
      }
    }
  }
}

/// center of the i-cell at d
fn center(g: &GMap, layout: &OrbitMap<(f64, f64)>, d: usize, i: usize) -> (f64, f64) {
  let ((x, y), n) =
    g.one_dart_per_incident_cell(d, 0, i, None)
      .fold(((0f64, 0f64), 0f64), |((x, y), n), d| {
        let &(x1, y1) = layout.map().get(&d).expect("missing vertex in layout");
        ((x + x1, y + y1), n + 1f64)
      });
  (x / n, y / n)
}
