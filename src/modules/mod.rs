// import reminder module
// 引入提醒功能模塊
pub mod reminder;

// import func module
// 引入通用功能函數模塊
pub mod func;

// import bot_process module
// 引入機器人處理邏輯模塊
pub mod bot_process;

// import anti_tiktok module
// 引入反 TikTok 模塊
pub mod anti_tiktok;

// This module file defines the core functional structure of the bot
// Each sub-module contains implementations of specific functionalities:
// - reminder: handles reminder-related functions, possibly including execution logic for timed tasks
// - func: contains various general utility functions, such as file operations, permission checks, etc.
// - bot_process: contains the main processing logic of the bot, such as command parsing and execution
// - anti_tiktok: handles TikTok-related functions, possibly including blocking or converting TikTok links

// Note: The structure of this file reflects the overall architecture of the bot,
// dividing functionalities into core processing logic (bot_process), specific feature implementations (reminder, anti_tiktok),
// and general utility functions (func) as three main components

// 這個模塊文件定義了機器人的核心功能結構
// 每個子模塊包含特定功能的實現：
// - reminder: 處理提醒相關的功能，包括定時任務的執行邏輯
// - func: 包含各種通用的輔助函數，如文件操作、權限檢查等
// - bot_process: 包含機器人的主要處理邏輯，如命令解析和執行
// - anti_tiktok: 處理與 TikTok 相關的功能，可能包括阻止或轉換 TikTok 鏈接

// 注意：此文件的結構反映了機器人的整體架構，
// 將功能分為以下幾個主要部分：
// 1. 核心處理邏輯（bot_process）
// 2. 特定功能實現（reminder, anti_tiktok）
// 3. 通用輔助函數（func）
