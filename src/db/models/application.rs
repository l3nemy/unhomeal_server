use crate::{
    db::{
        get_conn,
        schema::applications::{self, dsl},
        DbPool,
    },
    error::{Error, Result},
    routes::{ApplyParam, GetApplicationParam},
};
use actix_web::web::{block, Data};
use chrono::{Local, NaiveDateTime, Duration, Datelike, NaiveDate};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::user::UserDAO;

#[derive(Clone, Queryable, Insertable)]
#[diesel(table_name = applications)]
pub struct ApplicationDAO {
    pub id: u64,
    pub user_id: u64,
    pub created_at: NaiveDateTime,
}

fn next_first_day_of_month(year: i32, month: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap_or(NaiveDate::from_ymd(year + 1, 1, 1))
}

impl ApplicationDAO {
    pub async fn post(pool: Data<DbPool>, param: ApplyParam) -> Result<()> {
        let user = UserDAO::by_session_id(pool.clone(), &param.session_id).await?;

        if param.send_date != Local::now().date_naive() {
            Err(Error::DateChanged)
        } else {
            let mut conn = get_conn(pool).await;
            block(move || {
                diesel::insert_into(dsl::applications)
                    .values(dsl::user_id.eq(&user.id))
                    .execute(&mut conn)
            })
            .await??;
            Ok(())
        }
    }

    pub async fn get(
        pool: Data<DbPool>,
        param: GetApplicationParam,
    ) -> Result<Vec<ApplicationDAO>> {
        let user = UserDAO::by_session_id(pool.clone(), &param.session_id).await?;

        if user.is_teacher {
            let mut conn = get_conn(pool).await;
            block(move || dsl::applications.load::<ApplicationDAO>(&mut conn))
                .await?
                .map_err(Into::into)
        } else {
            Err(Error::Unprivileged)
        }
    }

    pub async fn get_one_month(
        pool: Data<DbPool>,
        param: GetApplicationParam,
    ) -> Result<Vec<ApplicationDAO>> {
        let user = UserDAO::by_session_id(pool.clone(), &param.session_id).await?;

        let today = Local::today();         
        let first_day = NaiveDate::from_ymd(today.year(), today.month(), 1);
        let next_first_day = next_first_day_of_month(today.year(), today.month());

        let today = Local::today().naive_local();
        let tomorrow = today + Duration::days(1);
        let mut conn = get_conn(pool).await;
        block(move || dsl::applications
            .filter(
                dsl::created_at.ge(today.and_hms(0, 0, 0))
            )
            .filter(
                dsl::created_at.lt(tomorrow.and_hms(0, 0, 0))
            )
            .load::<ApplicationDAO>(&mut conn))
            .await?
            .map_err(Into::into)
    }

    pub async fn get_one<T>(pool: Data<DbPool>, session_id: T) -> Result<ApplicationDAO>
    where
        T: Into<String>,
    {
        let user = UserDAO::by_session_id(pool.clone(), session_id.into()).await?;

        if user.is_teacher {
            let mut conn = get_conn(pool).await;
            block(move || {
                dsl::applications
                    .filter(dsl::user_id.eq(&user.id))
                    .first(&mut conn)
            })
            .await?
            .map_err(Error::not_found_on_db)
        } else {
            Err(Error::Unprivileged)
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Application {
    pub username: String,
    pub created_at: NaiveDateTime,
}

impl Application {
    pub async fn from_application_dto(
        pool: Data<DbPool>,
        original: ApplicationDAO,
    ) -> Result<Self> {
        let user = UserDAO::by_id(pool, original.user_id).await?;

        Ok(Self {
            username: user.username.clone(),
            created_at: user.created_at,
        })
    }
}
