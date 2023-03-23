use std::sync::Arc;
use actix_web::{HttpServer as ActixHttpServer, App, web::Data};
use actix_cors::Cors;
use crate::{session_pool::SessionPool, logger::Logger};

mod api;
mod error_handlers;
mod utils;

pub struct AppState {
    session_pool: Arc<SessionPool>,
    logger: Arc<Logger>
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
    logger: Arc<Logger>
}

impl HttpServer {
    pub fn new(session_pool: Arc<SessionPool>, logger: Arc<Logger>) -> Self {
        Self {session_pool, logger}
    }

    pub async fn run(&self) -> Result<(), Error> {
        let app_state = Data::new(AppState {
            logger: self.logger.clone(),
            session_pool: self.session_pool.clone()
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
        .bind(("127.0.0.1", 5000)).map_err(Error::FailedToBind)?
        .run()
        .await?;

        Ok(())
    }
}

type AppStateData = Data<AppState>;