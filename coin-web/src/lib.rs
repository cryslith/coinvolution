#[macro_use]
pub mod utils;
pub mod puzzle;
pub mod svg;

use puzzle::Puzzle;

use sauron::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() {
  #[cfg(feature = "console_error_panic_hook")]
  console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn run(solve_endpoint: Option<String>) {
  Program::mount_to_body(Puzzle::new(solve_endpoint));
}
