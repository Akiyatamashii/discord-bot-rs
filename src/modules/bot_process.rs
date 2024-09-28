use std::sync::Arc;

use colored::Colorize;
use serenity::all::{CommandInteraction, Context, Message};

use crate::{commands, modules::func::error_output, Handler};

use super::func::{check_permission, interaction_response, register_commands};

// 處理前綴命令的函數
pub async fn prefix_command_process(ctx: &Context, msg: &Message) {
    let content = &msg.content;

    // 處理 !register 命令
    if content == "!register" {
        println!("get command !register");
        let guild_id = msg.guild_id.unwrap();
        // 註冊斜線命令
        register_commands(ctx, &guild_id, false).await;
        // 刪除觸發命令的消息
        msg.delete(&ctx.http).await.unwrap();
    }
}

// 處理斜線命令的函數
pub async fn interaction_process(handler: &Handler, ctx: &Context, command: &CommandInteraction) {
    let _ = match command.data.name.as_str() {
        // 處理 ping 命令
        "ping" => {
            let msg = commands::base::ping::run(&command.data.options());
            interaction_response(ctx, command, msg, true).await;
            true
        }
        // 處理 info 命令
        "info" => {
            commands::base::info::run(ctx, command, &command.data.options()).await;
            true
        }
        // 處理 update 命令（查看更新日誌）
        "update" => {
            commands::base::update::run(ctx, command, &command.data.options())
                .await
                .unwrap();
            true
        }
        // 處理 look 命令（查看提醒）
        "look" => {
            let guild_id = command.guild_id.unwrap();
            let channel_id = command.channel_id;
            let msg = commands::reminder::look::run(guild_id, channel_id);
            interaction_response(ctx, command, msg, true).await;
            true
        }
        // 處理 remind 命令（設置提醒）
        "remind" => {
            if !check_permission(ctx, command).await {
                return;
            }
            let channel_id = command.channel_id;
            let guild_id = command.guild_id.unwrap();
            match commands::reminder::remind::run(
                &command.data.options(),
                handler.reminders.clone(),
                channel_id,
                guild_id,
                &handler.trigger_notify,
            )
            .await
            {
                Ok(msg) => {
                    interaction_response(ctx, command, msg, true).await;
                    true
                }
                Err(err) => {
                    println!(
                        "{} {} {}",
                        error_output(),
                        "Failed to set reminder:".red(),
                        err
                    );
                    false
                }
            }
        }
        // 處理 rm_remind 命令（刪除提醒）
        "rm_remind" => {
            if !check_permission(ctx, command).await {
                return;
            }
            let channel_id = command.channel_id;
            let guild_id = command.guild_id.unwrap();
            match commands::reminder::rm_remind::run(
                &command.data.options(),
                handler.reminders.clone(),
                channel_id,
                guild_id,
            )
            .await
            {
                Ok(msg) => {
                    interaction_response(ctx, command, msg, true).await;
                    true
                }
                Err(err) => {
                    println!(
                        "{} {} {}",
                        error_output(),
                        "Failed to remove reminder:".red(),
                        err
                    );
                    false
                }
            }
        }

        // 處理 chat 命令（OpenAI 聊天）
        "chat" => match commands::openai::chat::run(ctx, command, &command.data.options()).await {
            Ok(msg) => {
                if !msg.is_empty() {
                    interaction_response(ctx, command, msg, true).await;
                }
                true
            }
            Err(err) => {
                println!(
                    "{} {} {}",
                    error_output(),
                    "OpenAI mission filed:".red(),
                    err
                );
                false
            }
        },
        // 處理 image 命令（OpenAI 圖像生成）
        "image" => {
            match commands::openai::image::run(ctx, command, &command.data.options()).await {
                Ok(msg) => {
                    if msg.is_empty() {
                        interaction_response(ctx, command, msg, true).await;
                    }
                    true
                }
                Err(err) => {
                    println!(
                        "{} {} {}",
                        error_output(),
                        "OpenAI mission filed:".red(),
                        err
                    );
                    false
                }
            }
        }
        // 處理 model_list 命令（OpenAI 模型列表）
        "model_list" => {
            let msg = commands::openai::model_list::run(&command.data.options()).await;
            interaction_response(ctx, command, msg, true).await;
            true
        }

        // 處理 cash 命令（欠債系統）
        "cash" => {
            commands::cash::run(ctx, command, &command.data.options()).await;
            true
        }
        // 處理 tiktok_msg_add 命令（添加拒絕 TikTok 訊息）
        "tiktok_msg_add" => {
            let tiktok_refuse_msg = Arc::clone(&handler.tiktok_refuse_msg);
            if let Ok(msg) = commands::tiktok_refuse::tiktok_msg_add::run(
                &command.data.options(),
                tiktok_refuse_msg,
            )
            .await
            {
                interaction_response(ctx, command, msg, true).await;
            }
            true
        }

        // 未知命令
        _ => false,
    };
}
