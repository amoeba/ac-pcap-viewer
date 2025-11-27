use axum::{
    extract::Query,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;
use tracing::info;

mod discord;
mod bot;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
        )
        .init();

    // Get Discord bot token
    let discord_token = std::env::var("DISCORD_OAUTH_TOKEN")
        .expect("DISCORD_OAUTH_TOKEN environment variable is required");

    info!("Initializing with Discord bot token");

    // Spawn bot in background task
    let bot_token = discord_token.clone();
    tokio::spawn(async move {
        if let Err(e) = bot::start_bot(bot_token).await {
            tracing::error!("Bot error: {}", e);
        }
    });

    // Start web server
    let app = create_router();

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind listener");

    info!("Web server running on {}", addr);

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

fn create_router() -> Router {
    let dist_path = std::path::PathBuf::from("dist");

    Router::new()
        .route("/api/health", get(health))
        .route("/api/discord", get(discord_pull))
        .fallback_service(ServeDir::new(&dist_path))
}

/// Health check endpoint
async fn health() -> &'static str {
    "OK"
}

#[derive(Deserialize)]
struct DiscordQuery {
    #[serde(default)]
    channel: String,
    #[serde(default)]
    msg: String,
}

#[derive(Serialize)]
struct DiscordError {
    error: String,
}

/// Fetch PCAP from Discord message attachment
async fn discord_pull(
    Query(params): Query<DiscordQuery>,
) -> Result<Vec<u8>, (StatusCode, Json<DiscordError>)> {
    info!("Discord pull request: channel={}, msg={}", params.channel, params.msg);
    
    // Validate channel and message IDs
    if params.channel.is_empty() || params.msg.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(DiscordError {
                error: "Missing channel or msg parameters".to_string(),
            }),
        ));
    }

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
    let message = discord::fetch_message(&params.channel, &params.msg, &token)
        .await
        .map_err(|(status, error)| {
            (
                status,
                Json(DiscordError { error }),
            )
        })?;

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
        .map_err(|(status, error)| {
            (
                status,
                Json(DiscordError { error }),
            )
        })?;

    info!(
        "Successfully fetched PCAP from Discord: {} ({} bytes)",
        pcap_attachment.filename,
        pcap_data.len()
    );

    Ok(pcap_data)
}
