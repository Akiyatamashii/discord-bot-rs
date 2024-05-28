use std::{collections::HashMap, env, sync::Arc};

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
use tokio::sync::{Notify, RwLock};

mod commands;
mod modules;
use modules::func::{
    check_permission, ensure_guild_id_file_exists, error_output, interaction_response,
    load_reminders_from_file, register_commands, register_commands_guild_ids, system_output,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    reminders: Arc<RwLock<HashMap<GuildId, HashMap<ChannelId, Vec<Reminder>>>>>,
    trigger_notify: Arc<Notify>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        if msg.content.starts_with("!register") {
            let guild_id = msg.guild_id.unwrap();
            register_commands(&ctx, &guild_id, false).await;
            msg.delete(&ctx.http).await.unwrap();
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!(
                "{} {} {} {} {}",
                system_output(),
                "Received interaction:".green(),
                command.data.name.yellow().bold(),
                ",from:".green(),
                command.user.name.yellow().bold()
            );
            let _ = match command.data.name.as_str() {
                "ping" => {
                    let msg = commands::ping::run(&command.data.options());
                    interaction_response(&ctx, &command, msg, true).await;
                    true
                }
                "info" => {
                    commands::info::run(&ctx, &command, &command.data.options()).await;
                    true
                }
                "look" => {
                    let guild_id = command.guild_id.unwrap();
                    let channel_id = command.channel_id;
                    let msg = commands::look::run(guild_id, channel_id);
                    interaction_response(&ctx, &command, msg, true).await;
                    true
                }

                "chat" => {
                    match commands::chat::run(&ctx, &command, &command.data.options()).await {
                        Ok(msg) => {
                            if msg != "" {
                                interaction_response(&ctx, &command, msg, true).await;
                            }
                            true
                        }
                        Err(err) => {
                            println!(
                                "{} {} {}",
                                error_output(),
                                "OpenAI mission filed:".red(),
                                err
                            );
                            false
                        }
                    }
                }
                "image" => {
                    match commands::image::run(&ctx, &command, &command.data.options()).await {
                        Ok(msg) => {
                            if msg != "" {
                                interaction_response(&ctx, &command, msg, true).await;
                            }
                            true
                        }
                        Err(err) => {
                            println!(
                                "{} {} {}",
                                error_output(),
                                "OpenAI mission filed:".red(),
                                err
                            );
                            false
                        }
                    }
                }
                "remind" => {
                    if !check_permission(&ctx, &command).await {
                        return;
                    }
                    let channel_id = command.channel_id;
                    let guild_id = command.guild_id.unwrap();
                    match commands::remind::run(
                        &command.data.options(),
                        self.reminders.clone(),
                        channel_id,
                        guild_id,
                        &self.trigger_notify,
                    )
                    .await
                    {
                        Ok(msg) => {
                            interaction_response(&ctx, &command, msg, true).await;
                            true
                        }
                        Err(err) => {
                            println!(
                                "{} {} {}",
                                error_output(),
                                "Failed to set reminder:".red(),
                                err
                            );
                            false
                        }
                    }
                }
                "rm_remind" => {
                    if !check_permission(&ctx, &command).await {
                        return;
                    }
                    let channel_id = command.channel_id;
                    let guild_id = command.guild_id.unwrap();
                    match commands::rm_remind::run(
                        &command.data.options(),
                        self.reminders.clone(),
                        channel_id,
                        guild_id,
                    )
                    .await
                    {
                        Ok(msg) => {
                            interaction_response(&ctx, &command, msg, true).await;
                            true
                        }
                        Err(err) => {
                            println!(
                                "{} {} {}",
                                error_output(),
                                "Failed to remove reminder:".red(),
                                err
                            );
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
        let file_path = "guild_id.txt";
        ensure_guild_id_file_exists(file_path).unwrap();

        register_commands_guild_ids(&ctx).await;

        println!(
            "{} {} {}",
            system_output(),
            ready.user.name.green().bold(),
            "connect success".green().bold()
        );
    }
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
        reminders: Arc::clone(&reminders),
        trigger_notify: Arc::new(Notify::new()),
    };

    let mut client = Client::builder(&token, intents)
        .event_handler(handler.clone())
        .await
        .expect("Create client error");

    let http = Arc::clone(&client.http);

    tokio::spawn(modules::reminder::remind_task(
        http,
        Arc::clone(&handler.reminders),
        Arc::clone(&handler.trigger_notify),
    ));

    if let Err(why) = client.start().await {
        println!("{} {} {:?}", error_output(), "Client error:".red(), why);
    }
}
