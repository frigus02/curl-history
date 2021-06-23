use crate::BoxError;
use sqlx::{migrate::MigrateDatabase as _, Sqlite, SqlitePool};

pub async fn ensure_db() -> Result<SqlitePool, BoxError> {
    let database_path: String = if let Some(config_dir) = dirs::config_dir() {
        let dir = config_dir.join("curl-history");
        std::fs::create_dir_all(&dir)?;
        dir.join("history.db")
            .to_str()
            .expect("config dir should be valid utf8 string")
            .into()
    } else {
        "history.db".into()
    };
    let database_url = format!("sqlite:{}", database_path);
    Sqlite::create_database(&database_url).await?;
    let pool = SqlitePool::connect(&database_url).await?;
    sqlx::migrate!().run(&pool).await?;
    Ok(pool)
}
