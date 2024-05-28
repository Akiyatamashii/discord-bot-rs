use std::collections::HashMap;

use serenity::all::GuildId;
use serenity::{builder::CreateCommand, model::prelude::ChannelId};

use crate::modules::func::load_reminders_from_file;
use crate::Reminder;

pub fn register() -> CreateCommand {
    CreateCommand::new("look").description("查看當前設置的提醒")
}

pub fn run(guild_id: GuildId, channel_id: ChannelId) -> String {
    match load_reminders_from_file() {
        Ok(reminders) => format_reminders(reminders, guild_id, channel_id),
        Err(err) => format!("Failed to load reminders: {}", err),
    }
}

fn format_reminders(
    reminders: HashMap<GuildId, HashMap<ChannelId, Vec<Reminder>>>,
    o_guild_id: GuildId,
    o_channel_id: ChannelId,
) -> String {
    let mut output = String::new();

    if let Some(guild_reminders) = reminders.get(&o_guild_id) {
        // 先處理當前頻道的提醒
        if let Some(current_channel_reminders) = guild_reminders.get(&o_channel_id) {
            output.push_str(&format!("V 頻道 ID: {} (當前頻道) V\n", o_channel_id));
            if !current_channel_reminders.is_empty() {
                for (index, reminder) in current_channel_reminders.iter().enumerate() {
                    output.push_str(&format!(
                        "{}. 週期: {:?}, 時間: {}, 訊息: {}\n",
                        index, reminder.weekdays, reminder.time, reminder.message
                    ));
                }
            } else {
                output.push_str(">> 尚未新增提醒\n");
            }
            output.push_str("\n");
        } else {
            output.push_str(">> 當前頻道尚未新增提醒\n\n");
        }

        // 再處理其他頻道的提醒
        for (channel_id, reminder_list) in guild_reminders {
            if *channel_id != o_channel_id {
                output.push_str(&format!("V 頻道 ID: {} V\n", channel_id));
                if !reminder_list.is_empty() {
                    for (index, reminder) in reminder_list.iter().enumerate() {
                        output.push_str(&format!(
                            "{}. 週期: {:?}, 時間: {}, 訊息: {}\n",
                            index, reminder.weekdays, reminder.time, reminder.message
                        ));
                    }
                } else {
                    output.push_str(">> 尚未新增提醒\n");
                }
                output.push_str("\n");
            }
        }
    } else {
        output.push_str(">> 該群組尚未新增提醒\n");
    }

    output
}
