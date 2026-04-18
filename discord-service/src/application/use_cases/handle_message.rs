use crate::domain::command::BotCommand;

pub struct HandleSlashCommandUseCase;

impl HandleSlashCommandUseCase {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, command_name: &str) -> &'static str {
        let command = BotCommand::from_slash_name(command_name);
        let response = match command {
            BotCommand::Ping => "pong",
            BotCommand::Help => "Available commands: /ping, /help",
            BotCommand::Unknown => "Unknown command. Use /help",
        };
        response
    }
}
