use crate::application::error::AppError;
use crate::application::services::{AddMangaResult, AppServices};
use serenity::all::{
    ChannelId, Colour, CommandDataOptionValue, CommandInteraction, CommandOptionType, Context,
    CreateCommand, CreateCommandOption,
    CreateEmbed, CreateEmbedFooter, CreateInteractionResponse, CreateInteractionResponseMessage,
};
use std::sync::Arc;

pub fn register_commands() -> Vec<CreateCommand> {
    vec![register_manga_command(), register_channel_command()]
}

pub async fn run_command(
    ctx: &Context,
    command: &CommandInteraction,
    services: Arc<AppServices>,
) -> serenity::Result<()> {
    match command.data.name.as_str() {
        "manga" => run_manga_command(ctx, command, services).await,
        "channel" => run_channel_command(ctx, command, services).await,
        _ => respond_embed(ctx, command, "ไม่รู้จักคำสั่ง", "ไม่พบคำสั่งนี้", Colour::RED).await,
    }
}

fn register_manga_command() -> CreateCommand {
    CreateCommand::new("manga")
        .description("จัดการการ์ตูน")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "add", "เพิ่มการ์ตูนใหม่")
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "url",
                        "URL ของการ์ตูนที่ต้องการเพิ่ม",
                    )
                    .required(true),
                ),
        )
}

fn register_channel_command() -> CreateCommand {
    CreateCommand::new("channel")
        .description("จัดการช่อง")
        .add_option(
            CreateCommandOption::new(CommandOptionType::SubCommand, "register", "บันทึกข้อมูลช่อง")
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::Channel,
                        "channel",
                        "ช่องที่ต้องการบันทึก",
                    )
                    .required(true),
                ),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            "list",
            "ดูรายการช่องทั้งหมด",
        ))
}

async fn run_manga_command(
    ctx: &Context,
    command: &CommandInteraction,
    services: Arc<AppServices>,
) -> serenity::Result<()> {
    let subcommand_name = command
        .data
        .options
        .first()
        .map(|o| o.name.as_str())
        .unwrap_or_default();
    if subcommand_name != "add" {
        return respond_embed(ctx, command, "ไม่รู้จักคำสั่ง", "ไม่รู้จักคำสั่งย่อยนี้", Colour::RED)
            .await;
    }

    let url = command
        .data
        .options
        .iter()
        .find(|opt| opt.name == "add")
        .and_then(|opt| match &opt.value {
            CommandDataOptionValue::SubCommand(sub_opts) => Some(sub_opts),
            _ => None,
        })
        .and_then(|sub_opts| sub_opts.iter().find(|opt| opt.name == "url"))
        .and_then(|opt| opt.value.as_str())
        .unwrap_or_default();

    match services.manga_service.add_from_url(url).await {
        Ok(AddMangaResult::AlreadyExists) => {
            respond_embed(
                ctx,
                command,
                "การเพิ่มการ์ตูน",
                "การ์ตูนนี้มีอยู่ในระบบแล้ว",
                Colour::GOLD,
            )
            .await
        }
        Ok(AddMangaResult::Created(manga)) => {
            let description = format!(
                "**เพิ่มการ์ตูนสำเร็จ**\n**ชื่อเรื่อง:** {}\n**ตอนล่าสุด:** {}\n**URL:** {}",
                manga.title, manga.latest_chapter, manga.url
            );
            respond_embed(
                ctx,
                command,
                "เพิ่มการ์ตูนสำเร็จ",
                &description,
                Colour::DARK_GREEN,
            )
            .await
        }
        Err(error) => {
            respond_embed(ctx, command, "เกิดข้อผิดพลาด", &render_error(&error), Colour::RED).await
        }
    }
}

async fn run_channel_command(
    ctx: &Context,
    command: &CommandInteraction,
    services: Arc<AppServices>,
) -> serenity::Result<()> {
    let subcommand_name = command
        .data
        .options
        .first()
        .map(|o| o.name.as_str())
        .unwrap_or_default();

    match subcommand_name {
        "register" => register_channel(ctx, command, services).await,
        "list" => list_channels(ctx, command, services).await,
        _ => respond_embed(ctx, command, "ไม่รู้จักคำสั่ง", "ไม่รู้จักคำสั่งย่อยนี้", Colour::RED).await,
    }
}

async fn register_channel(
    ctx: &Context,
    command: &CommandInteraction,
    services: Arc<AppServices>,
) -> serenity::Result<()> {
    let channel_id: ChannelId = match command
        .data
        .options
        .iter()
        .find(|opt| opt.name == "register")
        .and_then(|opt| match &opt.value {
            CommandDataOptionValue::SubCommand(sub_opts) => Some(sub_opts),
            _ => None,
        })
        .and_then(|sub_opts| sub_opts.iter().find(|opt| opt.name == "channel"))
        .and_then(|opt| opt.value.as_channel_id())
    {
        Some(value) => value,
        None => {
            return respond_embed(
                ctx,
                command,
                "ข้อมูลไม่ครบ",
                "กรุณาเลือกช่องที่ต้องการบันทึก",
                Colour::RED,
            )
            .await;
        }
    };

    let guild_id = match command.guild_id {
        Some(value) => value,
        None => {
            return respond_embed(
                ctx,
                command,
                "ไม่รองรับ",
                "คำสั่งนี้ใช้ได้เฉพาะในเซิร์ฟเวอร์เท่านั้น",
                Colour::RED,
            )
            .await;
        }
    };

    let guild = match ctx.http.get_guild(guild_id).await {
        Ok(value) => value,
        Err(error) => {
            return respond_embed(
                ctx,
                command,
                "เกิดข้อผิดพลาด",
                &format!("ดึงข้อมูลเซิร์ฟเวอร์ไม่สำเร็จ: {error}"),
                Colour::RED,
            )
            .await;
        }
    };
    let channel = match channel_id.to_channel(&ctx.http).await {
        Ok(value) => value,
        Err(error) => {
            return respond_embed(
                ctx,
                command,
                "เกิดข้อผิดพลาด",
                &format!("ไม่พบช่องที่ระบุ: {error}"),
                Colour::RED,
            )
            .await;
        }
    };

    let guild_channel = match channel.guild() {
        Some(value) => value,
        None => {
            return respond_embed(
                ctx,
                command,
                "เกิดข้อผิดพลาด",
                "ไม่สามารถบันทึกข้อมูลช่องส่วนตัวได้",
                Colour::RED,
            )
            .await;
        }
    };

    match services
        .channel_service
        .register_channel(
            guild_id.to_string(),
            guild.name,
            guild_channel.id.to_string(),
            guild_channel.name,
        )
        .await
    {
        Ok(_) => {
            respond_embed(
                ctx,
                command,
                "บันทึกข้อมูลสำเร็จ",
                "บันทึกหรืออัปเดตข้อมูลช่องเรียบร้อย",
                Colour::DARK_GREEN,
            )
            .await
        }
        Err(error) => {
            respond_embed(ctx, command, "เกิดข้อผิดพลาด", &render_error(&error), Colour::RED).await
        }
    }
}

async fn list_channels(
    ctx: &Context,
    command: &CommandInteraction,
    services: Arc<AppServices>,
) -> serenity::Result<()> {
    let guild_id = match command.guild_id {
        Some(value) => value,
        None => {
            return respond_embed(
                ctx,
                command,
                "ไม่รองรับ",
                "คำสั่งนี้ใช้ได้เฉพาะในเซิร์ฟเวอร์เท่านั้น",
                Colour::RED,
            )
            .await;
        }
    };

    match services.channel_service.list_channels(&guild_id.to_string()).await {
        Ok(channels) if channels.is_empty() => {
            respond_embed(
                ctx,
                command,
                "รายการช่อง",
                "ไม่พบช่องที่บันทึกไว้ในระบบ",
                Colour::RED,
            )
            .await
        }
        Ok(channels) => {
            let description = channels
                .iter()
                .map(|channel| format!("**{}** ({})", channel.channel_name, channel.channel_id))
                .collect::<Vec<_>>()
                .join("\n");
            respond_embed(ctx, command, "รายการช่องทั้งหมด", &description, Colour::BLUE).await
        }
        Err(error) => {
            respond_embed(ctx, command, "เกิดข้อผิดพลาด", &render_error(&error), Colour::RED).await
        }
    }
}

async fn respond_embed(
    ctx: &Context,
    command: &CommandInteraction,
    title: &str,
    description: &str,
    color: Colour,
) -> serenity::Result<()> {
    let embed = CreateEmbed::new()
        .title(title)
        .description(description)
        .color(color)
        .footer(CreateEmbedFooter::new("discord-service"));
    command
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .embed(embed)
                    .ephemeral(true),
            ),
        )
        .await
}

fn render_error(error: &AppError) -> String {
    match error {
        AppError::Validation(message) | AppError::NotFound(message) => message.clone(),
        AppError::Infrastructure(_) => "เกิดข้อผิดพลาดภายในระบบ".to_string(),
    }
}
