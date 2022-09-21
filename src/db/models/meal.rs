use crate::db::schema::meals::{self, dsl::*};
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Clone, Queryable, Insertable)]
#[diesel(table_name = meals)]
pub struct MealDAO {
    pub id: u64,
    pub name: String,
    pub created_at: NaiveDate,
}
