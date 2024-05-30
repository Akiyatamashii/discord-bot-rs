use std::sync::Arc;

use serenity::all::CreateCommand;
use tokio::sync::RwLock;

use crate::MusicInfo;

pub fn register() -> CreateCommand {
    CreateCommand::new("music_look").description("查看音樂播放列表")
}

pub async fn run(music_list: Arc<RwLock<Vec<MusicInfo>>>) -> String {
    let mut content = String::new();
    if !music_list.read().await.is_empty() {
        content.push_str("V 音樂播放列表 V\n");
        for (index, music) in music_list.read().await.iter().enumerate() {
            content.push_str(format!("{}. {}\n", index + 1, music.title).as_str())
        }
    } else {
        content.push_str(">> 音樂列表中尚無音樂")
    }
    content
}
