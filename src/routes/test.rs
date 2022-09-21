use actix_web::{get, web::Data};

use crate::{
    db::{models::user::UserDAO, DbPool},
    error::Result,
};

#[get("/test")]
pub async fn test_route(pool: Data<DbPool>) -> Result<String> {
    let mut user = UserDAO::by_username(pool.clone(), "test").await?;

    if user.session_id.is_none() {
        user.get_new_session(pool).await?;
    }

    Ok(format!(
        "Hi! {}. Your session_id is {}",
        user.username,
        user.session_id.unwrap()
    ))
}
