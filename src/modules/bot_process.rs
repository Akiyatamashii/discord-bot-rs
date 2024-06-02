use colored::Colorize;
use serenity::all::{CommandInteraction, Context, Message};

use crate::{commands, modules::func::error_output, Handler};

use super::func::{check_permission, interaction_response, register_commands};

pub async fn prefix_command_process(ctx: &Context, msg: &Message) {
    let content = &msg.content;

    if content == "!register" {
        println!("get command !register");
        let guild_id = msg.guild_id.unwrap();
        register_commands(&ctx, &guild_id, false).await;
        msg.delete(&ctx.http).await.unwrap();
    }
}

pub async fn interaction_process(handler: &Handler, ctx: &Context, command: &CommandInteraction) {
    let _ = match command.data.name.as_str() {
        "join" => {
            let msg = commands::join::run(&ctx, &command).await.unwrap();
            interaction_response(&ctx, &command, msg, true).await;
            true
        }
        "leave" => {
            let msg = commands::leave::run(&ctx, &command).await.unwrap();
            interaction_response(&ctx, &command, msg, true).await;
            true
        }
        "ping" => {
            let msg = commands::ping::run(&command.data.options());
            interaction_response(&ctx, &command, msg, true).await;
            true
        }
        "info" => {
            commands::info::run(&ctx, &command, &command.data.options()).await;
            true
        }
        "look" => {
            let guild_id = command.guild_id.unwrap();
            let channel_id = command.channel_id;
            let msg = commands::look::run(guild_id, channel_id);
            interaction_response(&ctx, &command, msg, true).await;
            true
        }

        "chat" => match commands::chat::run(&ctx, &command, &command.data.options()).await {
            Ok(msg) => {
                if msg != "" {
                    interaction_response(&ctx, &command, msg, true).await;
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
        "image" => match commands::image::run(&ctx, &command, &command.data.options()).await {
            Ok(msg) => {
                if msg != "" {
                    interaction_response(&ctx, &command, msg, true).await;
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
        "remind" => {
            if !check_permission(&ctx, &command).await {
                return;
            }
            let channel_id = command.channel_id;
            let guild_id = command.guild_id.unwrap();
            match commands::remind::run(
                &command.data.options(),
                handler.reminders.clone(),
                channel_id,
                guild_id,
                &handler.trigger_notify,
            )
            .await
            {
                Ok(msg) => {
                    interaction_response(&ctx, &command, msg, true).await;
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
            if !check_permission(&ctx, &command).await {
                return;
            }
            let channel_id = command.channel_id;
            let guild_id = command.guild_id.unwrap();
            match commands::rm_remind::run(
                &command.data.options(),
                handler.reminders.clone(),
                channel_id,
                guild_id,
            )
            .await
            {
                Ok(msg) => {
                    interaction_response(&ctx, &command, msg, true).await;
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
        "music_search" => {
            match commands::music_search::run(
                &ctx,
                &command,
                handler.music_list_temp.clone(),
                handler.music_list.clone(),
                &command.data.options(),
            )
            .await
            {
                Ok(msg) => {
                    if msg != "" {
                        interaction_response(&ctx, &command, msg, true).await;
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
        "music_select" => {
            match commands::music_select::run(
                handler.music_list_temp.clone(),
                handler.music_list.clone(),
                &command.data.options(),
            )
            .await
            {
                Ok(msg) => {
                    if msg != "" {
                        interaction_response(&ctx, &command, msg, true).await;
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
        "music_look" => {
            let msg = commands::music_look::run(handler.music_list.clone()).await;
            interaction_response(&ctx, &command, msg, false).await;
            true
        }
        _ => false,
    };
}
