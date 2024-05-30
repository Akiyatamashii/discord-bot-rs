use std::{collections::HashMap, error::Error};

use serenity::all::{Context, Message, UserId, VoiceState};

pub async fn run(ctx: &Context, msg: &Message) -> Result<String, Box<dyn Error + Send + Sync>> {
    let guild_id = msg.guild_id.unwrap();

    let manager = songbird::get(ctx)
        .await
        .ok_or("Failed to retrieve songbird manager.")?;

    if let Some(call) = manager.get(guild_id) {
        let mut call_lock = call.lock().await;
        if call_lock.leave().await.is_ok() {
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
            Ok(format!(
                "Left the {} channel successfully.",
                channel_id.name(ctx).await.unwrap()
            )
            .to_string())
        } else {
            Ok("Failed to leave the voice channel.".to_string())
        }
    } else {
        Ok("Bot is not in a voice channel.".to_string())
    }
}
