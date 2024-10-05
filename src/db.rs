use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use anyhow::Result;
use bcrypt::{hash, verify};
use chrono::Utc;

use crate::data::{User, SearchHistory};

pub async fn establish_connection() -> Result<Pool<Postgres>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    Ok(pool)
}

pub async fn register_user(username: &str, password: &str) -> Result<()> {
    let pool = establish_connection().await?;
    let password_hash = hash(password, 4)?;
    sqlx::query!(
        "INSERT INTO users (username, password_hash) VALUES ($1, $2)",
        username,
        password_hash
    )
    .execute(&pool)
    .await?;
    Ok(())
}

pub async fn login_user(username: &str, password: &str) -> Result<bool> {
    let pool = establish_connection().await?;
    let user = sqlx::query_as!(
        User,
        "SELECT id, username, password_hash FROM users WHERE username = $1",
        username
    )
    .fetch_one(&pool)
    .await?;

    Ok(verify(password, &user.password_hash)?)
}

pub async fn save_search_history(username: &str, query: &str) -> Result<()> {
    let pool = establish_connection().await?;
    let user = sqlx::query_as!(
        User,
        "SELECT id, username, password_hash FROM users WHERE username = $1",
        username
    )
    .fetch_one(&pool)
    .await?;

    sqlx::query!(
        "INSERT INTO search_history (user_id, query, timestamp) VALUES ($1, $2, $3)",
        user.id,
        query,
        Utc::now().naive_utc()
    )
    .execute(&pool)
    .await?;
    Ok(())
}
