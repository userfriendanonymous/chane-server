use actix_web::{HttpServer, App, web::Data, HttpResponse};
use serde_json::json;
use crate::{db_pool::DbPool, session::Session};
use crate::shared::Shared;
pub use api::LiveChannel;

mod api;
mod middleware;
mod error_handlers;

pub struct AppState {
    db_pool: Shared<DbPool>,
    session: Option<Shared<Session<LiveChannel>>>,
    live_channel: Shared<LiveChannel>
}

fn extract_session_gen() -> HttpResponse {
    HttpResponse::InternalServerError().json(json!({
        "message": "user session not found"
    }))
}

macro_rules! extract_session {
    ($app_state:expr, $session:ident, $session_cloned:ident, $gen:ident) => {
        let $session_cloned = match &$app_state.session {
            Some(session) => session.clone(),
            None => return $gen()
        };

        let $session = $session_cloned.lock().await;
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

type AppStateData = Data<AppState>;

pub async fn launch() -> Result<(), Error> {
    let app_state = Data::new(AppState {
        db_pool: Shared::new(DbPool::new().await.map_err(Error::Db)?),
        session: None,
        live_channel: Shared::new(LiveChannel::default())
    });

    HttpServer::new(move || {
        App::new()
        .app_data(app_state.clone())
        .service(api::service())
    })
    .bind(("127.0.0.1", 5000)).map_err(Error::FailedToBind)?
    .run()
    .await?;

    Ok(())
}