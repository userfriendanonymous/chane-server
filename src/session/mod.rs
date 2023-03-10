use crate::db_pool::{DbPoolShared, self};

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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("db error: {0:?}")]
    Db(db_pool::Error)
}

pub struct Instance {
    db_pool: DbPoolShared
}