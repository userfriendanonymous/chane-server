#![allow(clippy::too_many_arguments)]
use serde::{Serialize, Deserialize};
use ts_rs::TS;
use super::{DbPool, Error, utils::as_obj_id};
use mongodb::bson::{doc, oid::ObjectId};

#[derive(Debug, Serialize, Deserialize)]
pub struct Channel {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    #[serde(rename = "type")]
    pub _type: ChannelType,
    pub roles: Vec<(String, String)>,
    pub default_role: String,
    pub labels: Vec<String>,
    pub description: String,
    pub pinned_block: String,
    pub title: String,
    pub activity_table: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
#[ts(export)]
pub enum ChannelType {
    #[serde(rename = "server_hosted")]
    ServerHosted,
    #[serde(rename = "ghosted")]
    Ghosted,
}

impl DbPool {
    pub async fn get_channel(&self, id: &str) -> Result<Channel, Error> {
        let filter = doc! {"id": as_obj_id(id)?};
        let result = self.channels.find_one(filter, None).await?;
        match result {
            Some(model) => Ok(model),
            None => Err(Error::NotFound)
        }
    }

    pub async fn create_channel(&self, _type: &ChannelType, title: &str, description: &str, roles: &[(String, String)], default_role: &str, labels: &[String], activity_table: &str) -> Result<String, Error> {
        let model = Channel {
            id: None,
            _type: _type.clone(),
            roles: roles.to_owned(),
            default_role: default_role.to_owned(),
            labels: labels.to_owned(),
            description: description.to_owned(),
            pinned_block: "".to_owned(),
            title: title.to_string(),
            activity_table: activity_table.to_string()
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
            "id": as_obj_id(id)?
        }, doc! {
            "$set": {
                "pinned_block": pinned_block_id
            }
        }, None).await?;

        if result.modified_count == 0 {
            Err(Error::NotFound)
        } else {
            Ok(())
        }
    }

    pub async fn change_channel_description(&self, id: &str, description: &str) -> Result<(), Error> {
        let result = self.channels.update_one(doc! {"_id": as_obj_id(id)?}, doc! {
            "$set": {
                "description": description
            }
        }, None).await?;
        if result.modified_count == 0 {
            Err(Error::NotFound)
        } else {
            Ok(())
        }
    }

    pub async fn change_channel_labels(&self, id: &str, labels: &[String]) -> Result<(), Error> {
        let result = self.channels.update_one(doc! {"_id": as_obj_id(id)?}, doc! {
            "$set": {
                "labels": labels
            }
        }, None).await?;
        if result.modified_count == 0 {
            Err(Error::NotFound)
        } else {
            Ok(())
        }
    }
}