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

static TW: Lazy<Tz> = Lazy::new(|| "Asia/Taipei".parse().unwrap());

#[derive(Clone, Default)]
struct ReminderStore {
    reminders_30_min: Arc<RwLock<Vec<(ChannelId, Reminder)>>>,
    reminders_2_min: Arc<RwLock<Vec<(ChannelId, Reminder)>>>,
    two_min_checking: Arc<RwLock<bool>>,
    one_secs_checking: Arc<RwLock<bool>>,
}

impl ReminderStore {
    fn new() -> Self {
        ReminderStore {
            two_min_checking: Arc::new(RwLock::new(false)),
            one_secs_checking: Arc::new(RwLock::new(false)),
            ..Default::default()
        }
    }
}

pub async fn remind_task(http: Arc<Http>, reminders: Reminders, notify: Arc<Notify>) {
    println!(
        "{} {}",
        system_output(),
        "Reminder remind_task start".green()
    );

    let mut wait_time = interval(Duration::from_secs(1800));
    let reminder_store = Arc::new(ReminderStore::new());

    loop {
        tokio::select! {
            _ = wait_time.tick() => {
                process_reminders(&reminders, &reminder_store).await;

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
            _ = notify.notified() =>{
                process_reminders(&reminders, &reminder_store).await;

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

async fn process_reminders(reminders: &Reminders, reminder_store: &Arc<ReminderStore>) {
    let now = Utc::now().with_timezone(&*TW);
    // println!("start process reminder check:{}", now.time());
    let target_time = now + chrono::Duration::minutes(30);
    let handler_reminder = Arc::clone(&reminders);
    {
        let mut guild_reminders_map = handler_reminder.write().await;
        for (_guild_id, reminders_map) in guild_reminders_map.iter_mut() {
            for (channel_id, reminders) in reminders_map.iter_mut() {
                for reminder in reminders.iter_mut() {
                    if reminder.weekdays.contains(&now.weekday())
                        && reminder.time > now.time()
                        && reminder.time <= target_time.time()
                        && reminder.last_executed != Some(now.date_naive())
                    {
                        reminder.last_executed = Some(now.date_naive());
                        let mut reminder_in_30min = reminder_store.reminders_30_min.write().await;
                        reminder_in_30min.push((channel_id.clone(), reminder.clone()));
                    }
                }
            }
        }
        save_reminders_to_file(&*guild_reminders_map).expect("Failed to save reminders");
    }
}

async fn check_2min_remind(http: Arc<Http>, remind_store: Arc<ReminderStore>) {
    let mut wait_time = interval(Duration::from_secs(120));
    loop {
        wait_time.tick().await;
        let now = Utc::now().with_timezone(&*TW);
        // println!("start 2min check:{}", now.time());
        let target_time = now + chrono::Duration::minutes(2);
        let mut new_list = Vec::new();
        {
            let reminder_in_30min = remind_store.reminders_30_min.read().await;
            for (channel_id, reminder) in reminder_in_30min.iter() {
                if reminder.time > now.time() && reminder.time < target_time.time() {
                    let mut reminder_in_2min = remind_store.reminders_2_min.write().await;
                    reminder_in_2min.push((channel_id.clone(), reminder.clone()));
                } else {
                    new_list.push((channel_id.clone(), reminder.clone()));
                }
            }
        }
        *remind_store.reminders_30_min.write().await = new_list;

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
    // println!("stop 2min check");
}

async fn check_1secs_remind(http: Arc<Http>, remind_store: Arc<ReminderStore>) {
    let mut wait_time = interval(Duration::from_secs(1));
    loop {
        wait_time.tick().await;
        let now = Utc::now().with_timezone(&*TW);
        // println!("start 1secs check:{}", now.time());
        let time = NaiveTime::from_hms_opt(now.hour(), now.minute(), now.second()).unwrap();
        let mut new_list = Vec::new();
        {
            let reminder_in_2min = remind_store.reminders_2_min.read().await;
            for (channel_id, reminder) in reminder_in_2min.iter() {
                if reminder.time == time {
                    if let Err(err) = channel_id.say(&http, &reminder.message).await {
                        println!("{} sending message: {:?}", error_output(), err);
                    }
                } else {
                    new_list.push((channel_id.clone(), reminder.clone()));
                }
            }
        }
        *remind_store.reminders_2_min.write().await = new_list;

        if remind_store.reminders_2_min.read().await.is_empty() {
            *remind_store.one_secs_checking.write().await = false;
            break;
        }
    }
    // println!("stop 1secs check");
}
