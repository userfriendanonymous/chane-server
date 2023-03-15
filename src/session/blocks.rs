use serde::{Serialize, Deserialize};
use crate::{db_pool, session::LiveMessage};

use super::{Session, extract_db, extract_auth, Error as GeneralError, LiveChannel};

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

impl<LC: LiveChannel> Session<LC> {
    pub async fn create_block(&self, content: &str) -> Result<String, GeneralError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_cloned);
        Ok(db_pool.create_block(content, auth.name.as_str(), &Vec::new()).await?)
    }

    pub async fn get_block(&self, id: &str) -> Result<Block, GeneralError> {
        extract_db!(self, db_pool, db_pool_cloned);
        Ok(Block::from(db_pool.get_block(id).await?))
    }

    pub async fn change_block(&self, id: &str, content: &str) -> Result<(), GeneralError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_cloned);
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