use std::fs;

use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, ResolvedOption, ResolvedValue,
};

use crate::modules::func::{error_output, info_path};

// Register the info command
// 註冊 info 命令
pub fn register() -> CreateCommand {
    CreateCommand::new("info")
        .description("get bot info and command list")
        .description_localized("zh-TW", "獲取機器人資訊與指令列表")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "type",
                "what part of info do you want to see",
            )
            .description_localized("zh-TW", "選擇要查看的資訊類型")
            .add_string_choice("common", "common")
            .add_string_choice("reminder", "reminder")
            .add_string_choice("ai", "ai")
            .add_string_choice("cash", "cash")
            .add_string_choice("anti_tiktok", "anti_tiktok")
            .add_string_choice("ban", "ban"),
        )
}

// Run the info command
// 運行 info 命令
pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    option: &'a [ResolvedOption<'a>],
) {
    // Find the "type" option from the input options
    // 從輸入選項中找到 "type" 選項
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

    // Call the appropriate function based on the info_type
    // 根據 info_type 調用適當的函數
    if info_type.is_empty() {
        info(ctx, command).await
    } else {
        info_with_type(ctx, command, info_type).await
    }
}

// Handle the info command without a specific type
// 處理沒有特定類型的 info 命令
async fn info(ctx: &Context, command: &CommandInteraction) {
    let mut file_path = info_path();
    file_path.push_str("info.md");

    // Read the content of the info file
    // 讀取 info 文件的內容
    let content = match fs::read_to_string(file_path) {
        Ok(ctx) => ctx
            .lines()
            .map(|line| format!("> {}", line).replace("+", "-"))
            .collect::<Vec<String>>()
            .join("\n"),
        Err(err) => {
            println!("{} Failed to read info file:{}", error_output(), err);
            let data = CreateInteractionResponseMessage::new().content(">> 讀取資訊失敗");
            let builder = CreateInteractionResponse::Message(data);
            if let Err(err) = command.create_response(&ctx.http, builder).await {
                println!("{} Failed to send respond:{}", error_output(), err)
            }
            return;
        }
    };

    // Send the response with the file content
    // 發送包含文件內容的回應
    let data = CreateInteractionResponseMessage::new().content(content);
    let builder = CreateInteractionResponse::Message(data);
    if let Err(err) = command.create_response(&ctx.http, builder).await {
        println!("{} Failed to send respond:{}", error_output(), err)
    }
}

// Handle the info command with a specific type
// 處理有特定類型的 info 命令
async fn info_with_type(ctx: &Context, command: &CommandInteraction, info_type: &str) {
    let fold_path = info_path();
    let file_name = format!("{}.md", info_type);

    // Check if the file exists
    // 檢查文件是否存在
    let file_exists = fs::read_dir(fold_path)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .any(|entry| *entry.file_name() == *file_name)
        })
        .unwrap_or(false);

    if !file_exists {
        let data =
            CreateInteractionResponseMessage::new().content(">> 參數不存在，請輸入正確的類別代號");
        let builder = CreateInteractionResponse::Message(data);
        if let Err(err) = command.create_response(&ctx.http, builder).await {
            println!("{} Failed to send respond:{}", error_output(), err)
        }
        return;
    }

    let mut file_path = info_path();
    file_path.push_str(format!("{}.md", info_type).as_str());

    // Read the content of the specific info file
    // 讀取特定 info 文件的內容
    let content = match fs::read_to_string(file_path) {
        Ok(ctx) => ctx
            .lines()
            .map(|line| format!("> {}", line).replace("+", "-"))
            .collect::<Vec<String>>()
            .join("\n"),
        Err(err) => {
            println!("{} Failed to read info file:{}", error_output(), err);
            let data = CreateInteractionResponseMessage::new().content(">> 讀取資訊失敗");
            let builder = CreateInteractionResponse::Message(data);
            if let Err(err) = command.create_response(&ctx.http, builder).await {
                println!("{} Failed to send respond:{}", error_output(), err)
            }
            return;
        }
    };

    // Send the response with the file content
    // 發送包含文件內容的回應
    let data = CreateInteractionResponseMessage::new().content(content);
    let builder = CreateInteractionResponse::Message(data);
    if let Err(err) = command.create_response(&ctx.http, builder).await {
        println!("{} Failed to send respond:{}", error_output(), err)
    }
}
