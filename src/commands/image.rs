use std::error::Error;

use async_openai::{
    types::{CreateImageRequestArgs, Image::Url, ImageModel, ImageSize, ResponseFormat},
    Client,
};
use serenity::{
    all::{
        CommandInteraction, CommandOptionType, Context, CreateCommandOption,
        CreateInteractionResponse, CreateInteractionResponseMessage, EditInteractionResponse,
        ResolvedOption, ResolvedValue,
    },
    builder::CreateCommand,
};

use crate::modules::func::{error_output, openai_config};

pub fn register() -> CreateCommand {
    CreateCommand::new("image")
        .description("生成圖片")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "prompt", "提示詞").required(true),
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
    let prompt = options.iter().find(|opt| opt.name == "prompt");
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
            if let Err(err) = image(ctx, command, prompt, &public).await {
                println!("{} OpenAI mission failed: {}", error_output(), err)
            }
            return Ok("".to_string());
        } else {
            return Ok("Prompt參數轉換有問題".to_string());
        }
    }
    Ok("未提供提示詞".to_string())
}

async fn image(
    ctx: &Context,
    command: &CommandInteraction,
    prompt: &str,
    public: &bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = Client::with_config(openai_config());

    let req = CreateImageRequestArgs::default()
        .prompt(prompt)
        .n(1)
        .response_format(ResponseFormat::Url)
        .size(ImageSize::S1024x1024)
        .model(ImageModel::DallE3)
        .user(&command.user.name)
        .build()?;

    let data = CreateInteractionResponseMessage::new()
        .content("生成中請稍後...")
        .ephemeral(!public);
    let builder = CreateInteractionResponse::Message(data);
    if let Err(err) = command.create_response(&ctx.http, builder).await {
        println!("{} Failed to send respond:{}", error_output(), err)
    }
    let res = client.images().create(req).await?;
    let mut img_url = String::new();

    if let Url {
        url,
        revised_prompt: _,
    } = &*res.data[0]
    {
        img_url.clone_from(url);
    } else {
        let builder = EditInteractionResponse::new().content("獲取圖片連結失敗");
        if let Err(err) = command.edit_response(&ctx.http, builder).await {
            println!("{} Failed to send respond:{}", error_output(), err)
        }
    }

    let builder = EditInteractionResponse::new().content(img_url);
    if let Err(err) = command.edit_response(&ctx.http, builder).await {
        println!("{} Failed to send respond:{}", error_output(), err)
    }

    Ok(())
}
