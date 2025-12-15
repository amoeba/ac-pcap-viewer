//! Web UI for Asheron's Call PCAP Parser
//!
//! WASM-compiled version of the AC PCAP viewer for web deployment.

#![cfg(target_arch = "wasm32")]

use app::PcapViewerApp;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

/// Query parameter variants
enum QueryParams {
    Url(String),
    Discord { channel: String, msg: String },
}

/// Get query parameters - either ?url=... or ?channel=X&msg=Y
fn get_query_params() -> Option<QueryParams> {
    let window = web_sys::window()?;
    let location = window.location();
    let search = location.search().ok()?;

    if search.is_empty() {
        return None;
    }

    // Remove the leading '?' and parse
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;

    // Check for Discord params first (channel and msg)
    let channel = params.get("channel");
    let msg = params.get("msg");

    if let (Some(channel), Some(msg)) = (channel, msg) {
        return Some(QueryParams::Discord { channel, msg });
    }

    // Fall back to URL param
    params.get("url").map(QueryParams::Url)
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

                    // Check for query parameters on web version
                    match get_query_params() {
                        Some(QueryParams::Url(url)) => {
                            log::info!("Found URL in query params: {}", url);
                            app.initial_url = Some(url);
                            app.status_message = "Loading PCAP from URL...".to_string();
                        }
                        Some(QueryParams::Discord { channel, msg }) => {
                            log::info!("Found Discord params: channel={}, msg={}", channel, msg);
                            app.discord_channel_id = channel;
                            app.discord_message_id = msg;
                            app.status_message = "Ready to load PCAP from Discord...".to_string();
                            // The app will auto-load on first frame via initial_discord_load
                            app.initial_discord_load = true;
                        }
                        None => {
                            log::info!("No query params found");
                        }
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
