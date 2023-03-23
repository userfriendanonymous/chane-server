use serde::{Serialize, Deserialize};
use crate::{db_pool::{self, Activity}, live_channel::LiveMessage};

use super::{Session, Error as GeneralError};

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub content: String,
    pub owner: String,
}

impl From<db_pool::Block> for Block {
    fn from(model: db_pool::Block) -> Self {
        Self {
            id: model.id.unwrap(),
            content: model.content,
            owner: model.owner
        }
    }
}

impl Session {
    pub async fn create_block(&self, content: &str) -> Result<String, GeneralError> {
        let (db_pool, auth) = self.auth_and_db()?;
        let id = db_pool.create_block(content, auth.name.as_str(), &Vec::new()).await?;

        self.activity_logger.log(&auth.activity_table_id, &[
            Activity::BlockCreated { by: auth.name.clone(), id: id.clone() }
        ]);
        Ok(id)
    }

    pub async fn get_block(&self, id: &str) -> Result<Block, GeneralError> {
        let db_pool = self.db_pool();
        Ok(Block::from(db_pool.get_block(id).await?))
    }

    pub async fn change_block(&self, id: &str, content: &str) -> Result<(), GeneralError> {
        let (db_pool, auth) = self.auth_and_db()?;
        let block = db_pool.get_block(id).await?;
        if block.owner != auth.name {
            return Err(GeneralError::Unauthorized("you don't have permissions to change this block".to_owned()));
        }
        db_pool.change_block(id, content).await?;
        let mut live_channel = self.live_channel.lock().await;
        
        let message = LiveMessage::BlockChanged { id: id.to_string() };
        for channel_id in block.connected_channels {
            live_channel.receive_message(channel_id.as_str(), &message).await;
        }
        Ok(())
    }
}