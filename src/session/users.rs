use serde::{Serialize, Deserialize};
use crate::db_pool;
use super::{Session, extract_db, Error as GeneralError};

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

impl<LC> Session<LC> {
    pub async fn get_user(&self, name: &str) -> Result<User, GeneralError> {
        extract_db!(self, db_pool, db_pool_cloned);
        Ok(User::from(db_pool.get_user(name).await?))
    }
}