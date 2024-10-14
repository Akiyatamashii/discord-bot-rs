use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption, EditMember, ResolvedOption, ResolvedValue, UserId
};

use crate::{modules::func::check_permission, BanList};

// Register the unban command
// 註冊解封命令
pub fn register() -> CreateCommand {
    CreateCommand::new("unban")
        .description("unban users")
        .description_localized("zh-TW", "解封禁用戶")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "member", "the member to unban")
                .description_localized("zh-TW", "要解封的用戶")
                .required(true),
        )
}

// Run the unban command
// 執行解封命令
pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    ban_list: BanList,
    options: &'a [ResolvedOption<'a>],
) -> String {
    // Check if the user has permission to use the command
    // 檢查用戶是否有權限使用該命令
    if !check_permission(ctx, command).await {
        return "你沒有許可權使用指令".to_string();
    }
    
    // Find the member option from the command
    // 從命令中找到成員選項
    let member = options.iter().find(|option| option.name == "member");

    // Extract member ID and name
    // 提取成員 ID 和名稱
    let (member_id, member_name) = if let Some(get_member) = member {
        if let ResolvedValue::User(member, _option) = get_member.value {
            (member.id, member.name.clone())
        } else {
            (UserId::default(), "".to_string())
        }
    } else {
        (UserId::default(), "".to_string())
    };

    // Prevent users from unbanning themselves
    // 防止用戶解封自己
    if command.member.clone().unwrap().user.id == member_id {
        return "你不能解封你自己".to_string();
    }
    
    // Check if the member is in the ban list
    // 檢查成員是否在封禁列表中
    let ban_list_value = ban_list.write().await;
    let baned_member = ban_list_value
        .iter()
        .find(|user| user.0 == member_id)
        .cloned();

    if baned_member.is_some() {
        drop(ban_list_value); // Release the write lock // 釋放寫鎖
        unban(ban_list.clone(), member_id).await;

        // Unmute the member in the guild
        // 在伺服器中取消成員的靜音
        let guild_id = command.guild_id.unwrap();
        let builder = EditMember::new().mute(false);
        guild_id.edit_member(ctx, member_id, builder).await.unwrap();
        
        format!("已將{}移出封禁名單", member_name)
    } else {
        "該用戶不在封禁名單中".to_string()
    }
}

// Remove a member from the ban list
// 從封禁列表中移除成員
pub async fn unban(ban_list: BanList, member_id: UserId) {
    let mut ban_list_guard = ban_list.write().await;
    ban_list_guard.retain(|user| user.0 != member_id);
    let json = serde_json::to_string(&*ban_list_guard).unwrap();
    drop(ban_list_guard);
    
    // Write the updated ban list to file
    // 將更新後的封禁列表寫入文件
    tokio::spawn(async move {
        if let Err(e) = tokio::fs::write("assets/ban_list.json", json).await {
            eprintln!("寫入 ban_list.json 時發生錯誤: {}", e);
        }
    });
}
