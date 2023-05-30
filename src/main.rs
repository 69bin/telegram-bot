use once_cell::sync::OnceCell;
use sqlx::MySqlPool;
use toml::Table;
use std::{error::Error, fs::File};
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        ChatKind, ChatPermissions, InlineKeyboardButton, InlineKeyboardMarkup,
        InlineQueryResultArticle, InputMessageContent, InputMessageContentText, Me, PublicChatKind,
    },
    utils::command::BotCommands,
};
use std::io::prelude::*;
pub mod service;
use service::{
    add_group_user, generate_10_num, generate_num, update_group_user_join_count,
    update_group_user_status, Group,
};

use crate::service::select_group_user;

static MYSQLPOOL: OnceCell<MySqlPool> = OnceCell::new();

#[inline]
pub fn get_mysql() -> &'static MySqlPool {
    unsafe { MYSQLPOOL.get_unchecked() }
}

#[derive(BotCommands)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    #[command(description = "Display this text")]
    Help,
    #[command(description = "Start")]
    Start,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    let mut file = match File::open("config.toml") {
        Ok(content) => {
            content
        }
        Err(_err) => {
            panic!("配置文件打开错误，请检查配置文件。");
        }
    };
    let mut config_str = String::new();
    let _: usize = match file.read_to_string(&mut config_str){
        Ok(text) => text,
        Err(_err) => panic!("配置文件读取错误。")
    };
    let con = config_str.parse::<Table>().expect("配置文件错误，请检查。");

    //连接mysql数据库
    log::info!("Start connecting to database");
    let mysql_pool = MySqlPool::connect(con["mysql_url"].as_str().unwrap()).await;
    match mysql_pool {
        Ok(pool) => {
            //把连接的数据库增加到全局变量
            MYSQLPOOL.set(pool).unwrap();
            println!("数据库连接成功!!!!");
        }
        Err(err) => {
            println!("err = ${:?}", err);
        }
    }
    log::info!("Starting buttons bot...");

    let bot = Bot::new("6136519942:AAE8YXmxbe97zcb2GFykku8uCbJ6fSIapR8");

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler))
        .branch(Update::filter_inline_query().endpoint(inline_query_handler))
        .branch(Update::filter_chat_member().endpoint(chat_member));

    Dispatcher::builder(bot, handler)
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
    Ok(())
}

/// 创建一个由大列中的按钮组成的键盘。
fn make_keyboard() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    let debian_versions = [
        "Buzz", "Rex", "Bo", "Hamm", "Slink", "Potato", "Woody", "Sarge", "Etch", "Lenny",
        "Squeeze", "Wheezy", "Jessie", "Stretch", "Buster", "Bullseye",
    ];

    for versions in debian_versions.chunks(3) {
        let row = versions
            .iter()
            .map(|&version| InlineKeyboardButton::callback(version.to_owned(), version.to_owned()))
            .collect();

        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

fn calculate(num: i32) -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    let number = generate_10_num(num);

    let debian_versions = [
        &number.0.to_string(),
        &number.1.to_string(),
        &number.2.to_string(),
        &number.3.to_string(),
    ];

    for versions in debian_versions.chunks(4) {
        let row = versions
            .iter()
            .map(|&version| InlineKeyboardButton::callback(version.to_owned(), version.to_owned()))
            .collect();

        keyboard.push(row);
    }

    InlineKeyboardMarkup::new(keyboard)
}

///分析Telegram上写的文本，并检查该文本是否为有效命令
///或否，则匹配该命令。如果命令为“/start”，则会写入
///使用`InlineKeyboardMarkup`进行标记。
async fn message_handler(
    bot: Bot,
    msg: Message,
    me: Me,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(text) = msg.text() {
        match BotCommands::parse(text, me.username()) {
            //如果是命令 help
            Ok(Command::Help) => {
                // Just send the description of all commands.
                bot.send_message(msg.chat.id, Command::descriptions().to_string())
                    .await?;
            }
            //如果是命令 start
            Ok(Command::Start) => {
                // Create a list of buttons and send them.
                let keyboard = make_keyboard();
                bot.send_message(msg.chat.id, "Debian versions:")
                    .reply_markup(keyboard)
                    .await?;
            }
            //其他信息 返回发送的消息
            Err(_) => {
                let _message = format!("you send message is {:?}", text);
                //bot.send_message(msg.chat.id, message).await?;
            }
        }
    }

    Ok(())
}

async fn inline_query_handler(
    bot: Bot,
    q: InlineQuery,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let choose_debian_version = InlineQueryResultArticle::new(
        "0",
        "Chose debian version",
        InputMessageContent::Text(InputMessageContentText::new("Debian versions:")),
    )
    .reply_markup(make_keyboard());
    bot.answer_inline_query(q.id, vec![choose_debian_version.into()])
        .await?;

    Ok(())
}

//用户修改消息回调
async fn chat_member(
    bot: Bot,
    chat_member: ChatMemberUpdated,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let username = chat_member.from.username.unwrap();
    let user_id = chat_member.from.id;
    let group_id = match chat_member.chat.kind.clone() {
        ChatKind::Public(p) => match p.kind {
            PublicChatKind::Supergroup(sgr) => sgr.username.unwrap(),
            _ => "".to_string(),
        },
        _ => "".to_string(),
    };
    //判断数据库是否已经存在该用户
    let group = select_group_user(&group_id, &user_id.0.to_string()).await;
    if group.is_err() {
        add_group_user(Group::new(&username, &user_id.0.to_string(), &group_id, 0)).await?;
    }
    println!("{:?}",chat_member.new_chat_member);
    //如果old 是left，则是加入，如果old是Member是退出
    match chat_member.new_chat_member.kind {
        teloxide::types::ChatMemberKind::Member => {
            //给用户禁用发现权限，并且发送一个用户验证消息。
            if group.unwrap().get_status() == 1{
                return Ok(());
            }
            //@用户并且发送消息
            if chat_member.from.is_bot == false {
                //生成计算公式，并计算出答案
                let nums = generate_num();
                let text = format!("@{} 计算:{} + {} = ", username, nums.0, nums.1);
                update_group_user_join_count(nums.2, &group_id, &user_id.0.to_string()).await?;
                bot.send_message(chat_member.chat.id, text)
                    .reply_markup(calculate(nums.2))
                    .await?;
                let permissions = ChatPermissions::empty();
                bot.restrict_chat_member(chat_member.chat.id, user_id, permissions)
                    .await?;
            }
        }
        teloxide::types::ChatMemberKind::Left => {
            //退出的话，把状态更改为2
            update_group_user_status(2, &group_id, &user_id.0.to_string()).await?;
        }
        _ => {}
    }
    Ok(())
}

///当它接收到来自按钮的回调时，它会编辑消息
///那些用选定的Debian版本编写文本的按钮。
///**重要**：不要以这种方式发送隐私敏感数据！！！
///任何人都可以读取存储在回调按钮中的数据。
async fn callback_handler(bot: Bot, q: CallbackQuery) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(version) = q.data {
        //告诉telegram我们已经看到这个查询，以删除🕑 的图标
        //客户。您也可以使用`answer_callback_query`的可选
        //参数来调整客户端上发生的事情。
        bot.answer_callback_query(q.id).await?;
        let permissions = ChatPermissions::all();
        let message = q.message;
        match message {
            Some(msg) => {
                let username = q.from.username.unwrap();
                let text = format!("欢迎 @{} 加入群组！！！",username);
                println!("{}",text);
                let group_id = match msg.chat.kind.clone() {
                    ChatKind::Public(p) => match p.kind {
                        PublicChatKind::Supergroup(sgr) => sgr.username.unwrap(),
                        _ => "".to_string(),
                    },
                    _ => "".to_string(),
                };
                let u = select_group_user(&group_id, &q.from.id.0.to_string()).await?;
                if u.get_join_count().to_string().eq(&version) {
                    update_group_user_status(1, &group_id, &q.from.id.0.to_string()).await?;
                    bot.restrict_chat_member(msg.chat.id, q.from.id, permissions)
                        .await?;
                    bot.edit_message_text(msg.chat.id, msg.id, &text).await?;
                } else {
                    bot.ban_chat_member(msg.chat.id, q.from.id).await?;
                    println!("{}-{}-{}:踢出群组",group_id,q.from.id.0,msg.id);
                }
            }
            None => {}
        }

        //编辑按钮所附邮件的文本
        /*if let Some(Message { id, chat, .. }) = q.message {
            bot.edit_message_text(chat.id, id, text).await?;
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, text).await?;
        }*/
        log::info!("You chose: {}", version);
    }

    Ok(())
}
