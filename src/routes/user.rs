use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{
    db::{models::user::UserDAO, DbPool},
    error::{Result, Error},
};

#[derive(Clone, Deserialize)]
pub struct UserParam {
    session_id: String,
    username: String,
}

#[derive(Serialize)]
struct UserResponse {
    is_error: bool,
    user: User,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct User {
    username: String,
    name: String,
    auto_apply: bool,
    is_teacher: bool,
    created_at: NaiveDateTime,
}

impl From<UserDAO> for User {
    fn from(u: UserDAO) -> Self {
        Self {
            username: u.username.clone(),
            name: u.name.clone(),
            auto_apply: u.auto_apply,
            is_teacher: u.is_teacher,
            created_at: u.created_at,
        }
    }
}

#[post("/user")]
pub async fn user_route(pool: Data<DbPool>, param: Json<UserParam>) -> Result<HttpResponse> {
    let user = UserDAO::by_session_id(pool.clone(), &param.session_id).await?;
    if user.is_teacher || user.username == param.username {
        Ok(HttpResponse::Accepted().json(UserResponse {
            is_error: false,
            user: user.into(),
        }))
    } else {
        Err(Error::Unprivileged)
    }   
}
