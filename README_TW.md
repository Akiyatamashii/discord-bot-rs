# Discord Bot in Rust

這是一個使用 Rust 語言開發的多功能 Discord 機器人。

## 功能特點

1. **提醒系統 (Reminder)**

   - 設置定時提醒
   - 查看當前設置的提醒
   - 移除指定的提醒

2. **OpenAI 整合 (AI)**

   - 與 ChatGPT 進行對話
   - 生成圖像
   - 查看可用的 AI 模型列表

3. **欠債系統 (Cash)**

   - 記錄欠款信息
   - 查看欠款列表
   - 刪除欠款記錄

4. **基礎命令 (Common)**
   - 查看機器人信息
   - 測試機器人響應時間

## 安裝與設置

1. 確保您已安裝 Rust 和 Cargo。
2. 克隆此倉庫：
   ```
   git clone https://github.com/Akiyatamashii/discord-bot-rs.git
   ```
3. 進入專案目錄：
   ```
   cd discord-bot-rs
   ```
4. 創建一個 `.env` 文件，並添加以下內容：
   ```
   TOKEN=your_discord_bot_token // 機器人 token
   API_KEY=your_openai_api_key // openai api key (如果沒有需要可以不填)
   ```
5. 編譯並運行機器人：
   ```
   cargo run // 運行
   ```
   or
   ```
   cargo build --release // 編譯
   ./target/release/discord-bot // 運行
   ```
6. 添加機器人到您的 Discord 伺服器：
   - 在 Discord 開發者門戶中，使用您的機器人令牌創建一個新的機器人。
   - 將機器人添加到您的伺服器。
   - 在伺服器中使用 `/register` 命令註冊機器人指令。
7. 享受你的 Discord 機器人吧！

## 使用方法

機器人支持以下斜線命令：

### 基本功能 (Common)

- `/info` - 查看機器人基本資訊
- `/info [type]` - 查看功能詳細指令
- `/ping` - 測試連線

### 提醒器 (Reminder)

- `/remind [weekdays] [time] [message]` - 設置提醒
- `/rm_remind [index]` - 移除指定的提醒
- `/look` - 查看當前設置的提醒

### AI 生成 (OpenAI)

- `/chat [message] [public] [model]` - 與 ChatGPT 對話
- `/image [prompt] [public] [model]` - 生成圖片
- `/model_list` - 查看可用的 AI 模型

### 欠債系統 (Cash)

- `/cash look` - 查看當前存在的債務
- `/cash add [debtor] [creditor] [debt] [ps]` - 增加債務記錄
- `/cash del [index]` - 刪除指定的債務記錄

詳細的命令使用說明請參考 `info/` 目錄下的相應 markdown 文件。

## 開發說明

本專案使用以下主要依賴：

- serenity: Discord API 客戶端
- async-openai: OpenAI API 客戶端
- tokio: 異步運行時
- chrono: 日期和時間處理
- serde: 序列化和反序列化

完整的依賴列表請查看 `Cargo.toml` 文件。

## 貢獻

歡迎提交 Pull Requests 來改進這個專案。對於重大更改，請先開 issue 討論您想要改變的內容。

## 授權

本專案採用 [MIT 授權](LICENSE)。

## 作者

Made By **_Akiyatamashii_**
