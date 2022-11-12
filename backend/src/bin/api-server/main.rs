use actix_cors::Cors;
use actix_web::{App, HttpServer};

mod api;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .service(api::restaurants)
            .service(api::dishes)
            .service(api::reviewes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;
    Ok(())
}
