use serde::{Serialize, Deserialize};
use super::{DbPool, Error, utils::as_object_id};
use mongodb::bson::{doc, oid::ObjectId};
#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl DbPool {
    pub async fn get_channel(&self, id: &str) -> Result<Model, Error> {
        let filter = doc! {"id": as_object_id!(id)};
        let result = self.channels.find_one(filter, None).await.map_err(Error::Query)?;
        match result {
            Some(model) => Ok(model),
            None => Err(Error::NotFound)
        }
    }

    pub async fn create_channel(&self) -> Result<String, Error> {
        let model = Model {
            id: None
        };
        let result = self.channels.insert_one(model, None).await.map_err(Error::Query)?;
        Ok(result.inserted_id.to_string())
    }
}