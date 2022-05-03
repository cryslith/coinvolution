use crate::svg::get_location;

use gmap::{grids::square, Alphas, Dart, GMap, OrbitMap};

use sauron::{html::attributes::style, prelude::*};

pub enum Msg {
  FaceClick(Dart, f64, f64),
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
}

pub struct Puzzle {
  g: GMap,
  layout: OrbitMap<(f64, f64)>, // positions of every vertex
  layers: Vec<Layer>,
  active_layer: Option<usize>,
}

impl Puzzle {
  pub fn new() -> Self {
    let (g, squares) = square::new(10, 10);
    let mut layout = OrbitMap::new(Alphas::VERTEX);
    for (i, row) in square::vertex_grid(&g, &squares).iter().enumerate() {
      for (j, &v) in row.iter().enumerate() {
        layout.insert(&g, v, (j as f64, i as f64))
      }
    }

    Puzzle {
      g,
      layout,
      layers: vec![Layer {
        data: LayerData::Enum {
          spec: vec![
            (Marker::Dot, "black".to_string()),
            (Marker::Dot, "red".to_string()),
          ],
          data: OrbitMap::new(Alphas::EDGE),
        },
        active_dart: None,
      }],
      active_layer: Some(0),
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
      }
    }
  }

  pub fn view_layer(&self, layer: &Layer) -> Vec<Node<Msg>> {
    let mut nodes = vec![];
    match &layer.data {
      LayerData::String { .. } => todo!(),
      LayerData::Enum { spec, data } => {
        let indices = data.indices();
        for dart in self.g.one_dart_per_orbit(indices) {
          let value = data.map().get(&dart);

          match value {
            None => {}
            Some(i) => {
              let (marker_type, color) = &spec[*i];
              match marker_type {
                Marker::Dot => {
                  let (center_x, center_y) = center(&self.g, &self.layout, dart, indices);
                  // todo abstract magic numbers
                  let new_marker = circle(
                    [
                      cx(center_x),
                      cy(center_y),
                      r(0.1),
                      stroke("none"),
                      fill(color),
                      pointer_events("none"),
                    ],
                    [],
                  );

                  nodes.push(new_marker);
                }
                _ => todo!(),
              }
            }
          }
        }
      }
    }
    nodes
  }
}

impl Application<Msg> for Puzzle {
  fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
    match msg {
      Msg::FaceClick(face, x, y) => {
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
    }
    Cmd::none()
  }

  fn view(&self) -> Node<Msg> {
    let g = &self.g;
    let mut face_clickers: Vec<Node<Msg>> = vec![];

    for face in g.one_dart_per_cell(2) {
      let mut segments = vec![];
      let mut v = face;
      loop {
        let &(x, y) = self.layout.map().get(&v).expect("missing vertex in layout");
        segments.push(format!("{},{}", x, y));

        v = g.al(v, [0, 1]);
        if v == face {
          break;
        }
      }

      let clicker = polygon(
        [
          points(&segments.join(" ")),
          stroke("gray"),
          stroke_width("0.05"),
          fill("transparent"),
          on_click(move |event: MouseEvent| {
            let coords = get_location("#puzzle", &event);
            let x = coords.x();
            let y = coords.y();
            Msg::FaceClick(face, x, y)
          }),
        ],
        [],
      );
      face_clickers.push(clicker);
    }

    article(
      [],
      [div(
        [
          style("height", "95vh"),
          style("display", "flex"),
          style("align-items", "center"),
          style("flex-direction", "column"),
        ],
        [svg(
          [id("puzzle"), viewBox([-2, -2, 14, 14])],
          face_clickers
            .into_iter()
            .chain(self.layers.iter().flat_map(|l| self.view_layer(l))),
        )],
      )],
    )
  }
}

/// center of the a-orbit at d
fn center(g: &GMap, layout: &OrbitMap<(f64, f64)>, d: Dart, a: Alphas) -> (f64, f64) {
  let ((x, y), n) = g.one_dart_per_incident_orbit(d, Alphas::VERTEX, a).fold(
    ((0f64, 0f64), 0f64),
    |((x, y), n), d| {
      let &(x1, y1) = layout.map().get(&d).expect("missing vertex in layout");
      ((x + x1, y + y1), n + 1f64)
    },
  );
  (x / n, y / n)
}
