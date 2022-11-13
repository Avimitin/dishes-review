use actix_web::{web, HttpResponse};
use meal_review::db as db_api;

pub(super) struct ApiState {
    db_pool: sqlx::SqlitePool,
}

impl ApiState {
    pub(super) async fn new(addr: &str) -> Self {
        let db_pool = sqlx::SqlitePool::connect(addr)
            .await
            .expect("fail to open database");
        Self { db_pool }
    }
}

#[derive(serde::Serialize)]
struct ErrJsonResp {
    message: String,
}

#[actix_web::get("/api/v1/restaurants")]
pub(super) async fn restaurants(data: web::Data<ApiState>) -> HttpResponse {
    let result = db_api::get_restaurant(&data.db_pool, db_api::RestaurantSearchProps::All).await;
    if let Err(err) = result {
        HttpResponse::Ok().json(ErrJsonResp {
            message: err.to_string(),
        })
    } else {
        HttpResponse::Ok().json(result.unwrap())
    }
}

#[derive(serde::Deserialize)]
pub(super) struct RestaurantPath {
    id: i64,
}

#[actix_web::get("/api/v1/restaurants/{id}")]
pub(super) async fn dishes(
    data: web::Data<ApiState>,
    path: web::Path<RestaurantPath>,
) -> HttpResponse {
    let result = db_api::get_dish(&data.db_pool, path.id, None).await;
    if let Err(err) = result {
        HttpResponse::Ok().json(ErrJsonResp {
            message: err.to_string(),
        })
    } else {
        HttpResponse::Ok().json(result.unwrap())
    }
}

#[derive(serde::Deserialize)]
pub(super) struct DishesPath {
    id: i64,
}

#[actix_web::get("/api/v1/dishes/{id}")]
pub(super) async fn reviewes(
    data: web::Data<ApiState>,
    path: web::Path<DishesPath>,
) -> HttpResponse {
    let prop = db_api::GetReviewPropsBuilder::default()
        .dish_id(path.id)
        .build()
        .unwrap();
    let result = db_api::get_review(&data.db_pool, prop).await;
    if let Err(err) = result {
        HttpResponse::Ok().json(ErrJsonResp {
            message: err.to_string(),
        })
    } else {
        HttpResponse::Ok().json(result.unwrap())
    }
}
