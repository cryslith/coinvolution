#[macro_use]
pub mod svg;
pub mod app;

use app::App;

use sauron::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
  #[cfg(feature = "console_error_panic_hook")]
  console_error_panic_hook::set_once();
  Program::mount_to_body(App::new());
}
