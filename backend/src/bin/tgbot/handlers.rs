use anyhow::Context;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use meal_review::db;
use sqlx::SqlitePool;
use teloxide::{
    dispatching::dialogue::InMemStorage, prelude::*, types::Message, utils::command::BotCommands,
    Bot,
};

macro_rules! send {
    ([$bot:expr, $msg:expr], $text:expr) => {
        if let Err(e) = $bot.send_message($msg.chat.id, $text).await {
            tracing::error!("fail to send message: {e}")
        }
    };
}

enum Commands {
    Rest,
    Dish,
    Review,
}

pub(super) fn handler_schema() -> teloxide::dispatching::UpdateHandler<anyhow::Error> {
    let command_handler = teloxide::filter_command::<Commands, _>()
        .branch(dptree::case![Commands::Rest].endpoint(restaurant_handler))
        .branch(
            dptree::case![Commands::Help].endpoint(|msg: Message, bot: Bot| async move {
                send!([bot, msg], Commands::descriptions().to_string());
                Ok(())
            }),
        );

    let message_handler = Update::filter_message().branch(command_handler);

    teloxide::dispatching::dialogue::enter::<Update, InMemStorage<ChatState>, ChatState, _>()
        .branch(message_handler)
}

enum AddRestaurantAction {
    Add(String, String),
    Search(String),
    Edit(i64),
}

struct BtnPrefix;
impl BtnPrefix {
    const RESTAURANT: &str = "RSTBTN";
    const UPDATE_RESTAURANT: &str = "RSTUPDBTN";
}

struct RstBtnAction;
impl RstBtnAction {
    const UPDATE: &str = "update";
    const ADD: &str = "new_dishes";
    const LIST: &str = "list_dishes";
    const DEL: &str = "delete";
}

impl AddRestaurantAction {
    // I need:
    //  /cmd add <name>
    fn new(args: &[&str]) -> Result<Self, &'static str> {
        if args.len() < 2 {
            return Err("too less argument");
        }

        match args[1] {
            "add" => {
                if args.len() < 4 {
                    Err("too less argument")
                } else {
                    Ok(AddRestaurantAction::Add(
                        args[2].to_string(),
                        args[3].to_string(),
                    ))
                }
            }
            "search" => {
                if args.len() < 3 {
                    Err("too less argument")
                } else {
                    Ok(Self::Search(args[2].to_string()))
                }
            }
            "edit" => {
                if args.len() < 3 {
                    Err("too less argument")
                } else {
                    let Ok(id) = args[2].parse() else {
                        return Err("Can not parse your argument into number")
                    };
                    Ok(Self::Edit(id))
                }
            }
            _ => Err("unexpected action"),
        }
    }

    // consumed the action
    async fn run(self, msg: &Message, bot: &Bot, pool: &SqlitePool) -> anyhow::Result<()> {
        match self {
            Self::Add(restaurant, address) => {
                db::add_restaurant(pool, &restaurant, &address).await?;
                send!([bot, msg], "Added.");
            }
            Self::Search(pattern) => {
                let rests = db::get_restaurant(pool, db::RestaurantSearchProps::All).await?;
                // XXX: move it somewhere else then here
                let matcher = SkimMatcherV2::default();
                let result: String = rests
                    .into_iter()
                    .filter(|restaurant| matcher.fuzzy_match(&restaurant.name, &pattern).is_some())
                    .fold(String::new(), |sumed, unit| {
                        format!("{sumed}\n{}. {}", unit.id, unit.name)
                    });
                send!([bot, msg], result);
            }
            Self::Edit(id) => {
                let rest = db::get_restaurant(pool, db::RestaurantSearchProps::Id(id)).await?;
                if rest.is_empty() {
                    send!([bot, msg], "Incorrect id, no restaurant found");
                    return Ok(());
                }
                let rest = &rest[0];
                // build the callback data by "{category}-{id}-{action}"
                let cbd = |action: &str| format!("{}-{}-{action}", BtnPrefix::RESTAURANT, rest.id);
                let btn = teloxide::types::InlineKeyboardButton::callback;
                let buttons = vec![
                    vec![
                        btn("Update Restaurant", cbd(RstBtnAction::UPDATE)),
                        btn("New Dish", cbd(RstBtnAction::ADD)),
                    ],
                    vec![
                        btn("List Dishes", cbd(RstBtnAction::LIST)),
                        btn("Delete", cbd(RstBtnAction::DEL)),
                    ],
                ];
                let markup = teloxide::types::InlineKeyboardMarkup::new(buttons);

                bot.send_message(
                    msg.chat.id,
                    format!("List of operation for: \n\n{} {}", rest.name, rest.address),
                )
                .reply_markup(markup)
                .await?;
            }
        }

        Ok(())
    }
}

async fn restaurant_handler(msg: Message, bot: Bot, pool: SqlitePool) -> anyhow::Result<()> {
    let Some(text) = msg.text() else { return Ok(()) };

    let arguments = text.split(' ').collect::<Vec<_>>();
    let action = AddRestaurantAction::new(&arguments);
    if let Err(hint) = action {
        send!([bot, msg], hint);
        return Ok(());
    }

    let action = action.unwrap();
    if let Err(e) = action.run(&msg, &bot, &pool).await {
        send!(
            [bot, msg],
            format!("Fail to take action on restaurant: {e}")
        );
        return Ok(());
    }

    Ok(())
}

async fn edit_restaurant_name_handler(
    msg: Message,
    bot: Bot,
    dialogue: Dialogue,
    rid: i64,
    pool: &SqlitePool,
) -> anyhow::Result<()> {
    let Some(text) = msg.text() else {
        send!([bot, msg],  "Text name is required, please resend a valid restaurant name, or /cancel .");
        return Ok(())
    };

    if text.contains("/cancel") {
        send!([bot, msg], "Process cancelled");
        dialogue.exit().await?;
        return Ok(());
    }

    db::update_restaurant(
        pool,
        rid,
        db::UpdateRestaurantProps::UpdateName(text.to_string()),
    )
    .await?;

    send!([bot, msg], "Restaurant name is changed to {text}");
    dialogue.exit().await?;

    Ok(())
}

async fn callback_dispatcher(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue,
) -> anyhow::Result<()> {
    // just silently exit
    let Some(data) = query.data else { return Ok(()) };
    let callback_action = data.split('-').collect::<Vec<_>>();
    if callback_action.is_empty() {
        anyhow::bail!("Get callback action without data")
    }
    // we don't handle inline query, so there must be a message
    let Some(message) = query.message else { return Ok(()) };
    match callback_action[0] {
        // This callback format is PREFIX-id-action
        BtnPrefix::RESTAURANT => {
            if callback_action.len() != 3 {
                return Ok(());
            }
            let id: i64 = callback_action[1]
                .parse()
                .with_context(|| {
                    format!("[RSTBTN dispatcher] original format: {callback_action:?}")
                })
                .expect("Met unexpected callback format, please check");

            rst_cb_handler(bot, message, id, callback_action[2]).await?;
        }
        BtnPrefix::UPDATE_RESTAURANT => {
            if callback_action.len() != 3 {
                anyhow::bail!("invalid callback action data")
            }
            let id: i64 = callback_action[1]
                .parse()
                .with_context(|| {
                    format!("[RSTBTN dispatcher] original format: {callback_action:?}")
                })
                .expect("Met unexpected callback format, please check");
            rstupd_cb_handler(bot, message, id, callback_action[2], &dialogue).await?;
        }
        _ => (),
    }

    Ok(())
}

struct RstBtnUpdActionBtn;
impl RstBtnUpdActionBtn {
    const NAME: &str = "name";
    const ADDR: &str = "address";
    const CANCEL: &str = "cancel";
}

async fn rst_cb_handler(bot: Bot, msg: Message, rst_id: i64, action: &str) -> anyhow::Result<()> {
    match action {
        RstBtnAction::UPDATE => {
            let new_text = "What you want to do with this restaurant";
            let cbd = |field: &str| format!("{}-{rst_id}-{field}", BtnPrefix::UPDATE_RESTAURANT);
            let btn = teloxide::types::InlineKeyboardButton::callback;
            let buttons = vec![
                btn("Update Name", cbd(RstBtnUpdActionBtn::NAME)),
                btn("Update Address", cbd(RstBtnUpdActionBtn::ADDR)),
                btn("Cancel", cbd(RstBtnUpdActionBtn::CANCEL)),
            ];
            let new_markup = teloxide::types::InlineKeyboardMarkup::default().append_row(buttons);
            bot.edit_message_text(msg.chat.id, msg.id, new_text)
                .reply_markup(new_markup)
                .await?;
        }
        _ => panic!("Unexpected action {action} present, please check your code"),
    }
    Ok(())
}

async fn rstupd_cb_handler(
    bot: Bot,
    msg: Message,
    rid: i64,
    field: &str,
    dialogue: &Dialogue,
) -> anyhow::Result<()> {
    match field {
        RstBtnUpdActionBtn::NAME => {
            send!([bot, msg], "Please send the new name");
            dialogue.update(ChatState::EditingRstName(rid)).await?;
        }
        RstBtnUpdActionBtn::ADDR => {
            send!([bot, msg], "Please send the new address");
            dialogue.update(ChatState::EditingRstAddr(rid)).await?;
        }
        _ => (),
    }

    Ok(())
}
