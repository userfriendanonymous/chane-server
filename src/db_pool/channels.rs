use serde::{Serialize, Deserialize};
use super::{DbPool, Error, utils::as_object_id};
use mongodb::bson::{doc, oid::ObjectId};

#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub _type: ChannelType,
    pub roles: Vec<(String, String)>,
    pub default_role: String,
    pub labels: Vec<String>,
    pub description: String,
    pub pinned_block: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ChannelType {
    ServerHosted = 0,
    Ghosted = 1,
}

impl DbPool {
    pub async fn get_channel(&self, id: &str) -> Result<Channel, Error> {
        let filter = doc! {"id": as_object_id!(id)};
        let result = self.channels.find_one(filter, None).await?;
        match result {
            Some(model) => Ok(model),
            None => Err(Error::NotFound)
        }
    }

    pub async fn create_channel(&self, _type: &ChannelType, description: &String, roles: &Vec<(String, String)>, default_role: &String, labels: &Vec<String>) -> Result<String, Error> {
        let model = Channel {
            id: None,
            _type: _type.clone(),
            roles: roles.clone(),
            default_role: default_role.clone(),
            labels: labels.clone(),
            description: description.clone(),
            pinned_block: "".to_owned()
        };
        let result = self.channels.insert_one(model, None).await?;
        Ok(result.inserted_id.to_string())
    }

    pub async fn pin_channel_block(&self, id: &str, block_id: &Option<String>) -> Result<(), Error> {
        let empty = "".to_owned();
        let pinned_block_id = match block_id {
            Some(id) => id,
            None => &empty
        };

        let result = self.channels.update_one(doc! {
            "id": as_object_id!(id)
        }, doc! {
            "$set": {
                "pinned_block": pinned_block_id
            }
        }, None).await?;

        if result.modified_count <= 0 {
            Err(Error::NotFound)
        } else {
            Ok(())
        }
    }

    pub async fn change_channel_description(&self, id: &str, description: &String) -> Result<(), Error> {
        let result = self.channels.update_one(doc! {"_id": as_object_id!(id)}, doc! {
            "$set": {
                "description": description
            }
        }, None).await?;
        if result.modified_count <= 0 {
            Err(Error::NotFound)
        } else {
            Ok(())
        }
    }

    pub async fn set_channel_labels(&self, id: &str, labels: &Vec<String>) -> Result<(), Error> {
        let result = self.channels.update_one(doc! {"_id": as_object_id!(id)}, doc! {
            "$set": {
                "labels": labels
            }
        }, None).await?;
        if result.modified_count <= 0 {
            Err(Error::NotFound)
        } else {
            Ok(())
        }
    }
}