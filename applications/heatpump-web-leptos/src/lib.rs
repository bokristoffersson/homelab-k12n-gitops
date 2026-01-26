use leptos::*;
use wasm_bindgen::prelude::*;

mod api;
mod app;
mod components;
mod models;
mod state;

/// WASM entry point - called when the WASM module loads
#[wasm_bindgen(start)]
pub fn main() {
    // Set up panic hook for better error messages in WASM
    console_error_panic_hook::set_once();

    // Initialize logging
    _ = console_log::init_with_level(log::Level::Debug);

    log::info!("Starting Heatpump Monitor");

    // Mount the app to the document body
    mount_to_body(|| view! { <app::App /> });
}
