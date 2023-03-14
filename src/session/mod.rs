use std::sync::Arc;
use tokio::sync::Mutex;
use crate::db_pool::{DbPoolShared, self};
use auth::Auth;
use self::auth::Tokens;
pub use roles::{RoleWrappedError, CreateRoleError};
pub use blocks::Block;

mod auth;
mod users;
mod blocks;
mod channels;
mod groups;
mod roles;

macro_rules! extract_db {
    ($self:expr, $db_pool:ident, $cloned:ident) => {
        let $cloned = $self.db_pool.clone();
        let $db_pool = $cloned.lock().await;
    };
}
pub(self) use extract_db;

macro_rules! extract_auth {
    ($self:expr, $error:expr, $error2:expr) => {
        $self.auth.as_result(|error| $error2($error(error.clone())))?
    };
    ($self:expr, $error:expr) => {
        $self.auth.as_result(|error| $error(error.clone()))?
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

impl From<db_pool::Error> for Error {
    fn from(value: db_pool::Error) -> Self {
        Self::Db(value)
    }
}

pub struct Session {
    db_pool: DbPoolShared,
    auth_keys: auth::Keys,
    auth: Auth
}

pub type SessionShared = Arc<Mutex<Session>>;

impl Session {
    pub fn new(db_pool: DbPoolShared, auth_keys: auth::Keys, tokens: Tokens) -> Self {
        Self {
            db_pool,
            auth_keys,
            auth: tokens.into_auth()
        }
    }

    pub fn new_shared(db_pool: DbPoolShared, auth_keys: auth::Keys, tokens: Tokens) -> SessionShared {
        Arc::new(Mutex::new(Self::new(db_pool, auth_keys, tokens)))
    }
}