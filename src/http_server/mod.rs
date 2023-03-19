use actix_web::{HttpServer, App, web::Data, HttpResponse};
use serde_json::json;
use actix_cors::Cors;
use crate::{db_pool::DbPool, session::Session};
use crate::shared::Shared;
pub use api::LiveChannel;

mod api;
mod middleware;
mod error_handlers;

pub struct AppState {
    db_pool: Shared<DbPool>,
    session: Shared<Option<Session<LiveChannel>>>,
    live_channel: Shared<LiveChannel>
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

type AppStateData = Data<AppState>;

pub async fn launch() -> Result<(), Error> {
    let session = Shared::new(None);
    let db_pool = Shared::new(DbPool::new().await.map_err(Error::Db)?);
    let live_channel = Shared::new(LiveChannel::default());

    let app_state = Data::new(AppState {
        db_pool: db_pool.clone(),
        session: session.clone(),
        live_channel: live_channel.clone()
    });

    HttpServer::new(move || {
        App::new()
        .wrap(
            Cors::permissive()
        )
        .wrap(middleware::session::MiddlewareFactory {
            session: session.clone(),
            auth_keys: crate::session::AuthKeys { access: "".to_owned(), key: "".to_owned() },
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