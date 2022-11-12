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

#[derive(Debug, Default, Clone)]
pub(super) enum ChatState {
    #[default]
    None,
    CreatingDishesStage1(i64),
    CreatingDisheFinal(i64, String),
    CreatingReviewStage1(i64),
    CreatingReviewStage2(i64, String),
    EditingRstName(i64),
    EditingRstAddr(i64),
}

type Dialogue = teloxide::prelude::Dialogue<ChatState, InMemStorage<ChatState>>;

#[derive(BotCommands, Clone, Debug)]
#[command(
    rename_rule = "lowercase",
    description = "Commands for operating the database"
)]
enum Commands {
    #[command(description = "Display this help page")]
    Help,
    #[command(description = "Operate the restaurant")]
    Rest,
    #[command(description = "Operate on dish")]
    Dish,
    #[command(description = "Operate the review")]
    Review,
}

pub(super) fn handler_schema() -> teloxide::dispatching::UpdateHandler<anyhow::Error> {
    use dptree::case;
    let command_handler = teloxide::filter_command::<Commands, _>()
        .branch(case![Commands::Rest].endpoint(restaurant_handler))
        .branch(case![Commands::Review].endpoint(cmd_review_handler))
        .branch(
            case![Commands::Help].endpoint(|msg: Message, bot: Bot| async move {
                send!([bot, msg], Commands::descriptions().to_string());
                Ok(())
            }),
        );

    let message_handler = Update::filter_message()
        .branch(case![ChatState::EditingRstName(_name)].endpoint(edit_restaurant_name_handler))
        .branch(case![ChatState::EditingRstAddr(_a)].endpoint(edit_restaurant_address_handler))
        .branch(case![ChatState::CreatingDishesStage1(_a)].endpoint(add_dish_stage1_handler))
        .branch(case![ChatState::CreatingDisheFinal(_a, _b)].endpoint(add_dish_final_handler))
        .branch(case![ChatState::CreatingReviewStage1(_a)].endpoint(review_stage1_handler))
        .branch(case![ChatState::CreatingReviewStage2(_a, _b)].endpoint(review_stage2_handler))
        .branch(command_handler);

    let callback_handler = Update::filter_callback_query().endpoint(callback_dispatcher);

    teloxide::dispatching::dialogue::enter::<Update, InMemStorage<ChatState>, ChatState, _>()
        .branch(callback_handler)
        .branch(message_handler)
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

enum AddRestaurantAction {
    Add(String, String),
    Search(String),
    Edit(i64),
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
                    Err("too less argument, usage: /rest add <name> <addr>")
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
            //
            Self::Search(pattern) => {
                let rests = db::get_restaurant(pool, db::RestaurantSearchProps::All).await?;
                // XXX: move it somewhere else then here
                let matcher = SkimMatcherV2::default();
                let result: String = rests
                    .into_iter()
                    .filter(|restaurant| matcher.fuzzy_match(&restaurant.name, &pattern).is_some())
                    .fold(String::new(), |sumed, unit| {
                        format!("{sumed}\n{}. {} {}", unit.id, unit.name, unit.address)
                    });
                send!([bot, msg], result);
            }
            //
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
    pool: SqlitePool,
) -> anyhow::Result<()> {
    let Some(text) = msg.text() else {
        send!([bot, msg],  "Text name is required, please resend a valid restaurant name, or /cancel.");
        return Ok(())
    };

    if text.contains("/cancel") {
        send!([bot, msg], "Process cancelled");
        dialogue.exit().await?;
        return Ok(());
    }

    db::update_restaurant(
        &pool,
        rid,
        db::UpdateRestaurantProps::UpdateName(text.to_string()),
    )
    .await?;

    send!([bot, msg], format!("Restaurant name is changed to {text}"));
    dialogue.exit().await?;

    Ok(())
}

async fn edit_restaurant_address_handler(
    bot: Bot,
    msg: Message,
    pool: SqlitePool,
    dialogue: Dialogue,
    rid: i64,
) -> anyhow::Result<()> {
    let Some(text) = msg.text() else {
        send!([bot, msg], "Required text message, please resend a valid restaurant address, or /cancel.");
        return Ok(())
    };

    if text.contains("/cancel") {
        send!([bot, msg], "Process cancelled");
        dialogue.exit().await?;
        return Ok(());
    }

    db::update_restaurant(
        &pool,
        rid,
        db::UpdateRestaurantProps::UpdateAddr(text.to_string()),
    )
    .await?;

    send!([bot, msg], format!("Restaurant name is changed to {text}"));
    dialogue.exit().await?;

    Ok(())
}

async fn callback_dispatcher(
    bot: Bot,
    query: CallbackQuery,
    dialogue: Dialogue,
    pool: SqlitePool,
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

            rst_cb_handler(bot, message, id, callback_action[2], &dialogue, &pool).await?;
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
}

async fn rst_cb_handler(
    bot: Bot,
    msg: Message,
    rst_id: i64,
    action: &str,
    dialogue: &Dialogue,
    pool: &SqlitePool,
) -> anyhow::Result<()> {
    match action {
        RstBtnAction::UPDATE => {
            let new_text = "What you want to do with this restaurant";
            let cbd = |field: &str| format!("{}-{rst_id}-{field}", BtnPrefix::UPDATE_RESTAURANT);
            let btn = teloxide::types::InlineKeyboardButton::callback;
            let buttons = vec![
                btn("Update Name", cbd(RstBtnUpdActionBtn::NAME)),
                btn("Update Address", cbd(RstBtnUpdActionBtn::ADDR)),
            ];
            let new_markup = teloxide::types::InlineKeyboardMarkup::default().append_row(buttons);
            bot.edit_message_text(msg.chat.id, msg.id, new_text)
                .reply_markup(new_markup)
                .await?;
        }
        RstBtnAction::ADD => {
            send!([bot, msg], "Please send the name of the dish");
            dialogue
                .update(ChatState::CreatingDishesStage1(rst_id))
                .await?;
        }
        RstBtnAction::LIST => {
            let dishes = db::get_dish(pool, rst_id, None).await?;
            let text = if dishes.is_empty() {
                String::from("No dishes found")
            } else {
                dishes.iter().fold(String::new(), |sum, now| {
                    format!("{sum}* {} {}\n", now.id, now.name)
                })
            };
            send!([bot, msg], text);
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
            send!(
                [bot, msg],
                "Please send the new name, press /cancel to cancel"
            );
            dialogue.update(ChatState::EditingRstName(rid)).await?;
        }
        RstBtnUpdActionBtn::ADDR => {
            send!(
                [bot, msg],
                "Please send the new address, press /cancel to cancel"
            );
            dialogue.update(ChatState::EditingRstAddr(rid)).await?;
        }
        _ => (),
    }

    Ok(())
}

async fn add_dish_stage1_handler(
    bot: Bot,
    msg: Message,
    rid: i64,
    dialogue: Dialogue,
) -> anyhow::Result<()> {
    let Some(text) = msg.text() else {
        send!([bot, msg], "Please send text message");
        return Ok(());
    };

    if text.contains("/cancel") {
        dialogue.exit().await?;
        send!([bot, msg], "Cancelled");
        return Ok(());
    }

    send!(
        [bot, msg],
        format!("{text} created, please send a picture, or just click /skip")
    );
    dialogue
        .update(ChatState::CreatingDisheFinal(rid, text.to_string()))
        .await?;
    Ok(())
}

async fn add_dish_final_handler(
    bot: Bot,
    msg: Message,
    stage1: (i64, String),
    dialogue: Dialogue,
    pool: SqlitePool,
) -> anyhow::Result<()> {
    let mut skipped = false;
    if let Some(text) = msg.text() {
        if text.contains("/skip") {
            skipped = true;
        } else {
            send!([bot, msg], "Need Image, not text, /skip ?");
            return Ok(());
        }
    }

    let image = if skipped {
        None
    } else {
        let Some(images) = msg.photo() else {
            send!([bot,msg], "Need images, please retry or /skip");
            return Ok(())
        };

        if images.len() > 1 {
            send!(
                [bot, msg],
                "Multiple images detected, choosing the first one"
            );
        }

        Some(images[0].file.id.clone())
    };

    db::add_dish(&pool, stage1.0, &stage1.1, image).await?;

    dialogue.exit().await?;

    send!([bot, msg], "Dish added");
    Ok(())
}

async fn cmd_review_handler(bot: Bot, msg: Message, dialogue: Dialogue) -> anyhow::Result<()> {
    let Some(text) = msg.text() else {
        return Ok(());
    };

    let args = text.split(' ').collect::<Vec<_>>();
    if args.len() != 2 {
        send!([bot, msg], "Usage: /review <Dish ID>");
        return Ok(());
    }
    let Ok(dish_id) = args[1].parse::<i64>() else {
        send!([bot, msg], format!("{} is not a valid number", args[1]));
        return Ok(())
    };

    dialogue
        .update(ChatState::CreatingReviewStage1(dish_id))
        .await?;

    send!([bot, msg], "Please send your review");

    Ok(())
}

async fn review_stage1_handler(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue,
    dish_id: i64,
) -> anyhow::Result<()> {
    let Some(text) = msg.text() else {
        send!([bot, msg], "I need text message");
        return Ok(())
    };
    send!([bot, msg], "Please send your rating for this dish, 0 - 5");
    dialogue
        .update(ChatState::CreatingReviewStage2(dish_id, text.to_string()))
        .await?;
    Ok(())
}

async fn review_stage2_handler(
    bot: Bot,
    msg: Message,
    dialogue: Dialogue,
    props: (i64, String),
    pool: SqlitePool,
) -> anyhow::Result<()> {
    let Some(text) = msg.text() else {
        send!([bot, msg], "I need number of the score, please retry");
        return Ok(());
    };

    let score = text.parse::<u8>();
    if score.is_err() {
        send!([bot, msg], "Invalid number, please retry");
        return Ok(());
    }
    let score = score.unwrap();
    if !(0..=5).contains(&score) {
        send!([bot, msg], "Invalid number, please send 0, 1, 2, 3, 4 or 5");
        return Ok(());
    }

    let review = db::NewReviewPropsBuilder::default()
        .dish(db::DishProp::Id(props.0))
        .reviewer(db::ReviewerProp::Id(
            // cast u64 to i64, this is probably safe to unwrap()
            msg.from().unwrap().id.0.try_into().unwrap(),
        ))
        .score(score)
        .details(props.1)
        .build()
        .unwrap();

    db::add_new_review(&pool, review).await?;
    dialogue.exit().await?;

    send!([bot, msg], "New review added");

    Ok(())
}
