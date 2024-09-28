use std::{
    fs::{self, File},
    io::{BufReader, Read, Write},
};

use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    EditMessage, MessageId, ResolvedOption, ResolvedValue,
};

use crate::modules::func::{error_output, interaction_response};

pub fn register() -> CreateCommand {
    CreateCommand::new("update")
        .description("查看更新日誌")
        .add_option(CreateCommandOption::new(
            CommandOptionType::Boolean,
            "all",
            "查看所有更新日誌",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::Boolean,
            "create_msg",
            "創建更新日誌消息",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::Boolean,
            "update_msg",
            "更新更新日誌消息",
        ))
}

pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    options: &'a [ResolvedOption<'a>],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let all = options.iter().find(|option| option.name == "all");
    let create_msg = options.iter().find(|option| option.name == "create_msg");
    let update_msg = options.iter().find(|option| option.name == "update_msg");

    let show_all = if let Some(show_all) = all {
        if let ResolvedValue::Boolean(show_all) = show_all.value {
            show_all
        } else {
            false
        }
    } else {
        false
    };

    let create_msg = if let Some(create_msg) = create_msg {
        if let ResolvedValue::Boolean(create_msg) = create_msg.value {
            create_msg
        } else {
            false
        }
    } else {
        false
    };

    let update_msg = if let Some(update_msg) = update_msg {
        if let ResolvedValue::Boolean(update_msg) = update_msg.value {
            update_msg
        } else {
            false
        }
    } else {
        false
    };
    #[allow(unused_assignments)]
    let mut all_logs = String::new();
    let update_dir = "assets/update";

    if update_msg {
        // 讀取更新日誌消息ID
        let msg_id = read_update_msg_id(update_dir);

        if msg_id.is_empty() {
            interaction_response(ctx, command, "未找到有效的更新日誌消息ID".to_string(), true)
                .await;
            return Ok(());
        }

        // 將 msg_id (String) 轉換為 u64，然後轉換為 MessageId
        let msg_id_u64 = match msg_id.parse::<u64>() {
            Ok(id) => id,
            Err(_) => {
                interaction_response(ctx, command, "更新日誌消息ID無效".to_string(), true).await;
                return Ok(());
            }
        };

        let msg = command
            .channel_id
            .message(ctx.http.clone(), MessageId::new(msg_id_u64))
            .await;

        match msg {
            Ok(mut message) => {
                // 讀取所有更新日誌
                let all_logs = read_all_update_logs(update_dir);

                // 編輯消息以包含所有更新日誌
                let builder = EditMessage::new().content(all_logs); // 使用 EditMessage 來編輯消息
                if let Err(err) = message.edit(&ctx.http, builder).await {
                    println!("Failed to edit message: {}", err);
                }
            }
            Err(err) => {
                interaction_response(ctx, command, "無法找到更新日誌消息".to_string(), true).await;
                println!("Error getting message: {:?}", err);
            }
        }
        return Ok(());
    }

    if create_msg {
        all_logs = read_all_update_logs(update_dir);
        let channel_id = command.channel_id;
        let msg = channel_id.say(ctx, all_logs).await;
        match msg {
            Ok(msg) => {
                // 保存更新日誌消息ID到assets/update/update_msg_id.txt
                let msg_id = msg.id.to_string();
                let mut file = File::create("assets/update/update_msg_id.txt").unwrap();
                file.write_all(msg_id.as_bytes()).unwrap();
            }
            Err(err) => println!("{} 發送回應失敗：{}", error_output(), err),
        }
        return Ok(());
    }

    if show_all {
        // 讀取所有更新日誌文件
        all_logs = read_all_update_logs(update_dir);
        interaction_response(ctx, command, all_logs, false).await;
    } else {
        // 讀取最新更新日誌文件
        all_logs = read_latest_update_log(update_dir);
        if create_msg {
            interaction_response(ctx, command, all_logs, true).await;
        } else if update_msg {
            interaction_response(ctx, command, all_logs, false).await;
        }
    }
    Ok(())
}

fn read_all_update_logs(update_dir: &str) -> String {
    let mut all_logs = String::new();
    if let Ok(entries) = fs::read_dir(update_dir) {
        // 收集所有文件路徑
        let mut file_paths: Vec<_> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
            .collect();

        // 按照文件名排序（假設文件名格式為日期）
        file_paths.sort();

        // 從舊到新讀取每個文件
        for path in file_paths {
            if let Ok(file) = File::open(&path) {
                let mut reader = BufReader::new(file);
                let mut content = Vec::new();
                if reader.read_to_end(&mut content).is_ok() {
                    if let Ok(utf8_content) = String::from_utf8(content) {
                        all_logs.push_str(&utf8_content);
                        all_logs.push_str("\n\n"); // 在每個日誌之間添加空行
                    }
                }
            }
        }
    } else {
        all_logs = "無法讀取更新日誌目錄".to_string();
    }
    all_logs
}

fn read_latest_update_log(update_dir: &str) -> String {
    #[allow(unused_assignments)]
    let mut all_logs = String::new();
    // 讀取目錄中的所有文件
    if let Ok(entries) = fs::read_dir(update_dir) {
        // 收集所有文件路徑
        let mut file_paths: Vec<_> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
            .collect();

        // 按照文件名排序（假設文件名格式為日期）
        file_paths.sort();

        // 讀取最新的文件
        if let Some(latest_file) = file_paths.last() {
            if let Ok(file) = File::open(latest_file) {
                let mut reader = BufReader::new(file);
                let mut content = Vec::new();
                if reader.read_to_end(&mut content).is_ok() {
                    if let Ok(utf8_content) = String::from_utf8(content) {
                        all_logs = utf8_content;
                    } else {
                        all_logs = "無法解析最新的更新日誌為UTF-8格式".to_string();
                    }
                } else {
                    all_logs = "無法讀取最新的更新日誌".to_string();
                }
            } else {
                all_logs = "無法打開最新的更新日誌文件".to_string();
            }
        } else {
            all_logs = "沒有找到更新日誌文件".to_string();
        }
    } else {
        all_logs = "無法讀取更新日誌目錄".to_string();
    }
    all_logs
}

fn read_update_msg_id(update_dir: &str) -> String {
    let mut msg_id = String::new();
    let mut file = File::open(format!("{}/update_msg_id.txt", update_dir)).unwrap();
    file.read_to_string(&mut msg_id).unwrap();
    msg_id
}
