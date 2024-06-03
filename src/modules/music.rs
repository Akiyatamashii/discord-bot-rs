use std::{collections::HashMap, process::Command};

use regex::Regex;
use reqwest;
use serde_json::Value;
use urlencoding::encode;

use crate::{MusicInfo, MusicTemp};

pub async fn catch(query: &str, music_list_temp: MusicTemp) {
    let encoded_query = encode(query); // 對查詢字串進行 URL 編碼
    let url = format!(
        "https://www.youtube.com/results?search_query={}",
        encoded_query
    );
    let body = reqwest::get(&url).await.unwrap().text().await.unwrap();
    let re = Regex::new(r#"var ytInitialData = (.*?);</script>"#).unwrap();
    let mut yt_query_list: HashMap<usize, MusicInfo> = HashMap::new();
    if let Some(captures) = re.captures(&body) {
        let json_str = &captures[1];
        let json_value: Value = serde_json::from_str(json_str).unwrap();
        let mut index = 1;

        // 提取 videoRenderer 資訊
        if let Some(items) = json_value.pointer("/contents/twoColumnSearchResultsRenderer/primaryContents/sectionListRenderer/contents/0/itemSectionRenderer/contents") {
            for item in items.as_array().unwrap() {
                if let Some(video_renderer) = item.get("videoRenderer") {
                    if let Some(_video_id) = video_renderer.get("videoId").and_then(|v| v.as_str()) {
                        if let Some(title_obj) = video_renderer.pointer("/title/runs/0/text") {
                            if let Some(title) = title_obj.as_str() {
                                // 提取 watchEndpoint 相關資訊
                                if let Some(watch_endpoint) = video_renderer.pointer("/navigationEndpoint/watchEndpoint") {
                                    if let Some(video_url) = watch_endpoint.get("videoId").and_then(|v| v.as_str()) {
                                        let full_url = format!("https://www.youtube.com/watch?v={}", video_url);
                                        yt_query_list.insert(index, MusicInfo { title: title.to_string(), http: full_url,watch:None });
                                        index += 1;
                                        if index > 10 {
                                            break; // 只提取前十個影片
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    {
        let mut temp = music_list_temp.write().await;
        *temp = yt_query_list;
    }
}

pub async fn get_video_info(video_url: &str) -> Option<MusicInfo> {
    let output = Command::new("youtube-dl")
        .arg("--get-title")
        .arg(video_url)
        .output()
        .expect("Failed to execute youtube-dl");

    if output.status.success() {
        let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(MusicInfo {
            title,
            http: video_url.to_string(),
            watch: None,
        })
    } else {
        None
    }
}

pub async fn get_audio_url(video_url: &str) -> Option<String> {
    let output = Command::new("youtube-dl")
        .arg("-f")
        .arg("bestaudio")
        .arg("--get-url")
        .arg(video_url)
        .output()
        .expect("Failed to execute youtube-dl");

    if output.status.success() {
        let audio_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(audio_url)
    } else {
        None
    }
}
