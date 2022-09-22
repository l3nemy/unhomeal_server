mod db;
mod error;
pub use error::Result;
mod routes;
mod token;

use actix_web::{
    middleware::Logger,
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use diesel::r2d2::Pool;
use diesel::{r2d2::ConnectionManager, MysqlConnection};
use routes::get_applications_route;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::File;

use crate::{
    db::DbPool,
    routes::{
        apply_route, get_rates_route, get_user_rate_route, login_route, logout_route,
        post_rate_route, rank_route, test_route,
    },
};

/*
use std::io::BufReader;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{read_one, Item};
*/

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    port: u16,
    cert_location: String,
    key_location: String,
    database_url: String,
    token_generation_key: String,
}

fn load_config() -> Result<Config> {
    let config_file = File::open("./config.yml")?;
    let config: Config = serde_yaml::from_reader(config_file)?;

    // TODO: Verify if config is not malformed (e.g. cert_location or key_location is empty)

    Ok(config)
}

#[actix_web::main]
async fn main() -> Result<()> {
    let config = load_config()?;

    /*
    // generating certs for localhost :
    //      openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'
    print!("Reading certificate: ");
    let cert_file = File::open(config.cert_location)?;
    let key_file = File::open(config.key_location)?;

    let mut cert_reader = BufReader::new(cert_file);
    let mut key_reader = BufReader::new(key_file);

    let cert_buf = match read_one(&mut cert_reader)?.unwrap() {
        Item::X509Certificate(der) => der,
        _ => {
            panic!("Invalid certificate")
        }
    };

    let key_buf = match read_one(&mut key_reader)?.unwrap() {
        Item::PKCS8Key(der) => der,
        _ => {
            panic!("Invalid private key")
        }
    };

    let cert_chain = vec![Certificate(cert_buf)];
    let key = PrivateKey(key_buf);
    println!("done");

    let rustls_config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)?;
    */
    let connection = ConnectionManager::<MysqlConnection>::new(&config.database_url);
    let pool = Pool::builder()
        .build(connection)
        .expect("Error creating dbpool");

    HttpServer::new(move || {
        App::new()
            .service(test_route) //서버 온라인 체크
            .service(login_route) // 로그인
            .service(logout_route) // 로그아웃
            .service(apply_route) // 신청
            .service(get_applications_route) // 신청 명단
            .service(post_rate_route) // 설문 제출
            .service(get_rates_route) // 학생 설문 정보
            .service(get_user_rate_route) // 단일 학생 설문 정보
            .service(rank_route) // 급식 랭킹
            .app_data(Data::new(pool.clone()))
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                actix_web::error::InternalError::from_response(
                    "",
                    HttpResponse::BadRequest()
                        .content_type("application/json")
                        .json(json!({
                            "is_error": true,
                            "error": {
                                "type": "JsonError",
                                "content": format!("{}", err)
                            }
                        })),
                )
                .into()
            }))
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
    })
    //.bind_rustls(("127.0.0.1", config.port), rustls_config)?
    .bind(("127.0.0.1", config.port))?
    .workers(8)
    .run()
    .await?;

    Ok(())
}
