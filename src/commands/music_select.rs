use std::{collections::HashMap, sync::Arc};

use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue,
};
use tokio::sync::RwLock;

use crate::modules::music::get_audio_url;
use crate::MusicInfo;

pub fn register() -> CreateCommand {
    CreateCommand::new("music_select")
        .description("選擇音樂")
        .add_option(
            CreateCommandOption::new(CommandOptionType::Number, "index", "搜尋結果之索引")
                .required(true),
        )
}

pub async fn run<'a>(
    music_list_temp: Arc<RwLock<HashMap<usize, MusicInfo>>>,
    music_list: Arc<RwLock<Vec<MusicInfo>>>,
    option: &'a [ResolvedOption<'a>],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let index = option.iter().find(|opt| opt.name == "index");
    if let Some(i) = index {
        if let ResolvedValue::Number(i) = i.value {
            let i = i as usize;
            let choice_music: MusicInfo;
            {
                let mut temp = music_list_temp.write().await;
                if temp.is_empty() {
                    return Ok("請先搜尋音樂".to_string());
                }
                let mut temp_music = temp.get(&i).unwrap().clone();
                temp_music.watch = get_audio_url(&temp_music.http).await;
                choice_music = temp_music;
                temp.clear()
            }
            let mut music = music_list.write().await;
            music.push(choice_music);
            return Ok("以將歌曲加入列表".to_string());
        } else {
            return Ok("獲取參數失敗".to_string());
        }
    }
    Ok("".to_string())
}
