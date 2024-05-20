use chrono::{Datelike, NaiveDate, NaiveTime, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{
        ChannelId, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, Http,
        Interaction,
    },
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::{collections::HashMap, env, fs, sync::Arc};
use tokio::sync::{Notify, RwLock};
use tokio::time::{interval, Duration};
mod commands;

#[derive(Serialize, Deserialize, Clone)]
struct Reminder {
    weekdays: Vec<Weekday>,
    time: NaiveTime,
    message: String,
    last_executed: Option<NaiveDate>,
}

#[derive(Clone)]
struct Handler {
    reminders: Arc<RwLock<HashMap<ChannelId, Vec<Reminder>>>>,
    cancel_notify: Arc<Notify>,
    trigger_notify: Arc<Notify>, // 用於觸發立即檢查
}

impl Handler {
    async fn save_reminders(&self) -> Result<(), Box<dyn std::error::Error>> {
        let reminders = self.reminders.read().await;
        let json_content = serde_json::to_string(&*reminders)?;
        fs::write("reminders.json", json_content)?;
        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        println!("get message:{}", msg.content);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {}", command.data.name);
            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                "remind" => {
                    let channel_id = command.channel_id;
                    match commands::remind::run(
                        &command.data.options(),
                        self.reminders.clone(),
                        channel_id,
                    )
                    .await
                    {
                        Ok(msg) => {
                            self.trigger_notify.notify_one(); // 立即觸發檢查
                            Some(msg)
                        }
                        Err(err) => Some(format!("Failed to set reminder: {}", err)),
                    }
                }
                _ => Some(String::from("no command")),
            };
            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(err) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {err}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is online", ready.user.name);
        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let command = guild_id
            .set_commands(
                &ctx,
                vec![commands::ping::register(), commands::remind::register()],
            )
            .await;
        match command {
            Ok(cmds) => {
                let command_names: Vec<_> = cmds.iter().map(|cmd| &cmd.name).collect();
                println!(
                    "I created the following global slash commands: {:?}",
                    command_names
                );
            }
            Err(err) => {
                println!("Failed to create commands: {:?}", err);
            }
        }
    }
}

async fn remind_task(
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

fn load_reminders_from_file(
) -> Result<HashMap<ChannelId, Vec<Reminder>>, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string("reminders.json")?;
    let reminders: HashMap<ChannelId, Vec<Reminder>> = serde_json::from_str(&file_content)?;
    Ok(reminders)
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("TOKEN").expect("missing token");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let reminders = match load_reminders_from_file() {
        Ok(r) => Arc::new(RwLock::new(r)),
        Err(_) => Arc::new(RwLock::new(HashMap::new())),
    };

    let handler = Handler {
        reminders: reminders.clone(),
        cancel_notify: Arc::new(Notify::new()),
        trigger_notify: Arc::new(Notify::new()), // 初始化觸發通知
    };

    let mut client = Client::builder(&token, intents)
        .event_handler(handler.clone())
        .await
        .expect("Create client error");

    let http = Arc::clone(&client.http);

    tokio::spawn(remind_task(
        http,
        reminders.clone(),
        Arc::new(handler.clone()),
    ));

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
