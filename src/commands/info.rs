use std::fs;

use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, ResolvedOption, ResolvedValue,
};

use crate::modules::func::{error_output, info_path};

pub fn register() -> CreateCommand {
    CreateCommand::new("info")
        .description("獲取機器人資訊與指令列表")
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "type",
            "功能類型選擇",
        ))
}

pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    option: &'a [ResolvedOption<'a>],
) {
    let info_type_result = option.iter().find(|opt| opt.name == "type");
    let info_type = if let Some(info_type) = info_type_result {
        if let ResolvedValue::String(info_type) = info_type.value {
            info_type
        } else {
            ""
        }
    } else {
        ""
    };

    if info_type.is_empty() {
        info(ctx, command).await
    } else {
        info_with_type(ctx, command, info_type).await
    }
}

async fn info(ctx: &Context, command: &CommandInteraction) {
    let mut file_path = info_path();
    file_path.push_str("info.md");
    let content = match fs::read_to_string(file_path) {
        Ok(ctx) => ctx,
        Err(err) => {
            println!("{} Failed to read info file:{}", error_output(), err);
            let data = CreateInteractionResponseMessage::new().content("讀取資訊失敗");
            let builder = CreateInteractionResponse::Message(data);
            if let Err(err) = command.create_response(&ctx.http, builder).await {
                println!("{} Failed to send respond:{}", error_output(), err)
            }
            return;
        }
    };
    let data = CreateInteractionResponseMessage::new().content(content);
    let builder = CreateInteractionResponse::Message(data);
    if let Err(err) = command.create_response(&ctx.http, builder).await {
        println!("{} Failed to send respond:{}", error_output(), err)
    }
}

async fn info_with_type(ctx: &Context, command: &CommandInteraction, info_type: &str) {
    let fold_path = info_path();
    let file_name = format!("{}.md", info_type);
    let file_exists = fs::read_dir(fold_path)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .any(|entry| *entry.file_name() == *file_name)
        })
        .unwrap_or(false);

    if !file_exists {
        let data =
            CreateInteractionResponseMessage::new().content("參數不存在，請輸入正確的類別代號");
        let builder = CreateInteractionResponse::Message(data);
        if let Err(err) = command.create_response(&ctx.http, builder).await {
            println!("{} Failed to send respond:{}", error_output(), err)
        }
        return;
    }

    let mut file_path = info_path();
    file_path.push_str(format!("{}.md", info_type).as_str());

    let content = match fs::read_to_string(file_path) {
        Ok(ctx) => ctx,
        Err(err) => {
            println!("{} Failed to read info file:{}", error_output(), err);
            let data = CreateInteractionResponseMessage::new().content("讀取資訊失敗");
            let builder = CreateInteractionResponse::Message(data);
            if let Err(err) = command.create_response(&ctx.http, builder).await {
                println!("{} Failed to send respond:{}", error_output(), err)
            }
            return;
        }
    };
    let data = CreateInteractionResponseMessage::new().content(content);
    let builder = CreateInteractionResponse::Message(data);
    if let Err(err) = command.create_response(&ctx.http, builder).await {
        println!("{} Failed to send respond:{}", error_output(), err)
    }
}
