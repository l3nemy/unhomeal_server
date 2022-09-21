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
    routes::{GetUserRatesParam, Rate, RateParam},
};
use actix_web::web::{block, Data};
use chrono::{Duration, Local, NaiveDate, NaiveDateTime};
use diesel::prelude::*;

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

    pub async fn get_one_today(pool: Data<DbPool>, param: GetUserRatesParam) -> Result<Vec<Rate>> {
        Self::get_one(pool, param, Local::today().naive_local()).await
    }

    pub async fn get(pool: Data<DbPool>, date: NaiveDate) -> Result<Vec<Rate>> {
        let mut conn = get_conn(pool).await;

        let next_date = date + Duration::days(1);
        block(move || {
            rates::table
                .left_join(users::table.on(users::id.eq(dsl::user_id)))
                .select((
                    users::username.assume_not_null(),
                    rates::food_name,
                    rates::rate_level,
                    rates::created_at,
                ))
                .filter(rates::created_at.ge(date.and_hms(0, 0, 0)))
                .filter(rates::created_at.lt(next_date.and_hms(0, 0, 0)))
                .get_results::<(String, String, i8, NaiveDateTime)>(&mut conn)
        })
        .await?
        .map_err(Error::not_found_on_db)
        .map(|v| v.iter().map(|elem| elem.into()).collect::<Vec<_>>())
    }

    pub async fn get_one(
        pool: Data<DbPool>,
        param: GetUserRatesParam,
        date: NaiveDate,
    ) -> Result<Vec<Rate>> {
        let user = UserDAO::by_session_id(pool.clone(), &param.session_id).await?;
        let target = UserDAO::by_username(pool.clone(), &param.username).await?;

        if user.is_teacher || user.id == target.id {
            let mut conn = get_conn(pool.clone()).await;

            block(move || {
                dsl::rates
                    .left_join(users::table.on(users::id.eq(dsl::user_id)))
                    .select((
                        users::username.assume_not_null(),
                        rates::food_name,
                        rates::rate_level,
                        rates::created_at,
                    ))
                    .filter(dsl::created_at.ge(date.and_hms(0, 0, 0)))
                    .filter(dsl::created_at.lt((date + Duration::days(1)).and_hms(0, 0, 0)))
                    .filter(dsl::user_id.eq(target.id))
                    .load::<(String, String, i8, NaiveDateTime)>(&mut conn)
            })
            .await?
            .map_err(Into::into)
            .map(|v| v.iter().map(|elem| elem.into()).collect::<Vec<_>>())
        } else {
            Err(Error::Unprivileged)
        }
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
