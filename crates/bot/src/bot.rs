//! Discord bot handler for event-driven interactions

use serenity::prelude::*;
use serenity::async_trait;
use serenity::model::prelude::*;
use tracing::{info, debug};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Bot connected as: {}", ready.user.name);
    }

    async fn message(&self, _ctx: Context, msg: Message) {
        debug!(
            "Message received in {}: {}",
            msg.channel_id, msg.content
        );
        
        // You can add bot commands/reactions here later
        // For now, just log messages for debugging
    }
}

/// Start the Discord bot
pub async fn start_bot(token: String) -> Result<(), Box<dyn std::error::Error>> {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await?;

    info!("Starting Discord bot...");
    client.start().await?;

    Ok(())
}
