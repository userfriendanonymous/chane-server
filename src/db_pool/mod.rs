use tokio::sync::MutexGuard;
use mongodb::{options::ClientOptions, Client, Database, Collection};

pub use users::User;
pub use channels::{Channel, ChannelType};
pub use blocks::Block;
pub use roles::{Role, RolePermissions};
pub use activity_table::{ActivityTable, Activity};

mod blocks;
mod channels;
mod users;
mod utils;
mod groups;
mod roles;
mod activity_table;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("db query error: {0}")]
    Query(mongodb::error::Error),
    #[error("failed to parse object id: {0}")]
    InvalidObjectId(mongodb::bson::oid::Error),
    #[error("not found")]
    NotFound,
    #[error("serialization error: {0}")]
    BsonSerialization(mongodb::bson::ser::Error)
}

impl From<mongodb::bson::ser::Error> for Error {
    fn from(value: mongodb::bson::ser::Error) -> Self {
        Self::BsonSerialization(value)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(value: mongodb::error::Error) -> Self {
        Self::Query(value)
    }
}

impl From<mongodb::bson::oid::Error> for Error {
    fn from(value: mongodb::bson::oid::Error) -> Self {
        Self::InvalidObjectId(value)
    }
}

pub struct DbPool {
    db: Database,
    blocks: Collection<Block>,
    users: Collection<User>,
    roles: Collection<Role>,
    channels: Collection<Channel>,
    activity_tables: Collection<ActivityTable>,
}

impl DbPool {
    pub async fn new() -> mongodb::error::Result<Self> {
        let client = Client::with_options(
            ClientOptions::parse("mongodb://localhost:27017").await?
        )?;
        let db = client
        .database("admin");
        Ok(Self {
            blocks: db.collection("blocks"),
            users: db.collection("users"),
            roles: db.collection("roles"),
            channels: db.collection("channels"),
            activity_tables: db.collection("activity_tables"),
            db
        })
    }
}

pub type DbPoolGuard<'a> = MutexGuard<'a, DbPool>;