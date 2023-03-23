use serde::{Serialize, Deserialize};
use crate::db_pool;
use super::{Session, Error as GeneralError};

#[derive(Serialize, Deserialize)]
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
        let db_pool = self.db_pool();
        Ok(User::from(db_pool.get_user(name).await?))
    }
}