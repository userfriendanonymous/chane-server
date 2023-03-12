use std::sync::{Arc, Mutex};
use mongodb::{options::ClientOptions, Client, Database, Collection};

mod blocks;
mod channels;
mod users;
mod utils;
mod groups;
mod roles;

pub use users::Model as User;
pub use channels::Model as Channel;
pub use blocks::Model as Block;
pub use groups::Model as Group;
pub use roles::Model as Role;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("db query error: {0}")]
    Query(mongodb::error::Error),
    #[error("failed to parse object id: {0}")]
    InvalidObjectId(mongodb::bson::oid::Error),
    #[error("not found")]
    NotFound,
}
pub struct DbPool {
    db: Database,
    blocks: Collection<Block>,
    users: Collection<User>,
    groups: Collection<Group>,
    roles: Collection<Role>,
    channels: Collection<Channel>
}

impl DbPool {
    pub async fn new_shared() -> mongodb::error::Result<DbPoolShared> {
        Ok(Arc::new(Mutex::new(Self::new().await?)))
    }

    pub async fn new() -> mongodb::error::Result<Self> {
        let client = Client::with_options(
            ClientOptions::parse("mongodb://localhost:27017").await?
        )?;
        let db = client
        .database("admin");
        Ok(Self {
            blocks: db.collection("blocks"),
            users: db.collection("users"),
            groups: db.collection("groups"),
            roles: db.collection("roles"),
            channels: db.collection("channels"),
            db
        })
    }
}

pub type DbPoolShared = Arc<Mutex<DbPool>>;