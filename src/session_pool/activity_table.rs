use serde::Serialize;
use ts_rs::TS;
use crate::db_pool::{Activity, self};
use super::{Session, Error as GeneralError};

#[derive(Serialize, TS)]
#[ts(export)]
pub struct ActivityTable {
    id: String,
    items: Vec<Activity>
}
impl From<db_pool::ActivityTable> for ActivityTable {
    fn from(value: db_pool::ActivityTable) -> Self {
        Self {
            id: value.id.unwrap().to_string(),
            items: value.items
        }
    }
}

impl Session {
    pub async fn get_activity_table(&self, id: &str) -> Result<ActivityTable, GeneralError> {
        Ok(self.db_pool.get_activity_table(id).await?.into())
    }
}