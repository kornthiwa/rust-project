#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BotCommand {
    Ping,
    Help,
    Unknown,
}

impl BotCommand {
    pub fn from_slash_name(name: &str) -> Self {
        match name.trim().to_ascii_lowercase().as_str() {
            "ping" => Self::Ping,
            "help" => Self::Help,
            _ => Self::Unknown,
        }
    }
}
