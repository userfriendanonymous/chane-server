use actix_web::{HttpRequest, web, HttpResponse, get};
use actix_ws::Session;
use crate::session::{self, LiveMessage};
use super::super::AppStateData;
use std::{sync::Arc, collections::HashMap};
use tokio::sync::Mutex;
use async_trait::async_trait;

pub struct Peer {
    pub session: Session
}

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

impl Peer {
    pub async fn receive_message(&mut self, message: &LiveMessage) -> Result<(), Error> {
        self.session.text(serde_json::to_string(message)?).await?;
        Ok(())
    }
}

type Channels = HashMap<String, Vec<PeerShared>>;
type PeerShared = Arc<Mutex<Peer>>;

#[derive(Default)]
pub struct State {
    channels: Mutex<Channels>,
}

#[async_trait]
impl session::LiveChannel for State {
    async fn receive_message(&mut self, channel_id: &str, message: &LiveMessage) {
        let empty_peers = Vec::new();
        let channels = self.channels.lock().await;
        let peers = channels.get(channel_id).unwrap_or(&empty_peers);
        let mut errors = Vec::new();
        for peer in peers {
            if let Err(error) = peer.lock().await.receive_message(message).await {
                errors.push(error);
            }
        }
        println!("errors: {errors:?}");
    }
}

pub type StateShared = Arc<Mutex<State>>;

impl State {
    pub fn default_shared() -> StateShared {
        Arc::new(Mutex::new(Self::default()))
    }

    pub async fn connect(&mut self, peer: &Arc<Mutex<Peer>>, channel_id: &String){
        let mut channels = self.channels.lock().await;
        match channels.get_mut(channel_id) {
            Some(peers) => peers.push(peer.clone()),
            None => {
                channels.insert(channel_id.clone(), vec![peer.clone()]);
            }
        };
    }
}

#[get("/chat")]
pub async fn service(app_state: AppStateData, request: HttpRequest, body: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    let (response, session, mut message_stream) = actix_ws::handle(&request, body)?;

    let peer = Arc::new(Mutex::new(Peer {
        session: session.clone()
    }));

    let chat_state = app_state.live_channel_state.clone();
    chat_state.lock().await.connect(&peer, &"".to_string()).await;

    actix_rt::spawn(async move {
        while let Some(Ok(message)) = message_stream.recv().await {
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}