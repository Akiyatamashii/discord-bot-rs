use crate::modules::func::{error_output, save_reminders_to_file, system_output};
use crate::{Handler, Reminder};
use chrono::{Datelike, NaiveTime, Timelike, Utc};
use chrono_tz::Tz;
use colored::Colorize;
use serenity::all::{ChannelId, Http};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::RwLock,
    time::{interval, Duration},
};

pub async fn remind_task(
    http: Arc<Http>,
    reminders: Arc<RwLock<HashMap<ChannelId, Vec<Reminder>>>>,
    handler: Arc<Handler>,
) {
    println!(
        "{} {}",
        system_output(),
        "Reminder remind_task start".green()
    );
    let timezone: Tz = "Asia/Taipei".parse().unwrap();
    let cancel_notify = Arc::clone(&handler.cancel_notify);

    let mut second_check = interval(Duration::from_secs(1));

    loop {
        tokio::select! {
            _ = second_check.tick() => {
                check_reminders(http.clone(), reminders.clone(), timezone).await;
            },
            _ = cancel_notify.notified() => {
                println!("Task cancellation received, exiting task loop.");
                break;
            },
        }
    }
    println!("{} {}", system_output(), "remind_task stop".red());
}

async fn check_reminders(
    http: Arc<Http>,
    reminders: Arc<RwLock<HashMap<ChannelId, Vec<Reminder>>>>,
    timezone: Tz,
) {
    let now = Utc::now().with_timezone(&timezone);

    // 获取当前要发送的提醒
    let reminders_to_send = {
        let reminders_lock = reminders.read().await;
        let mut reminders_to_send = Vec::new();

        for (channel_id, reminder_list) in reminders_lock.iter() {
            for reminder in reminder_list.iter() {
                if reminder.weekdays.contains(&now.weekday())
                    && reminder.time
                        == NaiveTime::from_hms_opt(now.hour(), now.minute(), now.second()).unwrap()
                    && reminder.last_executed != Some(now.date_naive())
                {
                    reminders_to_send.push((channel_id.clone(), reminder.clone()));
                }
            }
        }

        reminders_to_send
    }; // 在此释放读锁

    // 发送提醒并更新提醒的最后执行时间
    for (channel_id, reminder) in reminders_to_send {
        if let Err(err) = channel_id.say(&http, &reminder.message).await {
            println!("{} sending message: {:?}", error_output(), err);
        }

        let mut reminders_lock = reminders.write().await;
        if let Some(reminders) = reminders_lock.get_mut(&channel_id) {
            if let Some(rem) = reminders
                .iter_mut()
                .find(|r| r.time == reminder.time && r.message == reminder.message)
            {
                rem.last_executed = Some(now.date_naive());
                if let Err(err) = save_reminders_to_file(&reminders_lock) {
                    println!("{} saving reminders: {:?}", error_output(), err);
                }
            }
        }
    }
}
