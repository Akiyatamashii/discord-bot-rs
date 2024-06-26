use reqwest::Client;
use serenity::all::{CommandInteraction, Context, CreateCommand};
use songbird::{input::YoutubeDl, tracks::Track};

use crate::{MusicInfo, MusicList};

pub async fn run(
    ctx: &Context,
    command: &CommandInteraction,
    music_list: MusicList,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if music_list.read().await.is_empty() {
        return Ok("歌單內沒有歌曲".to_string());
    };

    let guild_id = command.guild_id.ok_or("GuildID not found")?;
    let manager = songbird::get(ctx)
        .await
        .ok_or("Failed to retrieve songbird manager.")?;
    if let Some(call) = manager.get(guild_id) {
        let mut call_lock = call.lock().await;
        let music: MusicInfo;
        {
            let music_lock = music_list.write().await;
            music = music_lock.first().unwrap().clone();
        }
        let url = music.http;
        let lazy = YoutubeDl::new(Client::new(), url);
        let lazy_c = lazy.clone();
        call_lock.play_input(lazy.into());
        call_lock.play(Track::from(lazy_c)).set_volume(1_f32)?;
    }

    Ok("成功".to_string())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("play").description("播放音樂")
}
