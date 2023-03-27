use std::sync::Arc;

use actix_web::{HttpRequest, web::{self, Path}, HttpResponse, get};
use serde_json::json;
use tokio::sync::Mutex;
use crate::{live_channel::{self, LiveMessage}, logger::Logger, http_server::errors::TransResponse};
use super::{AppStateData, Response, errors::ResultResponse};
use async_trait::async_trait;

// #[derive(thiserror::Error, Debug)]
// pub enum Error {
//     #[error("serialization: {0}")]
//     Json(serde_json::Error),
//     #[error("session already closed: {0}")]
//     SessionClosed(actix_ws::Closed),
// }

// impl From<serde_json::Error> for Error {
//     fn from(value: serde_json::Error) -> Self {
//         Self::Json(value)
//     }
// }
// impl From<actix_ws::Closed> for Error {
//     fn from(value: actix_ws::Closed) -> Self {
//         Self::SessionClosed(value)
//     }
// }

struct WebsocketPeer {
    session: Mutex<actix_ws::Session>,
    logger: Arc<Logger>
}

#[async_trait]
impl live_channel::Peer for WebsocketPeer {
    async fn receive_message(&self, message: &LiveMessage) {
        if let Err(error) = self.session.lock().await.text(
            match serde_json::to_string(&message).map_err(|error| { // rust is flexible with this but now it's quite hard to understand what exactly is going on here..
                self.logger.log(error.to_string());
            }) {
                Ok(data) => data,
                Err(()) => return
            }
        ).await {
            self.logger.log(error.to_string());
        }
    }
}

#[get("/live/{id}")]
pub async fn service(app_state: AppStateData, request: HttpRequest, body: web::Payload, id: Path<String>) -> HttpResponse {
    let session = app_state.session_from_request(&request);
    let handle = match session.live(&id).await {
        Ok(handle) => handle,
        Err(error) => return HttpResponse::InternalServerError().json(json!({"message": "this is wip"}))
    };

    let (response, session, mut message_stream) = match actix_ws::handle(&request, body) {
        Ok(result) => result,
        Err(error) => return HttpResponse::InternalServerError().json(json!({"message": error.to_string()}))
    };

    let peer = Arc::new(WebsocketPeer {
        session: Mutex::new(session.clone()),
        logger: app_state.logger.clone()
    });

    handle.connect(peer).await;

    actix_rt::spawn(async move {
        while let Some(Ok(_)) = message_stream.recv().await {}

        handle.disconnect().await;
        let _ = session.close(None).await;
    });

    response
}