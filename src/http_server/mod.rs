use std::sync::Arc;
use actix_web::{HttpServer as ActixHttpServer, App, web::Data, HttpRequest};
use actix_cors::Cors;
use crate::{session_pool::{SessionPool, Session}, logger::Logger, auth_validator::Tokens};

mod api;
mod errors;

fn extract_cookie_as_string(request: &HttpRequest, name: &str) -> String {
    match request.cookie(name) {
        Some(cookie) => cookie.value().to_owned(),
        None => "".to_owned()
    }
}

pub struct AppState {
    session_pool: Arc<SessionPool>,
    logger: Arc<Logger>,
}

impl AppState {
    pub fn session_from_request(&self, request: &HttpRequest) -> Session {
        self.session_pool.spawn_session(&Tokens {
            access: extract_cookie_as_string(request, "access-token"),
            key: extract_cookie_as_string(request, "key-token")
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("faild to bind server: {0}")]
    FailedToBind(std::io::Error),
    #[error("failed to create db pool: {0}")]
    Db(mongodb::error::Error),
    #[error("failed to run server")]
    Running(std::io::Error)
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Running(value)
    }
}

pub struct HttpServer {
    session_pool: Arc<SessionPool>,
    logger: Arc<Logger>,
}

impl HttpServer {
    pub fn new(session_pool: Arc<SessionPool>, logger: Arc<Logger>) -> Self {
        Self {session_pool, logger}
    }

    pub async fn run(self: Arc<Self>) {
        let this = self.clone();
        let app_state = Data::new(AppState {
            logger: this.logger.clone(),
            session_pool: this.session_pool.clone(),
        });

        ActixHttpServer::new(move || {
            App::new()
            .wrap(
                Cors::permissive()
                .allow_any_header()
                .allow_any_origin()
                .allow_any_method()
            )
            .app_data(app_state.clone())
            .service(api::service())
        })
        .bind(("127.0.0.1", 5000)).map_err(Error::FailedToBind).unwrap()
        .run()
        .await.unwrap();
    }
}

type AppStateData = Data<AppState>;