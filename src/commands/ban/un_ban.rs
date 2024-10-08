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

    println!("unban id: {}", member_id);

    println!("正在獲取封禁列表...");
    let ban_list_value = ban_list.write().await;
    println!("已獲取封禁列表");

    println!("正在查找被封禁成員...");
    let baned_member = ban_list_value
        .iter()
        .find(|user| user.0 == member_id)
        .cloned();
    println!("查找完成");

    if baned_member.is_some() {
        println!("正在釋放寫鎖...");
        drop(ban_list_value); // 釋放寫鎖
        println!("寫鎖已釋放");

        println!("正在執行解封操作...");
        unban(ban_list.clone(), member_id).await;
        println!("解封操作完成");

        println!("正在獲取公會ID...");
        let guild_id = command.guild_id.unwrap();
        println!("已獲取公會ID");

        println!("正在創建編輯成員的建構器...");
        let builder = EditMember::new().mute(false);
        println!("建構器已創建");

        println!("正在編輯成員狀態...");
        guild_id.edit_member(ctx, member_id, builder).await.unwrap();
        println!("成員狀態已更新");

        format!("已將{}移出封禁名單", member_name)
    } else {
        "該用戶不在封禁名單中".to_string()
    }
}

pub async fn unban(ban_list: BanList, member_id: UserId) {
    let mut ban_list = ban_list.write().await;
    ban_list.retain(|user| user.0 != member_id);
    let ban_list_clone = ban_list.clone();
    let json = serde_json::to_string(&ban_list_clone).unwrap();
    std::fs::write("assets/ban_list.json", json).unwrap();
}
