use std::{
    fs::{self, File},
    io::{BufReader, Read},
};

use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    ResolvedOption, ResolvedValue,
};

use crate::modules::func::interaction_response;

// Register the update command
// 註冊 update 命令
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
            "public",
            "是否公開",
        ))
}

// Run the update command
// 運行 update 命令
pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    options: &'a [ResolvedOption<'a>],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse command options
    // 解析命令選項
    let all = options.iter().find(|option| option.name == "all");
    let public = options.iter().find(|option| option.name == "public");

    let show_all = if let Some(show_all) = all {
        if let ResolvedValue::Boolean(show_all) = show_all.value {
            show_all
        } else {
            false
        }
    } else {
        false
    };

    let is_public = if let Some(create_msg) = public {
        if let ResolvedValue::Boolean(create_msg) = create_msg.value {
            create_msg
        } else {
            false
        }
    } else {
        false
    };

    let update_dir = "assets/update";

    // Handle the "show all" option
    // 處理 "顯示所有" 選項
    if show_all {
        interaction_response(ctx, command, "查看所有更新日誌".to_string(), !is_public).await;
        let all_logs = read_all_update_logs(update_dir);
        for log in all_logs {
            command.channel_id.say(ctx, log).await?;
        }
    }

    // Read and display the latest update log
    // 讀取並顯示最新的更新日誌
    let all_logs = read_latest_update_log(update_dir);
    interaction_response(ctx, command, all_logs[0].clone(), !is_public).await;

    Ok(())
}

// Read the latest update log file
// 讀取最新的更新日誌文件
fn read_latest_update_log(update_dir: &str) -> Vec<String> {
    let mut all_logs = Vec::new();
    // Read all files in the directory
    // 讀取目錄中的所有文件
    if let Ok(entries) = fs::read_dir(update_dir) {
        // Collect all file paths
        // 收集所有文件路徑
        let mut file_paths: Vec<_> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
            .collect();

        // Sort files by name (assuming the file name format is a date)
        // 按照文件名排序（假設文件名格式為日期）
        file_paths.sort();

        // Read the latest file
        // 讀取最新的文件
        if let Some(latest_file) = file_paths.last() {
            if let Ok(file) = File::open(latest_file) {
                let mut reader = BufReader::new(file);
                let mut content = Vec::new();
                if reader.read_to_end(&mut content).is_ok() {
                    if let Ok(utf8_content) = String::from_utf8(content) {
                        all_logs.push(utf8_content);
                    } else {
                        all_logs.push("無法解析最新的更新日誌為UTF-8格式".to_string());
                    }
                } else {
                    all_logs.push("無法讀取最新的更新日誌".to_string());
                }
            } else {
                all_logs.push("無法打開最新的更新日誌文件".to_string());
            }
        } else {
            all_logs.push("沒有找到更新日誌文件".to_string());
        }
    } else {
        all_logs.push("無法讀取更新日誌目錄".to_string());
    }
    all_logs
}

// Read all update log files
// 讀取所有更新日誌文件
fn read_all_update_logs(update_dir: &str) -> Vec<String> {
    let mut all_logs = Vec::new();

    if let Ok(entries) = fs::read_dir(update_dir) {
        let mut file_paths: Vec<_> = entries
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("md"))
            .collect();

        file_paths.sort();

        for file_path in file_paths {
            if let Ok(file) = File::open(&file_path) {
                let mut reader = BufReader::new(file);
                let mut content = String::new();
                if reader.read_to_string(&mut content).is_ok() {
                    all_logs.push(content);
                }
            }
        }
    }
    all_logs
}