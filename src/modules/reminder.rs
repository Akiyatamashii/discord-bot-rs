// Import necessary modules and dependencies
// 導入必要的模組和依賴
use crate::modules::func::{error_output, save_reminders_to_file, system_output};
use crate::{Reminder, Reminders};
use chrono::{Datelike, NaiveTime, Timelike, Utc};
use chrono_tz::Tz;
use colored::Colorize;
use once_cell::sync::Lazy;
use serenity::all::{ChannelId, Http};
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::{
    sync::RwLock,
    time::{interval, Duration},
};

// Define Taipei timezone
// 定義台北時區
static TW: Lazy<Tz> = Lazy::new(|| "Asia/Taipei".parse().unwrap());

// Define reminder storage structure
// 定義提醒存儲結構
#[derive(Clone, Default)]
struct ReminderStore {
    reminders_30_min: Arc<RwLock<Vec<(ChannelId, Reminder)>>>,
    reminders_2_min: Arc<RwLock<Vec<(ChannelId, Reminder)>>>,
    two_min_checking: Arc<RwLock<bool>>,
    one_secs_checking: Arc<RwLock<bool>>,
}

impl ReminderStore {
    // Create a new ReminderStore instance
    // 創建一個新的 ReminderStore 實例
    fn new() -> Self {
        ReminderStore {
            two_min_checking: Arc::new(RwLock::new(false)),
            one_secs_checking: Arc::new(RwLock::new(false)),
            ..Default::default()
        }
    }
}

// Main reminder task function
// 主要的提醒任務函數
pub async fn remind_task(http: Arc<Http>, reminders: Reminders, notify: Arc<Notify>) {
    println!(
        "{} {}",
        system_output(),
        "Reminder remind_task start".green()
    );

    // Set up interval for checking reminders every 30 minutes
    // 設置每30分鐘檢查一次提醒的間隔
    let mut wait_time = interval(Duration::from_secs(1800));
    let reminder_store = Arc::new(ReminderStore::new());

    loop {
        tokio::select! {
            _ = wait_time.tick() => {
                process_reminders(&reminders, &reminder_store).await;

                // Start 2-minute check if there are reminders within 30 minutes
                // 如果30分鐘內有提醒，開始2分鐘檢查
                if !reminder_store.reminders_30_min.read().await.is_empty()
                    && !*reminder_store.two_min_checking.read().await
                {
                    *reminder_store.two_min_checking.write().await = true;
                    tokio::spawn(check_2min_remind(
                        Arc::clone(&http),
                        Arc::clone(&reminder_store),
                    ));
                }
            }
            _ = notify.notified() => {
                // Process reminders immediately when a new reminder is added
                // 當有新的提醒被添加時，立即處理
                process_reminders(&reminders, &reminder_store).await;

                // Start 2-minute check if there are reminders within 30 minutes
                // 如果30分鐘內有提醒，開始2分鐘檢查
                if !reminder_store.reminders_30_min.read().await.is_empty()
                    && !*reminder_store.two_min_checking.read().await
                {
                    *reminder_store.two_min_checking.write().await = true;
                    tokio::spawn(check_2min_remind(
                        Arc::clone(&http),
                        Arc::clone(&reminder_store),
                    ));
                }
            }
        }
    }
}

// Process reminders function
// 處理提醒的函數
async fn process_reminders(reminders: &Reminders, reminder_store: &Arc<ReminderStore>) {
    let now = Utc::now().with_timezone(&*TW);
    let target_time = now + chrono::Duration::minutes(30);
    let handler_reminder = Arc::clone(reminders);
    {
        let mut guild_reminders_map = handler_reminder.write().await;
        for (_guild_id, reminders_map) in guild_reminders_map.iter_mut() {
            for (channel_id, reminders) in reminders_map.iter_mut() {
                for reminder in reminders.iter_mut() {
                    // Check if the reminder needs to be executed within the next 30 minutes
                    // 檢查提醒是否在接下來的30分鐘內需要執行
                    if reminder.weekdays.contains(&now.weekday())
                        && reminder.time > now.time()
                        && reminder.time <= target_time.time()
                        && reminder.last_executed != Some(now.date_naive())
                    {
                        reminder.last_executed = Some(now.date_naive());
                        let mut reminder_in_30min = reminder_store.reminders_30_min.write().await;
                        reminder_in_30min.push((*channel_id, reminder.clone()));
                    }
                }
            }
        }
        save_reminders_to_file(&guild_reminders_map).expect("Failed to save reminders");
    }
}

// Check reminders within 2 minutes
// 檢查2分鐘內的提醒
async fn check_2min_remind(http: Arc<Http>, remind_store: Arc<ReminderStore>) {
    let mut wait_time = interval(Duration::from_secs(120));
    loop {
        wait_time.tick().await;
        let now = Utc::now().with_timezone(&*TW);
        let target_time = now + chrono::Duration::minutes(2);
        let mut new_list = Vec::new();
        {
            let reminder_in_30min = remind_store.reminders_30_min.read().await;
            for (channel_id, reminder) in reminder_in_30min.iter() {
                if reminder.time > now.time() && reminder.time < target_time.time() {
                    let mut reminder_in_2min = remind_store.reminders_2_min.write().await;
                    reminder_in_2min.push((*channel_id, reminder.clone()));
                } else {
                    new_list.push((*channel_id, reminder.clone()));
                }
            }
        }
        *remind_store.reminders_30_min.write().await = new_list;

        // Start 1-second check if there are reminders within 2 minutes
        // 如果有2分鐘內的提醒，啟動1秒檢查
        if !remind_store.reminders_2_min.read().await.is_empty()
            && !*remind_store.one_secs_checking.read().await
        {
            *remind_store.one_secs_checking.write().await = true;
            tokio::spawn(check_1secs_remind(
                Arc::clone(&http),
                Arc::clone(&remind_store),
            ));
        }

        if remind_store.reminders_30_min.read().await.is_empty() {
            *remind_store.two_min_checking.write().await = false;
            break;
        }
    }
}

// Check reminders within 1 second and send them
// 檢查1秒內的提醒並發送
async fn check_1secs_remind(http: Arc<Http>, remind_store: Arc<ReminderStore>) {
    let mut wait_time = interval(Duration::from_secs(1));
    loop {
        wait_time.tick().await;
        let now = Utc::now().with_timezone(&*TW);
        let time = NaiveTime::from_hms_opt(now.hour(), now.minute(), now.second()).unwrap();
        let mut new_list = Vec::new();
        {
            let reminder_in_2min = remind_store.reminders_2_min.read().await;
            for (channel_id, reminder) in reminder_in_2min.iter() {
                if reminder.time == time {
                    // Send reminder message
                    // 發送提醒消息
                    if let Err(err) = channel_id.say(&http, &reminder.message).await {
                        println!("{} sending message: {:?}", error_output(), err);
                    }
                } else {
                    new_list.push((*channel_id, reminder.clone()));
                }
            }
        }
        *remind_store.reminders_2_min.write().await = new_list;

        if remind_store.reminders_2_min.read().await.is_empty() {
            *remind_store.one_secs_checking.write().await = false;
            break;
        }
    }
}