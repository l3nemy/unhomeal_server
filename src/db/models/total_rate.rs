use crate::{
    db::{
        get_conn,
        models::user::UserDAO,
        schema::total_rates::{self, dsl},
        DbPool,
    },
    error::{Error, Result},
    routes::{GetUserRatesParam, RateParam},
};
use actix_web::web::{block, Data};
use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use diesel::prelude::*;

#[derive(Clone, Queryable, Insertable)]
#[diesel(table_name = total_rates)]
pub struct TotalRateDAO {
    pub id: u64,
    pub user_id: u64,
    pub rate_level: u8,
    pub created_at: NaiveDateTime,
}

impl TotalRateDAO {
    pub async fn avg_today(pool: Data<DbPool>) -> Result<f32> {
        let mut conn = get_conn(pool).await;

        let today = Local::today().naive_local();
        let result = block(move || {
            dsl::total_rates
                .filter(dsl::created_at.ge(today.and_hms(0, 0, 0)))
                .filter(dsl::created_at.lt((today + Duration::days(1)).and_hms(0, 0, 0)))
                .load::<TotalRateDAO>(&mut conn)
        })
        .await?
        .map_err(Error::not_found_on_db)?;

        let rates = result.iter().map(|r| r.rate_level).collect::<Vec<u8>>();

        let mut total_avg: f32 = 0.0;
        for r in &rates {
            total_avg += *r as f32 / rates.len() as f32;
        }

        Ok(total_avg)
    }

    pub async fn get_one_today(pool: Data<DbPool>, param: GetUserRatesParam) -> Result<u8> {
        Self::get_one(pool, param, Local::today().naive_local()).await
    }

    pub async fn get_one(
        pool: Data<DbPool>,
        param: GetUserRatesParam,
        date: NaiveDate,
    ) -> Result<u8> {
        let user = UserDAO::by_session_id(pool.clone(), &param.session_id).await?;
        let target = UserDAO::by_username(pool.clone(), &param.username).await?;

        if user.is_teacher || user.id == target.id {
            let mut conn = get_conn(pool.clone()).await;
            block(move || {
                dsl::total_rates
                    .select(dsl::rate_level)
                    .filter(dsl::user_id.eq(&user.id))
                    .filter(dsl::created_at.ge(date.and_hms(0, 0, 0)))
                    .filter(dsl::created_at.lt((date + Duration::days(1)).and_hms(0, 0, 0)))
                    .first::<u8>(&mut conn)
            })
            .await?
            .map_err(Error::not_found_on_db)
        } else {
            Err(Error::Unprivileged)
        }
    }

    pub async fn post(pool: Data<DbPool>, rate_param: RateParam) -> Result<()> {
        let user = UserDAO::by_session_id(pool.clone(), rate_param.session_id.clone()).await?;

        let mut conn = get_conn(pool).await;
        block(move || {
            diesel::insert_into(dsl::total_rates)
                .values((
                    dsl::user_id.eq(&user.id),
                    dsl::rate_level.eq(rate_param.total_rate),
                ))
                .execute(&mut conn)
        })
        .await?
        .map_err(Into::into)
        .map(|_| ())
    }
}
