use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    ResolvedOption, ResolvedValue, UserId,
};

use crate::{modules::func::check_permission, FraudBotList};

pub fn register() -> CreateCommand {
    CreateCommand::new("remove_block")
        .description("Remove a user from the block list")
        .description_localized("zh-TW", "從黑名單移除使用者")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::User,
                "user",
                "The user to be removed from the block list",
            )
            .description_localized("zh-TW", "被移除的使用者")
            .required(true),
        )
}

pub async fn run<'a>(
    ctx: &Context,
    command: &CommandInteraction,
    fraud_bot_list: FraudBotList,
    options: &[ResolvedOption<'a>],
) -> String {
    if !check_permission(ctx, command).await {
        return "你沒有許可權使用指令".to_string();
    }

    let user_option = options.iter().find(|option| option.name == "user");

    let user_id = if let Some(get_user) = user_option {
        if let ResolvedValue::User(user,_option ) = get_user.value {
            user.id
        }else {
            UserId::new(0)
        }
    } else {
        UserId::new(0)
    } ;

    if user_id == UserId::from(0) {
        return "無法解析使用者訊息".to_string();
    }
    

    fraud_bot_list.write().await.remove(&user_id);
    
    format!("成功移除使用者 <@{}>",user_id)
}
