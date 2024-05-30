use std::{collections::HashMap, error::Error};

use serenity::all::{Context, Message, UserId, VoiceState};

pub async fn run(ctx: &Context, msg: &Message) -> Result<String, Box<dyn Error + Send + Sync>> {
    let guild_id = match msg.guild_id {
        Some(gid) => gid,
        None => return Ok("This command can only be used in a guild.".to_string()),
    };

    let user_id = msg.author.id;

    let channel_id = {
        let guild = guild_id
            .to_guild_cached(ctx)
            .ok_or("Failed to retrieve guild from cache.")?;
        let voice_state: &HashMap<UserId, VoiceState> = &guild.voice_states;
        voice_state
            .get(&user_id)
            .and_then(|voice_state| voice_state.channel_id)
            .ok_or("You must be in a voice channel to use this command!")?
    };

    let manager = songbird::get(ctx)
        .await
        .ok_or("Failed to retrieve songbird manager.")?;

    if let Some(call) = manager.get(guild_id) {
        let mut call_lock = call.lock().await;
        if let Some(bot_channel) = call_lock.current_channel() {
            if bot_channel == channel_id.into() {
                return Ok("Bot is already in the channel.".to_string());
            } else {
                match call_lock.join(channel_id).await {
                    Ok(_) => {
                        return Ok(format!(
                            "Changed to {} channel successfully.",
                            channel_id.name(&ctx).await?
                        )
                        .to_string());
                    }
                    Err(_) => {
                        return Ok("Failed to change channel.".to_string());
                    }
                }
            }
        } else {
            match manager.join(guild_id, channel_id).await {
                Ok(call) => {
                    if !call.lock().await.current_channel().is_none() {
                        return Ok(format!(
                            "Join to {} channel successfully.",
                            channel_id.name(&ctx).await?
                        )
                        .to_string());
                    } else {
                        return Ok(format!(
                            "Failed to Join {} channel.",
                            channel_id.name(&ctx).await?
                        )
                        .to_string());
                    }
                }
                Err(_) => {
                    return Ok("Failed to join channel.".to_string());
                }
            }
        }
    } else {
        match manager.join(guild_id, channel_id).await {
            Ok(call) => {
                if !call.lock().await.current_channel().is_none() {
                    return Ok(format!(
                        "Join to {} channel successfully.",
                        channel_id.name(&ctx).await?
                    )
                    .to_string());
                } else {
                    return Ok(
                        format!("Failed to Join {} channel.", channel_id.name(&ctx).await?)
                            .to_string(),
                    );
                }
            }
            Err(_) => {
                return Ok("Failed to join channel.".to_string());
            }
        }
    }
}
