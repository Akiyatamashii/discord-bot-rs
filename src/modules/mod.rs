// 引入提醒功能相關的模塊
pub mod reminder;

// 引入通用功能函數的模塊
pub mod func;

// 引入機器人處理邏輯的模塊
pub mod bot_process;

// 引入拒絕TikTok的模塊
pub mod anti_tiktok;

// 這個模塊文件定義了機器人的核心功能結構
// 每個子模塊包含特定功能的實現：
// - reminder: 處理提醒相關的功能，可能包括定時任務的執行邏輯
// - func: 包含各種通用的輔助函數，如文件操作、權限檢查等
// - bot_process: 包含機器人的主要處理邏輯，如命令解析和執行

// 注意：這個文件的結構反映了機器人的整體架構，
// 將功能分為核心處理邏輯（bot_process）、特定功能實現（reminder）
// 和通用輔助函數（func）三個主要部分
