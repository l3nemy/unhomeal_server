use crate::{
    db::{
        get_conn,
        schema::{
            rates::{self, dsl},
            users,
        },
        DbPool, UserDAO,
    },
    error::{Error, Result},
    routes::{self, Rate, RateLevel, RateParam},
};
use actix_web::web::{block, Data};
use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use serde_json::Value;

#[derive(Clone, Queryable, Insertable)]
#[diesel(table_name = rates)]
pub struct RateDAO {
    pub id: u64,
    pub user_id: u64,
    pub food_name: String,
    pub rate_level: i8,
    pub created_at: NaiveDateTime,
}

impl RateDAO {
    pub async fn get_today(pool: Data<DbPool>) -> Result<Vec<Rate>> {
        Self::get(pool, Local::today().naive_local()).await
    }

    pub async fn get(pool: Data<DbPool>, date: NaiveDate) -> Result<Vec<Rate>> {
        let mut conn = get_conn(pool).await;

        let next_date = date + Duration::days(1);
        block(move || {
            let mut join = rates::table
                .left_join(users::table.on(users::id.eq(dsl::user_id)))
                .select((
                    users::username.assume_not_null(),
                    rates::food_name,
                    rates::rate_level,
                    rates::created_at,
                ));

            join.filter(rates::created_at.ge(date.and_hms(0, 0, 0)))
                .filter(rates::created_at.lt(next_date.and_hms(0, 0, 0)))
                .get_results::<(String, String, i8, NaiveDateTime)>(&mut conn)
        })
        .await?
        .map_err(Error::not_found_on_db)
        .map(|v| {
            v.iter()
                .map(|rate| Rate {
                    username: rate.0.clone(),
                    food_name: rate.1.clone(),
                    rate_level: rate.2.into(),
                    created_at: rate.3,
                })
                .collect::<Vec<_>>()
        })
    }

    pub async fn post(pool: Data<DbPool>, rate_param: RateParam) -> Result<()> {
        let user = UserDAO::by_session_id(pool.clone(), rate_param.session_id).await?;

        if rate_param.send_date != Local::today().naive_local() {
            Err(Error::DateChanged)
        } else {
            for r in rate_param.rates.clone() {
                let mut conn = get_conn(pool.clone()).await;
                block(move || {
                    diesel::insert_into(dsl::rates)
                        .values((
                            dsl::food_name.eq(r.food_name),
                            dsl::rate_level.eq(r.level as i8),
                            dsl::user_id.eq(&user.id),
                        ))
                        .execute(&mut conn)
                })
                .await??;
            }
            Ok(())
        }
    }
}
