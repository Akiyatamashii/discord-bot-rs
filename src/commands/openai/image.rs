use std::error::Error;

use async_openai::{
    types::{CreateImageRequestArgs, Image::Url, ImageModel, ImageResponseFormat, ImageSize},
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

// Register the image command
// 註冊 image 命令
pub fn register() -> CreateCommand {
    CreateCommand::new("image")
        .description("Generate an image")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "prompt", "Prompt for image generation").required(true),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "model",
            "Choose a model",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::Boolean,
            "public",
            "Whether to display publicly",
        ))
}

// Main function to execute the image command
// 執行 image 命令的主函數
pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    options: &'a [ResolvedOption<'a>],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Get values for prompt, public, and model from options
    // 從選項中獲取 prompt、public 和 model 的值
    let prompt = options.iter().find(|opt| opt.name == "prompt");
    let public_result = options.iter().find(|opt| opt.name == "public");
    let model_result = options.iter().find(|opt| opt.name == "model");

    // Handle the public option, default is false
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

    // Handle the model option, default is "dall-e-3"
    // 處理 model 選項，默認為 "dall-e-3"
    let model = if let Some(model_option) = model_result {
        if let ResolvedValue::String(model) = model_option.value {
            model
        } else {
            "dall-e-3"
        }
    } else {
        "dall-e-3"
    };

    // Handle the prompt option
    // 處理 prompt 選項
    if let Some(prompt) = prompt {
        if let ResolvedValue::String(prompt) = prompt.value {
            // Call the image function to generate an image
            // 調用 image 函數生成圖片
            if let Err(err) = image(ctx, command, prompt, model, &public).await {
                println!("{} OpenAI mission failed: {}", error_output(), err)
            }
            return Ok("".to_string());
        } else {
            return Ok(">> Problem converting Prompt parameter".to_string());
        }
    }
    Ok(">> No prompt provided".to_string())
}

// Core function for generating an image
// 生成圖片的核心函數
async fn image(
    ctx: &Context,
    command: &CommandInteraction,
    prompt: &str,
    model: &str,
    public: &bool,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create OpenAI client
    // 創建 OpenAI 客戶端
    let client = Client::with_config(openai_config());

    // Build image generation request
    // 構建圖片生成請求
    let req = CreateImageRequestArgs::default()
        .prompt(prompt)
        .n(1)
        .response_format(ImageResponseFormat::Url)
        .size(ImageSize::S1024x1024)
        .model(ImageModel::Other(model.to_string()))
        .user(&command.user.name)
        .build()?;

    // Create initial response, informing the user that the image is being generated
    // 創建初始回應，告知用戶正在生成圖片
    let data = CreateInteractionResponseMessage::new()
        .content(">> Generating, please wait...")
        .ephemeral(!public);
    let builder = CreateInteractionResponse::Message(data);
    if let Err(err) = command.create_response(&ctx.http, builder).await {
        println!("{} Failed to send respond:{}", error_output(), err)
    }

    // Send image generation request
    // 發送圖片生成請求
    let res = client.images().create(req).await?;
    let mut img_url = String::new();

    // Handle the returned image URL
    // 處理返回的圖片 URL
    if let Url {
        url,
        revised_prompt: _,
    } = &*res.data[0]
    {
        img_url.clone_from(url);
    } else {
        // If failed to get URL, update the response
        // 如果獲取 URL 失敗，更新回應
        let builder = EditInteractionResponse::new().content(">> Failed to get image URL");
        if let Err(err) = command.edit_response(&ctx.http, builder).await {
            println!("{} Failed to send respond:{}", error_output(), err)
        }
    }

    // Update response, send the generated image URL
    // 更新回應，發送生成的圖片 URL
    let builder = EditInteractionResponse::new().content(img_url);
    if let Err(err) = command.edit_response(&ctx.http, builder).await {
        println!("{} Failed to send respond:{}", error_output(), err)
    }

    Ok(())
}