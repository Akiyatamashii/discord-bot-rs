use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption, EditMember,
    ResolvedOption, ResolvedValue, UserId,
};

use crate::{
    modules::{
        func::{check_permission, ensure_file_exists},
        reminder::TW,
    },
    BanList,
};

pub fn register() -> CreateCommand {
    CreateCommand::new("ban")
        .description("ban or punish users")
        .description_localized("zh-TW", "封禁或逞罰用戶")
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, "member", "the member to ban")
                .description_localized("zh-TW", "要封禁的用戶")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::Integer, "mins", "how long to ban")
                .description_localized("zh-TW", "封禁時間 (分鐘)")
                .required(true),
        )
}

pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    ban_list: BanList,
    options: &[ResolvedOption<'a>],
) -> String {
    if !check_permission(ctx, command).await {
        return "你沒有許可權使用指令".to_string();
    }
    let member = options.iter().find(|option| option.name == "member");
    let mins = options.iter().find(|option| option.name == "mins");

    let (member_id, member_name) = if let Some(get_member) = member {
        if let ResolvedValue::User(member, _option) = get_member.value {
            (member.id, member.name.clone())
        } else {
            (UserId::default(), "".to_string())
        }
    } else {
        (UserId::default(), "".to_string())
    };

    if member_id == UserId::from(412803325768237066) {
        return "你不能封禁豆腐".to_string();
    }

    let mins = if let Some(get_mins) = mins {
        if let ResolvedValue::Integer(mins) = get_mins.value {
            mins
        } else {
            0
        }
    } else {
        0
    };

    let mut ban_list = ban_list.write().await;
    if ban_list.iter().any(|(id, _)| *id == member_id) {
        return format!("{}已經在封禁名單中", member_name);
    }
    let now = chrono::Utc::now().with_timezone(&*TW).time();
    let ban_time = now + chrono::Duration::minutes(mins);
    ban_list.push((member_id, ban_time));

    ensure_file_exists("assets/ban_list.json").unwrap();
    let list_json = serde_json::to_string(&*ban_list).unwrap();
    std::fs::write("assets/ban_list.json", list_json).unwrap();

    println!("ban id: {}", member_id);

    let guild_id = command.guild_id.unwrap();
    let builder = EditMember::new().mute(true);
    guild_id.edit_member(ctx, member_id, builder).await.unwrap();
    drop(ban_list);

    format!("成功將{}加入封禁名單，封禁時間為{}分鐘", member_name, mins)
}
