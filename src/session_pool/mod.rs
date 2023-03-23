use crate::{db_pool::{self, DbPool}, auth_validator::{AuthValidator, Tokens, Auth, AuthInfo}, live_channel::LiveChannel, activity_logger::ActivityLogger};
use std::sync::Arc;
pub use roles::{RoleWrappedError, CreateRoleError};
pub use blocks::Block;
pub use auth::{RegisterError, LoginError};

mod auth;
mod users;
mod blocks;
mod channels;
mod groups;
mod roles;

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
    db_pool: Arc<DbPool>,
    live_channel: Arc<LiveChannel>,
    auth_validator: Arc<AuthValidator>,
    activity_logger: Arc<ActivityLogger>,
    auth: Auth
}

impl Session {
    pub fn new(db_pool: Arc<DbPool>, auth_validator: Arc<AuthValidator>, live_channel: Arc<LiveChannel>, activity_logger: ActivityLogger, tokens: &Tokens) -> Self {
        Self {
            db_pool,
            auth_validator,
            auth: auth_validator.tokens_as_auth(tokens),
            live_channel,
            activity_logger
        }
    }

    fn auth(&self) -> Result<&AuthInfo, Error> {
        self.auth.as_result().map_err(Error::Unauthorized)
    }

    fn auth_and_db(&self) -> Result<(Arc<DbPool>, &AuthInfo), Error> {
        Ok((self.db_pool, self.auth()?))
    }
}

pub struct SessionPool {
    db_pool: Arc<DbPool>,
    auth_validator: Arc<AuthValidator>,
    live_channel: Arc<LiveChannel>,
    activity_logger: Arc<ActivityLogger>,
}

impl SessionPool {
    pub fn new(db_pool: Arc<DbPool>, auth_validator: Arc<AuthValidator>, live_channel: Arc<LiveChannel>, activity_logger: Arc<ActivityLogger>){
        Self {db_pool, auth_validator, live_channel, activity_logger}
    }

    pub fn spawn_session(&self, tokens: &Tokens) -> Session {
        Session::new(self.db_pool.clone(), self.auth_validator.clone(), self.live_channel.clone(), self.activity_logger.clone(), tokens)
    }
}