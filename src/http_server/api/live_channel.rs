use actix_web::{HttpRequest, web, HttpResponse, get};
use actix_ws::{Message as WsMessage, Session};
use crate::session::Block;

use super::super::AppStateData;
use std::{sync::Arc, collections::HashMap};
use tokio::sync::Mutex;

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
    pub async fn receive_block(&mut self, block: &Block) -> Result<(), Error> {
        self.session.text(serde_json::to_string(block)?).await?;
        Ok(())
    }
}

#[derive(Default)]
pub struct State {
    channels: Mutex<
        HashMap<String, Vec<Arc<Mutex<Peer>>>>
    >,
}

impl State {
    pub async fn connect(&mut self, peer: &Arc<Mutex<Peer>>, channel_id: &String){
        let mut channels = self.channels.lock().await;
        match channels.get_mut(channel_id) {
            Some(peers) => peers.push(peer.clone()),
            None => {
                let mut peers = Vec::new();
                peers.push(peer.clone());
                channels.insert(channel_id.clone(), peers);
            }
        };
    }
  
    pub async fn send_block(&mut self, block: &Block, channel_id: &String) {
        let empty_peers = Vec::new();
        let channels = self.channels.lock().await;
        let peers = channels.get(channel_id).unwrap_or(&empty_peers);
        for peer in peers {
            peer.lock().await.receive_block(block).await;
        }
    }
}

#[get("/chat")]
pub async fn service(app_state: AppStateData, request: HttpRequest, body: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut message_stream) = actix_ws::handle(&request, body)?;

    let peer = Arc::new(Mutex::new(Peer {
        session: session.clone()
    }));

    let chat_state = app_state.live_channel_state.clone();
    chat_state.lock().await.connect(&peer, &"".to_string());

    actix_rt::spawn(async move {
        while let Some(Ok(message)) = message_stream.recv().await {
            match message {
                WsMessage::Text(message) => {
                    let message = message.to_string();
                    println!("Got text, {}", &message);

                    let block = serde_json::from_str(message.as_str()).unwrap(); // TO FIX THIS!
                    chat_state.lock().await.send_block(&block, &"".to_string()).await;
                },
                _ => break
            }
        }

        let _ = session.close(None).await;
    });

    Ok(response)
}