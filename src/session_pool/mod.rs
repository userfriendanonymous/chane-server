use crate::{db_pool::{self, DbPool}, auth_validator::{AuthValidator, Tokens, Auth, AuthInfo}, live_channel::LiveChannel, activity_logger::ActivityLogger, logger::Logger};
use std::sync::Arc;
pub use roles::{RoleWrappedError, CreateRoleError, Role, RoleError};
pub use blocks::Block;
pub use auth::{RegisterError, LoginError};
pub use channels::Channel;
pub use users::User;

mod auth;
mod users;
mod blocks;
mod channels;
mod groups;
mod roles;
mod live;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("db ( {0:?} )")]
    Db(db_pool::Error),
    #[error("unauthorized")]
    Unauthorized,
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
    auth: Auth,
    logger: Arc<Logger>
}

impl Session {
    pub fn new(tokens: &Tokens, db_pool: Arc<DbPool>, auth_validator: Arc<AuthValidator>, live_channel: Arc<LiveChannel>, activity_logger: Arc<ActivityLogger>, logger: Arc<Logger>) -> Self {
        let auth = auth_validator.tokens_as_auth(tokens);
        Self {
            db_pool,
            auth_validator,
            auth,
            live_channel,
            activity_logger,
            logger
        }
    }

    fn auth(&self) -> Result<&AuthInfo, Error> {
        self.auth.as_result().map_err(|_| Error::Unauthorized)
    }
}

pub struct SessionPool {
    db_pool: Arc<DbPool>,
    auth_validator: Arc<AuthValidator>,
    live_channel: Arc<LiveChannel>,
    activity_logger: Arc<ActivityLogger>,
    logger: Arc<Logger>
}

impl SessionPool {
    pub fn new(db_pool: Arc<DbPool>, auth_validator: Arc<AuthValidator>, live_channel: Arc<LiveChannel>, activity_logger: Arc<ActivityLogger>, logger: Arc<Logger>) -> Self {
        Self {db_pool, auth_validator, live_channel, activity_logger, logger}
    }

    pub fn spawn_session(&self, tokens: &Tokens) -> Session {
        Session::new(tokens, self.db_pool.clone(), self.auth_validator.clone(), self.live_channel.clone(), self.activity_logger.clone(), self.logger.clone())
    }
}