use std::{collections::HashMap, fmt::Debug};
use serde::Serialize;
use async_trait::async_trait;
use crate::shared::Shared;
use tokio::sync::Mutex;

type Channels<P: Peer> = HashMap<String, HashMap<i64, Shared<P>>>;

#[derive(Serialize)]
#[serde(tag = "topic", content = "data", rename = "snake_case")]
pub enum LiveMessage {
    BlockConnected {
        id: String
    },
    BlockDisconnected {
        id: String
    },
    LabelsChanged,
    DescriptionChanged,
    BlockPinned {
        id: Option<String>
    },
    BlockChanged {
        id: String
    },
}

#[async_trait]
pub trait Peer {
    async fn receive_message(&mut self, message: &LiveMessage);
}

pub struct Handle {
    channel_id: String,
    peer_id: i64,
}

#[derive(thiserror::Error, Debug)]
pub enum DisconnectError {
    #[error("channel id: {0} not found")]
    ChannelNotFound(String),
    #[error("peer id: {0} not found")]
    PeerNotFound(i64)
}

#[derive(Default)]
pub struct LiveChannel {
    channels: Mutex<Channels<dyn Peer>>,
    peer_id: Mutex<i64>
}

impl LiveChannel {
    pub async fn receive_message(&self, channel_id: &str, message: &LiveMessage) {
        let empty_peers = Vec::new();
        let channels = self.channels.lock().await;
        let peers = channels.get(channel_id).unwrap_or(&empty_peers);
        for peer in peers {
            peer.lock().await.receive_message(message).await
        }
    }

    pub async fn connect(&self, peer: &Shared<dyn Peer>, channel_id: &str) -> Handle {
        let handle = Handle {
            channel_id: channel_id.to_string(),
            peer_id: self.peer_id
        };

        let mut channels = self.channels.lock().await;
        match channels.get_mut(channel_id) {
            Some(peers) => {
                peers.insert(self.peer_id, peer.clone());
                *self.peer_id.lock().await += 1;
            },
            None => {
                channels.insert(channel_id.to_string(), vec![peer.clone()]);
            }
        };

        handle
    }

    pub async fn disconnect(&self, handle: Handle) -> Result<(), DisconnectError> {
        let mut channels = self.channels.lock().await;
        let channel = channels.get_mut(handle.channel_id.as_str()).ok_or(DisconnectError::ChannelNotFound(handle.channel_id))?;
        channel.remove(&handle.peer_id).ok_or(DisconnectError::PeerNotFound(handle.peer_id))?;
        Ok(())
    }
}