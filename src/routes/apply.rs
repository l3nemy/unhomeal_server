use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::{
    db::{
        models::{
            application::{Application, ApplicationDAO},
            user::UserDAO,
        },
        DbPool,
    },
    error::Error,
};

#[derive(Clone, Deserialize, Serialize)]
pub struct ApplyParam {
    pub session_id: String,
    pub apply: bool,
    pub send_date: NaiveDate,
}

#[derive(Serialize)]
pub struct ApplyResponse {
    is_error: bool,
}

#[post("/apply")]
pub async fn apply_route(pool: Data<DbPool>, param: Json<ApplyParam>) -> Result<HttpResponse> {
    //check session_id
    UserDAO::by_session_id(pool.clone(), &param.session_id).await?;

    ApplicationDAO::post(pool, param.0.clone()).await?;

    Ok(HttpResponse::Accepted().json(ApplyResponse { is_error: false }))
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GetApplicationParam {
    pub session_id: String,
}

#[derive(Serialize)]
pub struct GetApplicationResponse {
    is_error: bool,
    applications: Vec<Application>,
}

#[post("/applications")]
pub async fn get_applications_route(
    pool: Data<DbPool>,
    param: Json<GetApplicationParam>,
) -> Result<HttpResponse> {
    //check session_id
    let user = UserDAO::by_session_id(pool.clone(), &param.session_id).await?;

    if user.is_teacher {
        let applications = ApplicationDAO::get(pool.clone(), param.0.clone()).await?;

        let mut apps = Vec::new();
        for app in applications {
            apps.push(Application::from_application_dto(pool.clone(), app).await?)
        }

        Ok(HttpResponse::Accepted().json(GetApplicationResponse {
            is_error: false,
            applications: apps,
        }))
    } else {
        Err(Error::Unprivileged)
    }
}

#[derive(Clone, Deserialize)]
pub struct HasAppliedParam {
    session_id: String,
}

#[derive(Serialize)]
struct HasAppliedResponse {
    is_error: bool,
    applied: bool,
}

#[post("/has_applied")]
pub async fn has_applied_route(
    pool: Data<DbPool>,
    param: Json<HasAppliedParam>,
) -> Result<HttpResponse> {
    Ok(HttpResponse::Accepted().json(HasAppliedResponse {
        is_error: false,
        applied: ApplicationDAO::get_one(pool, param.session_id.clone())
            .await
            .is_err(),
    }))
}
