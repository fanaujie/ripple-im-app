use keyring::Entry;
use serde::{Deserialize, Serialize};
use sqlx::sqlite;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode};
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserProfile {
    pub user_id: String,
    pub nickname: String,
    pub avatar_path: Option<String>,
}
pub struct DB {
    pool: sqlite::SqlitePool,
}

impl Clone for DB {
    fn clone(&self) -> Self {
        DB {
            pool: self.pool.clone(),
        }
    }
}

impl DB {
    pub async fn new(app_data_dir: PathBuf) -> anyhow::Result<DB> {
        let db_path = app_data_dir.join("sqlite.db");
        println!("Using database path: {}", db_path.display());
        let db_url = format!("sqlite:{}", db_path.display());
        let pool = Self::load_db(db_url).await?;
        Ok(DB { pool })
    }

    pub async fn save_token(&self, access_token: &str, refresh_token: &str) -> anyhow::Result<i64> {
        if self.exists_token().await? {
            // Update existing token
            sqlx::query(
                r#"
UPDATE oauth_tokens SET access_token = ?, refresh_token = ?
        "#,
            )
            .bind(&access_token)
            .bind(&refresh_token)
            .execute(&self.pool)
            .await?;
            // Return the ID of the updated row (assuming there's only one)
            let id: (i64,) = sqlx::query_as(r#"SELECT id FROM oauth_tokens LIMIT 1"#)
                .fetch_one(&self.pool)
                .await?;
            Ok(id.0)
        } else {
            // Insert new token
            let id = sqlx::query(
                r#"
INSERT INTO oauth_tokens ( access_token,refresh_token ) VALUES ( ?, ? )
        "#,
            )
            .bind(&access_token)
            .bind(&refresh_token)
            .execute(&self.pool)
            .await?
            .last_insert_rowid();
            Ok(id)
        }
    }

    pub async fn exists_token(&self) -> anyhow::Result<bool> {
        let r: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM oauth_tokens"#)
            .fetch_one(&self.pool)
            .await?;
        Ok(r.0 == 1)
    }

    pub async fn get_token(&self) -> anyhow::Result<(String, String)> {
        let r: (String, String) =
            sqlx::query_as(r#"SELECT access_token, refresh_token FROM oauth_tokens"#)
                .fetch_one(&self.pool)
                .await?;
        Ok((r.0, r.1))
    }

    fn get_cipher_key() -> anyhow::Result<String> {
        let entry = Entry::new("ripple-im-app", "ripple")?;
        match entry.get_password() {
            Ok(password) => Ok(password),
            Err(keyring::Error::NoEntry) => {
                let new_password = Uuid::new_v4().simple().to_string();
                entry.set_password(&new_password)?;
                Ok(new_password)
            }
            Err(e) => Err(anyhow::anyhow!(
                "Failed to retrieve cipher key from keyring: {}",
                e
            )),
        }
    }

    async fn load_db(db_url: String) -> anyhow::Result<sqlite::SqlitePool> {
        let options = SqliteConnectOptions::from_str(&db_url)?
            .pragma("key", Self::get_cipher_key()?)
            .pragma("cipher_page_size", "1024")
            .pragma("kdf_iter", "64000")
            .pragma("cipher_hmac_algorithm", "HMAC_SHA1")
            .pragma("cipher_kdf_algorithm", "PBKDF2_HMAC_SHA1")
            .journal_mode(SqliteJournalMode::Delete)
            .foreign_keys(false)
            .create_if_missing(true);
        let pool = sqlite::SqlitePool::connect_with(options).await?;
        let migrator = sqlx::migrate!("./migrations");
        migrator.run(&pool).await?;
        Ok(pool)
    }

    pub async fn clear_tokens(&self) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM oauth_tokens")
            .execute(&self.pool)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to clear tokens from database: {}", e))?;
        Ok(())
    }
}
