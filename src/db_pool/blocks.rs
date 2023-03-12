use super::{DbPool, Error, utils::as_object_id};
use mongodb::bson::{doc, oid::ObjectId};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<u64>,
    content: String,
    author_name: String,
    channel_id: String,
}

impl DbPool {
    pub async fn get_block(&self, id: &str) -> Result<Model, Error> {
        let object_id = as_object_id!(id);
        let filter = doc! {"_id": object_id};
        let result = self.blocks.find_one(filter, None).await.map_err(Error::Query)?;
        match result {
            Some(model) => {
                Ok(model)
            },
            None => Err(Error::NotFound)
        }
    }

    pub async fn create_block(&self, content: &str, author_name: &str, channel_id: &str) -> Result<String, Error> {
        let document = Model {
            id: None,
            content: content.to_string(),
            author_name: author_name.to_string(),
            channel_id: channel_id.to_string()
        };
        let result = self.blocks.insert_one(document, None).await.map_err(Error::Query)?;
        Ok(result.inserted_id.to_string())
    }
}