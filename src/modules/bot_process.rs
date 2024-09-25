use colored::Colorize;
use serenity::all::{CommandInteraction, Context, Message};

use crate::{commands, modules::func::error_output, Handler};

use super::func::{check_permission, interaction_response, register_commands};

pub async fn prefix_command_process(ctx: &Context, msg: &Message) {
    let content = &msg.content;

    if content == "!register" {
        println!("get command !register");
        let guild_id = msg.guild_id.unwrap();
        register_commands(ctx, &guild_id, false).await;
        msg.delete(&ctx.http).await.unwrap();
    }
}

pub async fn interaction_process(handler: &Handler, ctx: &Context, command: &CommandInteraction) {
    let _ = match command.data.name.as_str() {
        "ping" => {
            let msg = commands::base::ping::run(&command.data.options());
            interaction_response(ctx, command, msg, true).await;
            true
        }
        "info" => {
            commands::base::info::run(ctx, command, &command.data.options()).await;
            true
        }
        "look" => {
            let guild_id = command.guild_id.unwrap();
            let channel_id = command.channel_id;
            let msg = commands::reminder::look::run(guild_id, channel_id);
            interaction_response(ctx, command, msg, true).await;
            true
        }
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
        "model_list" => {
            let msg = commands::openai::model_list::run(&command.data.options()).await;
            interaction_response(ctx, command, msg, true).await;
            true
        }

        "cash" => {
            commands::cash::run(ctx, command, &command.data.options()).await;
            true
        }
        _ => false,
    };
}
