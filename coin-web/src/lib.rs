#[macro_use]
pub mod utils;
pub mod puzzle;
pub mod svg;

use puzzle::Puzzle;

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

struct State {
  p: Puzzle,
  events: VecDeque<Event>,
}

impl State {
  fn handle(&mut self, e: Event, jstate: &JState) {
    match e {
      Event::Puzzle(e) => self.p.handle(e, &mut self.events, jstate),
    }
  }

  fn handle_all(&mut self, jstate: &JState) {
    loop {
      let e = if let Some(e) = self.events.pop_front() {
        e
      } else {
        break;
      };
      self.handle(e, jstate);
    }
  }
}

enum Event {
  Puzzle(puzzle::Event),
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct JState(Rc<RefCell<State>>);

impl JState {
  fn handle(&self, e: Event) {
    let mut state = self.0.borrow_mut();
    state.events.push_back(e);
    state.handle_all(self);
  }
}

#[wasm_bindgen]
pub fn initialize(svg: svg::SVG) -> JState {
  let p = Puzzle::new(svg);
  let jstate = JState(Rc::new(RefCell::new(State {
    p,
    events: VecDeque::new(),
  })));
  {
    let p = &mut jstate.0.borrow_mut().p;
    p.display(&jstate);
  }
  return jstate;
}
