//! 2048 split into a pure rules engine and a Yew view.
//!
//! `game` compiles and is tested on the host toolchain; `ui` exists only for
//! the `wasm32-unknown-unknown` target, which keeps `cargo test` free of any
//! browser dependency.

pub mod game;

#[cfg(target_arch = "wasm32")]
pub mod ui;
