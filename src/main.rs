use std::{collections::HashMap, env, sync::Arc};

use chrono::{NaiveDate, NaiveTime, Weekday};
use colored::*;
use dotenvy::dotenv;
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
use modules::anti_tiktok::tiktok_refuse;
use modules::func::{
    ensure_file_exists, error_output, load_reminders_from_file, register_commands_guild_ids,
    system_output,
};
use modules::{
    anti_tiktok::load_tiktok_refuse_msg,
    bot_process::{interaction_process, prefix_command_process},
};

// Define the Reminder structure
// 定義 Reminder 結構
#[derive(Serialize, Deserialize, Clone, Debug)]
struct Reminder {
    // Days of the week for the reminder
    // 要提醒的星期幾
    weekdays: Vec<Weekday>,
    // Time for the reminder
    // 提醒時間
    time: NaiveTime,
    // Content of the reminder message
    // 提醒訊息內容
    message: String,
    // Date of last execution
    // 上次執行的日期
    last_executed: Option<NaiveDate>,
}

// Define Reminders type for storing reminders for all servers and channels
// 定義 Reminders 類型，用於存儲所有伺服器和頻道的提醒
type Reminders = Arc<RwLock<HashMap<GuildId, HashMap<ChannelId, Vec<Reminder>>>>>;

// Define the Handler structure
// 定義 Handler 結構
#[derive(Clone)]
struct Handler {
    // Store all reminders
    // 存儲所有提醒
    reminders: Reminders,
    // Notifier for triggering reminder checks
    // 用於觸發提醒檢查的通知器
    trigger_notify: Arc<Notify>,
    // Regex for matching command prefixes
    // 用於匹配命令前綴的正則表達式
    prefix: Regex,
    // Replies for refusing TikTok messages
    // 用於拒絕TikTok消息的回覆
    tiktok_refuse_msg: Arc<RwLock<Vec<String>>>,
}

impl Handler {}

#[async_trait]
impl EventHandler for Handler {
    // Handle received messages
    // 處理收到的消息
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages sent by bots
        // 忽略機器人發送的消息
        if msg.author.bot {
            return;
        }

        // Check if the message matches the command prefix
        // 檢查消息是否匹配命令前綴
        if self.prefix.is_match(&msg.content) {
            prefix_command_process(&ctx, &msg).await
        };

        // Handle TikTok messages in a specific guild
        // 在特定伺服器中處理 TikTok 消息
        if msg.guild_id == Some(GuildId::new(1143403544599334992)) {
            tiktok_refuse(&ctx, &msg, Arc::clone(&self.tiktok_refuse_msg)).await;
        }
    }

    // Handle interaction commands
    // 處理交互命令
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            // Output information about the received interaction command
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

            // Process the interaction command
            // 處理交互命令
            interaction_process(self, &ctx, &command).await;
        }
    }

    // Handle bot ready event
    // 處理機器人準備就緒事件
    async fn ready(&self, ctx: Context, ready: Ready) {
        // Set the bot's activity status
        // 設置機器人活動狀態
        ctx.set_activity(Some(ActivityData::playing("記憶大賽....")));

        // Ensure the file for storing guild IDs exists
        // 確保存儲伺服器 ID 的文件存在
        let file_path = "assets/guild_id.txt";
        ensure_file_exists(file_path).unwrap();

        // Register commands for specified guild IDs
        // 註冊命令到指定的伺服器
        register_commands_guild_ids(&ctx).await;

        // Output bot connection success message
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
    // Load environment variables
    // 載入環境變量
    dotenv().ok();

    // Get Discord bot token
    // 獲取 Discord 機器人令牌
    let token = env::var("TOKEN").expect("missing token");

    // Set required bot permissions
    // 設置機器人所需的權限
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_VOICE_STATES;

    // Load reminders from file or create an empty HashMap
    // 從文件加載提醒，如果失敗則創建一個空的 HashMap
    let reminders = match load_reminders_from_file() {
        Ok(r) => Arc::new(RwLock::new(r)),
        Err(_) => Arc::new(RwLock::new(HashMap::new())),
    };

    // Create regex for matching command prefixes
    // 創建用於匹配命令前綴的正則表達式
    let prefix = Regex::new(r"^![A-Za-z]").unwrap();

    // Create Handler instance
    // 創建 Handler 實例
    let handler = Handler {
        reminders: Arc::clone(&reminders),
        trigger_notify: Arc::new(Notify::new()),
        prefix,
        tiktok_refuse_msg: Arc::new(RwLock::new(load_tiktok_refuse_msg())),
    };

    // Create Discord client
    // 創建 Discord 客戶端
    let mut client = Client::builder(&token, intents)
        .event_handler(handler.clone())
        .await
        .expect("Create client error");

    let http = Arc::clone(&client.http);

    // Start reminder task
    // 啟動提醒任務
    tokio::spawn(modules::reminder::remind_task(
        http,
        Arc::clone(&handler.reminders),
        Arc::clone(&handler.trigger_notify),
    ));

    // Start the client and output error message if it fails
    // 啟動客戶端，如果出錯則輸出錯誤信息
    if let Err(why) = client.start().await {
        println!("{} {} {:?}", error_output(), "Client error:".red(), why);
    }
}
