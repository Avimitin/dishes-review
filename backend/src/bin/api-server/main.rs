use actix_web::{App, HttpServer};

mod api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(api::restaurants)
            .service(api::dishes)
            .service(api::reviewes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;
    Ok(())
}
