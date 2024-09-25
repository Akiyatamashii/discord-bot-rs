use async_openai::Client;
use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue,
};
use std::collections::HashMap;

use crate::modules::func::openai_config;

// 註冊 model_list 命令
pub fn register() -> CreateCommand {
    CreateCommand::new("model_list")
        .description("模型列表")
        .add_option(
            CreateCommandOption::new(CommandOptionType::Integer, "model_type", "模型類型")
                .add_int_choice("自然語言模型", 1)
                .add_int_choice("圖片生成模型", 2)
                .add_int_choice("語音識別模型", 3)
                .add_int_choice("文字轉語音模型", 4)
                .add_int_choice("文本嵌入模型", 5)
                .add_int_choice("其他模型", 6)
                .required(false), // 設置為可選參數
        )
}

// 執行 model_list 命令的主函數
pub async fn run<'a>(option: &'a [ResolvedOption<'a>]) -> String {
    // 從選項中獲取 model_type 的值，如果沒有選擇則默認為 0（全部模型）
    let model_type = option
        .iter()
        .find(|opt| opt.name == "model_type")
        .and_then(|opt| {
            if let ResolvedValue::Integer(model_type) = opt.value {
                Some(model_type)
            } else {
                None
            }
        })
        .unwrap_or(0);

    // 創建 OpenAI 客戶端
    let client = Client::with_config(openai_config());

    // 獲取模型列表
    match client.models().list().await {
        Ok(models) => {
            // 創建一個 HashMap 來存儲不同類型的模型
            let mut model_groups: HashMap<&str, Vec<String>> = HashMap::new();

            // 將模型按類型分組
            for model in models.data.iter() {
                let model_type_str = get_model_type(&model.id);
                model_groups
                    .entry(model_type_str)
                    .or_default()
                    .push(model.id.clone());
            }

            let mut msg = String::new();

            // 定義模型類型的輸出順序
            let order = [
                "自然語言模型",
                "圖片生成模型",
                "語音識別模型",
                "文字轉語音模型",
                "文本嵌入模型",
                "其他模型",
            ];

            // 按順序輸出模型
            for (index, &group) in order.iter().enumerate() {
                // 根據 model_type 過濾輸出
                if model_type == 0 || model_type as usize == index + 1 {
                    if let Some(models) = model_groups.get(group) {
                        let mut models = models.clone();
                        models.sort();

                        // 特殊處理自然語言模型，將 chatgpt 模型放在 gpt 模型後面
                        if group == "自然語言模型" {
                            let chatgpt_models: Vec<String> = models
                                .iter()
                                .filter(|x| x.starts_with("chatgpt"))
                                .cloned()
                                .collect();
                            models.retain(|x| !x.starts_with("chatgpt"));
                            let last_gpt_index = models
                                .iter()
                                .rposition(|x| x.starts_with("gpt"))
                                .unwrap_or(0);
                            models.splice(last_gpt_index + 1..last_gpt_index + 1, chatgpt_models);
                        }

                        // 添加模型類型標題
                        msg.push_str(&format!("V {} V\n", group));
                        // 添加該類型的所有模型
                        for model in models {
                            msg.push_str(&format!("{}\n", model));
                        }
                        msg.push('\n');
                    }
                }
            }

            // 如果沒有找到符合條件的模型，返回提示信息
            if msg.is_empty() {
                "沒有找到符合條件的模型".to_string()
            } else {
                msg.trim_end().to_string()
            }
        }
        Err(err) => {
            println!("Error: {}", err);
            ">> 獲取模型列表失敗".to_string()
        }
    }
}

// 根據模型名稱判斷模型類型
fn get_model_type(model: &str) -> &str {
    if model.starts_with("gpt") || model.starts_with("chatgpt") || model.starts_with("text-davinci")
    {
        "自然語言模型"
    } else if model.starts_with("dall-e") {
        "圖片生成模型"
    } else if model.starts_with("whisper") {
        "語音識別模型"
    } else if model.starts_with("tts") {
        "文字轉語音模型"
    } else if model.starts_with("text-embedding") {
        "文本嵌入模型"
    } else {
        "其他模型"
    }
}
