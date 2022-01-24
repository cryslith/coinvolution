use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "../www/svg.js")]
extern "C" {
  #[derive(Clone)]
  pub type SVG;
  #[wasm_bindgen(structural, method)]
  pub fn path(s: &SVG) -> Object;

  #[derive(Clone)]
  pub type Object;
  #[wasm_bindgen(structural, method)]
  pub fn plot(o: &Object, spec: &str);
  #[wasm_bindgen(structural, method)]
  pub fn attr(o: &Object, key: &str, value: &str);
  #[wasm_bindgen(structural, method)]
  pub fn remove(o: &Object);
  #[wasm_bindgen(structural, method)]
  pub fn click(o: &Object, callback: &Closure<dyn FnMut(&JsEvent)>);

  pub type JsEvent;
  pub fn get_location(s: &SVG, e: &JsEvent) -> Point;

  pub type Point;
  #[wasm_bindgen(structural, method, getter)]
  pub fn x(p: &Point) -> f64;
  #[wasm_bindgen(structural, method, getter)]
  pub fn y(p: &Point) -> f64;
}
