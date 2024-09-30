use serenity::{builder::CreateCommand, model::application::ResolvedOption};

// Register the ping command
// 註冊 ping 命令
pub fn register() -> CreateCommand {
    CreateCommand::new("ping").description("連線測試")
}

// Run the ping command
// 運行 ping 命令
pub fn run(_options: &[ResolvedOption]) -> String {
    // Return a simple "Pong!" response
    // 返回簡單的 "Pong!" 回應
    "Pong!".to_string()
}
