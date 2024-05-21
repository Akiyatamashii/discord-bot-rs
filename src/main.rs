use chrono::{NaiveDate, NaiveTime, Weekday};
use colored::*;
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{ActivityData, ChannelId, GuildId, Interaction},
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use std::{collections::HashMap, env, fs, sync::Arc};
use tokio::sync::{Notify, RwLock};

mod commands;
mod modules;

#[derive(Serialize, Deserialize, Clone)]
struct Reminder {
    //提醒器結構
    weekdays: Vec<Weekday>,           //天數
    time: NaiveTime,                  //時間
    message: String,                  //發送訊息
    last_executed: Option<NaiveDate>, //最後執行時間
}

#[derive(Clone)]
struct Handler {
    //處理器結構
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
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!(
                "{} {} {} {}",
                "Received interaction:".green(),
                command.data.name.yellow().bold(),
                ",from:".green(),
                command.user.name.yellow().bold()
            );
            let _ = match command.data.name.as_str() {
                "ping" => {
                    let msg = commands::ping::run(&command.data.options());
                    modules::func::interaction_response(&ctx, &command, msg, true).await;
                    true
                }
                "look" => {
                    let msg = commands::look::run();
                    modules::func::interaction_response(&ctx, &command, msg, true).await;
                    true
                }
                "rm_remind" => {
                    if !modules::func::check_permission(&ctx, &command).await {
                        return;
                    }
                    let channel_id = command.channel_id;
                    match commands::rm_remind::run(
                        &command.data.options(),
                        self.reminders.clone(),
                        channel_id,
                    )
                    .await
                    {
                        Ok(msg) => {
                            modules::func::interaction_response(&ctx, &command, msg, false).await;
                            true
                        }
                        Err(err) => {
                            println!("{} {}", "Failed to remove reminder:".red(), err);
                            false
                        }
                    }
                }
                "remind" => {
                    if !modules::func::check_permission(&ctx, &command).await {
                        return;
                    }
                    let channel_id = command.channel_id;
                    match commands::remind::run(
                        &command.data.options(),
                        self.reminders.clone(),
                        channel_id,
                    )
                    .await
                    {
                        Ok(msg) => {
                            modules::func::interaction_response(&ctx, &command, msg, false).await;
                            self.trigger_notify.notify_one();
                            true
                        }
                        Err(err) => {
                            println!("{} {}", "Failed to set reminder:".red(), err);
                            false
                        }
                    }
                }
                _ => false,
            };
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Some(ActivityData::playing("記憶大賽....")));
        println!(
            "{} {}",
            ready.user.name.green().bold(),
            "connect success".green()
        );
        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let command = guild_id
            .set_commands(
                &ctx,
                vec![
                    commands::ping::register(),
                    commands::remind::register(),
                    commands::look::register(),
                    commands::rm_remind::register(),
                ],
            )
            .await;
        match command {
            Ok(cmds) => {
                let command_names: Vec<_> = cmds.iter().map(|cmd| &cmd.name).collect();
                println!(
                    "{} {:?}",
                    "Created slash commands: ".green(),
                    command_names
                );
            }
            Err(err) => {
                println!("{} {:?}", "Failed to create commands:".red(), err);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("TOKEN").expect("missing token");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let reminders = match modules::func::load_reminders_from_file() {
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

    tokio::spawn(modules::reminder::remind_task(
        http,
        reminders.clone(),
        Arc::new(handler.clone()),
    ));

    if let Err(why) = client.start().await {
        println!("{} {:?}", "Client error:".red(), why);
    }
}
