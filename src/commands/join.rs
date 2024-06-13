use std::{collections::HashMap, error::Error};

use serenity::all::{CommandInteraction, Context, CreateCommand, UserId, VoiceState};

use crate::modules::func::{error_output, voice_output};

pub async fn run(
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let guild_id = command.guild_id.ok_or("GuildID not found")?;
    let user_id = command.user.id;
    let guild = ctx.cache.guild(guild_id).ok_or("Guild not found")?.clone();
    let voice_state: HashMap<UserId, VoiceState> = guild.voice_states;
    let channel_id = voice_state.get(&user_id).and_then(|state| state.channel_id);

    let manager = songbird::get(ctx)
        .await
        .ok_or("Failed to retrieve songbird manager.")?;

    if let Some(channel_id) = channel_id {
        if let Some(call) = manager.get(guild_id) {
            let mut call_lock = call.lock().await;
            if let Some(bot_channel) = call_lock.current_channel() {
                if bot_channel == channel_id.into() {
                    Ok("機器人已經在頻道中".to_string())
                } else {
                    match call_lock.join(channel_id).await {
                        Ok(_) => {
                            println!(
                                "{} Change to {} channel success",
                                voice_output(),
                                channel_id.name(&ctx.http).await?
                            );
                            Ok("成功切換頻道".to_string())
                        }
                        Err(err) => {
                            println!(
                                "{} Failed to change to {} channel: {}",
                                error_output(),
                                channel_id.name(&ctx.http).await?,
                                err
                            );
                            Ok("切換頻道失敗".to_string())
                        }
                    }
                }
            } else {
                match call_lock.join(channel_id).await {
                    Ok(_) => {
                        println!(
                            "{} Join to {} channel success",
                            voice_output(),
                            channel_id.name(&ctx.http).await?
                        );
                        Ok("成功加入頻道".to_string())
                    }
                    Err(err) => {
                        println!(
                            "{} Failed to join to {} channel: {}",
                            error_output(),
                            channel_id.name(&ctx.http).await?,
                            err
                        );
                        Ok("加入頻道失敗".to_string())
                    }
                }
            }
        } else {
            match manager.join(guild_id, channel_id).await {
                Ok(_) => {
                    println!(
                        "{} Join to {} channel success",
                        voice_output(),
                        channel_id.name(&ctx.http).await?
                    );
                    Ok("成功加入頻道".to_string())
                }
                Err(err) => {
                    println!(
                        "{} Failed to join to {} channel: {}",
                        error_output(),
                        channel_id.name(&ctx.http).await?,
                        err
                    );
                    Ok("加入頻道失敗".to_string())
                }
            }
        }
    } else {
        Ok("請先加入一個語音頻道".to_string())
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("join").description("加入機器人到當前語音頻道")
}
