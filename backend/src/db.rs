use anyhow::Context;
use derive_builder::Builder;
use sqlx::{sqlite::SqlitePool, Row};

#[derive(Clone)]
pub enum ReviewerProp {
    Name(String),
    Id(i64),
}

impl ReviewerProp {
    async fn get_db_id(&self, db_conn: &SqlitePool) -> anyhow::Result<i64> {
        let id: i64 = match self {
            Self::Id(id) => *id,
            Self::Name(name) => {
                let row = sqlx::query("SELECT id FROM reviewer WHERE name = ?")
                    .bind(name)
                    .fetch_one(db_conn)
                    .await?;
                row.get("id")
            }
        };
        Ok(id)
    }
}

#[derive(Clone)]
pub enum DishProp {
    Id(i64),
    Name(String),
}

impl DishProp {
    async fn get_dish_id(&self, db_conn: &SqlitePool) -> anyhow::Result<i64> {
        let id: i64 = match self {
            Self::Id(id) => *id,
            Self::Name(name) => {
                let row = sqlx::query("SELECT id FROM dish WHERE name = ?")
                    .bind(name)
                    .fetch_one(db_conn)
                    .await?;
                row.get("id")
            }
        };
        Ok(id)
    }
}

#[derive(Builder)]
pub struct NewReviewProps {
    reviewer: ReviewerProp,
    dish: DishProp,
    details: String,
    score: u8,
}

pub async fn add_new_user(db_conn: &SqlitePool, user: (i64, &str)) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO reviewer (id, name) VALUES (?, ?)")
        .bind(user.0)
        .bind(user.1)
        .execute(db_conn)
        .await
        .with_context(|| format!("fail to add new user {}", user.1))?;
    Ok(())
}

pub async fn add_restaurant(db_conn: &SqlitePool, name: &str, addr: &str) -> anyhow::Result<i64> {
    let id = sqlx::query("INSERT INTO restaurant (name, address) VALUES (?, ?)")
        .bind(name)
        .bind(addr)
        .execute(db_conn)
        .await
        .with_context(|| format!("fail to add new restaurant {name}"))?
        .last_insert_rowid();
    Ok(id)
}

pub async fn add_dish(
    db_conn: &SqlitePool,
    restaurant: i64,
    name: &str,
    image: Option<String>,
) -> anyhow::Result<i64> {
    let row = if let Some(image) = image {
        sqlx::query("INSERT INTO dish (restaurant, name, image) VALUES (?, ?, ?)")
            .bind(restaurant)
            .bind(name)
            .bind(image)
            .execute(db_conn)
            .await?
    } else {
        sqlx::query("INSERT INTO dish (restaurant, name) VALUES (?, ?)")
            .bind(restaurant)
            .bind(name)
            .execute(db_conn)
            .await?
    };

    Ok(row.last_insert_rowid())
}

pub async fn add_new_review(db_conn: &SqlitePool, prop: NewReviewProps) -> anyhow::Result<()> {
    let NewReviewProps {
        reviewer,
        dish,
        details,
        score,
    } = prop;

    let reviewer_id = reviewer.get_db_id(db_conn).await?;
    let dish_id = dish.get_dish_id(db_conn).await?;

    sqlx::query(
        r#"
INSERT INTO review
    (reviewer, dish, details, score)
VALUES
    (?, ?, ?, ?)"#,
    )
    .bind(reviewer_id)
    .bind(dish_id)
    .bind(details)
    .bind(score)
    .execute(db_conn)
    .await?;

    Ok(())
}

#[derive(Builder)]
pub struct GetReviewProps {
    #[builder(setter(into, strip_option), default)]
    id: Option<i64>,
    #[builder(setter(into, strip_option), default)]
    dish_id: Option<i64>,
}

#[derive(sqlx::FromRow)]
pub struct Review {
    reviewer: i64,
    score: u8,
    details: String,
}

pub async fn get_review(db_conn: &SqlitePool, props: GetReviewProps) -> anyhow::Result<Review> {
    let GetReviewProps { id, dish_id } = props;
    let query = if let Some(id) = id {
        sqlx::query_as::<_, Review>("SELECT reviewer, details, score FROM review WHERE id=?")
            .bind(id)
    } else if let Some(id) = dish_id {
        sqlx::query_as::<_, Review>("SELECT reviewer, details, score FROM review WHERE dish=?")
            .bind(id)
    } else {
        // XXX
        panic!()
    };

    let row = query
        .fetch_one(db_conn)
        .await
        .with_context(|| "fail to get review")?;

    Ok(row)
}

#[tokio::test]
async fn test_add_new_review() {
    let db = sqlx::sqlite::SqlitePool::connect("sqlite:review.db")
        .await
        .unwrap();

    add_new_user(&db, (649191333, "Avimitin")).await.unwrap();
    let rid = add_restaurant(&db, "KFC", "WuHan").await.unwrap();
    let did = add_dish(&db, rid, "", None).await.unwrap();

    let comment = "Very good chicken, love from WuHan";
    let prop = NewReviewPropsBuilder::default()
        .dish(DishProp::Id(did))
        .reviewer(ReviewerProp::Id(649191333))
        .details(comment.to_string())
        .score(5)
        .build()
        .unwrap();
    add_new_review(&db, prop).await.unwrap();

    let review = get_review(
        &db,
        GetReviewPropsBuilder::default()
            .dish_id(did)
            .build()
            .unwrap(),
    )
    .await
    .unwrap();

    assert_eq!(review.details, comment);
}
