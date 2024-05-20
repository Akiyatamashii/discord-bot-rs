use async_openai::{
    config::OpenAIConfig,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
    },
    Client,
};
use dotenv::dotenv;
use serenity::futures::StreamExt;
use std::{
    env,
    error::Error,
    io::{self, stdout, Write},
};
use tiktoken_rs::p50k_base;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let bpe = p50k_base().unwrap();
    let api_key = env::var("API_KEY").unwrap();
    let config = OpenAIConfig::default().with_api_key(api_key);
    let client = Client::with_config(config);
    let mut messages: Vec<ChatCompletionRequestMessage> =
        vec![ChatCompletionRequestSystemMessageArgs::default()
            .content("you are a friendly chat partner. you will answer any question that i asked")
            .build()
            .unwrap()
            .into()];

    loop {
        print!("User:");
        io::stdout().flush().unwrap();
        let mut prompt = String::new();
        io::stdin().read_line(&mut prompt).expect("error");

        if prompt.trim() == "!q" {
            break;
        }
        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()
                .unwrap()
                .into(),
        );

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o")
            .n(1)
            .messages(messages.clone())
            .max_tokens(4096_u16)
            .temperature(0.9)
            .build()
            .unwrap();

        let mut stream = client.chat().create_stream(request).await.unwrap();

        let mut lock = stdout().lock();
        let mut response_message = String::new();
        print!("Chat:");
        io::stdout().flush().unwrap();
        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    response.choices.iter().for_each(|chat_choice| {
                        if let Some(ref content) = chat_choice.delta.content {
                            response_message.push_str(content);
                            write!(lock, "{}", content).unwrap();
                        }
                    });
                }
                Err(err) => {
                    writeln!(lock, "error: {err}").unwrap();
                }
            }
            stdout().flush()?;
        }
        println!("");
        io::stdout().flush().unwrap();
        messages.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(response_message)
                .build()
                .unwrap()
                .into(),
        );
        let total_tokens: usize = messages
            .iter()
            .map(|msg| match msg {
                ChatCompletionRequestMessage::User(msg) => match &msg.content {
                    async_openai::types::ChatCompletionRequestUserMessageContent::Text(msg) => {
                        bpe.encode_with_special_tokens(&msg).len()
                    }
                    _ => 0,
                },
                ChatCompletionRequestMessage::System(msg) => {
                    bpe.encode_with_special_tokens(&msg.content).len()
                }
                _ => 0,
            })
            .sum();

        if total_tokens > 4096 {
            messages.push(
                ChatCompletionRequestUserMessageArgs::default()
                    .content("幫我將以上內容做重點摘要")
                    .build()
                    .unwrap()
                    .into(),
            );
            let request = CreateChatCompletionRequestArgs::default()
                .model("gpt-4o")
                .n(1)
                .messages(messages.clone())
                .max_tokens(8192_u16)
                .temperature(0.9)
                .build()
                .unwrap();
            println!("摘要整理中....");

            let response = client.chat().create(request).await.unwrap();

            if let Some(choice) = response.choices.first() {
                match &choice.message.content {
                    Some(msg) => {
                        messages.clear();
                        messages.push(
                            ChatCompletionRequestSystemMessageArgs::default()
                                .content(msg)
                                .build()
                                .unwrap()
                                .into(),
                        )
                    }
                    None => {}
                }
            }
            println!("整理完畢")
        }
    }

    Ok(())
}
