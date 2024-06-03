use std::{collections::HashMap, env, sync::Arc};

use chrono::{NaiveDate, NaiveTime, Weekday};
use colored::*;
use dotenv::dotenv;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{ActivityData, ChannelId, GuildId, Interaction},
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use songbird::SerenityInit;
use tokio::sync::{Notify, RwLock};

mod commands;
mod modules;
use modules::bot_process::{interaction_process, prefix_command_process};
use modules::func::{
    ensure_guild_id_file_exists, error_output, load_reminders_from_file,
    register_commands_guild_ids, system_output,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Reminder {
    //提醒器結構
    weekdays: Vec<Weekday>,           //天數
    time: NaiveTime,                  //時間
    message: String,                  //發送訊息
    last_executed: Option<NaiveDate>, //最後執行時間
}

#[derive(Default, Debug, Clone)]
struct MusicInfo {
    title: String,
    http: String,
    watch: Option<String>,
}
type MusicTemp = Arc<RwLock<HashMap<usize, MusicInfo>>>;
type MusicList = Arc<RwLock<Vec<MusicInfo>>>;
type Reminders = Arc<RwLock<HashMap<GuildId, HashMap<ChannelId, Vec<Reminder>>>>>;

#[derive(Clone)]
struct Handler {
    //處理器結構
    reminders: Reminders,
    music_list_temp: MusicTemp,
    music_list: MusicList,
    trigger_notify: Arc<Notify>,
    prefix: Regex,
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if self.prefix.is_match(&msg.content) {
            prefix_command_process(&ctx, &msg).await
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

            let ctx = Arc::new(ctx);

            interaction_process(self, &ctx, &command).await;
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
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES;

    let reminders = match load_reminders_from_file() {
        Ok(r) => Arc::new(RwLock::new(r)),
        Err(_) => Arc::new(RwLock::new(HashMap::new())),
    };
    let music_list: MusicList = Arc::new(RwLock::new(Vec::new()));
    let music_list_temp: MusicTemp = Arc::new(RwLock::new(HashMap::new()));
    let prefix = Regex::new(r"^![A-Za-z]").unwrap();

    let handler = Handler {
        reminders: Arc::clone(&reminders),
        music_list: Arc::clone(&music_list),
        music_list_temp: Arc::clone(&music_list_temp),
        trigger_notify: Arc::new(Notify::new()),
        prefix,
    };

    let mut client = Client::builder(&token, intents)
        .event_handler(handler.clone())
        .register_songbird()
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
