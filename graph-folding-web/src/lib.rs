#[macro_use]
pub mod svg;
pub mod app;

use app::App;

use sauron::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() {
  #[cfg(feature = "console_error_panic_hook")]
  console_error_panic_hook::set_once();
  Program::mount_to_body(App::new());
}
