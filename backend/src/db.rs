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

#[derive(sqlx::FromRow)]
pub struct Dish {
    pub id: i64,
    #[sqlx(rename = "restaurant")]
    pub rid: i64,
    pub name: String,
    #[sqlx(default)]
    pub image: Option<String>,
}

pub async fn get_dish(
    db_conn: &SqlitePool,
    restaurant: i64,
    dish_id: Option<i64>,
) -> anyhow::Result<Vec<Dish>> {
    let query = if let Some(dish_id) = dish_id {
        sqlx::query_as("SELECT * FROM dish WHERE id=?").bind(dish_id)
    } else {
        sqlx::query_as("SELECT * FROM dish WHERE restaurant=?").bind(restaurant)
    };

    let dishes = query.fetch_all(db_conn).await?;

    Ok(dishes)
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
    pub reviewer: i64,
    pub score: u8,
    pub details: String,
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

#[derive(sqlx::FromRow)]
pub struct Restaurant {
    pub name: String,
    pub id: i64,
    pub address: String,
}

pub enum RestaurantSearchProps {
    Range(i64, i64),
    Id(i64),
    All,
}

// for future migration, maybe someday I don't want sqlite
type DB = sqlx::Sqlite;
type DBArg<'q> = sqlx::sqlite::SqliteArguments<'q>;

type SqliteQueryAs<'q, O> = sqlx::query::QueryAs<'q, DB, O, DBArg<'q>>;

impl RestaurantSearchProps {
    pub fn into_query_as<'q>(self) -> SqliteQueryAs<'q, Restaurant> {
        match self {
            Self::Range(s, e) => {
                sqlx::query_as::<_, Restaurant>("SELECT * FROM restaurant WHERE id BETWEEN ? AND ?")
                    .bind(s)
                    .bind(e)
            }
            Self::Id(id) => {
                sqlx::query_as::<_, Restaurant>("SELECT * FROM restaurant WHERE id=?").bind(id)
            }
            Self::All => sqlx::query_as::<_, Restaurant>("SELECT * FROM restaurant"),
        }
    }
}

pub async fn get_restaurant(
    db_conn: &SqlitePool,
    props: RestaurantSearchProps,
) -> anyhow::Result<Vec<Restaurant>> {
    let sql = props.into_query_as();

    let rsts: Vec<Restaurant> = sql
        .fetch_all(db_conn)
        .await
        .with_context(|| "Fail to get restaurants".to_string())?;

    Ok(rsts)
}

pub enum UpdateRestaurantProps {
    UpdateName(String),
    UpdateAddr(String),
    Delete,
}

impl UpdateRestaurantProps {
    fn into_query<'q>(self, id: i64) -> sqlx::query::Query<'q, DB, DBArg<'q>> {
        match self {
            Self::UpdateName(name) => sqlx::query("UPDATE restaurant SET name=? WHERE id=?")
                .bind(name)
                .bind(id),
            Self::UpdateAddr(addr) => sqlx::query("UPDATE restaurant SET address=? WHERE id=?")
                .bind(addr)
                .bind(id),
            Self::Delete => sqlx::query("DELETE FROM restaurant WHERE id=?").bind(id),
        }
    }
}

pub async fn update_restaurant(
    db_conn: &SqlitePool,
    id: i64,
    props: UpdateRestaurantProps,
) -> anyhow::Result<()> {
    props
        .into_query(id)
        .execute(db_conn)
        .await
        .with_context(|| "fail to update restaurant")?;

    Ok(())
}

#[tokio::test]
async fn test_add_new_review() {
    let db = sqlx::sqlite::SqlitePool::connect("sqlite:review.db")
        .await
        .unwrap();

    add_new_user(&db, (649191333, "Avimitin")).await.unwrap();
    let expect = "KFC";
    let rid = add_restaurant(&db, expect, "WuHan").await.unwrap();
    let restaurant = get_restaurant(&db, RestaurantSearchProps::All)
        .await
        .unwrap();
    assert!(!restaurant.is_empty());
    assert_eq!(restaurant[0].id, 1);
    assert_eq!(restaurant[0].name, expect);

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
