use serenity::{builder::CreateCommand, model::application::ResolvedOption};

pub fn run(_options: &[ResolvedOption]) -> String {
    "Pong!".to_string()
}

pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("A ping command")
}
