use wasm_bindgen::prelude::*;

pub struct Path {
  segments: Vec<String>,
  stroke: String,
  fill: String,
}

pub struct SVG(pub JsValue);

impl SVG {
  pub fn path(&self) -> Object {
    Object(path(&self.0))
  }
}

#[derive(Clone)]
pub struct Object(pub JsValue);

impl Object {
  pub fn plot(&self, spec: &str) {
    plot(&self.0, spec)
  }

  pub fn attr(&self, key: &str, value: &str) {
    attr(&self.0, key, value)
  }

  pub fn remove(&self) {
    remove(&self.0)
  }
}

#[wasm_bindgen(raw_module = "../www/svg.js")]
extern "C" {
  fn path(s: &JsValue) -> JsValue;
  fn plot(s: &JsValue, spec: &str);
  fn attr(s: &JsValue, key: &str, value: &str);
  fn remove(s: &JsValue);
}
