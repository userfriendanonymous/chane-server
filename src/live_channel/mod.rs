use std::{collections::HashMap, fmt::Debug, sync::Arc};
use serde::Serialize;
use async_trait::async_trait;
use tokio::sync::Mutex;

type PeerShared = Arc<dyn Peer + Send + Sync>;
type Channels = HashMap<String, HashMap<i64, PeerShared>>;

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
    async fn receive_message(&self, message: &LiveMessage);
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
    channels: Mutex<Channels>,
    peer_id: Mutex<i64>
}

impl LiveChannel {
    pub async fn receive_message(&self, channel_id: &str, message: &LiveMessage) {
        let empty_peers = HashMap::new();
        let channels = self.channels.lock().await;
        let peers = channels.get(channel_id).unwrap_or(&empty_peers);
        for (id, peer) in peers {
            peer.receive_message(message).await
        }
    }

    pub async fn connect(&self, peer: PeerShared, channel_id: &str) -> Handle {
        let mut peer_id = self.peer_id.lock().await;
        let handle = Handle {
            channel_id: channel_id.to_string(),
            peer_id: *peer_id
        };

        let mut channels = self.channels.lock().await;
        match channels.get_mut(channel_id) {
            Some(peers) => {
                peers.insert(*peer_id, peer);
            },
            None => {
                let mut peers = HashMap::new();
                peers.insert(*peer_id, peer);
                channels.insert(channel_id.to_string(), peers);
            }
        };
        *peer_id += 1;

        handle
    }

    pub async fn disconnect(&self, handle: Handle) -> Result<(), DisconnectError> {
        let mut channels = self.channels.lock().await;
        let channel = channels.get_mut(handle.channel_id.as_str()).ok_or(DisconnectError::ChannelNotFound(handle.channel_id))?;
        channel.remove(&handle.peer_id).ok_or(DisconnectError::PeerNotFound(handle.peer_id))?;
        Ok(())
    }
}