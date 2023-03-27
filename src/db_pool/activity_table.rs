use serde::{Serialize, Deserialize};
use mongodb::{bson::{doc, oid::ObjectId}};
use super::{DbPool, Error, utils::as_object_id};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
#[serde(tag = "type", content = "data")]
pub enum Activity {
    User {
        activity: UserActivity
    },
    Channel {
        activity: ChannelActivity
    },
    Global {
        activity: GlobalActivity
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
#[serde(tag = "type", content = "data")]
pub enum UserActivity {
    ChannelCreated {id: String},
    BlockCreated {id: String},
    Joined,
    RoleCreated {id: String},
    ChannelBlockPinned {block_id: Option<String>, id: String},
    ChannelDescriptionChanged {id: String},
    BlockConnectedToChannel {block_id: String, id: String},
    BlockDisconnectedFromChannel {block_id: String, id: String},
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
#[serde(tag = "type", content = "data")]
pub enum ChannelActivity {
    Created,
    BlockConnected {by: String, id: String},
    BlockDisconnected {by: String, id: String},
    BlockPinned {by: String, id: Option<String>},
    DescriptionChanged {by: String},
    LabelsChanged {by: String},
    RolesChanged {by: String},
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export)]
#[serde(tag = "type", content = "data")]
pub enum GlobalActivity {
    ChannelCreated {by: String, id: String},
    BlockCreated {by: String, id: String},
    Joined {by: String},
    ChannelBlockPinned {by: String, id: Option<String>, channel_id: String},
    ChannelDescriptionChanged {by: String, id: String},
    BlockChanged {by: String, id: String},
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityTable {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    items: Vec<Activity>
}

impl DbPool {
    pub async fn create_activity_table(&self) -> Result<String, Error> {
        let document = ActivityTable {
            id: None,
            items: Vec::new()
        };
        let result = self.activity_tables.insert_one(document, None).await?;
        Ok(result.inserted_id.to_string())
    }

    pub async fn get_activity_table(&self, id: &str) -> Result<ActivityTable, Error> {
        match self.activity_tables.find_one(doc! {
            "_id": as_object_id!(id)
        }, None).await? {
            Some(table) => Ok(table),
            None => Err(Error::NotFound)
        }
    }

    pub async fn push_to_activity_table(&self, id: &str, items: &[Activity]) -> Result<(), Error> {
        let items_bson = mongodb::bson::to_bson(items)?;
        let result = self.activity_tables.update_one(doc! {
            "_id": as_object_id!(id)
        }, doc! {
            "$push": {
                "items": {
                    "$each": items_bson,
                    "$slice": -50
                }
            }
        }, None).await?;
        if result.modified_count == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }
}