use serenity::{builder::CreateCommand, model::application::ResolvedOption};

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("連線測試")
}

pub fn run(_options: &[ResolvedOption]) -> String {
    "Pong!".to_string()
}
