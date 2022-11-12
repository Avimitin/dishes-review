use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use meal_review::db;
use sqlx::SqlitePool;
use teloxide::{prelude::*, types::Message, Bot};

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
                let cbd = |action: &str| format!("RSTBTN-{}-{action}", rest.id);
                let btn = teloxide::types::InlineKeyboardButton::callback;
                let buttons = vec![
                    vec![
                        btn("Update Restaurant", cbd("update")),
                        btn("New Dish", cbd("new_dishes")),
                    ],
                    vec![
                        btn("List Dishes", cbd("list_dishes")),
                        btn("Delete", cbd("delete")),
                    ],
                ];
                let markup = teloxide::types::InlineKeyboardMarkup::new(buttons);

                bot.send_message(
                    msg.chat.id,
                    format!("List of operation for: \n\n{}", rest.name),
                )
                .reply_markup(markup)
                .await?;
            }
        }

        Ok(())
    }
}

async fn restaurant_handler(msg: Message, bot: Bot, pool: &SqlitePool) -> anyhow::Result<()> {
    let Some(text) = msg.text() else { return Ok(()) };

    let arguments = text.split(' ').collect::<Vec<_>>();
    let help = "Usage: /restaurant add <name>";
    let action = AddRestaurantAction::new(&arguments);
    if let Err(hint) = action {
        send!([bot, msg], format!("{hint}\n\n{help}"));
        return Ok(());
    }

    let action = action.unwrap();
    if let Err(e) = action.run(&msg, &bot, pool).await {
        send!(
            [bot, msg],
            format!("Fail to take action on restaurant: {e}")
        );
        return Ok(());
    }

    Ok(())
}
