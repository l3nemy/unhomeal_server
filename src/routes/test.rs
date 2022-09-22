use actix_web::{get, HttpResponse};

use crate::error::Result;

#[get("/test")]
pub async fn test_route() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("Ok"))
}
