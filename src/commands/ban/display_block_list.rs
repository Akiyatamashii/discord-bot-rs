use serenity::all::{CommandInteraction, Context, CreateCommand};

use crate::{modules::func::check_permission, FraudBotList};

pub fn register() -> CreateCommand {
    CreateCommand::new("display_block_list")
        .description("Display block list")
        .description_localized("zh-TW", "顯示黑名單")
}

pub async fn run(ctx: &Context, command: &CommandInteraction, list: FraudBotList) -> String {
    if !check_permission(ctx, command).await {
        return "你沒有許可權使用指令".to_string();
    }

    let get_list = list.read().await.clone();
    let mut list_text: Vec<_> = get_list.iter().collect();
    list_text.sort();
    let msg = list_text
        .iter()
        .map(|id| format!("<@{}>", id))
        .collect::<Vec<_>>()
        .join("\n");
    if msg.is_empty() {
        return "黑名單是空的".to_string();
    }
    msg
}
