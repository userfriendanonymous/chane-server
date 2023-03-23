use serde::{Serialize, Deserialize};
use crate::{db_pool::{self, ChannelType, Activity}, session_pool::{roles::RolePermissionValidator, Block}, live_channel::LiveMessage};
use super::{Session, Error as GeneralError, roles::{resolve_user_role, RoleWrappedError}};

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

impl Session {
    pub async fn create_channel(&self, _type: &ChannelType, title: &str, description: &str, default_role: &str, labels: &[String]) -> Result<String, GeneralError> {
        let (db_pool, auth) = self.auth_and_db()?;

        let id = db_pool.create_channel(_type, title, description, &Vec::new(), default_role, labels).await?;
        Ok(id)
    }

    pub async fn get_channel(&self, id: &str) -> Result<Channel, GeneralError> {
        let db_pool = self.db_pool();
        Ok(Channel::from(db_pool.get_channel(id).await?))
    }

    pub async fn connect_block_to_channel(&self, id: &str, block_id: &str) -> Result<(), RoleWrappedError> {
        let (db_pool, auth) = self.auth_and_db()?;
        
        let (role, channel) = resolve_user_role(db_pool.clone(), id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);

        if validator.can_connect_blocks() {
            db_pool.connect_block_to_channel(block_id, id).await?;
            let mut live_channel = self.live_channel.lock().await;
            live_channel.receive_message(id, &LiveMessage::BlockConnected { id: block_id.to_string() }).await;

            db_pool.push_to_activity_table(&auth.activity_table_id, &[
                Activity::BlockConnected { by: auth.name.clone(), id: id.to_string() }
            ]).await?;

            Ok(())
        } else {
            Err(GeneralError::Unauthorized("you don't have permissions to connect blocks".to_owned()).into())
        }
    }

    pub async fn disconnect_block_from_channel(&self, id: &str, block_id: &str) -> Result<(), RoleWrappedError> {
        let (db_pool, auth) = self.auth_and_db()?;
        
        let (role, channel) = resolve_user_role(db_pool.clone(), id, &auth.name).await?;
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
        let (db_pool, auth) = self.auth_and_db()?;
        let (role, channel) = resolve_user_role(db_pool.clone(), id, &auth.name).await?;
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
        let (db_pool, auth) = self.auth_and_db()?;
        let (role, channel) = resolve_user_role(db_pool.clone(), id, &auth.name).await?;
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
        let (db_pool, auth) = self.auth_and_db()?;
        let (role, channel) = resolve_user_role(db_pool.clone(), id, &auth.name).await?;
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

    pub async fn get_channel_blocks(&self, id: &str, limit: &Option<i64>, offset: &Option<u64>) -> Result<(Vec<Block>, Vec<mongodb::error::Error>), RoleWrappedError> {
        let (db_pool, auth) = self.auth_and_db()?;
        let (role, channel) = resolve_user_role(db_pool.clone(), id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);

        if validator.can_view_blocks() {
            let (db_blocks, blocks_errors) = db_pool.get_channel_blocks(id, limit, offset).await?;
            let mut blocks = Vec::new();
            for block in db_blocks {
                blocks.push(Block::from(block));
            }
            Ok((blocks, blocks_errors))
            
        } else {
            Err(GeneralError::Unauthorized("you don't have permissions to view blocks of this channel".to_owned()).into())
        }
    }
}