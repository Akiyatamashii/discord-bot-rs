use chrono::{Datelike, NaiveTime, Timelike, Utc};
use chrono_tz::Tz;

use serenity::all::{ChannelId, Http};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

use crate::{Handler, Reminder};

pub async fn remind_task(
    http: Arc<Http>,
    reminders: Arc<RwLock<HashMap<ChannelId, Vec<Reminder>>>>,
    handler: Arc<Handler>,
) {
    let timezone: Tz = "Asia/Taipei".parse().unwrap();
    let mut thirty_min_check = interval(Duration::from_secs(1800));
    let cancel_notify = Arc::clone(&handler.cancel_notify);
    let trigger_notify = Arc::clone(&handler.trigger_notify);

    loop {
        tokio::select! {
            _ = thirty_min_check.tick() => {
                // 每30分鐘檢查一次提醒
                check_reminders(http.clone(), reminders.clone(), handler.clone(), timezone, 30).await;
            },
            _ = trigger_notify.notified() => {
                // 當新提醒被設定時，立即檢查提醒
                check_reminders(http.clone(), reminders.clone(), handler.clone(), timezone, 30).await;
            },
            _ = cancel_notify.notified() => {
                println!("Task cancellation received, exiting task loop.");
                break;
            },
        }
    }
}

async fn check_reminders(
    http: Arc<Http>,
    reminders: Arc<RwLock<HashMap<ChannelId, Vec<Reminder>>>>,
    handler: Arc<Handler>,
    timezone: Tz,
    minutes_ahead: i64,
) {
    let now = Utc::now().with_timezone(&timezone);
    let target_time = now + chrono::Duration::minutes(minutes_ahead);

    let reminders_lock = reminders.read().await;
    let mut upcoming_reminders: Vec<(ChannelId, Reminder)> = Vec::new();
    // println!("Checking 30m reminders at: {}", now);

    for (channel_id, reminder_list) in reminders_lock.iter() {
        for reminder in reminder_list.iter() {
            if reminder.weekdays.contains(&now.weekday())
                && reminder.time > now.time()
                && reminder.time <= target_time.time()
                && reminder.last_executed != Some(now.date_naive())
            {
                println!(
                    "Upcoming reminder for channel {}: {}",
                    channel_id, reminder.message
                ); // 新增除錯資訊
                upcoming_reminders.push((*channel_id, reminder.clone()));
            }
        }
    }

    if !upcoming_reminders.is_empty() {
        // 啟動每2分鐘和每秒檢查即將觸發的提醒
        start_imminent_check(
            http,
            reminders.clone(),
            handler.clone(),
            timezone,
            upcoming_reminders,
        )
        .await;
    }
}

async fn start_imminent_check(
    http: Arc<Http>,
    reminders: Arc<RwLock<HashMap<ChannelId, Vec<Reminder>>>>,
    handler: Arc<Handler>,
    timezone: Tz,
    upcoming_reminders: Vec<(ChannelId, Reminder)>,
) {
    let mut two_min_check = interval(Duration::from_secs(120));

    loop {
        two_min_check.tick().await;
        let now = Utc::now().with_timezone(&timezone);
        let target_time = now + chrono::Duration::minutes(2);
        // println!("Checking 2-min reminders at: {}", now);

        let mut imminent_reminders = Vec::new();
        for (channel_id, reminder) in &upcoming_reminders {
            if reminder.time > now.time() && reminder.time <= target_time.time() {
                println!(
                    "2-min reminder for channel {}: {}",
                    channel_id, reminder.message
                ); // 新增除錯資訊
                imminent_reminders.push((channel_id.clone(), reminder.clone()));
            }
        }

        if !imminent_reminders.is_empty() {
            start_second_check(
                http.clone(),
                reminders.clone(),
                handler.clone(),
                timezone,
                imminent_reminders,
            )
            .await;
        }
    }
}

async fn start_second_check(
    http: Arc<Http>,
    reminders: Arc<RwLock<HashMap<ChannelId, Vec<Reminder>>>>,
    handler: Arc<Handler>,
    timezone: Tz,
    imminent_reminders: Vec<(ChannelId, Reminder)>,
) {
    let mut second_check = interval(Duration::from_secs(1));

    loop {
        second_check.tick().await;
        let now = Utc::now().with_timezone(&timezone);
        let mut reminders_to_send = Vec::new();

        // println!("Checking imminent reminders at: {}", now);

        for (channel_id, reminder) in &imminent_reminders {
            if reminder.time
                == NaiveTime::from_hms_opt(now.hour(), now.minute(), now.second()).unwrap()
            {
                reminders_to_send.push((channel_id.clone(), reminder.clone()));
            }
        }

        for (channel_id, reminder) in reminders_to_send {
            if let Err(err) = channel_id.say(&http, &reminder.message).await {
                println!("Error sending message: {:?}", err);
            }
            // 更新提醒的最後執行時間
            let mut reminders_lock = reminders.write().await;
            if let Some(reminders) = reminders_lock.get_mut(&channel_id) {
                if let Some(rem) = reminders
                    .iter_mut()
                    .find(|r| r.time == reminder.time && r.message == reminder.message)
                {
                    rem.last_executed = Some(now.date_naive());
                    if let Err(err) = handler.save_reminders().await {
                        println!("Error saving reminders: {:?}", err);
                    }
                }
            }
        }
    }
}
