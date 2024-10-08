// import base module
// 引入基礎命令
pub mod base;

// import reminder module
// 引入提醒功能相關的命令
pub mod reminder;

// import openai module
// 引入 OpenAI 相關的命令
pub mod openai;

// import cash module
// 引入現金管理相關的命令
pub mod cash;

// import anti_tiktok module
// 引入拒絕 TikTok 相關的命令
pub mod anti_tiktok;

// import ban module
// 引入封禁或逞罰相關的模塊
pub mod ban;

// This module file defines the main command structure of the bot
// Each sub-module contains commands for specific functionalities:
// - cash: handles commands related to debts and loans
// - openai: commands for interacting with OpenAI API, such as chat and image generation
// - reminder: handles commands for reminders and scheduled tasks
// - base: contains some basic or general commands
// - anti_tiktok: reply to users who send TikTok links in a rude manner
// - ban: handles commands for banning or punishing users

// 這個模塊文件定義了機器人的主要命令結構
// 每個子模塊包含特定功能的命令：
// - base: 包含一些基礎或通用的命令
// - reminder: 處理提醒和定時任務的命令
// - openai: 與 OpenAI API 交互的命令，如聊天和圖像生成
// - cash: 處理欠款和債務相關的命令
// - anti_tiktok: 對發送Tiktok連結的用戶進行不友好的回覆
// - ban: 處理封禁或逞罰用戶的命令