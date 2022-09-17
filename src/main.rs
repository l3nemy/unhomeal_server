use actix_web::{
    get, middleware::Logger, post, web::Json, App, HttpResponse, HttpServer, Responder,
};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{read_one, Item};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};

// TODO: Create crate::Error for better error handling and replace all BoxError to crate::Error
type BoxError = Box<dyn std::error::Error>;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    http_port: u16,
    https_port: u16,
    cert_location: String,
    key_location: String,
}

fn load_config() -> Result<Config, BoxError> {
    let config_file = File::open("./config.yml")?;
    let config = serde_yaml::from_reader(config_file)?;

    // TODO: Verify if config is not malformed (e.g. cert_location or key_location is empty)

    Ok(config)
}

#[actix_web::main]
async fn main() -> Result<(), BoxError> {
    let config = load_config()?;
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

    HttpServer::new(|| {
        App::new()
            .service(root)
            .service(login)
            .wrap(Logger::default())
    })
    .bind_rustls(("127.0.0.1", 7912), rustls_config)?
    .workers(8)
    .run()
    .await?;

    Ok(())
}

#[get("/")]
async fn root() -> impl Responder {
    "unho root"
}

// TODO: remove #[allow(unused)]
#[allow(unused)]
#[derive(Deserialize)]
struct LoginParam {
    user_id: String,
    // password: String,
}

/// LoginResponse is used to send login result to the client    
/// * Response body which is serialized into JSON
/// * And sent to clients
#[derive(Serialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
    access_token_expiry: u64,
    refresh_token_expiry: u64,
}

/// Login procedure
/// * Receives JSON request body as struct LoginParam
/// * Responds with JSON body with access_token, refresh_token and their expiry
#[post("/login")]
async fn login(_param: Json<LoginParam>) -> Result<HttpResponse, actix_web::Error> {
    // TODO: implement login procedure
    // 1. Check if uid is in database
    // 2. Error handle
    // 2-1. If not in database return Error
    // 2-2. Else continue
    // 3. Generate two Bearer token (access_token, refresh_token)
    // 3-1. + expiry dates
    // 4. Finally send Response
    todo!()
}
