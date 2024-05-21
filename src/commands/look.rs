use std::collections::HashMap;

use serenity::{builder::CreateCommand, model::prelude::ChannelId};

use crate::modules::func::load_reminders_from_file;
use crate::Reminder;

pub fn register() -> CreateCommand {
    CreateCommand::new("look").description("查看當前設置的提醒")
}

pub fn run() -> String {
    match load_reminders_from_file() {
        Ok(reminders) => format_reminders(reminders),
        Err(err) => format!("Failed to load reminders: {}", err),
    }
}

fn format_reminders(reminders: HashMap<ChannelId, Vec<Reminder>>) -> String {
    let mut output = String::new();
    for (channel_id, reminder_list) in reminders {
        output.push_str(&format!("V 頻道 ID: {} V\n", channel_id));
        if !reminder_list.is_empty() {
            for (index, reminder) in reminder_list.iter().enumerate() {
                output.push_str(&format!(
                    "{}. 週期: {:?}, 時間: {}, 訊息: {}\n",
                    index, reminder.weekdays, reminder.time, reminder.message
                ));
            }
        } else {
            output.push_str(">>尚無提醒")
        }
        output.push_str("\n");
    }
    output
}
