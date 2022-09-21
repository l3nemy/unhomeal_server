use std::collections::HashMap;

use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse,
};
use chrono::{NaiveDate, NaiveDateTime};
use diesel::Queryable;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    db::{
        models::{rate::RateDAO, total_rate::TotalRateDAO, user::UserDAO},
        DbPool,
    },
    error::Result,
};

#[derive(Clone, Deserialize, Serialize)]
pub struct RateReq {
    pub food_name: String,
    pub level: RateLevel,
}

#[derive(Clone, Deserialize_repr, Serialize_repr)]
#[repr(i8)]
pub enum RateLevel {
    Bad = -1,
    Soso,
    Good,
}

impl From<i8> for RateLevel {
    fn from(level: i8) -> Self {
        match level {
            -1 => Self::Bad,
            0 => Self::Soso,
            1 => Self::Good,
            _ => unimplemented!(),
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct RateParam {
    pub session_id: String,
    pub rates: Vec<RateReq>,
    pub total_rate: u8, // 0~10
    pub send_date: NaiveDate,
}

#[derive(Clone, Serialize)]
pub struct RateResponse {
    is_error: bool,
}

#[post("/post_rate")]
pub async fn post_rate_route(pool: Data<DbPool>, param: Json<RateParam>) -> Result<HttpResponse> {
    RateDAO::post(pool.clone(), param.0.clone()).await?;
    TotalRateDAO::post(pool, param.0.clone()).await?;
    //TODO : UPDATE RATE IF EXISTS

    Ok(HttpResponse::Accepted().json(RateResponse { is_error: false }))
}

#[derive(Clone, Deserialize)]
pub struct GetRateParam {
    session_id: String,
    food_name: Option<String>,
    date: Option<NaiveDate>,
}

#[derive(Clone, Serialize)]
pub struct GetRateResponse {
    is_error: bool,
    rates: Vec<Rate>,
    total_avg_rate: f32,
}

#[derive(Clone, Deserialize, Serialize, Queryable)]
pub struct Rate {
    pub username: String,
    pub food_name: String,
    pub rate_level: RateLevel,
    pub created_at: NaiveDateTime,
}

#[post("/get_rate")]
pub async fn get_rate_route(pool: Data<DbPool>, param: Json<GetRateParam>) -> Result<HttpResponse> {
    //checking session_id
    UserDAO::by_session_id(pool.clone(), &param.session_id).await?;

    let rates = if let Some(date) = param.date {
        RateDAO::get(pool.clone(), date).await?
    } else {
        RateDAO::get_today(pool.clone()).await?
    };

    let rates = if let Some(food_name) = param.food_name.clone() {
        rates
            .iter()
            .filter(|r| r.food_name == food_name)
            .map(|r| r.to_owned())
            .collect::<Vec<_>>()
    } else {
        rates
    };

    let total_avg_rate = TotalRateDAO::avg_today(pool).await?;

    Ok(HttpResponse::Accepted().json(GetRateResponse {
        is_error: false,
        rates,
        total_avg_rate,
    }))
}

#[derive(Deserialize)]
pub struct RankParam {
    session_id: String,
}

#[derive(Serialize)]
pub struct RankResponse {
    is_error: bool,
    rank: HashMap<String, i32>,
}

#[post("/rank")]
pub async fn rank(pool: Data<DbPool>, param: Json<RankParam>) -> Result<HttpResponse> {
    //checking session_id
    UserDAO::by_session_id(pool.clone(), &param.session_id).await?;

    let mut rank_hash: HashMap<String, i32> = HashMap::new();

    let rates = RateDAO::get_today(pool).await?;
    for r in rates {
        let entry = rank_hash.entry(r.food_name).or_insert(0);
        *entry += r.rate_level as i32;
    }

    Ok(HttpResponse::Accepted().json(RankResponse {
        is_error: false,
        rank: rank_hash,
    }))
}
