use crate::{
    application::use_cases::handle_message::HandleSlashCommandUseCase,
};
use serenity::{
    all::{
        Command, CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
        Interaction, Ready,
    },
    async_trait,
    client::{Context as SerenityContext, EventHandler},
};
use std::sync::Arc;

pub struct DiscordEventHandler {
    use_case: Arc<HandleSlashCommandUseCase>,
}

impl DiscordEventHandler {
    pub fn new(use_case: HandleSlashCommandUseCase) -> Self {
        Self {
            use_case: Arc::new(use_case),
        }
    }
}

#[async_trait]
impl EventHandler for DiscordEventHandler {
    async fn ready(&self, ctx: SerenityContext, ready: Ready) {
        let commands = vec![
            CreateCommand::new("ping").description("Check bot latency"),
            CreateCommand::new("help").description("Show available commands"),
        ];
        if let Err(error) = Command::set_global_commands(&ctx.http, commands).await {
            eprintln!("failed to register slash commands: {error}");
        }

        println!("{} is connected", ready.user.name);
    }

    async fn interaction_create(&self, ctx: SerenityContext, interaction: Interaction) {
        let Interaction::Command(command) = interaction else {
            return;
        };

        let response = self.use_case.execute(&command.data.name);
        if let Err(error) = command
            .create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(response),
                ),
            )
            .await
        {
            eprintln!("failed to process interaction: {error}");
        }
    }
}
