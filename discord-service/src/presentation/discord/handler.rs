use serenity::{
    async_trait,
    all::{
        Context,
        EventHandler,
        Ready,
        Interaction,
        Command,
        CreateCommand,
        CreateCommandOption,
        CommandOptionType,
        CreateInteractionResponse,
        CreateInteractionResponseMessage,
        GuildId,
    },
};

use crate::application::use_cases::register_guild::RegisterGuildUseCase;
use crate::application::use_cases::add_manga::AddMangaUseCase;
use crate::infrastructure::memory::{
    guild_repo::InMemoryGuildRepo,
    manga_repo::InMemoryMangaRepo,
};

pub struct DiscordHandler {
    register_uc: RegisterGuildUseCase<InMemoryGuildRepo>,
    add_manga_uc: AddMangaUseCase<InMemoryMangaRepo>,
}

impl DiscordHandler {
    pub fn new(
        register_uc: RegisterGuildUseCase<InMemoryGuildRepo>,
        add_manga_uc: AddMangaUseCase<InMemoryMangaRepo>,
    ) -> Self {
        Self {
            register_uc,
            add_manga_uc,
        }
    }
}

#[async_trait]
impl EventHandler for DiscordHandler {

    async fn ready(&self, ctx: Context, ready: Ready) {
        let commands = vec![
            CreateCommand::new("register")
                .description("Register this discord server"),

                CreateCommand::new("add_manga")
                .description("Add manga to system")
            
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "site",
                        "Select manga website",
                    )
                    .add_string_choice("GoManga", "gomanga")
                    .add_string_choice("SingManga", "singmanga")
                    .required(true),
                )
            
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "url",
                        "manga url",
                    )
                    .required(true),
                ),
        ];

        if let Err(e) = Command::set_global_commands(&ctx.http, commands).await {
            eprintln!("failed to register commands: {e}");
        }

        println!("bot ready: {}", ready.user.name);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Interaction::Command(cmd) = interaction else {
            return;
        };

        // -----------------------------
        // register command
        // -----------------------------
        if cmd.data.name == "register" {
            let guild_id = cmd
                .guild_id
                .map(|g: GuildId| g.to_string())
                .unwrap_or_else(|| "no-guild".to_string());

            let channel_id = cmd.channel_id.to_string();

            let response = self
                .register_uc
                .execute(guild_id, channel_id)
                .await;

            let _ = cmd
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(response),
                    ),
                )
                .await;

            return;
        }

        // -----------------------------
        // add_manga command
        // -----------------------------
        if cmd.data.name == "add_manga" {
            let site = cmd.data.options
                .get(0)
                .and_then(|o| o.value.as_str())
                .unwrap_or("custom");
        
            let url = cmd.data.options
                .get(1)
                .and_then(|o| o.value.as_str())
                .unwrap_or("")
                .to_string();
        
            let response = self
                .add_manga_uc
                .execute(site, "unknown".to_string(), url) 
                .await;
        
            let _ = cmd.create_response(
                &ctx.http,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(response),
                ),
            ).await;
        }
    }
}