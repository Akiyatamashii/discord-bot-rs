use serenity::{
    all::{CommandOptionType, GuildId, ResolvedOption, ResolvedValue},
    builder::{CreateCommand, CreateCommandOption},
    model::id::ChannelId,
};

use crate::{modules::func::save_reminders_to_file, Reminders};

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
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "channel_id",
            "頻道ID請至 /look 查看",
        ))
}

pub async fn run<'a>(
    options: &'a [ResolvedOption<'a>],
    reminders: Reminders,
    channel_id: ChannelId,
    guild_id: GuildId,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let index_option = options.iter().find(|opt| opt.name == "index");
    let channel_id_option = options.iter().find(|opt| opt.name == "channel_id");

    let rm_channel_id = if let Some(channel_id_option) = channel_id_option {
        if let ResolvedValue::String(channel_id_str) = channel_id_option.value {
            if let Ok(channel_id_parse) = channel_id_str.parse::<u64>() {
                ChannelId::new(channel_id_parse)
            } else {
                channel_id
            }
        } else {
            channel_id
        }
    } else {
        channel_id
    };

    if let Some(index_option) = index_option {
        if let ResolvedValue::Integer(index) = index_option.value {
            let index = index - 1;
            {
                let mut reminders_lock = reminders.write().await;
                if let Some(guild_reminder) = reminders_lock.get_mut(&guild_id) {
                    if let Some(reminder_list) = guild_reminder.get_mut(&rm_channel_id) {
                        if (index as usize) < reminder_list.len() {
                            reminder_list.remove(index as usize);
                            // 如果頻道內的提醒列表為空，則移除該頻道 ID
                            if reminder_list.is_empty() {
                                guild_reminder.remove(&rm_channel_id);
                            }
                            // 如果公會內的提醒列表為空，則移除該公會 ID
                            if guild_reminder.is_empty() {
                                reminders_lock.remove(&guild_id);
                            }
                            save_reminders_to_file(&reminders_lock).unwrap();
                            return Ok(format!("提醒索引 '{}' 已移除", index + 1));
                        } else {
                            return Ok(format!("索引 '{}' 無效", index + 1));
                        }
                    } else {
                        return Ok("該頻道沒有設置任何提醒".to_string());
                    }
                } else {
                    return Ok("該公會沒有設置任何提醒".to_string());
                }
            }
        }
    }

    Ok("未提供有效的索引".to_string())
}
