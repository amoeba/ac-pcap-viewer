//! Discord bot handler for event-driven interactions

use serenity::async_trait;
use serenity::model::prelude::*;
use serenity::prelude::*;
use tracing::{debug, error, info};

use crate::db::{CommandLog, Database};

pub struct Handler {
    pub web_url: String,
    pub db: Database,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("Bot connected as: {}", ready.user.name);
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages from bots (including ourselves)
        if msg.author.bot {
            return;
        }

        debug!(
            "Message received in {}: {} ({} attachments)",
            msg.channel_id,
            msg.content,
            msg.attachments.len()
        );

        // Check for PCAP attachments (must end with .pcap or .pcapng)
        let pcap_attachment = msg.attachments.iter().find(|a| {
            let filename = a.filename.to_lowercase();
            filename.ends_with(".pcap") || filename.ends_with(".pcapng")
        });

        if let Some(attachment) = pcap_attachment {
            info!(
                "PCAP attachment detected: {} in channel {} message {}",
                attachment.filename, msg.channel_id, msg.id
            );

            // Create web UI link with query parameters
            let web_link = format!("{}?channel={}&msg={}", self.web_url, msg.channel_id, msg.id);

            // Reply with the link
            let reply = format!(
                "📊 PCAP file detected: `{}`\n\n[View in PCAP Parser]({})",
                attachment.filename, web_link
            );

            let success = if let Err(e) = msg.reply(&ctx.http, reply).await {
                error!("Failed to send reply: {}", e);
                false
            } else {
                true
            };

            // Log command to database
            let log = CommandLog {
                command_name: "pcap_detect".to_string(),
                user_id: msg.author.id.to_string(),
                user_name: msg.author.name.clone(),
                channel_id: msg.channel_id.to_string(),
                guild_id: msg.guild_id.map(|id| id.to_string()),
                message_id: msg.id.to_string(),
                success,
                error_message: if success {
                    None
                } else {
                    Some("Failed to send reply".to_string())
                },
            };

            if let Err(e) = self.db.log_command(log).await {
                error!("Failed to log command to database: {}", e);
            }
        }
    }
}

/// Start the Discord bot
pub async fn start_bot(
    token: String,
    web_url: String,
    db: Database,
) -> Result<(), Box<dyn std::error::Error>> {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let handler = Handler { web_url, db };

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await?;

    info!("Starting Discord bot...");
    client.start().await?;

    Ok(())
}
