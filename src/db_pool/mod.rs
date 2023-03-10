use std::sync::{Arc, Mutex};
use surrealdb::{Datastore, Session};

mod blocks;
mod channels;
mod users;
mod utils;

pub use users::User;
pub use channels::Channel;
pub use blocks::Block;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("db query error: {0}")]
    Query(String),
    #[error("db objects not found")]
    ObjectsNotFound,
    #[error("surreal value expected to be object but is not")]
    ValueShouldBeObject,
    #[error("failed to extract first object (no objects were found)")]
    FailedToExtractFirstObject,
    #[error("object error: {0}")]
    ObjectFailure(String),
    #[error("failed to convert surreal type to custom struct: {0}")]
    Conversion(String)
}
pub struct DbPool {
    datastore: Datastore,
    session: Session
}

impl DbPool {
    pub async fn new_shared() -> DbPoolShared {
        Arc::new(Mutex::new(Self::new().await))
    }

    pub async fn new() -> Self {
        let datastore = Datastore::new("memory").await.unwrap();
        let session = Session::for_db("ns", "db");

        Self {
            datastore,
            session
        }
    }
}

pub type DbPoolShared = Arc<Mutex<DbPool>>;