use crate::svg::client_to_svg;

use gmap::{grids::hex, Alphas, Dart, GMap, OrbitMap};

use itertools::chain;
use sauron::{
  html::attributes::{name, style, tabindex},
  prelude::*,
};
use wasm_bindgen::JsCast;
use web_sys::WheelEvent;

const GRID_STROKE_WIDTH: f64 = 0.05;
const DOT_RADIUS: f64 = 0.1;
const CROSS_SIZE: f64 = 0.12;
const CROSS_STROKE_WIDTH: f64 = 0.05;
const FILL_STROKE_WIDTH: f64 = 0.02;
const LINE_STROKE_WIDTH: f64 = 0.07;
const ZOOM_BASE: f64 = 1.2;

pub enum Msg {
  FaceClick(Dart, f64, f64),
  SelectLayer(usize),
  KeyPressed(char),
  Backspace,
  Zoom(f64, f64, f64),
  ChangeName(usize, String),
  Solve,
  None,
}

pub enum Marker {
  Dot,
  Cross,
  Fill,
  LineVE,
  LineEF,
  LineVF,
  Arrow,
}

pub type Color = String;

pub enum LayerData {
  String {
    color: Color,
    data: OrbitMap<String>,
    size: f64,
    size_scaling: f64,
  },
  Enum {
    spec: Vec<(Marker, Color)>,
    data: OrbitMap<usize>,
  },
}

pub struct Layer {
  name: String,
  data: LayerData,
  active_dart: Option<Dart>,
}

pub struct Puzzle {
  g: GMap,
  solve_endpoint: Option<String>,
  layout: OrbitMap<(f64, f64)>, // positions of every vertex
  layers: Vec<Layer>,
  active_layer: Option<usize>,
  viewbox: [f64; 4],
}

impl Puzzle {
  pub fn new(solve_endpoint: Option<String>) -> Self {
    let (g, rows) = hex::new(10, 10);
    let coords = hex::vertex_coords(&g, &rows);
    let mut layout = OrbitMap::new(Alphas::VERTEX);
    for v in g.one_dart_per_cell(0) {
      let (a, b) = coords.map()[&v];
      let a = a as f64;
      let b = b as f64;
      layout.insert(&g, v, (a * 3f64.sqrt() / 4., a / 4. + b / 2.));
    }
    let viewbox = [-2., -2., 14., 14.];

    Puzzle {
      g,
      solve_endpoint,
      layout,
      layers: vec![
        Layer {
          name: "vertex".to_string(),
          data: LayerData::Enum {
            spec: vec![
              (Marker::Dot, "black".to_string()),
              (Marker::Cross, "red".to_string()),
              (Marker::Fill, "green".to_string()),
              (Marker::LineVE, "blue".to_string()),
              (Marker::LineVF, "magenta".to_string()),
            ],
            data: OrbitMap::new(Alphas::VERTEX),
          },
          active_dart: None,
        },
        Layer {
          name: "edge".to_string(),
          data: LayerData::Enum {
            spec: vec![
              (Marker::Dot, "black".to_string()),
              (Marker::Cross, "red".to_string()),
              (Marker::Fill, "green".to_string()),
              (Marker::LineVE, "blue".to_string()),
              (Marker::LineEF, "cyan".to_string()),
            ],
            data: OrbitMap::new(Alphas::EDGE),
          },
          active_dart: None,
        },
        Layer {
          name: "face".to_string(),
          data: LayerData::Enum {
            spec: vec![
              (Marker::Dot, "black".to_string()),
              (Marker::Cross, "red".to_string()),
              (Marker::Fill, "green".to_string()),
              (Marker::LineEF, "cyan".to_string()),
              (Marker::LineVF, "magenta".to_string()),
            ],
            data: OrbitMap::new(Alphas::FACE),
          },
          active_dart: None,
        },
        Layer {
          name: "slitherlink".to_string(),
          data: LayerData::Enum {
            spec: vec![
              (Marker::LineVE, "black".to_string()),
              (Marker::Cross, "red".to_string()),
            ],
            data: OrbitMap::new(Alphas::EDGE),
          },
          active_dart: None,
        },
        Layer {
          name: "text".to_string(),
          data: LayerData::String {
            color: "black".to_string(),
            data: OrbitMap::new(Alphas::FACE),
            size: 1.0,
            size_scaling: 1.0,
          },
          active_dart: None,
        },
      ],
      active_layer: None,
      viewbox,
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

  fn view_layer<'a>(&'a self, layer: &'a Layer) -> Box<dyn Iterator<Item = Node<Msg>> + 'a> {
    match &layer.data {
      LayerData::String {
        color,
        data,
        size,
        size_scaling,
      } => {
        let indices = data.indices();
        Box::new(self.g.one_dart_per_orbit(indices).filter_map(move |dart| {
          let value = data.map().get(&dart);
          let (center_x, center_y) = center(&self.g, &self.layout, dart, indices);
          value.map(|s| {
            svg::tags::text(
              [
                x(center_x),
                y(center_y),
                dominant_baseline("central"),
                text_anchor("middle"),
                fill(color),
                font_size(size * (1.0 / (s.len() as f64).max(1.0)).powf(*size_scaling)),
                pointer_events("none"),
              ],
              [text(s)],
            )
          })
        }))
      }
      LayerData::Enum { spec, data } => {
        let indices = data.indices();
        Box::new(self.g.one_dart_per_orbit(indices).filter_map(move |dart| {
          let value = data.map().get(&dart);

          value.map(|i| {
            let (marker_type, color) = &spec[*i];
            match marker_type {
              Marker::Dot => {
                let (center_x, center_y) = center(&self.g, &self.layout, dart, indices);
                circle(
                  [
                    cx(center_x),
                    cy(center_y),
                    r(DOT_RADIUS),
                    stroke("none"),
                    fill(color),
                    pointer_events("none"),
                  ],
                  [],
                )
              }
              Marker::Cross => {
                let (center_x, center_y) = center(&self.g, &self.layout, dart, indices);
                g(
                  [
                    stroke(color),
                    stroke_width(CROSS_STROKE_WIDTH),
                    fill("none"),
                    pointer_events("none"),
                  ],
                  [
                    line(
                      [
                        x1(center_x - CROSS_SIZE),
                        y1(center_y - CROSS_SIZE),
                        x2(center_x + CROSS_SIZE),
                        y2(center_y + CROSS_SIZE),
                      ],
                      [],
                    ),
                    line(
                      [
                        x1(center_x - CROSS_SIZE),
                        y1(center_y + CROSS_SIZE),
                        x2(center_x + CROSS_SIZE),
                        y2(center_y - CROSS_SIZE),
                      ],
                      [],
                    ),
                  ],
                )
              }
              Marker::Fill => g(
                [
                  stroke(color),
                  stroke_width(FILL_STROKE_WIDTH),
                  fill(color),
                  pointer_events("none"),
                ],
                self.g.orbit(dart, indices).map(|tri| {
                  let vertex = self.layout.map().get(&tri).unwrap();
                  let edge = center(&self.g, &self.layout, tri, Alphas::EDGE);
                  let face = center(&self.g, &self.layout, tri, Alphas::FACE);
                  polygon(
                    [points(format!(
                      "{},{} {},{} {},{}",
                      vertex.0, vertex.1, edge.0, edge.1, face.0, face.1,
                    ))],
                    [],
                  )
                }),
              ),
              Marker::LineVE => self.draw_line(indices, Alphas::VERTEX, Alphas::EDGE, dart, color),
              Marker::LineEF => self.draw_line(indices, Alphas::EDGE, Alphas::FACE, dart, color),
              Marker::LineVF => self.draw_line(indices, Alphas::VERTEX, Alphas::FACE, dart, color),
              _ => todo!(),
            }
          })
        }))
      }
    }
  }

  fn draw_line(
    &self,
    indices: Alphas,
    from: Alphas,
    to: Alphas,
    dart: Dart,
    color: &Color,
  ) -> Node<Msg> {
    g(
      [
        stroke(color),
        stroke_width(LINE_STROKE_WIDTH),
        fill("none"),
        pointer_events("none"),
      ],
      self.g.orbit(dart, indices).map(|tri| {
        let start = center(&self.g, &self.layout, tri, from);
        let end = center(&self.g, &self.layout, tri, to);
        line([x1(start.0), y1(start.1), x2(end.0), y2(end.1)], [])
      }),
    )
  }

  fn view_face_clickers(&self) -> impl Iterator<Item = Node<Msg>> + '_ {
    self.g.one_dart_per_cell(2).map(|face| {
      let mut segments = vec![];
      let mut v = face;
      loop {
        let &(x, y) = self.layout.map().get(&v).expect("missing vertex in layout");
        segments.push(format!("{},{}", x, y));

        v = self.g.al(v, [0, 1]);
        if v == face {
          break;
        }
      }

      polygon(
        [
          points(&segments.join(" ")),
          stroke("gray"),
          stroke_width(GRID_STROKE_WIDTH),
          fill("transparent"),
          on_mousedown(move |event: MouseEvent| {
            if event.button() != 0 {
              return Msg::None;
            }
            let coords = client_to_svg("#puzzle", event.client_x(), event.client_y());
            let x = coords.x();
            let y = coords.y();
            Msg::FaceClick(face, x, y)
          }),
        ],
        [],
      )
    })
  }

  fn view_layer_selector(&self) -> Node<Msg> {
    fieldset(
      [],
      [legend([], [text("Layer")])]
        .into_iter()
        .chain(self.layers.iter().enumerate().map(|(i, l)| {
          label(
            [],
            [
              input(
                [
                  r#type("radio"),
                  name("layer"),
                  on_click(move |_| Msg::SelectLayer(i)),
                  checked(self.active_layer == Some(i)),
                ],
                [],
              ),
              text(&l.name),
            ],
          )
        })),
    )
  }

  fn view_layer_options(&self) -> Option<Node<Msg>> {
    let layer_index = self.active_layer?;
    let active_layer = &self.layers[layer_index];
    Some(fieldset(
      [],
      [
        legend([], [text("Layer Options")]),
        label(
          [],
          [
            text("Name "),
            input(
              [
                r#type("text"),
                key(layer_index),
                on_input(move |event: InputEvent| {
                  Msg::ChangeName(layer_index, event.value.to_string())
                }),
                value(&active_layer.name),
              ],
              [],
            ),
          ],
        ),
      ],
    ))
  }

  fn view_solve_ui(&self) -> Option<Node<Msg>> {
    self.solve_endpoint.as_ref().map(|solve_endpoint| {
      div(
        [],
        [button(
          [on_click(|event: MouseEvent| Msg::Solve)],
          [text("Solve")],
        )],
      )
    })
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
      Msg::SelectLayer(i) => {
        log!("event: selected layer {}", i);
        self.active_layer = Some(i);
      }
      Msg::KeyPressed(_) | Msg::Backspace => {
        log!("event: key pressed");
        if let Some(i) = self.active_layer {
          let layer = &mut self.layers[i];
          if let LayerData::String { ref mut data, .. } = layer.data {
            if let Some(d) = layer.active_dart {
              let mut x = data.map().get(&d).cloned().unwrap_or_else(String::new);
              match msg {
                Msg::KeyPressed(s) => x.push(s),
                Msg::Backspace => {
                  if !x.is_empty() {
                    x.truncate(x.len() - 1);
                  }
                }
                _ => unreachable!(),
              }
              data.insert(&self.g, d, x);
            }
          }
        }
      }
      Msg::Zoom(magnitude, x, y) => {
        log!("event: zoom");
        let r = ZOOM_BASE.powf(magnitude);
        let [bx, by, bw, bh] = self.viewbox;
        let nx = bx + (x - bx) * (1. - r);
        let ny = by + (y - by) * (1. - r);
        let nw = bw * r;
        let nh = bh * r;
        self.viewbox = [nx, ny, nw, nh];
      }
      Msg::ChangeName(layer_index, name) => {
        log!("event: change name {}", layer_index);
        self.layers[layer_index].name = name;
      }
      Msg::Solve => {
        log!("event: solve");
      }
      Msg::None => {}
    }
    Cmd::none()
  }

  fn view(&self) -> Node<Msg> {
    article(
      [],
      [div(
        [
          style("display", "flex"),
          style("align-items", "center"),
          style("flex-direction", "column"),
        ],
        chain!(
          self.view_solve_ui(),
          [
            svg(
              [
                id("puzzle"),
                viewBox(self.viewbox),
                tabindex(0),
                style("border-style", "solid"),
                style("width", "400px"),
                style("height", "400px"),
                on_wheel(|event: MouseEvent| {
                  event.prevent_default();
                  event.stop_propagation();
                  let event: WheelEvent = if let Ok(e) = event.dyn_into() {
                    e
                  } else {
                    return Msg::None;
                  };
                  let coords = client_to_svg("#puzzle", event.client_x(), event.client_y());
                  let x = coords.x();
                  let y = coords.y();
                  Msg::Zoom(event.delta_y(), x, y)
                }),
                on_keydown(|event: KeyboardEvent| {
                  if event.alt_key() || event.ctrl_key() || event.meta_key() {
                    return Msg::None;
                  }
                  let key = event.key();
                  match &key[..] {
                    "Backspace" => {
                      event.prevent_default();
                      event.stop_propagation();
                      Msg::Backspace
                    }
                    _ if key.len() == 1 => {
                      event.prevent_default();
                      event.stop_propagation();
                      Msg::KeyPressed(key.chars().next().unwrap())
                    }
                    _ => Msg::None,
                  }
                }),
              ],
              self
                .view_face_clickers()
                .chain(self.layers.iter().flat_map(|l| self.view_layer(l))),
            ),
            self.view_layer_selector(),
          ],
          self.view_layer_options(),
        ),
      )],
    )
  }
}

/// center of the a-orbit at d
fn center(g: &GMap, layout: &OrbitMap<(f64, f64)>, d: Dart, a: Alphas) -> (f64, f64) {
  if !a.has(0) {
    return *layout.map().get(&d).expect("missing vertex in layout");
  }

  let ((x, y), n) = g.one_dart_per_incident_orbit(d, Alphas::VERTEX, a).fold(
    ((0f64, 0f64), 0f64),
    |((x, y), n), d| {
      let &(x1, y1) = layout.map().get(&d).expect("missing vertex in layout");
      ((x + x1, y + y1), n + 1f64)
    },
  );
  (x / n, y / n)
}
