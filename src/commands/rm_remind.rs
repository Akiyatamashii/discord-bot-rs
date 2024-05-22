use std::collections::HashMap;
use std::sync::Arc;

use serenity::{
    all::{CommandOptionType, ResolvedOption, ResolvedValue},
    builder::{CreateCommand, CreateCommandOption},
    model::id::ChannelId,
};
use tokio::sync::RwLock;

use crate::modules::func::save_reminders_to_file;
use crate::Reminder;

pub fn register() -> CreateCommand {
    CreateCommand::new("rm_remind")
        .description("移除指定的提醒")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                "index",
                "提醒的索引：請參照\"/look\"產生的索引",
            )
            .required(true),
        )
}

pub async fn run<'a>(
    options: &'a [ResolvedOption<'a>],
    reminders: Arc<RwLock<HashMap<ChannelId, Vec<Reminder>>>>,
    channel_id: ChannelId,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let index_option = options.iter().find(|opt| opt.name == "index");

    if let Some(index_option) = index_option {
        if let ResolvedValue::Integer(index) = index_option.value {
            let index = index - 1;
            {
                let mut reminders_lock = reminders.write().await;
                if let Some(reminder_list) = reminders_lock.get_mut(&channel_id) {
                    if (index as usize) < reminder_list.len() {
                        reminder_list.remove(index as usize);
                        save_reminders_to_file(&*reminders_lock).unwrap();
                        return Ok(format!("提醒索引 '{}' 已移除", index + 1));
                    } else {
                        return Ok(format!("索引 '{}' 無效", index + 1));
                    }
                } else {
                    return Ok("該頻道沒有設置任何提醒".to_string());
                }
            }
        }
    }

    Ok("未提供有效的索引".to_string())
}
