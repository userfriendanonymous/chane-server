use actix_web::{HttpServer, App, web::Data, HttpResponse, Responder};
use serde_json::json;
use tokio::sync::MutexGuard;
use crate::{db_pool::{DbPoolShared, DbPool, Error as DbError}, session::{SessionShared, Error as SessionError, Session, self}};

mod api;
mod middleware;

struct AppState {
    db_pool: DbPoolShared,
    session: Option<SessionShared>
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

fn handle_session_error(error: SessionError) -> HttpResponse {
    match error {
        SessionError::Db(error) => match error {
            DbError::InvalidObjectId(message) => HttpResponse::BadRequest().json(json!({
                "message": "invalid object id"
            })),
            DbError::NotFound => HttpResponse::NotFound().json(json!({
                "message": "not found"
            })),
            DbError::Query(error) => HttpResponse::InternalServerError().json(json!({
                "db query error": error.to_string()
            })),
        },
        SessionError::Unauthorized(message) => HttpResponse::Unauthorized().json(json!({
            "unauthorized": message
        }))
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("faild to bind server: {0}")]
    FailedToBind(std::io::Error),
    #[error("failed to create db pool: {0}")]
    Db(mongodb::error::Error),
}

type AppStateData = Data<AppState>;

pub async fn launch() -> Result<(), Error> {
    let app_state = Data::new(AppState {
        db_pool: DbPool::new_shared().await.map_err(Error::Db)?,
        session: None,
    });

    HttpServer::new(move || {
        App::new()
        .app_data(app_state.clone())
        .service(api::service())
    })
    .bind(("127.0.0.1", 5000)).map_err(Error::FailedToBind)?
    .run()
    .await;

    Ok(())
}