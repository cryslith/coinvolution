use crate::svg::{self, get_location, JsEvent, SVG};
use crate::JState;

use gmap::{grids::square, GMap, OrbitMap, Dart, Alphas};

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

pub enum LayerData {
  String {
    color: Color,
    data: OrbitMap<String>,
  },
  Enum {
    spec: Vec<(Marker, Color)>,
    data: OrbitMap<usize>,
  },
}

pub struct Layer {
  data: LayerData,
  active_dart: Option<Dart>,
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

impl Puzzle {
  pub fn new(svg: svg::SVG) -> Self {
    let (g, squares) = square::new(10, 10);
    let mut layout = OrbitMap::new(Alphas::VERTEX);
    for (i, row) in square::vertex_grid(&g, &squares).iter().enumerate() {
      for (j, &v) in row.iter().enumerate() {
        layout.insert(&g, v, (j as f64, i as f64))
      }
    }

    Puzzle {
      g,
      svg,
      layout,
      face_clickers: OrbitMap::new(Alphas::FACE),
      layers: vec![Layer {
        data: LayerData::Enum {
          spec: vec![
            (Marker::Dot, "black".to_string()),
            (Marker::Dot, "red".to_string()),
          ],
          data: OrbitMap::new(Alphas::EDGE),
        },
        active_dart: None,
        markers: OrbitMap::new(Alphas::EDGE),
      }],
      active_layer: Some(0),
    }
  }

  pub fn display(&mut self, jstate: &JState) {
    let g = &self.g;
    for face in g.one_dart_per_cell(2) {
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
        let mut state = jstate_onclick.0.borrow_mut();
        let p = get_location(&svg_onclick, &e);
        state.p.handle_click(face, p.x(), p.y());
      });
      clicker.on("mousedown", &onclick);
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

  pub fn identify_dart(&self, face: Dart, x: f64, y: f64) -> Dart {
    let g = &self.g;
    let mut best_vertex = None;
    let mut best_distance = 0f64;
    let dist = |v: Dart| {
      let &(vx, vy) = self.layout.map().get(&v).expect("missing vertex in layout");
      let dx = vx - x;
      let dy = vy - y;
      return dx * dx + dy * dy;
    };
    for v in g.one_dart_per_incident_cell(face, 0, 2) {
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

  pub(crate) fn handle_click(&mut self, face: Dart, x: f64, y: f64) {
    let dart = self.identify_dart(face, x, y);
    log!(
      "event: face {} clicked at ({}, {}).  dart: {}",
      face,
      x,
      y,
      dart
    );
    self.click_dart(dart);
  }

  fn click_dart(&mut self, dart: Dart) {
    let layer = if let Some(layer) = self.active_layer {
      &mut self.layers[layer]
    } else {
      return;
    };
    match &mut layer.data {
      LayerData::String { .. } => {
        layer.active_dart = Some(dart);
      }
      LayerData::Enum { spec, data } => {
        let i = data.map().get(&dart).map(|x| x + 1).unwrap_or(0);
        if i < spec.len() {
          data.insert(&self.g, dart, i);
        } else {
          data.remove(&self.g, dart);
        }
        self.redraw_active_layer(dart);
      }
    }
  }

  pub fn redraw_active_layer(&mut self, dart: Dart) {
    let layer = if let Some(layer) = self.active_layer {
      &mut self.layers[layer]
    } else {
      return;
    };
    match &layer.data {
      LayerData::String { .. } => todo!(),
      LayerData::Enum { spec, data } => {
        let value = data.map().get(&dart);
        let indices = data.indices();

        log!("dart {} updated, value: {:?}", dart, value);

        if let Some(old_marker) = layer.markers.map().get(&dart) {
          old_marker.remove();
        }

        match value {
          None => {}
          Some(i) => {
            let (marker_type, color) = &spec[*i];
            match marker_type {
              Marker::Dot => {
                let (cx, cy) = center(&self.g, &self.layout, dart, indices);
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
        }
      }
    }
  }
}

/// center of the a-orbit at d
fn center(g: &GMap, layout: &OrbitMap<(f64, f64)>, d: Dart, a: Alphas) -> (f64, f64) {
  let ((x, y), n) = g
    .one_dart_per_incident_orbit(d, Alphas::VERTEX, a)
    .fold(((0f64, 0f64), 0f64), |((x, y), n), d| {
      let &(x1, y1) = layout.map().get(&d).expect("missing vertex in layout");
      ((x + x1, y + y1), n + 1f64)
    });
  (x / n, y / n)
}
