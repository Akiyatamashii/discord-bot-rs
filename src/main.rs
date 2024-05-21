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
    check_permission, error_output, interaction_response, load_reminders_from_file, system_output,
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
    reminders: Arc<RwLock<HashMap<ChannelId, Vec<Reminder>>>>,
    cancel_notify: Arc<Notify>,
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
                "look" => {
                    let msg = commands::look::run();
                    interaction_response(&ctx, &command, msg, true).await;
                    true
                }

                "chat" => {
                    match commands::chat::run(&ctx, &command, &command.data.options()).await {
                        Ok(msg) => {
                            if msg != "" {
                                interaction_response(&ctx, &command, msg, false).await;
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
                "rm_remind" => {
                    if !check_permission(&ctx, &command).await {
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
                            interaction_response(&ctx, &command, msg, false).await;
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
                "remind" => {
                    if !check_permission(&ctx, &command).await {
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
                            interaction_response(&ctx, &command, msg, false).await;
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
                _ => false,
            };
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        ctx.set_activity(Some(ActivityData::playing("記憶大賽....")));

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
                    commands::chat::register(),
                ],
            )
            .await;
        match command {
            Ok(cmds) => {
                let command_names: Vec<_> = cmds.iter().map(|cmd| &cmd.name).collect();
                println!(
                    "{} {} {:?}",
                    system_output(),
                    "Created slash commands: ".green(),
                    command_names
                );
            }
            Err(err) => {
                println!(
                    "{} {} {:?}",
                    error_output(),
                    "Failed to create commands:".red(),
                    err
                );
            }
        }

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
        reminders: reminders.clone(),
        cancel_notify: Arc::new(Notify::new()),
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
        println!("{} {} {:?}", error_output(), "Client error:".red(), why);
    }
}
