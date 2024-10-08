use std::{
    collections::HashMap,
    error::Error,
    fs,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};
use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, ResolvedOption,
    ResolvedValue, UserId,
};

// Define Cash struct to store individual debt information
// 定義 Cash 結構體，用於存儲單筆欠款信息
#[derive(Deserialize, Serialize)]
struct Cash {
    creator: UserId,
    debtor: String,
    creditor: String,
    debt: usize,
    ps: String,
}

// Define CashList struct to store all debt information
// 定義 CashList 結構體，用於存儲所有欠款信息
#[derive(Deserialize, Serialize)]
struct CashList(HashMap<GuildId, Vec<Cash>>);

impl CashList {
    // Create a new CashList instance
    // 創建新的 CashList 實例
    fn new() -> Self {
        let list: HashMap<GuildId, Vec<Cash>> = HashMap::new();
        CashList(list)
    }
}

// Implement Deref trait to allow direct use of HashMap methods on CashList
// 實現 Deref trait，允許 CashList 直接使用 HashMap 的方法
impl Deref for CashList {
    type Target = HashMap<GuildId, Vec<Cash>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Implement DerefMut trait to allow direct use of mutable HashMap methods on CashList
// 實現 DerefMut trait，允許 CashList 直接使用 HashMap 的可變方法
impl DerefMut for CashList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Register the cash command
// 註冊 cash 命令
pub fn register() -> CreateCommand {
    CreateCommand::new("cash")
        .description("欠債系統")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "type", "要做的指令")
                .add_string_choice("look", "look")
                .add_string_choice("add", "add")
                .add_string_choice("del", "del")
                .required(true),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "debtor",
            "債務人 (@Somebody)",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "creditor",
            "債權人 (@Somebody)",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::Integer,
            "debt",
            "債務金額",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            "ps",
            "備註",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::Integer,
            "index",
            "刪除索引",
        ))
}

// Main function to execute the cash command
// 執行 cash 命令的主函數
pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    options: &'a [ResolvedOption<'a>],
) {
    // 從選項中獲取各個參數的值
    let command_type = options
        .iter()
        .find(|opt| opt.name == "type")
        .and_then(|opt| match opt.value {
            ResolvedValue::String(s) => Some(s),
            _ => None,
        })
        .unwrap_or("");

    let debtor = options
        .iter()
        .find(|opt| opt.name == "debtor")
        .and_then(|opt| match opt.value {
            ResolvedValue::String(s) => Some(s.to_string()),
            _ => None,
        })
        .unwrap_or(String::new());

    let creditor = options
        .iter()
        .find(|opt| opt.name == "creditor")
        .and_then(|opt| match opt.value {
            ResolvedValue::String(s) => Some(s.to_string()),
            _ => None,
        })
        .unwrap_or(String::new());

    let debt = options
        .iter()
        .find(|opt| opt.name == "debt")
        .and_then(|opt| match opt.value {
            ResolvedValue::Integer(s) => Some(s as usize),
            _ => None,
        })
        .unwrap_or(0);

    let ps = options
        .iter()
        .find(|opt| opt.name == "ps")
        .and_then(|opt| match opt.value {
            ResolvedValue::String(s) => Some(s.to_string()),
            _ => None,
        })
        .unwrap_or(String::new());

    let index = options
        .iter()
        .find(|opt| opt.name == "index")
        .and_then(|opt| match opt.value {
            ResolvedValue::Integer(i) => Some(i as usize),
            _ => None,
        });

    // Execute the corresponding operation based on the command type
    // 根據命令類型執行相應的操作
    match command_type {
        "look" => look(ctx, command).await,
        "add" => {
            let creator = command.user.id;
            let cash = Cash {
                creator,
                debtor,
                creditor,
                debt,
                ps,
            };
            add(ctx, command, cash).await
        }
        "del" => {
            if let Some(index) = index {
                del(ctx, command, index).await
            } else {
                let content = String::from(">> 請輸入索引");
                let data = CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true);
                let builder = CreateInteractionResponse::Message(data);
                command.create_response(&ctx.http, builder).await.ok();
            }
        }
        _ => {
            let content = String::from(">> 未知的指令類型");
            let data = CreateInteractionResponseMessage::new()
                .content(content)
                .ephemeral(true);
            let builder = CreateInteractionResponse::Message(data);
            command.create_response(&ctx.http, builder).await.ok();
        }
    }
}

// View the debt list
// 查看欠款列表
async fn look(ctx: &Context, command: &CommandInteraction) {
    let cash_lists = match load_cash_data() {
        Ok(cash_lists) => cash_lists,
        Err(_) => CashList::new(),
    };
    let guild_id = command.guild_id.unwrap();
    let cash: Vec<Cash> = Vec::new();
    let cash_list = cash_lists.get(&guild_id).unwrap_or(&cash);

    let content = if cash_list.is_empty() {
        "V 目前沒有任何欠款 V".to_string()
    } else {
        let mut content = String::from("V 欠債列表 ( 糙你媽欠錢不還 ) V\n");
        for (index, cash) in cash_list.iter().enumerate() {
            if cash.ps.is_empty() {
                content.push_str(
                    format!(
                        "{}. {} 欠 {} {}元\n",
                        index, cash.debtor, cash.creditor, cash.debt
                    )
                    .as_str(),
                )
            } else {
                content.push_str(
                    format!(
                        "{}. {} 欠 {} {}元 ，備註:{}\n",
                        index, cash.debtor, cash.creditor, cash.debt, cash.ps
                    )
                    .as_str(),
                )
            }
        }
        content
    };

    let data = CreateInteractionResponseMessage::new()
        .content(content)
        .ephemeral(true);
    let builder = CreateInteractionResponse::Message(data);
    command.create_response(&ctx.http, builder).await.ok();
}

// Add a new debt record
// 添加新的欠款記錄
async fn add(ctx: &Context, command: &CommandInteraction, cash: Cash) {
    let guild_id = command.guild_id.unwrap();
    let mut cash_lists = match load_cash_data().ok() {
        Some(cash_lists) => cash_lists,
        None => CashList::new(),
    };

    let cash_list = cash_lists.entry(guild_id).or_insert(Vec::new());
    cash_list.push(cash);
    if let Err(e) = save_cash_data(&cash_lists) {
        let content = format!(">> 儲存資料時發生錯誤: {}", e);
        let data = CreateInteractionResponseMessage::new()
            .content(content)
            .ephemeral(true);
        let builder = CreateInteractionResponse::Message(data);
        command.create_response(&ctx.http, builder).await.unwrap();
    } else {
        let content = String::from(">> 已加入欠債");
        let data = CreateInteractionResponseMessage::new()
            .content(content)
            .ephemeral(true);
        let builder = CreateInteractionResponse::Message(data);
        command.create_response(&ctx.http, builder).await.unwrap();
    }
}

// Delete a specified debt record
// 刪除指定的欠款記錄
async fn del(ctx: &Context, command: &CommandInteraction, index: usize) {
    let mut cash_lists = match load_cash_data().ok() {
        Some(cash_lists) => cash_lists,
        None => {
            let content = String::from(">> 沒有可刪除的債務");
            let data = CreateInteractionResponseMessage::new()
                .content(content)
                .ephemeral(true);
            let builder = CreateInteractionResponse::Message(data);
            command.create_response(&ctx.http, builder).await.ok();
            return;
        }
    };
    let guild_id = command.guild_id.unwrap();

    if let Some(cash_list) = cash_lists.get_mut(&guild_id) {
        if cash_list[index - 1].creator != command.user.id {
            let content = String::from(">> 你沒有刪除此債務的權力");
            let data = CreateInteractionResponseMessage::new()
                .content(content)
                .ephemeral(true);
            let builder = CreateInteractionResponse::Message(data);
            command.create_response(&ctx.http, builder).await.unwrap();
            return;
        }
        if index > 0 && index <= cash_list.len() {
            cash_list.remove(index - 1);
            if let Err(e) = save_cash_data(&cash_lists) {
                let content = format!(">> 儲存資料時發生錯誤: {}", e);
                let data = CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true);
                let builder = CreateInteractionResponse::Message(data);
                command.create_response(&ctx.http, builder).await.unwrap();
            } else {
                let content = String::from(">> 已刪除所選債務");
                let data = CreateInteractionResponseMessage::new()
                    .content(content)
                    .ephemeral(true);
                let builder = CreateInteractionResponse::Message(data);
                command.create_response(&ctx.http, builder).await.ok();
            }
        } else {
            let content = String::from(">> 索引超出範圍");
            let data = CreateInteractionResponseMessage::new()
                .content(content)
                .ephemeral(true);
            let builder = CreateInteractionResponse::Message(data);
            command.create_response(&ctx.http, builder).await.ok();
        }
    } else {
        let content = String::from(">> 沒有可刪除的債務");
        let data = CreateInteractionResponseMessage::new()
            .content(content)
            .ephemeral(true);
        let builder = CreateInteractionResponse::Message(data);
        command.create_response(&ctx.http, builder).await.ok();
    }
}

// Save debt data to file
// 保存欠款數據到文件
fn save_cash_data(cash_list: &CashList) -> Result<(), Box<dyn Error + Send + Sync>> {
    let json_content = serde_json::to_string(cash_list)?;
    fs::write("./cash.json", json_content)?;
    Ok(())
}

// Load debt data from file
// 從文件加載欠款數據
fn load_cash_data() -> Result<CashList, Box<dyn Error + Send + Sync>> {
    let json_content = fs::read_to_string("./cash.json")?;
    let cash: CashList = serde_json::from_str(&json_content)?;
    Ok(cash)
}
