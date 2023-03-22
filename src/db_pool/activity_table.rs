use serde::{Serialize, Deserialize};
use mongodb::{bson::{doc, oid::ObjectId}};
use super::{DbPool, Error, utils::as_object_id};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Activity {
    ChannelCreated {
        by: String,
        id: String,
    },
    BlockCreated {
        by: String,
        id: String,
    },
    RoleCreated {
        by: String,
        id: String
    },
    UserJoined {
        name: String,
    },
    BlockChanged {
        by: String,
        id: String,
    },
    BlockConnected {
        by: String,
        id: String,
    },
    BlockConnectedToChannel {
        by: String,
        to: String,
        id: String,
    },
    BlockDisconnected {
        by: String,
        id: String,
    },
    BlockDisconnectedFromChannel {
        by: String,
        from: String,
        id: String,
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityTable {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    items: Vec<Activity>
}

impl DbPool {
    pub async fn create_activity_table(&self, items: &[Activity]) -> Result<String, Error> {
        let document = ActivityTable {
            id: None,
            items: items.to_vec()
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