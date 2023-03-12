use serde::{Serialize, Deserialize};
use crate::db_pool;

use super::{Session, extract_db, extract_auth, Error as GeneralError};

#[derive(Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub content: String,
    pub author_name: String,
}

impl From<db_pool::Block> for Block {
    fn from(model: db_pool::Block) -> Self {
        Self {
            id: model.id.unwrap(), // UGH I HATE THE UNWRAP!!!
            content: model.content,
            author_name: model.author_name
        }
    }
}

impl Session {
    pub async fn create_block(&self, content: &str) -> Result<String, GeneralError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_cloned);
        Ok(db_pool.create_block(content, auth.name.as_str()).await.map_err(GeneralError::Db)?)
    }

    pub async fn get_block(&self, id: &str) -> Result<Block, GeneralError> {
        extract_db!(self, db_pool, db_pool_cloned);
        Ok(Block::from(db_pool.get_block(id).await.map_err(GeneralError::Db)?))
    }
}