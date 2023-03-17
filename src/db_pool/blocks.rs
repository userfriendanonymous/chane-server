use super::{DbPool, Error, utils::as_object_id};
use futures::StreamExt;
use mongodb::{bson::{doc, oid::ObjectId}, options::FindOptions};
use serde::{Serialize, Deserialize};

const QUERY_LIMIT: i64 = 30;

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub content: String,
    pub owner: String,
    pub connected_channels: Vec<String>
}

impl DbPool {
    pub async fn get_block(&self, id: &str) -> Result<Block, Error> {
        let object_id = as_object_id!(id);
        let filter = doc! {"_id": object_id};
        let result = self.blocks.find_one(filter, None).await?;
        match result {
            Some(model) => {
                Ok(model)
            },
            None => Err(Error::NotFound)
        }
    }

    pub async fn create_block(&self, content: &str, owner: &str, connected_channels: &[String]) -> Result<String, Error> {
        let document = Block {
            id: None,
            content: content.to_string(),
            owner: owner.to_string(),
            connected_channels: connected_channels.to_owned()
        };
        let result = self.blocks.insert_one(document, None).await?;
        Ok(result.inserted_id.to_string())
    }

    pub async fn change_block(&self, id: &str, content: &str) -> Result<(), Error> {
        let result = self.blocks.update_one(doc! {"_id": as_object_id!(id)}, doc! {"$set": {
            "content": content
        }}, None).await?;
        Ok(())
    }

    pub async fn connect_block_to_channel(&self, id: &str, channel_id: &str) -> Result<(), Error> {
        match self.blocks.update_one(doc! {"_id": id}, doc! {"$push": {"connected_channels": channel_id}}, None).await {
            Ok(result) => if result.modified_count > 1 {
                Ok(())
            } else {
                Err(Error::NotFound)
            },
            Err(error) => Err(Error::Query(error))
        }
    }

    pub async fn disconnect_block_from_channel(&self, id: &str, channel_id: &str) -> Result<(), Error> {
        match self.blocks.update_one(doc! {"_id": id}, doc! {"$pull": {"connected_channels": channel_id}}, None).await {
            Ok(result) => if result.modified_count > 1 {
                Ok(())
            } else {
                Err(Error::NotFound)
            },
            Err(error) => Err(Error::Query(error))
        }
    }

    pub async fn get_channel_blocks(&self, channel_id: &str, limit: &Option<i64>, offset: &Option<u64>) -> Result<(Vec<Block>, Vec<mongodb::error::Error>), Error> {
        let limit = match *limit {
            Some(limit) => limit.clamp(0, QUERY_LIMIT),
            None => QUERY_LIMIT
        };
        match self.blocks.find(doc! {"connected_channels": channel_id}, Some(FindOptions::builder().limit(Some(limit)).skip(*offset).build())).await {
            Ok(mut result) => {
                let mut blocks = Vec::new();
                let mut errors = Vec::new();
                while let Some(block_result) = result.next().await {
                    match block_result {
                        Ok(block) => blocks.push(block),
                        Err(error) => errors.push(error)
                    }
                }
                Ok((blocks, errors))
            },
            Err(error) => Err(Error::Query(error))
        }
    }
}