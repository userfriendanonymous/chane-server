use serde::{Serialize, Deserialize};
use crate::db_pool;

use super::{Error as GeneralError, extract_db, extract_auth, Session};

#[derive(Serialize, Deserialize)]
struct Group {
    pub id: String,
    pub owner: String,
    pub editors: Vec<String>,
    pub extends: Vec<String>,
    pub names: Vec<String>,
}

impl From<db_pool::Group> for Group {
    fn from(group: db_pool::Group) -> Self {
        Self {
            editors: group.editors,
            id: group.id.unwrap(),
            extends: group.extends,
            names: group.names,
            owner: group.owner
        }
    }
}

impl Session {
    pub async fn get_group(&self, id: &str) -> Result<Group, GeneralError> {
        extract_db!(self, db_pool, db_pool_cloned);
        Ok(Group::from(db_pool.get_group(id).await.map_err(GeneralError::Db)?))
    }

    pub async fn create_group(&self, editors: &Vec<String>, extends: &Vec<String>, names: &Vec<String>) -> Result<String, GeneralError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_cloned);
        db_pool.create_group(&auth.name, editors, extends, names).await.map_err(GeneralError::Db)
    }
}