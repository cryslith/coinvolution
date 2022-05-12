use gmap::{Alphas, Dart, GMap, OrbitReprs};
use graph_folding::{examples, Angle, Color, Constraints, Length, Problem};

use std::collections::HashMap;

use sauron::{html::attributes::style, prelude::*};

pub enum Msg {}

pub struct App {
  problem: Problem,
  or: OrbitReprs,
  layout: HashMap<Dart, (f64, f64)>, // positions of every vertex
  constraints: Result<Constraints, graph_folding::Error>,
}

impl App {
  pub fn new() -> Self {
    let problem = examples::kite();
    let g = problem.g();
    let mut or = OrbitReprs::new();
    or.ensure_all(g, Alphas::VERTEX);

    let mut layout = HashMap::new();
    let d = Dart(0);
    for (vertex, position) in [
      (d, (0., 0.)),
      (g.al(d, [1, 0]), (0., 1.)),
      (g.al(d, [1, 0, 1, 0]), (2., 2.)),
      (g.al(d, [1, 0, 1, 0, 1, 0]), (1., 0.)),
    ] {
      layout.insert(or[(Alphas::VERTEX, vertex)], position);
    }
    let constraints = problem.constraint_graph();

    Self {
      problem,
      or,
      layout,
      constraints,
    }
  }

  fn view_problem(&self) -> impl Iterator<Item = Node<Msg>> + '_ {
    let g = self.problem.g();
    g.one_dart_per_cell(2).filter_map(|face| {
      if face == self.problem.exterior_face() {
        return None;
      }

      let mut segments = vec![];
      let mut v = face;
      loop {
        let &(x, y) = self
          .layout
          .get(&self.or[(Alphas::VERTEX, v)])
          .expect("missing vertex in layout");
        segments.push(format!("{},{}", x, y));

        v = g.al(v, [0, 1]);
        if v == face {
          break;
        }
      }

      Some(polygon(
        [
          points(&segments.join(" ")),
          stroke("gray"),
          stroke_width("0.05"),
          fill("transparent"),
        ],
        [],
      ))
    })
  }

  // fn constraint_exterior(&self) -> HashMap<Dart, bool> {
  //   let constraints = self.constraints.unwrap();
  //   let cg = constraints.cg();
  //   let mut cgor = OrbitReprs::new();
  //   cgor.ensure_all(Alphas::VERTEX);

  //   let angle_to_cg = constraints.angle_to_cg();
  //   let mut cg_to_angle: HashMap<Dart, Dart> = angle_to_cg.iter().map(|(x, y)| (y, x)).collect();
  // }

  fn constraint_positions(&self, constraints: &Constraints) -> HashMap<Dart, (f64, f64)> {
    // Maybe a better approach would be to lay out each face via physics simulation...

    let cg = constraints.cg();
    let mut cgor = OrbitReprs::new();
    cgor.ensure_all(cg, Alphas::VERTEX);
    let angle_to_cg = constraints.angle_to_cg();
    let cg_to_vert: HashMap<Dart, Dart> = angle_to_cg
      .iter()
      .map(|(&x, &y)| (cgor[(Alphas::VERTEX, y)], self.or[(Alphas::VERTEX, x)]))
      .collect();

    let mut guess: HashMap<Dart, (f64, f64)> = HashMap::new();
    for vertex in cg.one_dart_per_cell(0) {
      let old_vertex_position = cg_to_vert
        .get(&vertex)
        .and_then(|v| self.layout.get(v))
        .cloned()
        .unwrap_or((0., 0.));
      guess.insert(vertex, old_vertex_position);
    }

    // this doesn't work at all because lots of things want to be in the same place :(

    for _ in 0..2 {
      let mut new_guess: HashMap<Dart, (f64, f64)> = HashMap::new();
      for vertex in cg.one_dart_per_cell(0) {
        if let Some(old_vertex_position) = cg_to_vert
          .get(&vertex)
          .and_then(|v| self.layout.get(v))
          .cloned()
        {
          new_guess.insert(vertex, old_vertex_position);
          continue;
        }

        let mut x = 0.;
        let mut y = 0.;
        let mut n = 0.;
        for edge in cg.one_dart_per_incident_cell(vertex, 1, 0) {
          let other_vertex = cgor[(Alphas::VERTEX, cg[(edge, 0)])];
          let (x1, y1) = guess[&other_vertex];
          x += x1;
          y += y1;
          n += 1.;
        }
        x = x / n;
        y = y / n;
        new_guess.insert(vertex, (x, y));
      }
      guess = new_guess;
    }

    web_sys::console::log_1(&format!("{:#?}", guess).into());
    guess
  }

  fn view_constraints(&self) -> Box<dyn Iterator<Item = Node<Msg>> + '_> {
    let constraints = match self.constraints.as_ref() {
      Ok(c) => c,
      Err(e) => {
        return Box::new([svg::tags::text([x(0), y(0)], [text(e.to_string())])].into_iter());
      }
    };
    let positions = self.constraint_positions(&constraints);
    let cg = constraints.cg();
    let mut cgor = OrbitReprs::new();
    cgor.ensure_all(cg, Alphas::VERTEX);
    Box::new(cg.one_dart_per_cell(2).filter_map(move |face| {
      // would be nice to hide CG parts from the original exterior face somehow

      let mut segments = vec![];
      let mut v = face;
      loop {
        let &(x, y) = positions
          .get(&cgor[(Alphas::VERTEX, v)])
          .expect("missing vertex in layout");
        segments.push(format!("{},{}", x, y));

        v = cg.al(v, [0, 1]);
        if v == face {
          break;
        }
      }

      Some(polygon(
        [
          points(&segments.join(" ")),
          stroke("gray"),
          stroke_width("0.05"),
          fill("transparent"),
        ],
        [],
      ))
    }))
  }
}

impl Application<Msg> for App {
  fn update(&mut self, msg: Msg) -> Cmd<Self, Msg> {
    match msg {}
  }

  fn view(&self) -> Node<Msg> {
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
          [viewBox([-2, -2, 14, 14])],
          self
            .view_problem()
            .chain([g([transform("translate(0 5)")], self.view_constraints())]),
        )],
      )],
    )
  }
}

/// center of the a-orbit at d
fn center(
  g: &GMap,
  or: OrbitReprs,
  layout: &HashMap<Dart, (f64, f64)>,
  d: Dart,
  a: Alphas,
) -> (f64, f64) {
  let ((x, y), n) = or
    .orbit_repr_per_incident_orbit(g, d, Alphas::VERTEX, a)
    .fold(((0f64, 0f64), 0f64), |((x, y), n), d| {
      let &(x1, y1) = layout.get(&d).expect("missing vertex in layout");
      ((x + x1, y + y1), n + 1f64)
    });
  (x / n, y / n)
}
