use std::error::Error;

use async_openai::{
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

use crate::modules::func::{error_output, openai_config};

pub fn register() -> CreateCommand {
    CreateCommand::new("chat")
        .description("與ChatGPT聊天")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "message", "給ChatGPT的訊息")
                .required(true),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::Boolean,
            "public",
            "是否公開顯示",
        ))
}

pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    options: &'a [ResolvedOption<'a>],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let prompt = options.iter().find(|opt| opt.name == "message");
    let public_result = options.iter().find(|opt| opt.name == "public");

    let public = if let Some(public_option) = public_result {
        if let ResolvedValue::Boolean(public) = public_option.value {
            public
        } else {
            false
        }
    } else {
        false
    };
    if let Some(prompt) = prompt {
        if let ResolvedValue::String(prompt) = prompt.value {
            if let Err(err) = chat(ctx, command, prompt, &public).await {
                println!("{} OpenAI mission failed: {}", error_output(), err)
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
    message: &str,
    public: &bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = Client::with_config(openai_config());
    let req = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o")
        .max_tokens(4096_u16)
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content("請用繁體中文回覆，並將字數控制為2000字以內。")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(message)
                .build()?
                .into(),
        ])
        .build()?;

    let mut stream = client.chat().create_stream(req).await?;

    let data = CreateInteractionResponseMessage::new()
        .content("回覆中請稍後...".to_string())
        .ephemeral(!public);
    let builder = CreateInteractionResponse::Message(data);
    command.create_response(&ctx.http, builder).await?;
    let mut message = String::from("");
    let mut count = 1;

    while let Some(result) = stream.next().await {
        match result {
            Ok(res) => {
                for choice in res.choices.iter() {
                    if let Some(ref content) = choice.delta.content {
                        message.push_str(content);
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
