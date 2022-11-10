use crate::db;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
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

enum AddRestaurantAction {
    Add(String, String),
    Search(String),
}

impl AddRestaurantAction {
    // I need:
    //  /cmd add <name>
    fn new(args: &[&str]) -> Result<Self, &'static str> {
        if args.len() < 4 {
            return Err("too less argument");
        }

        match args[1] {
            "add" => (),
            _ => return Err("unexpected action"),
        }

        Ok(AddRestaurantAction::Add(
            args[2].to_string(),
            args[3].to_string(),
        ))
    }

    // consumed the action
    async fn run(self, msg: &Message, bot: &Bot, pool: &SqlitePool) -> anyhow::Result<()> {
        match self {
            Self::Add(restaurant, address) => {
                db::add_restaurant(pool, &restaurant, &address).await?;
                send!([bot, msg], "Added.");
            }
            Self::Search(pattern) => {
                let rests = db::get_restaurant(pool, None).await?;
                let matcher = SkimMatcherV2::default();
                let result: String = rests
                    .into_iter()
                    .filter(|restaurant| matcher.fuzzy_match(&restaurant.name, &pattern).is_some())
                    .fold(String::new(), |sumed, unit| {
                        format!("{sumed}\n{}. {}", unit.id, unit.name)
                    });
                send!([bot, msg], result);
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
