//! Browser entry point. Built for wasm by `trunk`; the host build is a stub so
//! that `cargo test` and `cargo clippy` work without a wasm toolchain.

#[cfg(target_arch = "wasm32")]
fn main() {
    yew::Renderer::<rust_2048::ui::App>::new().render();
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("rust-2048 runs in the browser. Build it with `trunk serve --open`.");
    std::process::exit(1);
}
