use std::{io, sync::Arc};

use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue,
};
use tokio::sync::RwLock;

use crate::modules::anti_tiktok::add_tiktok_refuse_msg;

// Register the tiktok_msg_add command
// 註冊 tiktok_msg_add 命令
pub fn register() -> CreateCommand {
    CreateCommand::new("tiktok_msg_add")
        .description("拒絕 TikTok 連結")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "message", "訊息：拒絕訊息")
                .required(true),
        )
}

// Run the command to add a new TikTok refuse message
// 運行命令以添加新的 TikTok 拒絕消息
pub async fn run<'a>(
    options: &[ResolvedOption<'a>],
    tiktok_refuse_msg: Arc<RwLock<Vec<String>>>,
) -> Result<String, io::Error> {
    // Find the option named "message" from the input options
    // 從選項中找到名為 "message" 的選項
    let message = options.iter().find(|opt| opt.name == "message");
    
    // Parse the message content, use default message if not found or parsing fails
    // 解析消息內容，如果沒有找到或解析失敗，則使用默認消息
    let msg = if let Some(msg) = message {
        if let ResolvedValue::String(message) = msg.value {
            message.to_string()
        } else {
            "未輸入拒絕訊息".to_string()
        }
    } else {
        "未輸入拒絕訊息".to_string()
    };

    // Clone the Arc to use in the async context
    // 克隆 Arc 以在異步上下文中使用
    let tiktok_refuse_msg = Arc::clone(&tiktok_refuse_msg);

    // Try to add the refuse message
    // 嘗試添加拒絕消息
    if let Err(err) = add_tiktok_refuse_msg(&msg, tiktok_refuse_msg).await {
        // If adding fails, print the error and return a failure message
        // 如果添加失敗，打印錯誤並返回失敗消息
        println!("Add TikTok Refuse Msg Error: {}", err);
        return Ok("增加拒絕訊息失敗".to_string());
    }

    // If successful, return a success message
    // 如果成功，返回成功消息
    Ok("增加拒絕訊息成功".to_string())
}
