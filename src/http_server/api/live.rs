use std::sync::Arc;
use actix_web::{HttpRequest, web::{self, Path}, HttpResponse, get, Scope};
use serde_json::json;
use tokio::sync::Mutex;
use crate::{live_channel::{self, LiveMessage}, logger::Logger};
use super::AppStateData;
use async_trait::async_trait;
use futures::StreamExt;

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

pub fn service() -> Scope {
    web::scope("/live")
    .service(connect)
}

struct WebsocketPeer {
    session: Mutex<actix_ws::Session>,
    logger: Arc<Logger>
}

#[async_trait]
impl live_channel::Peer for WebsocketPeer {
    async fn receive_message(&self, message: &LiveMessage) {
        if let Err(error) = self.session.lock().await.text(
            match serde_json::to_string(&message).map_err(|error| {
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

#[get("/{id}")]
pub async fn connect(app_state: AppStateData, request: HttpRequest, body: web::Payload, id: Path<String>) -> HttpResponse {
    let session = app_state.session_from_request(&request);
    let handle = match session.live(&id).await.map_err(|e| {
        println!("{e:?}");
        HttpResponse::InternalServerError().json(json!({"message": "this is wip"}))
    }) {
        Ok(handle) => handle,
        Err(error) => return error
    };

    let (response, session, mut message_stream) = match actix_ws::handle(&request, body).map_err(|e| {
        println!("{e:?}");
        HttpResponse::InternalServerError().json(json!({"message": e.to_string()}))
    }) {
        Ok(result) => result,
        Err(error) => return error
    };

    let peer = Arc::new(WebsocketPeer {
        session: Mutex::new(session.clone()),
        logger: app_state.logger.clone()
    });

    handle.connect(peer).await;

    actix_rt::spawn(async move {
        while let Some(Ok(msg)) = message_stream.next().await {
            println!("{msg:?}");
        }

        handle.disconnect().await;
        let _ = session.close(None).await;
    });

    response
}