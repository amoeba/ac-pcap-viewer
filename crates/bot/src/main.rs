use axum::{
    extract::Path,
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::info;

mod bot;
mod discord;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
        )
        .init();

    // Print version info
    println!("Starting bot (version: {})", env!("GIT_SHA_SHORT"));
    info!("Bot version: {}", env!("GIT_SHA"));

    // Get Discord bot token (optional)
    if let Ok(discord_token) = std::env::var("DISCORD_OAUTH_TOKEN") {
        info!("Discord bot token found - starting bot");

        // Get web URL from environment or use default
        let web_url =
            std::env::var("WEB_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        info!("Web UI URL: {}", web_url);

        // Spawn bot in background task
        let bot_token = discord_token.clone();
        tokio::spawn(async move {
            if let Err(e) = bot::start_bot(bot_token, web_url).await {
                tracing::error!("Bot error: {}", e);
            }
        });
    } else {
        println!("DISCORD_OAUTH_TOKEN not set - Discord bot disabled");
        println!("Web server will still run, but Discord integration will not be available");
        info!("Discord bot disabled (no token provided)");
    }

    // Start web server
    let app = create_router();

    // Use PORT env var if set (for Dokku/Heroku), otherwise default to 3000
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind listener");

    info!("Web server running on {}", addr);

    // Setup graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server failed");

    info!("Server shutdown complete");
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("\nReceived Ctrl+C, shutting down gracefully...");
        },
        _ = terminate => {
            println!("\nReceived termination signal, shutting down gracefully...");
        },
    }
}

async fn log_requests(req: Request<axum::body::Body>, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    println!(">>> HTTP {method} {uri}");
    next.run(req).await
}

fn create_router() -> Router {
    let dist_path = std::path::PathBuf::from("dist");

    Router::new()
        .route("/api/health", get(health))
        .route(
            "/api/discord/channels/{channel_id}/messages/{message_id}/attachments",
            get(discord_pull),
        )
        .fallback_service(ServeDir::new(&dist_path))
        .layer(middleware::from_fn(log_requests))
        .layer(TraceLayer::new_for_http())
}

/// Health check endpoint
async fn health() -> &'static str {
    info!("Health check endpoint called");
    "OK"
}

#[derive(Deserialize)]
struct DiscordParams {
    channel_id: String,
    message_id: String,
}

#[derive(Serialize)]
struct DiscordError {
    error: String,
}

/// Fetch PCAP from Discord message attachment
async fn discord_pull(
    Path(params): Path<DiscordParams>,
) -> Result<Vec<u8>, (StatusCode, Json<DiscordError>)> {
    println!(
        "==> Discord pull request: channel={}, msg={}",
        params.channel_id, params.message_id
    );
    info!(
        "Discord pull request: channel={}, msg={}",
        params.channel_id, params.message_id
    );

    // Check if token is available
    let token = std::env::var("DISCORD_OAUTH_TOKEN").map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(DiscordError {
                error: "Discord OAuth token not configured".to_string(),
            }),
        )
    })?;

    // Fetch message from Discord API
    let message = discord::fetch_message(&params.channel_id, &params.message_id, &token)
        .await
        .map_err(|(status, error)| (status, Json(DiscordError { error })))?;

    // Find first PCAP attachment
    let pcap_attachment = message
        .attachments
        .iter()
        .find(|a| a.filename.ends_with(".pcap") || a.filename.ends_with(".pcapng"))
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(DiscordError {
                    error: "No PCAP attachments found in message".to_string(),
                }),
            )
        })?;

    // Download the attachment
    let pcap_data = discord::download_attachment(&pcap_attachment.url)
        .await
        .map_err(|(status, error)| (status, Json(DiscordError { error })))?;

    info!(
        "Successfully fetched PCAP from Discord: {} ({} bytes)",
        pcap_attachment.filename,
        pcap_data.len()
    );

    Ok(pcap_data)
}
