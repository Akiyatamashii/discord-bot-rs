use std::sync::Arc;

use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, ResolvedOption, ResolvedValue,
};

use crate::{
    modules::music::{catch, get_video_info},
    MusicList, MusicTemp,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("music_search")
        .description("搜尋音樂")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "query_or_url",
                "搜尋關鍵字 or 影片網址",
            )
            .required(true),
        )
}

pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    music_list_temp: MusicTemp,
    music_list: MusicList,
    option: &'a [ResolvedOption<'a>],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let query = option.iter().find(|opt| opt.name == "query_or_url");
    if let Some(query) = query {
        if let ResolvedValue::String(query) = query.value {
            if query.trim().starts_with("https://www.youtube.com/") {
                let music = get_video_info(query).await.unwrap();
                let content = format!("已將 \"{}\" 加入至音樂列表", music.title);
                {
                    let mut music_lock = music_list.write().await;
                    music_lock.push(music);
                }
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                command.create_response(&ctx.http, builder).await?;
                return Ok("".to_string());
            } else {
                {
                    music_list_temp.write().await.clear();
                }
                catch(&query, Arc::clone(&music_list_temp)).await;
                if !music_list_temp.read().await.is_empty() {
                    let mut content = String::new();
                    content.push_str("V 搜尋結果 V\n");
                    for i in 1..=10 {
                        if let Some(music) = music_list_temp.read().await.get(&i) {
                            content.push_str(format!("{}. {}\n", i, music.title.clone()).as_str())
                        }
                    }
                    let data = CreateInteractionResponseMessage::new().content(content);
                    let builder = CreateInteractionResponse::Message(data);
                    command.create_response(&ctx.http, builder).await?;
                }
            }
        } else {
            return Ok("獲取參數失敗".to_string());
        }
    } else {
        return Ok("獲取參數失敗".to_string());
    }
    return Ok("".to_string());
}
