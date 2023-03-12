use crate::db_pool::{self, ChannelType};
use super::{Session, Error as GeneralError, extract_auth, extract_db};

struct Channel {
    id: String,
    _type: ChannelType,
}

impl From<db_pool::Channel> for Channel {
    fn from(model: db_pool::Channel) -> Self {
        Self {
            id: model.id.unwrap(),
            _type: model._type
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CreateError {
    #[error("general: {0}")]
    General(GeneralError)
}

impl Session {
    pub async fn create_channel(&self, _type: ChannelType) -> Result<String, CreateError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized, CreateError::General);
        extract_db!(self, db_pool, db_pool_cloned);

        let id = db_pool.create_channel(_type).await.map_err(|error| CreateError::General(GeneralError::Db(error)))?;
        Ok(id)
    }

    pub async fn get_channel(&self, id: &str) -> Result<Channel, GeneralError> {
        extract_db!(self, db_pool, db_pool_cloned);
        Ok(Channel::from(db_pool.get_channel(id).await.map_err(GeneralError::Db)?))
    }
}