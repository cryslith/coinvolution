use sauron::MouseEvent;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "../www/svg.js")]
extern "C" {
  pub fn get_location(s: &str, e: &MouseEvent) -> Point;

  pub type Point;
  #[wasm_bindgen(structural, method, getter)]
  pub fn x(p: &Point) -> f64;
  #[wasm_bindgen(structural, method, getter)]
  pub fn y(p: &Point) -> f64;
}
