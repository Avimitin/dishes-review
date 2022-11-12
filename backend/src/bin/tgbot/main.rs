use teloxide::{
    dispatching::dialogue::InMemStorage,
    dptree,
    prelude::{Dispatcher, LoggingErrorHandler},
    Bot,
};

mod handlers;
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(true)
        .with_file(false)
        .pretty()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("fail to setup logging");

    let schema = handlers::handler_schema();

    let bot = Bot::new(&std::env::var("TGBOT_TOKEN").expect("TGBOT_TOKEN env not found"));
    let dbpool = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL env not found"))
        .await
        .expect("fail to connect to sqlite database");

    // TODO: add error handler
    Dispatcher::builder(bot, schema)
        .dependencies(dptree::deps![InMemStorage::<handlers::ChatState>::new(), dbpool])
        .enable_ctrlc_handler()
        .default_handler(|_| async move {})
        .error_handler(LoggingErrorHandler::with_custom_text(
            "Error occur when handling update",
        ))
        .build()
        .dispatch()
        .await;
}
