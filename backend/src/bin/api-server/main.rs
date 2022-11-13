use actix_cors::Cors;
use actix_web::{web, App, HttpServer};

mod api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(true)
        .with_file(false)
        .pretty()
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("fail to setup logging");
    let state = api::ApiState::new("sqlite://review.db").await;
    let data = web::Data::new(state);
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .app_data(data.clone())
            .service(api::restaurants)
            .service(api::dishes)
            .service(api::reviewes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;
    Ok(())
}
