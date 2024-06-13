use std::error::Error;

use serenity::all::{CommandInteraction, Context, CreateCommand};

use crate::modules::func::{error_output, voice_output};

pub async fn run(
    ctx: &Context,
    command: &CommandInteraction,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let guild_id = command.guild_id.ok_or("GuildID not found")?;
    let manager = songbird::get(ctx)
        .await
        .ok_or("Failed to retrieve songbird manager.")?;

    if let Some(call) = manager.get(guild_id) {
        let mut call_lock = call.lock().await;
        if let Some(_bot_channel) = call_lock.current_channel() {
            match call_lock.leave().await {
                Ok(_) => {
                    println!("{} Leave channel success", voice_output(),);
                    Ok("機器人已離開頻道".to_string())
                }
                Err(err) => {
                    println!("{} Failed to leave channel: {}", error_output(), err);
                    Ok("機器人離開頻道失敗".to_string())
                }
            }
        } else {
            Ok("機器人不在語音頻道中".to_string())
        }
    } else {
        Ok("機器人不在語音頻道中".to_string())
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("leave").description("讓機器人離開語音頻道")
}
