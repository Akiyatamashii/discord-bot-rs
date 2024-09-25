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
    weekdays: Vec<Weekday>,           //要提醒的星期幾
    time: NaiveTime,                  //提醒時間
    message: String,                  //提醒訊息內容
    last_executed: Option<NaiveDate>, //上次執行的日期
}

// 定義 Reminders 類型，用於存儲所有伺服器和頻道的提醒
type Reminders = Arc<RwLock<HashMap<GuildId, HashMap<ChannelId, Vec<Reminder>>>>>;

#[derive(Clone)]
struct Handler {
    //處理器結構
    reminders: Reminders,             // 存儲所有提醒
    trigger_notify: Arc<Notify>,      // 用於觸發提醒檢查的通知器
    prefix: Regex,                    // 用於匹配命令前綴的正則表達式
}

#[async_trait]
impl EventHandler for Handler {
    // 處理收到的消息
    async fn message(&self, ctx: Context, msg: Message) {
        // 忽略機器人發送的消息
        if msg.author.bot {
            return;
        }

        // 檢查消息是否匹配命令前綴
        if self.prefix.is_match(&msg.content) {
            prefix_command_process(&ctx, &msg).await
        }
    }

    // 處理交互命令
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            // 輸出接收到的交互命令信息
            println!(
                "{} {} {} {} {}",
                system_output(),
                "Received interaction:".green(),
                command.data.name.yellow().bold(),
                ",from:".green(),
                command.user.name.yellow().bold()
            );

            let ctx = Arc::new(ctx);

            // 處理交互命令
            interaction_process(self, &ctx, &command).await;
        }
    }

    // 機器人準備就緒時的處理
    async fn ready(&self, ctx: Context, ready: Ready) {
        // 設置機器人活動狀態
        ctx.set_activity(Some(ActivityData::playing("記憶大賽....")));
        
        // 確保存儲伺服器 ID 的文件存在
        let file_path = "guild_id.txt";
        ensure_guild_id_file_exists(file_path).unwrap();

        // 註冊命令到指定的伺服器
        register_commands_guild_ids(&ctx).await;

        // 輸出機器人連接成功的信息
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
    // 載入環境變量
    dotenv().ok();

    // 獲取 Discord 機器人令牌
    let token = env::var("TOKEN").expect("missing token");
    
    // 設置機器人所需的權限
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES;

    // 從文件加載提醒，如果失敗則創建一個空的 HashMap
    let reminders = match load_reminders_from_file() {
        Ok(r) => Arc::new(RwLock::new(r)),
        Err(_) => Arc::new(RwLock::new(HashMap::new())),
    };

    // 創建用於匹配命令前綴的正則表達式
    let prefix = Regex::new(r"^![A-Za-z]").unwrap();

    // 創建 Handler 實例
    let handler = Handler {
        reminders: Arc::clone(&reminders),
        trigger_notify: Arc::new(Notify::new()),
        prefix,
    };

    // 創建 Discord 客戶端
    let mut client = Client::builder(&token, intents)
        .event_handler(handler.clone())
        .await
        .expect("Create client error");

    let http = Arc::clone(&client.http);

    // 啟動提醒任務
    tokio::spawn(modules::reminder::remind_task(
        http,
        Arc::clone(&handler.reminders),
        Arc::clone(&handler.trigger_notify),
    ));

    // 啟動客戶端，如果出錯則輸出錯誤信息
    if let Err(why) = client.start().await {
        println!("{} {} {:?}", error_output(), "Client error:".red(), why);
    }
}
