use async_openai::Client;
use serenity::all::{
    CommandOptionType, CreateCommand, CreateCommandOption, ResolvedOption, ResolvedValue,
};
use std::collections::HashMap;

use crate::modules::func::openai_config;

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
                .required(false),
        )
}

pub async fn run<'a>(option: &'a [ResolvedOption<'a>]) -> String {
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

    let client = Client::with_config(openai_config());

    match client.models().list().await {
        Ok(models) => {
            let mut model_groups: HashMap<&str, Vec<String>> = HashMap::new();

            for model in models.data.iter() {
                let model_type_str = get_model_type(&model.id);
                model_groups
                    .entry(model_type_str)
                    .or_default()
                    .push(model.id.clone());
            }

            let mut msg = String::new();

            let order = [
                "自然語言模型",
                "圖片生成模型",
                "語音識別模型",
                "文字轉語音模型",
                "文本嵌入模型",
                "其他模型",
            ];

            for (index, &group) in order.iter().enumerate() {
                if model_type == 0 || model_type as usize == index + 1 {
                    if let Some(models) = model_groups.get(group) {
                        let mut models = models.clone();
                        models.sort();

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

                        msg.push_str(&format!("V {} V\n", group));
                        for model in models {
                            msg.push_str(&format!("{}\n", model));
                        }
                        msg.push('\n');
                    }
                }
            }

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
