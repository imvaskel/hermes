use std::ops::Deref;

use axum_login::AuthUser;
use eos::Timestamp;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Timestamp does not impl From<i64>, so we need to wrap it to allow us to convert from database.
#[derive(Debug, Clone)]
pub struct DatabaseTimestamp(pub Timestamp);

impl TryFrom<i64> for DatabaseTimestamp {
    type Error = ();

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        Ok(Self(Timestamp::from_milliseconds(value)))
    }
}

impl Deref for DatabaseTimestamp {
    type Target = Timestamp;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    #[sqlx(rename = "user_id")]
    pub id: String,
    pub username: String,
    pub password_hash: String,
}

impl AuthUser for User {
    type Id = String;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.as_bytes()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, FromRow)]
pub struct Note {
    #[sqlx(rename = "note_id")]
    pub id: String,
    #[sqlx(rename = "note_name")]
    pub name: String,
    #[sqlx(rename = "note_owner", flatten)]
    pub owner: User,
}

#[derive(Debug, Clone, FromRow)]
pub struct Entry {
    #[sqlx(try_from = "i64")]
    pub created_at: DatabaseTimestamp,
    pub content: String,
    #[sqlx(rename = "entry_id")]
    pub id: String,
    #[sqlx(flatten)]
    pub parent: Note,
}
