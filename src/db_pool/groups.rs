use serde::{Serialize, Deserialize};
use crate::db_pool::utils::as_object_id;
use super::{DbPool, Error};
use mongodb::bson::{doc, oid::ObjectId};

#[derive(Serialize, Deserialize, Debug)]
pub struct Group {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub owner: String,
    pub editors: Vec<String>,
    pub extends: Vec<String>,
    pub names: Vec<String>,
}

impl DbPool {
    pub async fn get_group(&self, id: &str) -> Result<Group, Error> {
        match self.groups.find_one(doc! {"_id": as_object_id!(id)}, None).await.map_err(Error::Query)? {
            Some(model) => Ok(model),
            None => Err(Error::NotFound)
        }
    }

    pub async fn create_group(&self, owner: String, editors: Vec<String>, extends: Vec<String>, names: Vec<String>) -> Result<String, Error> {
        let model = Group {
            id: None,
            owner,
            editors,
            extends,
            names
        };
        let result = self.groups.insert_one(model, None).await.map_err(Error::Query)?;
        Ok(result.inserted_id.to_string())
    }

    pub async fn update_group(&self, id: String, editors: Option<Vec<String>>, extends: Vec<String>, names: Vec<String>) -> Result<(), Error> {
        let result = self.groups.update_one(doc! {"_id": id}, doc! {"$set": {
            "editors": editors,
            "extends": extends,
            "names": names
        }}, None).await.map_err(Error::Query)?;
        Ok(())
    }
}