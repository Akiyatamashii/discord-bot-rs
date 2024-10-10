use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption, EditMember, ResolvedOption, ResolvedValue, UserId
};

use crate::{modules::func::check_permission, BanList};

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

pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    ban_list: BanList,
    options: &'a [ResolvedOption<'a>],
) -> String {
    if !check_permission(ctx, command).await {
        return "你沒有許可權使用指令".to_string();
    }
    let member = options.iter().find(|option| option.name == "member");

    let (member_id, member_name) = if let Some(get_member) = member {
        if let ResolvedValue::User(member, _option) = get_member.value {
            (member.id, member.name.clone())
        } else {
            (UserId::default(), "".to_string())
        }
    } else {
        (UserId::default(), "".to_string())
    };

    if command.member.clone().unwrap().user.id == member_id {
        return "你不能解封你自己".to_string();
    }
    
    let ban_list_value = ban_list.write().await;
    let baned_member = ban_list_value
        .iter()
        .find(|user| user.0 == member_id)
        .cloned();

    if baned_member.is_some() {
        drop(ban_list_value); // 釋放寫鎖
        unban(ban_list.clone(), member_id).await;

        let guild_id = command.guild_id.unwrap();
        let builder = EditMember::new().mute(false);
        guild_id.edit_member(ctx, member_id, builder).await.unwrap();
        
        format!("已將{}移出封禁名單", member_name)
    } else {
        "該用戶不在封禁名單中".to_string()
    }
}

pub async fn unban(ban_list: BanList, member_id: UserId) {
    let mut ban_list_guard = ban_list.write().await;
    ban_list_guard.retain(|user| user.0 != member_id);
    let json = serde_json::to_string(&*ban_list_guard).unwrap();
    drop(ban_list_guard);
    tokio::spawn(async move {
        if let Err(e) = tokio::fs::write("assets/ban_list.json", json).await {
            eprintln!("寫入 ban_list.json 時發生錯誤: {}", e);
        }
    });
}
