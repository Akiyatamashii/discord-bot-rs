use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand_chacha::ChaCha12Rng;
use serenity::all::{Context, Message};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use super::func::ensure_file_exists;

pub fn load_tiktok_refuse_msg() -> Vec<String> {
    ensure_file_exists("./assets/tiktok_refuse_msg.txt").unwrap();
    let mut msg_vec = Vec::new();
    let file_path = "./assets/tiktok_refuse_msg.txt";
    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        msg_vec.push(line.unwrap());
    }
    msg_vec
}

pub fn save_tiktok_refuse_msg(tiktok_refuse_msg: &Vec<String>) -> Result<(), io::Error> {
    let file_path = "./assets/tiktok_refuse_msg.txt";
    let mut file = File::create(file_path).unwrap();
    for msg in tiktok_refuse_msg {
        file.write_all(format!("{}\n", msg).as_bytes()).unwrap();
    }
    Ok(())
}

pub async fn add_tiktok_refuse_msg(
    msg: &str,
    tiktok_refuse_msg: Arc<RwLock<Vec<String>>>,
) -> Result<(), io::Error> {
    let mut tiktok_refuse_msg = tiktok_refuse_msg.write().await;
    tiktok_refuse_msg.push(msg.to_string());

    save_tiktok_refuse_msg(&tiktok_refuse_msg)?;

    Ok(())
}

pub async fn tiktok_refuse(
    ctx: &Context,
    msg: &Message,
    tiktok_refuse_msg: Arc<RwLock<Vec<String>>>,
) {
    // 建立一個執行緒安全的隨機數生成器
    let rng = Arc::new(Mutex::new(ChaCha12Rng::from_entropy()));

    if msg.content.contains("xiaohongshu") {
        msg.reply(ctx, "小紅書仔閉嘴").await.unwrap();
        return;
    }
    
    if msg.content.contains("tiktok.com") || msg.content.contains("douyin.com") {
        let refuse_msg = tiktok_refuse_msg.read().await;
        let selected_msg = refuse_msg
            .choose(&mut *rng.lock().unwrap()) // 使用 lock() 來獲取互斥鎖
            .unwrap_or(&"抖音仔閉嘴".to_string())
            .to_string();

        msg.reply(ctx, &format!("# {}", selected_msg)).await.unwrap();
    }
}
