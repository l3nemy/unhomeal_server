use actix_web::{
    error::BlockingError,
    http::{header::HeaderValue, StatusCode},
    web, ResponseError,
};
use serde_json::json;
use std::fmt::Write;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    RustlsError(#[from] rustls::Error),

    #[error("Config file is malformed. Check config.yml (source: {source:?})")]
    ConfigError {
        #[from]
        source: serde_yaml::Error,
    },

    #[error("Error while executing blocking function")]
    BlockingError(#[from] BlockingError),

    #[error("Error while quering database")]
    DBError(#[from] diesel::result::Error),

    #[error("Database connection error `{0}`")]
    DBConnectionError(#[from] diesel::r2d2::Error),

    #[error(transparent)]
    ActixWebError(#[from] actix_web::Error),

    #[error("Not found on the database")]
    NotFoundOnDB,

    #[error(transparent)]
    LoginError(#[from] anyhow::Error),

    #[error(transparent)]
    TokenError(#[from] jsonwebtoken::errors::Error),

    #[error("Token has expired")]
    TokenExpired,

    #[error("Already logged in")]
    AlreadyLoggedIn(String),

    #[error("No such session")]
    NoSuchSession,

    #[error("Date has changed")]
    DateChanged,

    #[error("Unprivileged request")]
    Unprivileged,

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

impl Error {
    pub fn tag_name(&self) -> String {
        String::from(match self {
            Error::RustlsError(_) => "RustlsError",
            Error::ConfigError { source: _ } => "ConfigError",
            Error::BlockingError(_) => "BlockingError",
            Error::DBError(_) => "DBError",
            Error::DBConnectionError(_) => "DBConnectionError",
            Error::ActixWebError(_) => "ActixWebError",
            Error::NotFoundOnDB => "NotFoundOnDB",
            Error::LoginError(_) => "LoginError",
            Error::TokenError(_) => "TokenError",
            Error::TokenExpired => "TokenExpired",
            Error::AlreadyLoggedIn(_) => "AlreadyLoggedIn",
            Error::NoSuchSession => "NoSuchSession",
            Error::DateChanged => "DateChanged",
            Error::Unprivileged => "Unprivileged",
            Error::IOError(_) => "IOError",
        })
    }

    pub fn not_found_on_db(e: diesel::result::Error) -> Self {
        match e {
            diesel::result::Error::NotFound => Self::NotFoundOnDB,
            _ => e.into(),
        }
    }

    pub fn no_such_session(_e: impl std::error::Error) -> Self {
        Error::NoSuchSession
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        use Error::*;
        match *self {
            NotFoundOnDB | LoginError(_) | AlreadyLoggedIn(_) | NoSuchSession | Unprivileged => {
                StatusCode::BAD_REQUEST
            }
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut res = actix_web::HttpResponse::new(self.status_code());

        let mut buf = web::BytesMut::new();
        let _ = write!(
            buf,
            "{}",
            json! ({
                "is_error": true,
                "error": {
                    "type": self.tag_name(),
                    "content": format!("{}", self)
                }
            })
        );

        res.headers_mut().insert(
            actix_web::http::header::CONTENT_TYPE,
            HeaderValue::from_str(mime::APPLICATION_JSON.essence_str()).unwrap(),
        );

        res.set_body(actix_web::body::BoxBody::new(buf))
    }
}
