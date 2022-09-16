use std::io;

use actix_web::{get, App, HttpServer, Responder};

#[tokio::main]
async fn main() -> io::Result<()> {
    HttpServer::new(|| App::new().service(root))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await?;

    Ok(())
}

#[get("/")]
async fn root() -> impl Responder {
    "unho root"
}
