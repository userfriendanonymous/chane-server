use serde::{Serialize, Deserialize};
use crate::{db_pool::{self, ChannelType}, session::roles::RolePermissionValidator};
use super::{Session, Error as GeneralError, extract_auth, extract_db, roles::get_user_role};

#[derive(Serialize, Deserialize)]
struct Channel {
    pub id: String,
    pub _type: ChannelType,
    pub roles: Vec<(String, String)>,
    pub default_role: String,
    pub labels: Vec<String>
}

impl From<db_pool::Channel> for Channel {
    fn from(model: db_pool::Channel) -> Self {
        Self {
            id: model.id.unwrap(),
            _type: model._type,
            roles: model.roles,
            default_role: model.default_role,
            labels: model.labels
        }
    }
}

impl Session {
    pub async fn create_channel(&self, _type: ChannelType, default_role: &String, labels: &Vec<String>) -> Result<String, GeneralError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_cloned);

        let id = db_pool.create_channel(&_type, &Vec::new(), default_role, labels).await.map_err(|error| GeneralError::Db(error))?;
        Ok(id)
    }

    pub async fn get_channel(&self, id: &str) -> Result<Channel, GeneralError> {
        extract_db!(self, db_pool, db_pool_cloned);
        Ok(Channel::from(db_pool.get_channel(id).await.map_err(GeneralError::Db)?))
    }

    pub async fn connect_block(&self, id: &str, block_id: &str) -> Result<(), GeneralError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_cloned);
        
        let (role, channel) = get_user_role(&db_pool, id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);

        if validator.can_connect_blocks() {
            return db_pool.connect_block_to_channel(block_id, id).await.map_err(GeneralError::Db);
        } else {
            return Err(GeneralError::Unauthorized("you don't have permissions to connect blocks".to_owned()))
        }
    }

    pub async fn disconnect_block(&self, id: &str, block_id: &str) -> Result<(), GeneralError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_cloned);
        
        let (role, channel) = get_user_role(&db_pool, id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);

        if validator.can_disconnect_blocks() {
            return db_pool.disconnect_block_from_channel(block_id, id).await.map_err(GeneralError::Db);
        } else {
            return Err(GeneralError::Unauthorized("you don't have permissions to disconnect blocks".to_owned()))
        }
    }
}