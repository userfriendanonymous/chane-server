use std::{collections::HashMap, fmt::Debug, sync::Arc};
use serde::Serialize;
use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc::{UnboundedReceiver, UnboundedSender, self}};
use ts_rs::TS;

use crate::logger::Logger;

type PeerShared = Arc<dyn Peer + Send + Sync>;
type Channels = HashMap<String, HashMap<i64, PeerShared>>;

#[derive(Serialize, Clone, TS)]
#[ts(export)]
#[serde(tag = "is", content = "data")]
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

#[derive(Clone)]
pub struct Handle {
    pub channel_id: String,
    pub peer_id: i64,
}

#[derive(thiserror::Error, Debug)]
pub enum DisconnectError {
    #[error("channel id: {0} not found")]
    ChannelNotFound(String),
    #[error("peer id: {0} not found")]
    PeerNotFound(i64)
}

type MpscMessage = (String, LiveMessage);

pub struct LiveChannel {
    channels: Mutex<Channels>,
    logger: Arc<Logger>,
    peer_id: Mutex<i64>,
    receiver: Mutex<UnboundedReceiver<MpscMessage>>,
    sender: UnboundedSender<MpscMessage>
}

impl LiveChannel {
    pub fn new(logger: Arc<Logger>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            receiver: Mutex::new(receiver),
            sender,
            channels: Default::default(),
            peer_id: Default::default(),
            logger
        }
    }

    pub fn receive_message(&self, channel_id: &str, message: &LiveMessage) {
        if let Err(error) = self.sender.send((channel_id.to_string(), message.clone())) {
            self.logger.log(error.to_string());
        }
    }

    async fn handle_message(&self, channel_id: &str, message: &LiveMessage) {
        let empty_peers = HashMap::new();
        let channels = self.channels.lock().await;
        let peers = channels.get(channel_id).unwrap_or(&empty_peers);
        for peer in peers.values() {
            peer.receive_message(message).await
        }
    }

    pub async fn connect(&self, peer: PeerShared, channel_id: &str) -> Handle {
        let mut peer_id = self.peer_id.lock().await;
        let handle = Handle {
            channel_id: channel_id.to_string(),
            peer_id: *peer_id,
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
        let channel = channels.get_mut(handle.channel_id.as_str()).ok_or(DisconnectError::ChannelNotFound(handle.channel_id.clone()))?;
        channel.remove(&handle.peer_id).ok_or(DisconnectError::PeerNotFound(handle.peer_id))?;
        Ok(())
    }

    pub async fn run(&self){
        let mut receiver = self.receiver.lock().await;
        while let Some((channel_id, message)) = receiver.recv().await {
            self.handle_message(channel_id.as_str(), &message).await;
        }
    }
}