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

// 註冊 chat 命令
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
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "model", "選擇的模型")
                .add_string_choice("gpt-4o", "chatgpt-4o-latest")
                .add_string_choice("gpt-4-turbo", "gpt-4-turbo")
                .add_string_choice("gpt-3.5-turbo", "gpt-3.5-turbo"),
        )
}

// 執行 chat 命令的主函數
pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    options: &'a [ResolvedOption<'a>],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // 從選項中獲取 message、public 和 model 的值
    let prompt = options.iter().find(|opt| opt.name == "message");
    let public_result = options.iter().find(|opt| opt.name == "public");
    let model_result = options.iter().find(|opt| opt.name == "model");

    // 處理 public 選項，默認為 false
    let public = if let Some(public_option) = public_result {
        if let ResolvedValue::Boolean(public) = public_option.value {
            public
        } else {
            false
        }
    } else {
        false
    };

    // 處理 model 選項，默認為 "gpt-4o-mini"
    let model = if let Some(model_option) = model_result {
        if let ResolvedValue::String(model) = model_option.value {
            model.trim()
        } else {
            "gpt-4o-mini"
        }
    } else {
        "gpt-4o-mini"
    };

    // 處理 message 選項
    if let Some(prompt) = prompt {
        if let ResolvedValue::String(prompt) = prompt.value {
            // 調用 chat 函數進行對話
            if let Err(err) = chat(ctx, command, prompt, &public, model).await {
                println!("{} OpenAI mission failed: {}", error_output(), err)
            }
            return Ok("".to_string());
        } else {
            return Ok(">> Prompt參數轉換有問題".to_string());
        }
    }
    Ok(">> 未提供提示詞".to_string())
}

// 進行對話的核心函數
async fn chat(
    ctx: &Context,
    command: &CommandInteraction,
    message: &str,
    public: &bool,
    model: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // 創建 OpenAI 客戶端
    let client = Client::with_config(openai_config());

    // 構建對話請求
    let req = CreateChatCompletionRequestArgs::default()
        .model(model)
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

    // 創建流式回應
    let mut stream = client.chat().create_stream(req).await?;

    // 創建初始回應，告知用戶正在處理
    let data = CreateInteractionResponseMessage::new()
        .content(">> 回覆中請稍後...".to_string())
        .ephemeral(!public);
    let builder = CreateInteractionResponse::Message(data);
    command.create_response(&ctx.http, builder).await?;

    let mut message = String::from("");
    let mut count = 1;

    // 處理流式回應
    while let Some(result) = stream.next().await {
        match result {
            Ok(res) => {
                for choice in res.choices.iter() {
                    if let Some(ref content) = choice.delta.content {
                        message.push_str(content);
                        // 每10次更新一次回應
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
                // 處理錯誤
                let builder = EditInteractionResponse::new().content(">> 獲取OpenAI訊息失敗");
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

    // 更新最終回應
    let builder = EditInteractionResponse::new().content(message);
    command.edit_response(&ctx.http, builder).await?;

    Ok(())
}
