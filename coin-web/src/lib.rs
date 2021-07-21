#[macro_use]
pub mod utils;

use wasm_bindgen::prelude::*;

use gmap::GMap;

#[wasm_bindgen]
pub struct GMapWrapper(GMap);

fn trace<T>(x: T) -> T
where
  T: std::fmt::Debug,
{
  log!("[rust] {:?}", x);
  x
}

#[wasm_bindgen]
pub fn initialize_graph() -> GMapWrapper {
  GMapWrapper(GMap::grid(10, 10).0)
}

#[wasm_bindgen]
pub fn count_darts(g: &GMapWrapper) -> usize {
  trace(g.0.alpha().len())
}
