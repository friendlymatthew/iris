#![warn(clippy::nursery)]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub mod font;
pub mod png;
pub mod renderer;
pub mod util;
