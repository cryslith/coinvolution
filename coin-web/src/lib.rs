#[macro_use]
pub mod utils;
pub mod puzzle;
pub mod svg;

use puzzle::Puzzle;

use sauron::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
  #[cfg(feature = "console_error_panic_hook")]
  console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn run(solve_endpoint: Option<String>) {
  log!("{:?}", solve_endpoint);
  Program::mount_to_body(Puzzle::new(solve_endpoint));
}
