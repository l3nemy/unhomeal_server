// use super::{db::models::User, error::Result};
// use chrono::Utc;
// use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// pub static KEY: [u8; 16] = *include_bytes!("../secret.key");

#[derive(Serialize, Deserialize)]
pub struct UserToken {
    pub user: String,
    pub session: String,
    pub issued_at: i64,
    pub expire_at: i64,
}

// const ONE_MONTH: i64 = 60 * 60 * 24 * 30;
impl UserToken {
    pub fn new_session_id() -> String {
        Uuid::new_v4().as_simple().to_string()
    }

    //TODO: used in future
    /*pub fn generate(user: &User) -> Result<String> {
        if let Some(session_id) = user.session_id.clone() {
            let now = Utc::now().timestamp_nanos() / 1_000_000_000;
            let payload = UserToken {
                user: user.user_id.clone(),
                session: session_id,
                issued_at: now,
                expire_at: now + ONE_MONTH,
            };

            jsonwebtoken::encode(
                &Header::default(),
                &payload,
                &EncodingKey::from_secret(&KEY),
            )
            .map_err(Into::into)
        } else {
            unimplemented!()
        }
    }*/
}
