use std::{collections::HashMap, error::Error, sync::Arc};

use chrono::{NaiveTime, Weekday};
use serenity::{
    all::{GuildId, ResolvedValue},
    builder::{CreateCommand, CreateCommandOption},
    model::{
        application::{CommandOptionType, ResolvedOption},
        id::ChannelId,
    },
};
use tokio::sync::Notify;

use crate::Reminder;
use crate::{modules::func::save_reminders_to_file, Reminders};

// 註冊 remind 命令
pub fn register() -> CreateCommand {
    CreateCommand::new("remind")
        .description("設置提醒")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "weekdays",
                "日期：需要提醒的日期，以 \"1,4,7\" 格式表示",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "time",
                "時間：提醒時間，以 \"HH:MM\" 格式表示",
            )
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "message", "提醒訊息")
                .required(true),
        )
}

// 執行 remind 命令的主函數
pub async fn run<'a>(
    options: &'a [ResolvedOption<'a>],
    reminder: Reminders,
    channel_id: ChannelId,
    guild_id: GuildId,
    notify: &Arc<Notify>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    // 從選項中獲取 weekdays、time 和 message 的值
    let weekdays = options
        .iter()
        .find(|opt| opt.name == "weekdays")
        .and_then(|opt| match &opt.value {
            ResolvedValue::String(s) => Some(s),
            _ => None,
        })
        .unwrap_or(&"");

    let time = options
        .iter()
        .find(|opt| opt.name == "time")
        .and_then(|opt| match &opt.value {
            ResolvedValue::String(s) => Some(s),
            _ => None,
        })
        .unwrap_or(&"");

    let message = options
        .iter()
        .find(|opt| opt.name == "message")
        .and_then(|opt| match &opt.value {
            ResolvedValue::String(s) => Some(s),
            _ => None,
        })
        .unwrap_or(&"");

    // 解析星期幾
    let weekdays_result: Result<Vec<Weekday>, _> = weekdays
        .split(',')
        .map(|s| s.trim().parse::<u32>())
        .map(|day_result| {
            day_result.map(|day| match day {
                1 => Weekday::Mon,
                2 => Weekday::Tue,
                3 => Weekday::Wed,
                4 => Weekday::Thu,
                5 => Weekday::Fri,
                6 => Weekday::Sat,
                7 => Weekday::Sun,
                _ => Weekday::Mon, // 默認為星期一
            })
        })
        .collect();

    let weekdays = match weekdays_result {
        Ok(days) => days,
        Err(_) => return Err(">> 錯誤的日期格式：ex. 1,2,3".into()),
    };

    // 解析時間
    let time = match NaiveTime::parse_from_str(time.trim(), "%H:%M") {
        Ok(t) => t,
        Err(_) => return Err(">> 錯誤的時間格式(24小時制)：ex. 01:24 or 23:34".into()),
    };

    let reminder_message = message.to_string();

    // 添加新的提醒
    {
        let mut reminders = reminder.write().await;
        let guild_reminder = reminders.entry(guild_id).or_insert_with(HashMap::new);
        let channel_reminder = guild_reminder.entry(channel_id).or_insert_with(Vec::new);
        channel_reminder.push(Reminder {
            weekdays,
            time,
            message: reminder_message,
            last_executed: None,
        });
        // 保存提醒到文件
        save_reminders_to_file(&reminders).expect("Failed to save reminders");
    }

    // 通知提醒處理器有新的提醒
    notify.notify_one();

    Ok(">> 已設定每週提醒".to_string())
}
