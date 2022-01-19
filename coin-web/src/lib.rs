#[macro_use]
pub mod utils;
pub mod svg;
pub mod puzzle;

use puzzle::Puzzle;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

struct State {
  p: Puzzle,
}

#[wasm_bindgen]
pub struct JState(Rc<RefCell<State>>);

#[wasm_bindgen]
pub fn initialize(svg: svg::SVG) -> JState {
  let mut p = Puzzle::new(svg);
  p.display();
  return JState(Rc::new(RefCell::new(State {
    p,
  })));
}
