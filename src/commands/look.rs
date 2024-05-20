use serenity::builder::CreateCommand;
use serenity::model::prelude::ChannelId;
use std::{collections::HashMap, fs};

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

fn load_reminders_from_file(
) -> Result<HashMap<ChannelId, Vec<Reminder>>, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string("reminders.json")?;
    let reminders: HashMap<ChannelId, Vec<Reminder>> = serde_json::from_str(&file_content)?;
    Ok(reminders)
}

fn format_reminders(reminders: HashMap<ChannelId, Vec<Reminder>>) -> String {
    let mut output = String::new();
    for (channel_id, reminder_list) in reminders {
        output.push_str(&format!("V 頻道 ID: {} V\n", channel_id));
        for (index, reminder) in reminder_list.iter().enumerate() {
            output.push_str(&format!(
                "{}. 週期: {:?}, 時間: {}, 訊息: {}\n",
                index, reminder.weekdays, reminder.time, reminder.message
            ));
        }
        output.push_str("\n");
    }
    output
}
