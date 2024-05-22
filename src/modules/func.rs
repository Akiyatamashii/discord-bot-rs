use std::{collections::HashMap, error::Error, fs};

use colored::*;
use serenity::{
    all::{
        ChannelId, CommandInteraction, CreateInteractionResponse, CreateInteractionResponseMessage,
    },
    prelude::*,
};

use crate::Reminder;

const SYSTEM_OUTPUT: &str = "[SYSTEM_OUTPUT]:";
const ERROR_OUTPUT: &str = "[ERROR]:";

pub fn system_output() -> ColoredString {
    SYSTEM_OUTPUT.blue()
}

pub fn error_output() -> ColoredString {
    ERROR_OUTPUT.red()
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
    if let Err(err) = command.create_response(ctx.http.clone(), builder).await {
        println!("Failed to send respond:{}", err)
    }
}

pub fn load_reminders_from_file(
) -> Result<HashMap<ChannelId, Vec<Reminder>>, Box<dyn std::error::Error>> {
    let file_content = fs::read_to_string("reminders.json")?;
    let reminders: HashMap<ChannelId, Vec<Reminder>> = serde_json::from_str(&file_content)?;
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
    reminders: &HashMap<ChannelId, Vec<Reminder>>,
) -> Result<(), Box<dyn Error>> {
    let json_content = serde_json::to_string(reminders)?;
    std::fs::write("reminders.json", json_content)?;
    Ok(())
}