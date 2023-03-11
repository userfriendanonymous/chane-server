use crate::db_pool::{DbPoolShared, self};
use auth::Auth;
use self::auth::Tokens;

mod auth;
mod users;
mod blocks;
mod channels;

macro_rules! extract_db {
    ($self:expr, $db_pool:ident, $cloned:ident) => {
        let $cloned = $self.db_pool.clone();
        let $db_pool = $cloned.lock().unwrap();
    };
}
pub(self) use extract_db;

macro_rules! extract_auth {
    ($self:expr, $error:expr) => {
        $self.auth.into_result(|error| $error(GeneralError::Unauthorized(error.clone())))?
    };
}
pub(self) use extract_auth;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("db error: {0:?}")]
    Db(db_pool::Error),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
}

pub struct Instance {
    db_pool: DbPoolShared,
    auth_keys: auth::Keys,
    auth: Auth
}

impl Instance {
    pub fn new(db_pool: DbPoolShared, auth_keys: auth::Keys, tokens: Tokens) -> Self {
        Self {
            db_pool,
            auth_keys,
            auth: tokens.into_auth()
        }
    }
}