use std::sync::Arc;

use actix_web::{HttpRequest, web, HttpResponse, get};
use tokio::sync::Mutex;
use crate::live_channel::{self, LiveMessage};
use super::super::AppStateData;
use async_trait::async_trait;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("serialization: {0}")]
    Json(serde_json::Error),
    #[error("session already closed: {0}")]
    SessionClosed(actix_ws::Closed),
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
impl From<actix_ws::Closed> for Error {
    fn from(value: actix_ws::Closed) -> Self {
        Self::SessionClosed(value)
    }
}

struct WebsocketPeer {
    session: Mutex<actix_ws::Session>
}

#[async_trait]
impl live_channel::Peer for WebsocketPeer {
    async fn receive_message(&self, message: &LiveMessage) {
        self.session.lock().await.text("receive").await;
    }
}

#[get("/live")]
pub async fn service(app_state: AppStateData, request: HttpRequest, body: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    let (response, session, mut message_stream) = actix_ws::handle(&request, body)?;

    let peer = Arc::new(WebsocketPeer {
        session: Mutex::new(session.clone())
    });

    let live_channel = app_state.live_channel.clone();
    let handle = live_channel.connect(peer.clone(), &"".to_string()).await;

    actix_rt::spawn(async move {
        while let Some(Ok(message)) = message_stream.recv().await {
        }

        live_channel.disconnect(handle).await;
        let _ = session.close(None).await;
    });

    Ok(response)
}