#![allow(unused)]

pub(crate) mod models;
pub(crate) mod schema;

use crate::{db::models::user::UserDAO, error::Error, token::UserToken, Result};
use actix_web::web::{block, Data};

pub use diesel::prelude::*;
use diesel::{
    r2d2::{self, ConnectionManager, PooledConnection},
    MysqlConnection,
};

pub type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

pub async fn get_conn(pool: Data<DbPool>) -> PooledConnection<ConnectionManager<MysqlConnection>> {
    block(move || pool.get())
        .await
        .expect("Couldn't block function execution")
        .expect("Couldn't get DB connection")
}

/*pub async fn is_valid_token(pool: Data<DbPool>, user_token: UserToken) -> bool {
    use schema::users::dsl;

    let mut conn = get_conn(pool).await;
    block(move || {
        dsl::users
            .filter(dsl::username.eq(user_token.user))
            .filter(dsl::session_id.eq(Some(user_token.session)))
            .first::<User>(&mut conn)
    })
    .await
    .unwrap()
    .is_ok()
}*/
