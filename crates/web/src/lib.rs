//! Web UI for Asheron's Call PCAP Parser
//!
//! WASM-compiled version of the AC PCAP viewer for web deployment.

#![cfg(target_arch = "wasm32")]

use app::PcapViewerApp;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Get the URL from query parameters (?url=...)
fn get_url_from_query_params() -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();
    let search = location.search().ok()?;

    if search.is_empty() {
        return None;
    }

    // Remove the leading '?' and parse
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    params.get("url")
}

// WASM entry point
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // Redirect panics to console.error
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document
            .get_element_by_id("ac_pcap_canvas")
            .expect("Failed to find canvas element")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("Element is not a canvas");

        // Create app with URL from query params if available
        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc: &eframe::CreationContext<'_>| {
                    let mut app = PcapViewerApp::new(cc);

                    // Check for URL query parameter on web version
                    if let Some(url) = get_url_from_query_params() {
                        log::info!("Found URL in query params: {}", url);
                        app.initial_url = Some(url);
                        app.status_message = "Loading PCAP from URL...".to_string();
                    }

                    Ok(Box::new(app))
                }),
            )
            .await;

        // Remove loading text and spinner
        if let Some(loading) = document.get_element_by_id("loading") {
            loading.remove();
        }

        if let Err(e) = start_result {
            log::error!("Failed to start eframe: {:?}", e);
        }
    });

    Ok(())
}
