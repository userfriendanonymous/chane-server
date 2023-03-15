use serde::{Serialize, Deserialize};
use crate::{db_pool::{self, ChannelType}, session::{roles::RolePermissionValidator, LiveMessage}};
use super::{Session, Error as GeneralError, extract_auth, extract_db, roles::{resolve_user_role, RoleWrappedError}, LiveChannel};

#[derive(Serialize, Deserialize)]
pub struct Channel {
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

impl<LC: LiveChannel> Session<LC> {
    pub async fn create_channel(&self, _type: &ChannelType, description: &str, default_role: &str, labels: &[String]) -> Result<String, GeneralError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_cloned);

        let id = db_pool.create_channel(_type, description, &Vec::new(), default_role, labels).await?;
        Ok(id)
    }

    pub async fn get_channel(&self, id: &str) -> Result<Channel, GeneralError> {
        extract_db!(self, db_pool, db_pool_cloned);
        Ok(Channel::from(db_pool.get_channel(id).await?))
    }

    pub async fn connect_block_to_channel(&self, id: &str, block_id: &str) -> Result<(), RoleWrappedError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized, RoleWrappedError::General);
        extract_db!(self, db_pool, db_pool_cloned);
        
        let (role, channel, errors) = resolve_user_role(&db_pool, id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);

        if validator.can_connect_blocks() {
            db_pool.connect_block_to_channel(block_id, id).await?;
            let mut live_channel = self.live_channel.lock().await;
            live_channel.receive_message(id, &LiveMessage::BlockConnected { id: block_id.to_string() }).await;
            Ok(())
        } else {
            Err(GeneralError::Unauthorized("you don't have permissions to connect blocks".to_owned()).into())
        }
    }

    pub async fn disconnect_block_from_channel(&self, id: &str, block_id: &str) -> Result<(), RoleWrappedError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_cloned);
        
        let (role, channel, errors) = resolve_user_role(&db_pool, id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);

        if validator.can_disconnect_blocks() {
            db_pool.disconnect_block_from_channel(block_id, id).await?;
            let mut live_channel = self.live_channel.lock().await;
            live_channel.receive_message(id, &LiveMessage::BlockDisconnected { id: block_id.to_string() }).await;
            Ok(())
        } else {
            Err(GeneralError::Unauthorized("you don't have permissions to disconnect blocks".to_owned()).into())
        }
    }

    pub async fn pin_channel_block(&self, id: &str, block_id: &Option<String>) -> Result<(), RoleWrappedError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_shared);
        let (role, channel, errors) = resolve_user_role(&db_pool, id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);

        if validator.can_pin_block() {
            db_pool.pin_channel_block(id, block_id).await?;
            let mut live_channel = self.live_channel.lock().await;
            live_channel.receive_message(id, &LiveMessage::BlockPinned { id: block_id.clone() }).await;
            Ok(())
        } else {
            Err(GeneralError::Unauthorized("you don't have permission to pin block".to_owned()).into())
        }
    }

    pub async fn change_channel_description(&self, id: &str, description: &str) -> Result<(), RoleWrappedError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_shared);
        let (role, channel, errors) = resolve_user_role(&db_pool, id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);
        if validator.can_change_description() {
            db_pool.change_channel_description(id, description).await?;
            let mut live_channel = self.live_channel.lock().await;
            live_channel.receive_message(id, &LiveMessage::DescriptionChanged).await;
            Ok(())
        } else {
            Err(GeneralError::Unauthorized("you don't have permissions to change channel description".to_owned()).into())
        }
    }

    pub async fn change_channel_labels(&self, id: &str, labels: &[String]) -> Result<(), RoleWrappedError> {
        let auth = extract_auth!(self, GeneralError::Unauthorized);
        extract_db!(self, db_pool, db_pool_shared);
        let (role, channel, errors) = resolve_user_role(&db_pool, id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);
        if validator.can_set_labels() {
            db_pool.change_channel_labels(id, labels).await?;
            let mut live_channel = self.live_channel.lock().await;
            live_channel.receive_message(id, &LiveMessage::LabelsChanged).await;
            Ok(())
        } else {
            Err(GeneralError::Unauthorized("you don't have permissions to set channel labels".to_owned()).into())
        }
    }
}