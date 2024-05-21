use std::{env, error::Error};

use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client,
};
use colored::*;
use serenity::{
    all::{
        CommandInteraction, CommandOptionType, Context, CreateCommandOption,
        CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse,
        ResolvedOption, ResolvedValue,
    },
    builder::CreateCommand,
    futures::StreamExt,
};

use crate::modules::func::error_output;

pub fn register() -> CreateCommand {
    CreateCommand::new("chat")
        .description("openai chat")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "prompt", "提示詞").required(true),
        )
}

pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    options: &'a [ResolvedOption<'a>],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let prompt = options.iter().find(|opt| opt.name == "prompt");
    if let Some(prompt) = prompt {
        if let ResolvedValue::String(prompt) = prompt.value {
            if let Err(err) = chat(&ctx, command, prompt).await {
                println!("{} {} {}", error_output(), "OpenAI mission failed:", err)
            }
            return Ok("".to_string());
        } else {
            return Ok("Prompt參數轉換有問題".to_string());
        }
    }
    Ok("未提供提示詞".to_string())
}

async fn chat(
    ctx: &Context,
    command: &CommandInteraction,
    prompt: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = env::var("API_KEY").expect("無法獲取API_KEY");
    let config = OpenAIConfig::new().with_api_key(api_key);

    let client = Client::with_config(config);
    let req = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o")
        .max_tokens(4096 as u16)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content("請用繁體中文回覆，並將字數控制為2000字以內。")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into(),
        ])
        .build()?;

    let mut stream = client.chat().create_stream(req).await?;

    let data = CreateInteractionResponseMessage::new().content("回覆中請稍後...".to_string());
    let builder = CreateInteractionResponse::Message(data);
    command.create_response(&ctx.http, builder).await?;
    let mut message = String::from("");
    let mut count = 1;

    while let Some(result) = stream.next().await {
        match result {
            Ok(res) => {
                for choice in res.choices.iter() {
                    if let Some(ref content) = choice.delta.content {
                        message.push_str(&content);
                        if count > 10 {
                            let builder = EditInteractionResponse::new().content(message.clone());
                            command.edit_response(&ctx.http, builder).await?;
                            count = 1;
                        } else {
                            count += 1;
                        }
                    }
                }
            }
            Err(err) => {
                let builder = EditInteractionResponse::new().content("獲取OpenAI訊息失敗");
                command.edit_response(&ctx.http, builder).await?;
                println!(
                    "{} {} {}",
                    error_output(),
                    "Response message failed:".red(),
                    err
                );
            }
        }
    }
    let builder = EditInteractionResponse::new().content(message);
    command.edit_response(&ctx.http, builder).await?;

    Ok(())
}
