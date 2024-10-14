# Discord Bot in Rust

這是一個使用 Rust 程式語言開發的多功能 Discord 機器人。

[English](README.md)

## 功能

1. **提醒系統**

   - 設置定時提醒
   - 查看當前提醒
   - 移除特定提醒

2. **OpenAI 整合**

   - 與 ChatGPT 聊天
   - 生成圖像
   - 查看可用的 AI 模型

3. **欠債追蹤系統**

   - 記錄欠債信息
   - 查看欠債列表
   - 刪除欠債記錄

4. **基本指令**

   - 查看機器人信息
   - 測試機器人回應時間

5. **封禁處罰系統**

   - 封禁用戶
   - 解除用戶封禁

6. **反 TikTok 功能**
   - 自動處理 TikTok 連結（目前僅限特定群組使用）

## 安裝和設置

1. 確保已安裝 Rust 和 Cargo。
2. 克隆此存儲庫並進入項目目錄：
   ```
   git clone https://github.com/Akiyatamashii/discord-bot-rs.git
   cd discord-bot-rs
   ```
3. 創建 `.env` 文件並添加以下內容：
   ```
   TOKEN=your_discord_bot_token // Discord 機器人令牌
   API_KEY=your_openai_api_key // OpenAI API 密鑰（如不需要可選）
   ```
4. 編譯並運行機器人：
   ```
   cargo run // 運行
   ```
   或
   ```
   cargo build --release // 編譯
   ./target/release/discord-bot // 運行
   ```
5. 將機器人添加到您的 Discord 伺服器：
   - 在 Discord 開發者門戶使用您的機器人令牌創建新的機器人。
   - 將機器人添加到您的伺服器。
   - 在伺服器中使用 `!register` 命令註冊機器人命令。
   - 使用 `/info` 獲取更多關於機器人的信息。
6. 享受您的 Discord 機器人！

## 使用方法

機器人支持以下斜線命令：

### 基本功能（Common）

- `/info` - 查看基本機器人信息
- `/info [type]` - 查看特定功能的詳細說明
- `/ping` - 測試連接

### 提醒系統（Reminder）

- `/remind [weekdays] [time] [message]` - 設置提醒
- `/rm_remind [index]` - 移除特定提醒
- `/look` - 查看當前提醒

### AI 生成（OpenAI）

- `/chat [message] [public] [model]` - 與 ChatGPT 聊天
- `/image [prompt] [public] [model]` - 生成圖像
- `/model_list` - 查看可用的 AI 模型

### 欠債追蹤系統（Cash）

- `/cash look` - 查看當前欠債
- `/cash add [debtor] [creditor] [debt] [ps]` - 添加欠債記錄
- `/cash del [index]` - 刪除特定欠債記錄

### 封禁處罰系統（Ban）

- `/ban [member] [mins]` - 封禁用戶
- `/unban [member]` - 解除用戶封禁

有關詳細的命令使用說明，請參閱 `info/` 目錄中相應的 markdown 文件。

## 開發注意事項

本項目使用以下主要依賴：

- serenity: Discord API 客戶端
- async-openai: OpenAI API 客戶端
- tokio: 異步運行時
- chrono: 日期和時間處理
- serde: 序列化和反序列化

完整的依賴列表，請查看 `Cargo.toml` 文件。

## 貢獻

歡迎提交 Pull Request 以改進此項目。對於重大更改，請先開啟一個 Issue 討論您想要更改的內容。

## 版本

當前版本：0.7.7

## 許可證

本項目採用 [MIT 許可證](LICENSE)。

## 作者

由 **_Akiyatamashii_** 製作
