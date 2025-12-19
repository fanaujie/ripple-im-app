use crate::ripple_api::api_response::{
    GroupMemberData, MessageCommandType, MessageItem, MessageItemType, RelationUser, UserGroupData,
    UserProfileData,
};
use crate::store_engine::store_engine::{
    ConversationRecord, ConversationStorageAction, GroupMemberStorageAction, RelationStorageAction,
    RippleStorage, Token, UserGroupStorageAction,
};
use keyring::Entry;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool};
use std::path::PathBuf;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Clone)]
pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn new(app_data_dir: PathBuf) -> anyhow::Result<Self> {
        let db_path = app_data_dir.join("sqlite.db");
        println!("[SqliteStore] Using database path: {}", db_path.display());
        let db_url = format!("sqlite:{}", db_path.display());
        let pool = Self::load_db(db_url).await?;
        Ok(SqliteStore { pool })
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

    async fn load_db(db_url: String) -> anyhow::Result<SqlitePool> {
        let options = SqliteConnectOptions::from_str(&db_url)?
            .pragma("key", Self::get_cipher_key()?)
            .pragma("cipher_page_size", "1024")
            .pragma("kdf_iter", "64000")
            .pragma("cipher_hmac_algorithm", "HMAC_SHA1")
            .pragma("cipher_kdf_algorithm", "PBKDF2_HMAC_SHA1")
            .journal_mode(SqliteJournalMode::Delete)
            .foreign_keys(false)
            .create_if_missing(true);
        let pool = SqlitePool::connect_with(options).await?;
        let migrator = sqlx::migrate!("./migrations");
        migrator.run(&pool).await?;
        Ok(pool)
    }
}

impl RippleStorage for SqliteStore {
    async fn exists_token(&self) -> anyhow::Result<bool> {
        let r: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM oauth_tokens")
            .fetch_one(&self.pool)
            .await?;
        Ok(r.0 > 0)
    }

    async fn exist_relations(&self) -> anyhow::Result<bool> {
        let r: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM relations")
            .fetch_one(&self.pool)
            .await?;
        Ok(r.0 > 0)
    }

    async fn exist_conversations(&self) -> anyhow::Result<bool> {
        let r: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM conversations")
            .fetch_one(&self.pool)
            .await?;
        Ok(r.0 > 0)
    }

    async fn get_device_id(&self) -> anyhow::Result<Option<Uuid>> {
        let r: (Option<String>,) =
            sqlx::query_as("SELECT device_id FROM app_metadata WHERE id = 1")
                .fetch_one(&self.pool)
                .await?;
        match r.0 {
            Some(id) => Ok(Some(Uuid::parse_str(&id)?)),
            None => Ok(None),
        }
    }

    async fn save_device_id(&self, device_id: &Uuid) -> anyhow::Result<()> {
        sqlx::query("UPDATE app_metadata SET device_id = ? WHERE id = 1")
            .bind(device_id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_token(&self) -> anyhow::Result<Option<Token>> {
        let r: Option<(String, String)> =
            sqlx::query_as("SELECT access_token, refresh_token FROM oauth_tokens LIMIT 1")
                .fetch_optional(&self.pool)
                .await?;
        match r {
            Some((access_token, refresh_token)) => Ok(Some(Token {
                access_token,
                refresh_token,
            })),
            None => Ok(None),
        }
    }

    async fn save_token(&self, access_token: &str, refresh_token: &str) -> anyhow::Result<()> {
        if self.exists_token().await? {
            sqlx::query("UPDATE oauth_tokens SET access_token = ?, refresh_token = ?")
                .bind(access_token)
                .bind(refresh_token)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query("INSERT INTO oauth_tokens (access_token, refresh_token) VALUES (?, ?)")
                .bind(access_token)
                .bind(refresh_token)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    async fn clear_token(&self) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM oauth_tokens")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_stored_user_id(&self) -> anyhow::Result<Option<String>> {
        let r: (Option<String>,) =
            sqlx::query_as("SELECT user_id FROM app_metadata WHERE id = 1")
                .fetch_one(&self.pool)
                .await?;
        Ok(r.0)
    }

    async fn save_user_id(&self, user_id: &str) -> anyhow::Result<()> {
        sqlx::query("UPDATE app_metadata SET user_id = ? WHERE id = 1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn clear_all_data(&self) -> anyhow::Result<()> {
        // Clear all user data except device_id and tokens
        sqlx::query("DELETE FROM user_profile")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM relations")
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE relations_version SET version = NULL WHERE id = 1")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM conversations")
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE conversations_version SET version = NULL WHERE id = 1")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM messages")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM user_groups")
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE user_groups_version SET version = NULL WHERE id = 1")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM group_members")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM group_member_versions")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn get_user_profile(&self) -> anyhow::Result<Option<UserProfileData>> {
        let r: Option<(String, String, Option<String>)> =
            sqlx::query_as("SELECT user_id, nick_name, avatar FROM user_profile LIMIT 1")
                .fetch_optional(&self.pool)
                .await?;
        match r {
            Some((user_id, nick_name, avatar)) => Ok(Some(UserProfileData {
                user_id,
                nick_name,
                avatar,
            })),
            None => Ok(None),
        }
    }

    async fn save_user_profile(&self, profile: UserProfileData) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO user_profile (user_id, nick_name, avatar) VALUES (?, ?, ?)",
        )
        .bind(&profile.user_id)
        .bind(&profile.nick_name)
        .bind(&profile.avatar)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn apply_relation_all(
        &self,
        relations: Vec<RelationUser>,
        last_version: &str,
    ) -> anyhow::Result<()> {
        // Clear existing relations and insert new ones
        sqlx::query("DELETE FROM relations")
            .execute(&self.pool)
            .await?;

        for relation in relations {
            sqlx::query(
                "INSERT INTO relations (user_id, nick_name, avatar, remark_name, relation_flags) VALUES (?, ?, ?, ?, ?)",
            )
            .bind(&relation.user_id)
            .bind(&relation.nick_name)
            .bind(&relation.avatar)
            .bind(&relation.remark_name)
            .bind(relation.relation_flags)
            .execute(&self.pool)
            .await?;
        }

        // Update version
        sqlx::query("UPDATE relations_version SET version = ? WHERE id = 1")
            .bind(last_version)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn apply_relation_action(
        &self,
        action: RelationStorageAction,
        version: String,
        need_result: bool,
    ) -> anyhow::Result<Option<RelationUser>> {
        // Update version
        sqlx::query("UPDATE relations_version SET version = ? WHERE id = 1")
            .bind(&version)
            .execute(&self.pool)
            .await?;

        match action {
            RelationStorageAction::Upsert(relation) => {
                sqlx::query(
                    "INSERT OR REPLACE INTO relations (user_id, nick_name, avatar, remark_name, relation_flags) VALUES (?, ?, ?, ?, ?)",
                )
                .bind(&relation.user_id)
                .bind(&relation.nick_name)
                .bind(&relation.avatar)
                .bind(&relation.remark_name)
                .bind(relation.relation_flags)
                .execute(&self.pool)
                .await?;
                if need_result {
                    Ok(Some(relation))
                } else {
                    Ok(None)
                }
            }
            RelationStorageAction::UpdateRemarkName {
                user_id,
                remark_name,
            } => {
                sqlx::query("UPDATE relations SET remark_name = ? WHERE user_id = ?")
                    .bind(&remark_name)
                    .bind(&user_id)
                    .execute(&self.pool)
                    .await?;
                if need_result {
                    self.get_relation(&user_id).await
                } else {
                    Ok(None)
                }
            }
            RelationStorageAction::UpdateNickName { user_id, nick_name } => {
                sqlx::query("UPDATE relations SET nick_name = ? WHERE user_id = ?")
                    .bind(&nick_name)
                    .bind(&user_id)
                    .execute(&self.pool)
                    .await?;
                if need_result {
                    self.get_relation(&user_id).await
                } else {
                    Ok(None)
                }
            }
            RelationStorageAction::UpdateAvatar { user_id, avatar } => {
                sqlx::query("UPDATE relations SET avatar = ? WHERE user_id = ?")
                    .bind(&avatar)
                    .bind(&user_id)
                    .execute(&self.pool)
                    .await?;
                if need_result {
                    self.get_relation(&user_id).await
                } else {
                    Ok(None)
                }
            }
            RelationStorageAction::UpdateFlags { user_id, flags } => {
                sqlx::query("UPDATE relations SET relation_flags = ? WHERE user_id = ?")
                    .bind(flags)
                    .bind(&user_id)
                    .execute(&self.pool)
                    .await?;
                if need_result {
                    self.get_relation(&user_id).await
                } else {
                    Ok(None)
                }
            }
            RelationStorageAction::Delete { user_id } => {
                sqlx::query("DELETE FROM relations WHERE user_id = ?")
                    .bind(&user_id)
                    .execute(&self.pool)
                    .await?;
                Ok(None)
            }
            RelationStorageAction::UpdateNickNameAvatar {
                user_id,
                nick_name,
                avatar,
            } => {
                sqlx::query("UPDATE relations SET nick_name = ?, avatar = ? WHERE user_id = ?")
                    .bind(&nick_name)
                    .bind(&avatar)
                    .bind(&user_id)
                    .execute(&self.pool)
                    .await?;
                if need_result {
                    self.get_relation(&user_id).await
                } else {
                    Ok(None)
                }
            }
        }
    }

    async fn get_relation(&self, user_id: &str) -> anyhow::Result<Option<RelationUser>> {
        let r: Option<(String, String, Option<String>, Option<String>, i32)> = sqlx::query_as(
            "SELECT user_id, nick_name, avatar, remark_name, relation_flags FROM relations WHERE user_id = ?",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match r {
            Some((user_id, nick_name, avatar, remark_name, relation_flags)) => {
                Ok(Some(RelationUser {
                    user_id,
                    nick_name,
                    avatar,
                    remark_name,
                    relation_flags,
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_all_relations(&self) -> anyhow::Result<Vec<RelationUser>> {
        let rows: Vec<(String, String, Option<String>, Option<String>, i32)> = sqlx::query_as(
            "SELECT user_id, nick_name, avatar, remark_name, relation_flags FROM relations",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(user_id, nick_name, avatar, remark_name, relation_flags)| RelationUser {
                    user_id,
                    nick_name,
                    avatar,
                    remark_name,
                    relation_flags,
                },
            )
            .collect())
    }

    async fn get_relation_version(&self) -> anyhow::Result<Option<String>> {
        let r: (Option<String>,) =
            sqlx::query_as("SELECT version FROM relations_version WHERE id = 1")
                .fetch_one(&self.pool)
                .await?;
        Ok(r.0)
    }

    async fn clear_all_relations(&self) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM relations")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn apply_conversation_all(
        &self,
        conversations: Vec<ConversationRecord>,
        last_version: &str,
    ) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM conversations")
            .execute(&self.pool)
            .await?;

        for conv in conversations {
            sqlx::query(
                "INSERT INTO conversations (conversation_id, peer_id, group_id, last_message_id, last_read_message_id, unread_count, last_message_text, last_message_timestamp, name, avatar) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&conv.conversation_id)
            .bind(&conv.peer_id)
            .bind(&conv.group_id)
            .bind(&conv.last_message_id)
            .bind(&conv.last_read_message_id)
            .bind(conv.unread_count)
            .bind(&conv.last_message_text)
            .bind(conv.last_message_timestamp)
            .bind(&conv.name)
            .bind(&conv.avatar)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query("UPDATE conversations_version SET version = ? WHERE id = 1")
            .bind(last_version)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn apply_conversation_action(
        &self,
        action: ConversationStorageAction,
        version: String,
        need_result: bool,
    ) -> anyhow::Result<Option<ConversationRecord>> {
        sqlx::query("UPDATE conversations_version SET version = ? WHERE id = 1")
            .bind(&version)
            .execute(&self.pool)
            .await?;

        match action {
            ConversationStorageAction::Create(conv) => {
                sqlx::query(
                    "INSERT OR REPLACE INTO conversations (conversation_id, peer_id, group_id, last_message_id, last_read_message_id, unread_count, last_message_text, last_message_timestamp, name, avatar) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&conv.conversation_id)
                .bind(&conv.peer_id)
                .bind(&conv.group_id)
                .bind(&conv.last_message_id)
                .bind(&conv.last_read_message_id)
                .bind(conv.unread_count)
                .bind(&conv.last_message_text)
                .bind(conv.last_message_timestamp)
                .bind(&conv.name)
                .bind(&conv.avatar)
                .execute(&self.pool)
                .await?;
                if need_result {
                    Ok(Some(conv))
                } else {
                    Ok(None)
                }
            }
            ConversationStorageAction::UpdateLastReadMessageId {
                conversation_id,
                last_read_message_id,
            } => {
                sqlx::query(
                    "UPDATE conversations SET last_read_message_id = ? WHERE conversation_id = ?",
                )
                .bind(&last_read_message_id)
                .bind(&conversation_id)
                .execute(&self.pool)
                .await?;
                if need_result {
                    self.get_conversation_by_id(&conversation_id).await
                } else {
                    Ok(None)
                }
            }
            ConversationStorageAction::UpdateName {
                conversation_id,
                name,
            } => {
                sqlx::query("UPDATE conversations SET name = ? WHERE conversation_id = ?")
                    .bind(&name)
                    .bind(&conversation_id)
                    .execute(&self.pool)
                    .await?;
                if need_result {
                    self.get_conversation_by_id(&conversation_id).await
                } else {
                    Ok(None)
                }
            }
            ConversationStorageAction::UpdateAvatar {
                conversation_id,
                avatar,
            } => {
                sqlx::query("UPDATE conversations SET avatar = ? WHERE conversation_id = ?")
                    .bind(&avatar)
                    .bind(&conversation_id)
                    .execute(&self.pool)
                    .await?;
                if need_result {
                    self.get_conversation_by_id(&conversation_id).await
                } else {
                    Ok(None)
                }
            }
            ConversationStorageAction::UpdateNameAvatar {
                conversation_id,
                name,
                avatar,
            } => {
                sqlx::query(
                    "UPDATE conversations SET name = ?, avatar = ? WHERE conversation_id = ?",
                )
                .bind(&name)
                .bind(&avatar)
                .bind(&conversation_id)
                .execute(&self.pool)
                .await?;
                if need_result {
                    self.get_conversation_by_id(&conversation_id).await
                } else {
                    Ok(None)
                }
            }
            ConversationStorageAction::Delete { conversation_id } => {
                let removed = if need_result {
                    self.get_conversation_by_id(&conversation_id).await?
                } else {
                    None
                };
                sqlx::query("DELETE FROM conversations WHERE conversation_id = ?")
                    .bind(&conversation_id)
                    .execute(&self.pool)
                    .await?;
                Ok(removed)
            }
        }
    }

    async fn conversation_exists(&self, conversation_id: &str) -> anyhow::Result<bool> {
        let r: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM conversations WHERE conversation_id = ?")
                .bind(conversation_id)
                .fetch_one(&self.pool)
                .await?;
        Ok(r.0 > 0)
    }

    async fn get_conversation_by_id(
        &self,
        conversation_id: &str,
    ) -> anyhow::Result<Option<ConversationRecord>> {
        let r: Option<(
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            i64,
            Option<String>,
            Option<i64>,
            String,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT conversation_id, peer_id, group_id, last_message_id, last_read_message_id, unread_count, last_message_text, last_message_timestamp, name, avatar FROM conversations WHERE conversation_id = ?",
        )
        .bind(conversation_id)
        .fetch_optional(&self.pool)
        .await?;

        match r {
            Some((
                conversation_id,
                peer_id,
                group_id,
                last_message_id,
                last_read_message_id,
                unread_count,
                last_message_text,
                last_message_timestamp,
                name,
                avatar,
            )) => Ok(Some(ConversationRecord {
                conversation_id,
                peer_id,
                group_id,
                last_message_id,
                last_read_message_id,
                unread_count,
                last_message_text,
                last_message_timestamp,
                name,
                avatar,
            })),
            None => Ok(None),
        }
    }

    async fn get_all_conversations(&self) -> anyhow::Result<Vec<ConversationRecord>> {
        let rows: Vec<(
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            i64,
            Option<String>,
            Option<i64>,
            String,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT conversation_id, peer_id, group_id, last_message_id, last_read_message_id, unread_count, last_message_text, last_message_timestamp, name, avatar FROM conversations",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    conversation_id,
                    peer_id,
                    group_id,
                    last_message_id,
                    last_read_message_id,
                    unread_count,
                    last_message_text,
                    last_message_timestamp,
                    name,
                    avatar,
                )| ConversationRecord {
                    conversation_id,
                    peer_id,
                    group_id,
                    last_message_id,
                    last_read_message_id,
                    unread_count,
                    last_message_text,
                    last_message_timestamp,
                    name,
                    avatar,
                },
            )
            .collect())
    }

    async fn get_conversation_version(&self) -> anyhow::Result<Option<String>> {
        let r: (Option<String>,) =
            sqlx::query_as("SELECT version FROM conversations_version WHERE id = 1")
                .fetch_one(&self.pool)
                .await?;
        Ok(r.0)
    }

    async fn clear_all_conversations(&self) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM conversations")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_conversation_summary(
        &self,
        conversation_id: &str,
        unread_count: i64,
        last_message_id: Option<String>,
        last_message_text: Option<String>,
        last_message_timestamp: Option<i64>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE conversations SET unread_count = ?, last_message_id = ?, last_message_text = ?, last_message_timestamp = ? WHERE conversation_id = ?",
        )
        .bind(unread_count)
        .bind(&last_message_id)
        .bind(&last_message_text)
        .bind(last_message_timestamp)
        .bind(conversation_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn store_message(&self, message: MessageItem) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO messages (message_id, conversation_id, sender_id, receiver_id, group_id, send_timestamp, message_type, text, file_url, file_name, command_type, command_data) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&message.message_id)
        .bind(&message.conversation_id)
        .bind(&message.sender_id)
        .bind(&message.receiver_id)
        .bind(&message.group_id)
        .bind(&message.send_timestamp)
        .bind(i32::from(message.message_type))
        .bind(&message.text)
        .bind(&message.file_url)
        .bind(&message.file_name)
        .bind(i32::from(message.command_type))
        .bind(&message.command_data)
        .execute(&self.pool)
        .await?;

        // Update conversation's last_message_id if this message is newer
        let conv = self.get_conversation_by_id(&message.conversation_id).await?;
        if let Some(conv) = conv {
            let should_update = match &conv.last_message_id {
                Some(existing_id) => message.message_id > *existing_id,
                None => true,
            };
            if should_update {
                println!(
                    "[SqliteStore] store_message: updating last_message_id from {:?} to {}",
                    conv.last_message_id, message.message_id
                );
                sqlx::query("UPDATE conversations SET last_message_id = ? WHERE conversation_id = ?")
                    .bind(&message.message_id)
                    .bind(&message.conversation_id)
                    .execute(&self.pool)
                    .await?;
            }
        } else {
            println!(
                "[SqliteStore] store_message: conversation {} not found in store",
                message.conversation_id
            );
        }

        Ok(())
    }

    async fn get_latest_message(
        &self,
        conversation_id: &str,
    ) -> anyhow::Result<Option<MessageItem>> {
        let r: Option<(
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            i32,
            Option<String>,
            Option<String>,
            Option<String>,
            i32,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT message_id, conversation_id, sender_id, receiver_id, group_id, send_timestamp, message_type, text, file_url, file_name, command_type, command_data FROM messages WHERE conversation_id = ? ORDER BY message_id DESC LIMIT 1",
        )
        .bind(conversation_id)
        .fetch_optional(&self.pool)
        .await?;

        match r {
            Some((
                message_id,
                conversation_id,
                sender_id,
                receiver_id,
                group_id,
                send_timestamp,
                message_type,
                text,
                file_url,
                file_name,
                command_type,
                command_data,
            )) => Ok(Some(MessageItem {
                message_id,
                conversation_id,
                sender_id,
                receiver_id,
                group_id,
                send_timestamp,
                message_type: MessageItemType::from(message_type),
                text,
                file_url,
                file_name,
                command_type: MessageCommandType::from(command_type),
                command_data,
            })),
            None => Ok(None),
        }
    }

    async fn get_latest_messages(
        &self,
        conversation_id: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<MessageItem>> {
        // Get in descending order then reverse to get ascending order
        let rows: Vec<(
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            i32,
            Option<String>,
            Option<String>,
            Option<String>,
            i32,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT message_id, conversation_id, sender_id, receiver_id, group_id, send_timestamp, message_type, text, file_url, file_name, command_type, command_data FROM messages WHERE conversation_id = ? ORDER BY message_id DESC LIMIT ?",
        )
        .bind(conversation_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut result: Vec<MessageItem> = rows
            .into_iter()
            .map(
                |(
                    message_id,
                    conversation_id,
                    sender_id,
                    receiver_id,
                    group_id,
                    send_timestamp,
                    message_type,
                    text,
                    file_url,
                    file_name,
                    command_type,
                    command_data,
                )| MessageItem {
                    message_id,
                    conversation_id,
                    sender_id,
                    receiver_id,
                    group_id,
                    send_timestamp,
                    message_type: MessageItemType::from(message_type),
                    text,
                    file_url,
                    file_name,
                    command_type: MessageCommandType::from(command_type),
                    command_data,
                },
            )
            .collect();

        result.reverse();
        println!(
            "[SqliteStore] Returning {} latest messages",
            result.len()
        );
        Ok(result)
    }

    async fn get_messages_before(
        &self,
        conversation_id: &str,
        before_message_id: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<MessageItem>> {
        let rows: Vec<(
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            i32,
            Option<String>,
            Option<String>,
            Option<String>,
            i32,
            Option<String>,
        )> = sqlx::query_as(
            "SELECT message_id, conversation_id, sender_id, receiver_id, group_id, send_timestamp, message_type, text, file_url, file_name, command_type, command_data FROM messages WHERE conversation_id = ? AND message_id < ? ORDER BY message_id ASC LIMIT ?",
        )
        .bind(conversation_id)
        .bind(before_message_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(
                    message_id,
                    conversation_id,
                    sender_id,
                    receiver_id,
                    group_id,
                    send_timestamp,
                    message_type,
                    text,
                    file_url,
                    file_name,
                    command_type,
                    command_data,
                )| MessageItem {
                    message_id,
                    conversation_id,
                    sender_id,
                    receiver_id,
                    group_id,
                    send_timestamp,
                    message_type: MessageItemType::from(message_type),
                    text,
                    file_url,
                    file_name,
                    command_type: MessageCommandType::from(command_type),
                    command_data,
                },
            )
            .collect())
    }

    async fn exist_user_groups(&self) -> anyhow::Result<bool> {
        let r: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_groups")
            .fetch_one(&self.pool)
            .await?;
        Ok(r.0 > 0)
    }

    async fn apply_user_group_all(
        &self,
        groups: Vec<UserGroupData>,
        version: &str,
    ) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM user_groups")
            .execute(&self.pool)
            .await?;

        for group in groups {
            sqlx::query(
                "INSERT INTO user_groups (group_id, group_name, group_avatar) VALUES (?, ?, ?)",
            )
            .bind(&group.group_id)
            .bind(&group.group_name)
            .bind(&group.group_avatar)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query("UPDATE user_groups_version SET version = ? WHERE id = 1")
            .bind(version)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn apply_user_group_action(
        &self,
        action: UserGroupStorageAction,
        version: String,
        need_result: bool,
    ) -> anyhow::Result<Option<UserGroupData>> {
        sqlx::query("UPDATE user_groups_version SET version = ? WHERE id = 1")
            .bind(&version)
            .execute(&self.pool)
            .await?;

        match action {
            UserGroupStorageAction::Upsert(group) => {
                sqlx::query(
                    "INSERT OR REPLACE INTO user_groups (group_id, group_name, group_avatar) VALUES (?, ?, ?)",
                )
                .bind(&group.group_id)
                .bind(&group.group_name)
                .bind(&group.group_avatar)
                .execute(&self.pool)
                .await?;
                if need_result {
                    Ok(Some(group))
                } else {
                    Ok(None)
                }
            }
            UserGroupStorageAction::UpdateName { group_id, name } => {
                sqlx::query("UPDATE user_groups SET group_name = ? WHERE group_id = ?")
                    .bind(&name)
                    .bind(&group_id)
                    .execute(&self.pool)
                    .await?;
                if need_result {
                    let r: Option<(String, String, Option<String>)> = sqlx::query_as(
                        "SELECT group_id, group_name, group_avatar FROM user_groups WHERE group_id = ?",
                    )
                    .bind(&group_id)
                    .fetch_optional(&self.pool)
                    .await?;
                    Ok(r.map(|(group_id, group_name, group_avatar)| UserGroupData {
                        group_id,
                        group_name,
                        group_avatar,
                    }))
                } else {
                    Ok(None)
                }
            }
            UserGroupStorageAction::UpdateAvatar { group_id, avatar } => {
                sqlx::query("UPDATE user_groups SET group_avatar = ? WHERE group_id = ?")
                    .bind(&avatar)
                    .bind(&group_id)
                    .execute(&self.pool)
                    .await?;
                if need_result {
                    let r: Option<(String, String, Option<String>)> = sqlx::query_as(
                        "SELECT group_id, group_name, group_avatar FROM user_groups WHERE group_id = ?",
                    )
                    .bind(&group_id)
                    .fetch_optional(&self.pool)
                    .await?;
                    Ok(r.map(|(group_id, group_name, group_avatar)| UserGroupData {
                        group_id,
                        group_name,
                        group_avatar,
                    }))
                } else {
                    Ok(None)
                }
            }
            UserGroupStorageAction::Delete { group_id } => {
                sqlx::query("DELETE FROM user_groups WHERE group_id = ?")
                    .bind(&group_id)
                    .execute(&self.pool)
                    .await?;
                // Also remove group members when user quits a group
                sqlx::query("DELETE FROM group_members WHERE group_id = ?")
                    .bind(&group_id)
                    .execute(&self.pool)
                    .await?;
                sqlx::query("DELETE FROM group_member_versions WHERE group_id = ?")
                    .bind(&group_id)
                    .execute(&self.pool)
                    .await?;
                Ok(None)
            }
        }
    }

    async fn get_all_user_groups(&self) -> anyhow::Result<Vec<UserGroupData>> {
        let rows: Vec<(String, String, Option<String>)> =
            sqlx::query_as("SELECT group_id, group_name, group_avatar FROM user_groups")
                .fetch_all(&self.pool)
                .await?;

        Ok(rows
            .into_iter()
            .map(|(group_id, group_name, group_avatar)| UserGroupData {
                group_id,
                group_name,
                group_avatar,
            })
            .collect())
    }

    async fn get_user_group_version(&self) -> anyhow::Result<Option<String>> {
        let r: (Option<String>,) =
            sqlx::query_as("SELECT version FROM user_groups_version WHERE id = 1")
                .fetch_one(&self.pool)
                .await?;
        Ok(r.0)
    }

    async fn clear_all_user_groups(&self) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM user_groups")
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE user_groups_version SET version = NULL WHERE id = 1")
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn exist_group_members(&self, group_id: &str) -> anyhow::Result<bool> {
        let r: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM group_members WHERE group_id = ?")
            .bind(group_id)
            .fetch_one(&self.pool)
            .await?;
        Ok(r.0 > 0)
    }

    async fn apply_group_member_all(
        &self,
        group_id: &str,
        members: Vec<GroupMemberData>,
        version: &str,
    ) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM group_members WHERE group_id = ?")
            .bind(group_id)
            .execute(&self.pool)
            .await?;

        for member in members {
            sqlx::query(
                "INSERT INTO group_members (group_id, user_id, name, avatar) VALUES (?, ?, ?, ?)",
            )
            .bind(group_id)
            .bind(&member.user_id)
            .bind(&member.name)
            .bind(&member.avatar)
            .execute(&self.pool)
            .await?;
        }

        sqlx::query(
            "INSERT OR REPLACE INTO group_member_versions (group_id, version) VALUES (?, ?)",
        )
        .bind(group_id)
        .bind(version)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn apply_group_member_action(
        &self,
        group_id: &str,
        action: GroupMemberStorageAction,
        version: String,
        need_result: bool,
    ) -> anyhow::Result<Option<GroupMemberData>> {
        sqlx::query(
            "INSERT OR REPLACE INTO group_member_versions (group_id, version) VALUES (?, ?)",
        )
        .bind(group_id)
        .bind(&version)
        .execute(&self.pool)
        .await?;

        match action {
            GroupMemberStorageAction::Upsert(member) => {
                sqlx::query(
                    "INSERT OR REPLACE INTO group_members (group_id, user_id, name, avatar) VALUES (?, ?, ?, ?)",
                )
                .bind(group_id)
                .bind(&member.user_id)
                .bind(&member.name)
                .bind(&member.avatar)
                .execute(&self.pool)
                .await?;
                if need_result {
                    Ok(Some(member))
                } else {
                    Ok(None)
                }
            }
            GroupMemberStorageAction::UpdateName { user_id, name } => {
                sqlx::query(
                    "UPDATE group_members SET name = ? WHERE group_id = ? AND user_id = ?",
                )
                .bind(&name)
                .bind(group_id)
                .bind(&user_id)
                .execute(&self.pool)
                .await?;
                if need_result {
                    self.get_group_member(group_id, &user_id).await
                } else {
                    Ok(None)
                }
            }
            GroupMemberStorageAction::UpdateAvatar { user_id, avatar } => {
                sqlx::query(
                    "UPDATE group_members SET avatar = ? WHERE group_id = ? AND user_id = ?",
                )
                .bind(&avatar)
                .bind(group_id)
                .bind(&user_id)
                .execute(&self.pool)
                .await?;
                if need_result {
                    self.get_group_member(group_id, &user_id).await
                } else {
                    Ok(None)
                }
            }
            GroupMemberStorageAction::Delete { user_id } => {
                sqlx::query("DELETE FROM group_members WHERE group_id = ? AND user_id = ?")
                    .bind(group_id)
                    .bind(&user_id)
                    .execute(&self.pool)
                    .await?;
                Ok(None)
            }
        }
    }

    async fn get_all_group_members(&self, group_id: &str) -> anyhow::Result<Vec<GroupMemberData>> {
        let rows: Vec<(String, String, Option<String>)> =
            sqlx::query_as("SELECT user_id, name, avatar FROM group_members WHERE group_id = ?")
                .bind(group_id)
                .fetch_all(&self.pool)
                .await?;

        Ok(rows
            .into_iter()
            .map(|(user_id, name, avatar)| GroupMemberData {
                user_id,
                name,
                avatar,
            })
            .collect())
    }

    async fn get_group_member(
        &self,
        group_id: &str,
        user_id: &str,
    ) -> anyhow::Result<Option<GroupMemberData>> {
        let r: Option<(String, String, Option<String>)> = sqlx::query_as(
            "SELECT user_id, name, avatar FROM group_members WHERE group_id = ? AND user_id = ?",
        )
        .bind(group_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(r.map(|(user_id, name, avatar)| GroupMemberData {
            user_id,
            name,
            avatar,
        }))
    }

    async fn get_group_member_version(&self, group_id: &str) -> anyhow::Result<Option<String>> {
        let r: Option<(String,)> =
            sqlx::query_as("SELECT version FROM group_member_versions WHERE group_id = ?")
                .bind(group_id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(r.map(|(v,)| v))
    }

    async fn clear_group_members(&self, group_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM group_members WHERE group_id = ?")
            .bind(group_id)
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM group_member_versions WHERE group_id = ?")
            .bind(group_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
