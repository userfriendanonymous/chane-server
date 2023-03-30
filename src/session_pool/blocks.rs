use serde::{Serialize, Deserialize};
use ts_rs::TS;
use crate::{db_pool, live_channel::LiveMessage, activity_logger::Activity};

use super::{Session, Error as GeneralError};

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Block {
    pub id: String,
    pub content: String,
    pub owner: String,
}

impl From<db_pool::Block> for Block {
    fn from(model: db_pool::Block) -> Self {
        Self {
            id: model.id.unwrap().to_string(),
            content: model.content,
            owner: model.owner
        }
    }
}

impl Session {
    pub async fn create_block(&self, content: &str) -> Result<String, GeneralError> {
        let auth = self.auth()?;
        let id = self.db_pool.create_block(content, auth.name.as_str(), &Vec::new()).await?;

        self.activity_logger.log(Activity::BlockCreated { id: id.clone(), by: auth.name.clone() });
        Ok(id)
    }

    pub async fn get_block(&self, id: &str) -> Result<Block, GeneralError> {
        Ok(Block::from(self.db_pool.get_block(id).await?))
    }

    pub async fn change_block(&self, id: &str, content: &str) -> Result<(), GeneralError> {
        let auth = self.auth()?;
        let block = self.db_pool.get_block(id).await?;
        if block.owner != auth.name {
            return Err(GeneralError::Unauthorized);
        }
        self.db_pool.change_block(id, content).await?;
        
        let message = LiveMessage::BlockChanged { id: id.to_string() };
        for channel_id in block.connected_channels {
            self.live_channel.receive_message(channel_id.as_str(), &message);
        }
        Ok(())
    }
}