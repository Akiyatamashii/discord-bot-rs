use std::{io, sync::Arc};

use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue,
};
use tokio::sync::RwLock;

use crate::modules::tiktok_refuse::add_tiktok_refuse_msg;

// 註冊 tiktok_refuse 命令
pub fn register() -> CreateCommand {
    CreateCommand::new("tiktok_msg_add")
        .description("拒絕 TikTok 連結")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "message", "訊息：拒絕訊息")
                .required(true),
        )
}

pub async fn run<'a>(
    options: &[ResolvedOption<'a>],
    tiktok_refuse_msg: Arc<RwLock<Vec<String>>>,
) -> Result<String, io::Error> {
    let message = options.iter().find(|opt| opt.name == "message");
    let msg = if let Some(msg) = message {
        if let ResolvedValue::String(message) = msg.value {
            message
        } else {
            &"未輸入拒絕訊息".to_string()
        }
    } else {
        &"未輸入拒絕訊息".to_string()
    };

    let tiktok_refuse_msg = Arc::clone(&tiktok_refuse_msg);

    if let Err(err) = add_tiktok_refuse_msg(msg, tiktok_refuse_msg).await {
        println!("Add TikTok Refuse Msg Error: {}", err);
        return Ok("增加拒絕訊息失敗".to_string());
    }

    Ok("增加拒絕訊息成功".to_string())
}
