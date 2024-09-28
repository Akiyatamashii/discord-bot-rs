// 引入現金管理相關的命令
pub mod cash;

// 引入 OpenAI 相關的命令
pub mod openai;

// 引入提醒功能相關的命令
pub mod reminder;

// 引入基礎命令
pub mod base;

// 引入拒絕 TikTok 相關的命令
pub mod tiktok_refuse;

// 這個模塊文件定義了機器人的主要命令結構
// 每個子模塊包含特定功能的命令：
// - cash: 處理欠款和債務相關的命令
// - openai: 與 OpenAI API 交互的命令，如聊天和圖像生成
// - reminder: 處理提醒和定時任務的命令
// - base: 包含一些基礎或通用的命令

