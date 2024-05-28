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
        GuildId,
    },
    prelude::*,
};

use crate::{commands, Reminder};

const SYSTEM_OUTPUT: &str = "[SYSTEM_OUTPUT]:";
const ERROR_OUTPUT: &str = "[ERROR]:";
const INFO_PATH: &str = "./info/";

pub fn system_output() -> ColoredString {
    SYSTEM_OUTPUT.blue()
}

pub fn error_output() -> ColoredString {
    ERROR_OUTPUT.red()
}

pub fn openai_config() -> OpenAIConfig {
    let api = env::var("API_KEY").unwrap();
    OpenAIConfig::new().with_api_key(api)
}

pub fn info_path() -> String {
    INFO_PATH.to_string()
}

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

pub fn load_reminders_from_file(
) -> Result<HashMap<GuildId, HashMap<ChannelId, Vec<Reminder>>>, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string("reminders.json")?;
    let reminders: HashMap<GuildId, HashMap<ChannelId, Vec<Reminder>>> =
        serde_json::from_str(&file_content)?;
    Ok(reminders)
}

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

pub fn save_reminders_to_file(
    reminders: &HashMap<GuildId, HashMap<ChannelId, Vec<Reminder>>>,
) -> Result<(), Box<dyn Error>> {
    let json_content = serde_json::to_string(reminders)?;
    std::fs::write("reminders.json", json_content)?;
    Ok(())
}

pub async fn register_commands_guild_ids(ctx: &Context) {
    let file_path = "guild_id.txt";

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
        .filter_map(|line| line.ok())
        .filter_map(|line| line.parse::<u64>().ok())
        .map(GuildId::from)
        .collect();

    for guild_id in guild_ids {
        // 註冊指令
        register_commands(ctx, &guild_id, true).await;
    }
    println!(
        "{} {}",
        system_output(),
        "Slash command already register for every guild".green()
    );
}

pub async fn register_commands(ctx: &Context, guild_id: &GuildId, for_guilds: bool) {
    let command = guild_id
        .set_commands(
            ctx,
            vec![
                commands::ping::register(),
                commands::remind::register(),
                commands::look::register(),
                commands::rm_remind::register(),
                commands::chat::register(),
                commands::image::register(),
                commands::info::register(),
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

fn save_guild_id_to_file(guild_id: &GuildId) -> io::Result<()> {
    let file_path = "guild_id.txt";

    // 讀取已存在的 guild_id
    let existing_ids = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => String::new(),
    };

    if !existing_ids.contains(&guild_id.to_string()) {
        // 如果文件中不包含當前 guild_id，則寫入文件
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(file_path)?;

        writeln!(file, "{}", guild_id)?;
        println!("{} Saved guild_id to file.", system_output());
    }

    Ok(())
}

// 輔助函數，用於檢查文件是否存在，若不存在則創建
pub fn ensure_guild_id_file_exists(file_path: &str) -> io::Result<()> {
    if !Path::new(file_path).exists() {
        File::create(file_path)?;
    }
    Ok(())
}
