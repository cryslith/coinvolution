#[macro_use]
pub mod utils;
pub mod puzzle;
pub mod svg;

use puzzle::Puzzle;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

struct State {
  p: Puzzle,
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct JState(Rc<RefCell<State>>);

#[wasm_bindgen]
pub fn initialize(svg: svg::SVG) -> JState {
  let p = Puzzle::new(svg);
  let jstate = JState(Rc::new(RefCell::new(State { p })));
  {
    let p = &mut jstate.0.borrow_mut().p;
    p.display(&jstate);
  }
  return jstate;
}
