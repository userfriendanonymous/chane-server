use crate::{db_pool::{self, DbPool}, shared::Shared};
use auth::Auth;
use self::auth::{Tokens, AuthInfo};
use std::sync::Arc;
pub use roles::{RoleWrappedError, CreateRoleError};
pub use blocks::Block;
pub use live_channel::{LiveChannel, LiveMessage};
pub use auth::{RegisterError, LoginError, Keys as AuthKeys, Tokens as AuthTokens};

mod auth;
mod users;
mod blocks;
mod channels;
mod groups;
mod roles;
mod live_channel;

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

pub struct Session<LC: LiveChannel> {
    db_pool: Arc<DbPool>,
    live_channel: Shared<LC>,
    auth_keys: auth::Keys,
    auth: Auth
}

impl<LC: LiveChannel> Session<LC> {
    pub fn new(db_pool: Arc<DbPool>, auth_keys: auth::Keys, tokens: Tokens, live_channel: Shared<LC>) -> Self {
        Self {
            db_pool,
            auth_keys: auth_keys.clone(),
            auth: tokens.into_auth(auth_keys),
            live_channel
        }
    }

    fn db_pool(&self) -> Arc<DbPool> {
        self.db_pool.clone()
    }

    fn auth(&self) -> Result<&AuthInfo, Error> {
        self.auth.as_result().map_err(Error::Unauthorized)
    }

    fn auth_and_db(&self) -> Result<(Arc<DbPool>, &AuthInfo), Error> {
        Ok((self.db_pool(), self.auth()?))
    }
}