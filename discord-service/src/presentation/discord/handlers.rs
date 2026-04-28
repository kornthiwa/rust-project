use crate::application::services::AppServices;
use crate::presentation::discord::commands;
use crate::presentation::discord::notifier::DiscordNotifier;
use serenity::all::{
    Command, Context, CreateInteractionResponse, CreateInteractionResponseMessage, EventHandler,
    Interaction, Ready,
};
use serenity::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct DiscordEventHandler {
    services: Arc<AppServices>,
    auto_update_started: AtomicBool,
}

impl DiscordEventHandler {
    pub fn new(services: Arc<AppServices>) -> Self {
        Self {
            services,
            auto_update_started: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            if command.data.options.is_empty() {
                let _ = command
                    .create_response(
                        &ctx.http,
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("ไม่พบคำสั่งย่อย กรุณาระบุตัวเลือก")
                                .ephemeral(true),
                        ),
                    )
                    .await;
                return;
            }

            if let Err(error) = commands::run_command(&ctx, &command, self.services.clone()).await {
                eprintln!("command failed: {error}");
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} เชื่อมต่อแล้ว!", ready.user.name);
        let commands = commands::register_commands();
        if let Err(error) = Command::set_global_commands(&ctx.http, commands).await {
            eprintln!("register commands failed: {error}");
        }

        if !self.auto_update_started.swap(true, Ordering::SeqCst) {
            let notifier = Arc::new(DiscordNotifier::new(
                self.services.channel_repository.clone(),
                ctx.clone(),
            ));
            let auto_update = self.services.auto_update_service.clone_with_notifier(notifier);
            tokio::spawn(async move {
                auto_update.run_periodic_update().await;
            });
        }
    }
}
