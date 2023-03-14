use serde::Serialize;
use async_trait::async_trait;

#[async_trait]
pub trait LiveChannel {
    async fn receive_message(&mut self, channel_id: &str, message: &LiveMessage);
}

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
}