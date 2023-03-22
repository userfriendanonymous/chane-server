use std::sync::Arc;

use actix_web::{HttpServer as ActixHttpServer, App, web::Data, HttpResponse, FromRequest};
use serde_json::json;
use actix_cors::Cors;
use crate::{db_pool::DbPool, session::{Session, AuthKeys}};
use crate::shared::Shared;
pub use api::LiveChannel;

mod api;
mod error_handlers;
mod utils;

pub struct AppState {
    db_pool: Shared<DbPool>,
    session: Shared<Option<Session<LiveChannel>>>,
    live_channel: Shared<LiveChannel>,
    auth_keys: Arc<AuthKeys>
}

fn extract_session_gen() -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({
        "message": "user session not found"
    }))
}

macro_rules! extract_session {
    ($app_state:expr, $session:ident, $gen:ident) => {
        let $session = $app_state.session.clone();
        let $session = &*$session.lock().await; // shadow like a pro
        let $session = match $session {
            Some(session) => session,
            None => return $gen()
        };
    };
}
pub(self) use extract_session;

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

pub struct HttpServer;
impl HttpServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn run() -> Result<(), Error> {
        ActixHttpServer::new(move || {

        })
    }
}

type AppStateData = Data<AppState>;

pub async fn launch(auth_keys: AuthKeys) -> Result<(), Error> {
    let session = Shared::new(None);
    let db_pool = Shared::new(DbPool::new().await.map_err(Error::Db)?);
    let live_channel = Shared::new(LiveChannel::default());

    let app_state = Data::new(AppState {
        db_pool: db_pool.clone(),
        session: session.clone(),
        live_channel: live_channel.clone()
    });

    ActixHttpServer::new(move || {
        App::new()
        .wrap(
            Cors::permissive()
            .allow_any_header()
            .allow_any_origin()
            .allow_any_method()
        )
        .wrap(middleware::session::MiddlewareFactory {
            session: session.clone(),
            auth_keys: auth_keys.clone(),
            db_pool: db_pool.clone(),
            live_channel: live_channel.clone()
        })
        .app_data(app_state.clone())
        .service(api::service())
    })
    .bind(("127.0.0.1", 5000)).map_err(Error::FailedToBind)?
    .run()
    .await?;

    Ok(())
}