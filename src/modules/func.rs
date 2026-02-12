use std::{
    collections::HashMap,
    env,
    error::Error,
    fs::{self, File, OpenOptions},
    io::{self, BufRead, BufReader, Write},
    path::Path,
};

use async_openai::config::OpenAIConfig;
use colored::*;
use serenity::{
    all::{
        ChannelId, CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
        GuildId, Message,
    },
    prelude::*,
};

use crate::{commands, Reminder};

// Type alias for a simplified version of the Reminder structure
// Reminder 結構的簡化版本的類型別名
type ArcLessReminder = HashMap<GuildId, HashMap<ChannelId, Vec<Reminder>>>;

// Constants definition
// 常量定義
const SYSTEM_OUTPUT: &str = "[SYSTEM_OUTPUT]:";
const ERROR_OUTPUT: &str = "[ERROR]:";
const INFO_PATH: &str = "./info/";

// Return blue colored system output prefix
// 返回藍色的系統輸出前綴
pub fn system_output() -> ColoredString {
    SYSTEM_OUTPUT.blue()
}

// Return red bold colored error output prefix
// 返回紅色加粗的錯誤輸出前綴
pub fn error_output() -> ColoredString {
    ERROR_OUTPUT.red().bold()
}

// Create OpenAI configuration
// 創建 OpenAI 配置
pub fn openai_config() -> OpenAIConfig {
    let api = env::var("API_KEY").unwrap();
    OpenAIConfig::new().with_api_key(api)
}

// Return info path
// 返回信息路徑
pub fn info_path() -> String {
    INFO_PATH.to_string()
}

// Handle interaction response
// 處理交互響應
pub async fn interaction_response(
    ctx: &Context,
    command: &CommandInteraction,
    msg: String,
    ephemeral: bool,
) {
    let data = CreateInteractionResponseMessage::new()
        .content(msg)
        .ephemeral(ephemeral);
    let builder = CreateInteractionResponse::Message(data);
    if let Err(err) = command.create_response(&ctx.http, builder).await {
        println!("{} Failed to send respond:{}", error_output(), err)
    }
}

// Check user permissions
// 檢查用戶權限
pub async fn check_permission(ctx: &Context, command: &CommandInteraction) -> bool {
    if let Some(permissions) = command.member.clone().unwrap().permissions {
        if !permissions.administrator() {
            let data = CreateInteractionResponseMessage::new()
                .content("你沒有許可權使用指令")
                .ephemeral(true);
            let builder = CreateInteractionResponse::Message(data);
            if let Err(err) = command.create_response(&ctx.http, builder).await {
                println!("Cannot respond to slash command: {err}");
            }
            return false;
        }
    }
    true
}

pub async fn is_user_admin(ctx: &Context, msg: &Message) -> serenity::Result<bool> {
    let member = msg.guild_id.unwrap().member(&ctx.http, msg.author.id).await?;

    if let Some(permissions) = member.permissions {
        Ok(permissions.administrator())
    }else {
        Ok(false)
    }
}

// Load reminders from file
// 從文件加載提醒
pub fn load_reminders_from_file() -> Result<ArcLessReminder, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string("assets/reminders.json")?;
    let reminders: HashMap<GuildId, HashMap<ChannelId, Vec<Reminder>>> =
        serde_json::from_str(&file_content)?;
    Ok(reminders)
}

// Save reminders to file
// 保存提醒到文件
pub fn save_reminders_to_file(
    reminders: &HashMap<GuildId, HashMap<ChannelId, Vec<Reminder>>>,
) -> Result<(), Box<dyn Error>> {
    let json_content = serde_json::to_string(reminders)?;
    std::fs::write("assets/reminders.json", json_content)?;
    Ok(())
}

// Register commands for all guilds
// 為所有 guild 註冊命令
pub async fn register_commands_guild_ids(ctx: &Context) {
    let file_path = "assets/guild_id.txt";

    // Read all guild_ids from guild_id.txt
    // 讀取 guild_id.txt 內的所有 guild_id
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => {
            println!("{} Failed to open guild_id.txt: {:?}", error_output(), err);
            return;
        }
    };

    let reader = BufReader::new(file);
    let guild_ids: Vec<GuildId> = reader
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| line.parse::<u64>().ok())
        .map(GuildId::from)
        .collect();

    for guild_id in guild_ids {
        // Register commands
        // 註冊指令
        register_commands(ctx, &guild_id, true).await;
    }
    println!(
        "{} {}",
        system_output(),
        "Slash command already register for every guild".green()
    );
}

// Register commands for a specific guild
// 為特定 guild 註冊命令
pub async fn register_commands(ctx: &Context, guild_id: &GuildId, for_guilds: bool) {
    let command = guild_id
        .set_commands(
            ctx,
            vec![
                // Basic commands
                // 基本命令
                commands::base::info::register(),
                commands::base::ping::register(),
                commands::base::update::register(),
                // Reminder related commands
                // 提醒相關命令
                commands::reminder::remind::register(),
                commands::reminder::look::register(),
                commands::reminder::rm_remind::register(),
                // OpenAI related commands
                // OpenAI 相關命令
                commands::openai::chat::register(),
                commands::openai::image::register(),
                commands::openai::model_list::register(),
                // Ban related commands
                // 封禁相關命令
                commands::ban::add_ban::register(),
                commands::ban::un_ban::register(),
                commands::ban::remove_block::register(),
                commands::ban::display_block_list::register(),
                // Other feature commands
                // 其他功能命令
                commands::cash::register(),
                commands::anti_tiktok::tiktok_msg_add::register(),

            ],
        )
        .await;
    match command {
        Ok(cmds) => {
            let command_names: Vec<_> = cmds.iter().map(|cmd| &cmd.name).collect();
            if !for_guilds {
                println!(
                    "{} {} {:?} for guild {}",
                    system_output(),
                    "Created slash commands:".green(),
                    command_names,
                    guild_id
                );
                // Save guild_id to file
                // 儲存 guild_id 到文件
                save_guild_id_to_file(guild_id).unwrap();
            }
        }
        Err(err) => {
            println!(
                "{} {} {:?} for guild {}",
                error_output(),
                "Failed to create commands:".red(),
                err,
                guild_id
            );
        }
    }
}

// Save guild_id to file
// 保存 guild_id 到文件
fn save_guild_id_to_file(guild_id: &GuildId) -> io::Result<()> {
    let file_path = "assets/guild_id.txt";

    // Read existing guild_ids
    // 讀取已存在的 guild_id
    let existing_ids = fs::read_to_string(file_path).unwrap_or_default();

    if !existing_ids.contains(&guild_id.to_string()) {
        // If the file doesn't contain the current guild_id, write it to the file
        // 如果文件中不包含當前 guild_id，則寫入文件
        let mut file = OpenOptions::new().append(true).open(file_path)?;

        writeln!(file, "{}", guild_id)?;
        println!("{} Saved guild_id to file.", system_output());
    }

    Ok(())
}

// Ensure folder and file exist
// 確保資料夾與文件存在
pub fn ensure_file_exists(file_path: &str) -> io::Result<()> {
    // Ensure directory exists
    // 確保目錄存在
    if let Some(parent) = Path::new(file_path).parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Ensure file exists
    // 確保文件存在
    if !Path::new(file_path).exists() {
        File::create(file_path)?;
    }
    Ok(())
}