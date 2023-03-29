use serde::{Serialize, Deserialize};
use ts_rs::TS;
use crate::db_pool;
use super::{Session, Error as GeneralError};

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct User {
    name: String,
}

impl From<db_pool::User> for User {
    fn from(user: db_pool::User) -> Self {
        Self {
            name: user.name
        }
    }
}

impl Session {
    pub async fn get_user(&self, name: &str) -> Result<User, GeneralError> {
        Ok(User::from(self.db_pool.get_user(name).await?))
    }
}