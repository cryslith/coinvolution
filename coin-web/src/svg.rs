use wasm_bindgen::prelude::*;

#[wasm_bindgen(raw_module = "../www/svg.js")]
extern "C" {
  pub type SVG;
  #[derive(Clone)]
  pub type Object;

  #[wasm_bindgen(structural, method)]
  pub fn path(s: &SVG) -> Object;
  #[wasm_bindgen(structural, method)]
  pub fn plot(s: &Object, spec: &str);
  #[wasm_bindgen(structural, method)]
  pub fn attr(s: &Object, key: &str, value: &str);
  #[wasm_bindgen(structural, method)]
  pub fn remove(s: &Object);
  #[wasm_bindgen(structural, method)]
  pub fn click(s: &Object, callback: &Closure<dyn FnMut()>);
}
