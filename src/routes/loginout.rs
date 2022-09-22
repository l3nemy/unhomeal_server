use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use serde::{Deserialize, Serialize};

use crate::{
    db::{models::user::UserDAO, DbPool},
    error::Result,
};

#[derive(Deserialize)]
pub struct LoginParam {
    username: String,
    // password: String,
}

/// LoginResponse is used to send login result to the client    
/// * Response body which is serialized into JSON
/// * And sent to clients
#[derive(Serialize, Clone)]
struct LoginResponse {
    is_error: bool,
    session_id: String,
}

/// Login procedure
/// * Receives JSON request body as struct LoginParam
/// * Responds with JSON body with session_id
#[post("/login")]
pub async fn login_route(pool: Data<DbPool>, param: Json<LoginParam>) -> Result<HttpResponse> {
    let user = UserDAO::login(pool, param.username.clone()).await?;

    Ok(HttpResponse::Accepted().json(LoginResponse {
        is_error: false,
        session_id: user.session_id.unwrap().clone(),
    }))
}

#[derive(Deserialize)]
pub struct LogoutParam {
    username: String,
    session_id: String,
}

#[derive(Serialize)]
pub struct LogoutResponse {
    is_error: bool,
    username: String,
}

#[post("/logout")]
pub async fn logout_route(pool: Data<DbPool>, param: Json<LogoutParam>) -> Result<HttpResponse> {
    UserDAO::by_session_id(pool.clone(), &param.session_id)
        .await?
        .logout(pool)
        .await?;

    Ok(HttpResponse::Accepted().json(LogoutResponse {
        is_error: false,
        username: param.username.clone(),
    }))
}
