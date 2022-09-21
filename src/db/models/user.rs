use crate::{
    db::{
        get_conn,
        schema::users::{self, dsl},
    },
    error::{Error, Result},
    token::UserToken,
    DbPool,
};
use actix_web::web::{block, Data};
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Clone, Queryable, Insertable)]
#[diesel(table_name = users)]
pub struct UserDAO {
    pub id: u64,
    pub username: String,
    pub name: String,
    //password_hash: Binary
    pub session_id: Option<String>,

    pub auto_apply: bool,
    pub is_teacher: bool,
    pub created_at: NaiveDateTime,
}

impl UserDAO {
    pub async fn login(pool: Data<DbPool>, user_id: String) -> Result<UserDAO> {
        let mut user = Self::by_username(pool.clone(), user_id).await?;

        if let Some(sid) = user.session_id {
            Err(Error::AlreadyLoggedIn(sid))
        } else {
            user.get_new_session(pool).await?;
            Ok(user)
        }
    }

    pub async fn get_new_session(&mut self, pool: Data<DbPool>) -> Result<String> {
        self.update_session(pool, UserToken::new_session_id())
            .await?;
        Ok(self.session_id.clone().unwrap())
    }

    pub async fn logout(&mut self, pool: Data<DbPool>) -> Result<()> {
        self.update_session(pool, String::new()).await
    }

    pub async fn by_id(pool: Data<DbPool>, id: u64) -> Result<Self> {
        let mut conn = get_conn(pool).await;
        block(move || {
            dsl::users
                .filter(dsl::id.eq(id))
                .first::<UserDAO>(&mut conn)
        })
        .await?
        .map_err(Error::not_found_on_db)
    }

    pub async fn by_username<T>(pool: Data<DbPool>, username: T) -> Result<Self>
    where
        T: Into<String>,
    {
        let user_id: String = username.into();
        let mut conn = get_conn(pool).await;
        block(move || {
            dsl::users
                .filter(dsl::username.eq(user_id))
                .first::<UserDAO>(&mut conn)
        })
        .await?
        .map_err(Error::not_found_on_db)
    }

    pub async fn by_session_id<T>(pool: Data<DbPool>, session_id: T) -> Result<Self>
    where
        T: Into<String>,
    {
        let session_id: String = session_id.into();
        let mut conn = get_conn(pool).await;
        block(move || {
            dsl::users
                .filter(dsl::session_id.eq(session_id))
                .first::<UserDAO>(&mut conn)
        })
        .await?
        .map_err(Error::no_such_session)
    }

    pub async fn update_session<T>(&mut self, pool: Data<DbPool>, session_id: T) -> Result<()>
    where
        T: Into<String>,
    {
        let session_id: String = session_id.into();
        let mut conn = get_conn(pool).await;

        self.session_id = if session_id.is_empty() {
            None
        } else {
            Some(session_id.clone())
        };

        let id = self.id;
        let sid = self.session_id.clone();
        block(move || {
            diesel::update(dsl::users.find(id))
                .set(dsl::session_id.eq(&sid))
                .execute(&mut conn)
        })
        .await??;

        Ok(())
    }
}
