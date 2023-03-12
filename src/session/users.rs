use crate::db_pool;
use super::{Session, extract_db, Error as GeneralError};

struct User {
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
        extract_db!(self, db_pool, db_pool_cloned);
        Ok(User::from(db_pool.get_user(name).await.map_err(|error| GeneralError::Db(error))?))
    }
}