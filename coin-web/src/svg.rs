use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "./svg.js")]
extern "C" {
  pub fn client_to_svg(s: &str, x: i32, y: i32) -> Point;

  pub type Point;
  #[wasm_bindgen(structural, method, getter)]
  pub fn x(p: &Point) -> f64;
  #[wasm_bindgen(structural, method, getter)]
  pub fn y(p: &Point) -> f64;
}
