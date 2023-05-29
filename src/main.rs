use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use sqlx::MySqlPool;
use std::error::Error;
use teloxide::{
    payloads::SendMessageSetters,
    prelude::*,
    types::{
        InlineKeyboardButton, InlineKeyboardMarkup, InlineQueryResultArticle, InputMessageContent,
        InputMessageContentText, Me, ChatKind, PublicChatKind,ChatPermissions,
    },
    utils::command::BotCommands,
};

pub mod service;
use service::{add_group_user, Group};

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

    //连接mysql数据库
    log::info!("Start connecting to database");
    let mysql_pool = MySqlPool::connect("mysql://root:123456@127.0.0.1:3306/telegram_bot_db").await;
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

fn calculate() -> InlineKeyboardMarkup {
    let mut keyboard: Vec<Vec<InlineKeyboardButton>> = vec![];

    let debian_versions = [
        "10", "6", "9", "16", 
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
                let message = format!("you send message is {:?}", text);
                bot.send_message(msg.chat.id, message).await?;
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
    let username = chat_member.chat.username().unwrap();
    let user_id = chat_member.from.id;
    //如果old 是left，则是加入，如果old是Member是退出
    match chat_member.new_chat_member.kind {
        teloxide::types::ChatMemberKind::Member => {
            println!("有新用户加入，输出欢迎词！");
            let group_id = match chat_member.chat.kind.clone() {
                ChatKind::Public(p) => {
                    match p.kind {
                        PublicChatKind::Supergroup(sgr) => {
                            sgr.username.unwrap()
                        }
                        _ => {
                            "".to_string()
                        }
                    }
                }
                _ => {
                    "".to_string()
                }
            };
            add_group_user(Group::new(username, &user_id.0.to_string(), &group_id, 0)).await?;
            //给用户禁用发现权限，并且发送一个用户验证消息。
            //禁用消息
            let time = "2023-5-30T21:00:09+09:00".parse::<DateTime<Utc>>().unwrap();
            //直接封禁用户
            bot.kick_chat_member(chat_member.chat.id, user_id).until_date(time).await?;
            bot.send_message(
                chat_member.chat.id,
                "请选择正确答案：8 + 8 = ?"
            ).reply_markup(calculate())
            .await?;
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
        let text = format!("You chose: {version}");

        //告诉telegram我们已经看到这个查询，以删除🕑 的图标
        //客户。您也可以使用`answer_callback_query`的可选项
        //参数来调整客户端上发生的事情。
        bot.answer_callback_query(q.id).await?;
        
        //编辑按钮所附邮件的文本
        if let Some(Message { id, chat, .. }) = q.message {
            bot.edit_message_text(chat.id, id, text).await?;
        } else if let Some(id) = q.inline_message_id {
            bot.edit_message_text_inline(id, text).await?;
        }
        log::info!("You chose: {}", version);
    }

    Ok(())
}
