use serde::{Serialize, Deserialize};
use ts_rs::TS;
use crate::{db_pool::{self, ChannelType}, session_pool::{roles::RolePermissionValidator, Block}, live_channel::LiveMessage, activity_logger::Activity};
use super::{Session, Error as GeneralError, roles::{resolve_user_role, RoleWrappedError}};

#[derive(Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Channel {
    pub id: String,
    #[serde(rename = "type")]
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
        let auth = self.auth()?;

        let activity_table_id = self.db_pool.create_activity_table().await?;
        let id = self.db_pool.create_channel(_type, title, description, &Vec::new(), default_role, labels, &activity_table_id).await?;
        Ok(id)
    }

    pub async fn get_channel(&self, id: &str) -> Result<Channel, GeneralError> {
        Ok(Channel::from(self.db_pool.get_channel(id).await?))
    }

    pub async fn connect_block_to_channel(&self, id: &str, block_id: &str) -> Result<(), RoleWrappedError> {
        let auth = self.auth()?;
        
        let (role, channel) = resolve_user_role(self.db_pool.clone(), id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);

        if validator.can_connect_blocks() {
            self.db_pool.connect_block_to_channel(block_id, id).await?;
            self.live_channel.receive_message(id, &LiveMessage::BlockConnected { id: block_id.to_string() });

            self.activity_logger.log(Activity::BlockConnectedToChannel { block_id: block_id.to_string(), id: id.to_string(), by: auth.name.clone() });
            Ok(())
        } else {
            Err(GeneralError::Unauthorized.into())
        }
    }

    pub async fn disconnect_block_from_channel(&self, id: &str, block_id: &str) -> Result<(), RoleWrappedError> {
        let auth = self.auth()?;
        
        let (role, channel) = resolve_user_role(self.db_pool.clone(), id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);

        if validator.can_disconnect_blocks() {
            self.db_pool.disconnect_block_from_channel(block_id, id).await?;
            self.live_channel.receive_message(id, &LiveMessage::BlockDisconnected { id: block_id.to_string() });

            self.activity_logger.log(Activity::BlockDisconnectedFromChannel { block_id: block_id.to_string(), id: id.to_string(), by: auth.name.to_string() });
            Ok(())
        } else {
            Err(GeneralError::Unauthorized.into())
        }
    }

    pub async fn pin_channel_block(&self, id: &str, block_id: &Option<String>) -> Result<(), RoleWrappedError> {
        let auth = self.auth()?;
        let (role, channel) = resolve_user_role(self.db_pool.clone(), id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);

        if validator.can_pin_block() {
            self.db_pool.pin_channel_block(id, block_id).await?;
            self.live_channel.receive_message(id, &LiveMessage::BlockPinned { id: block_id.clone() });

            self.activity_logger.log(Activity::BlockPinnedOnChannel { block_id: block_id.clone(), id: id.to_string(), by: auth.name.clone() });
            Ok(())
        } else {
            Err(GeneralError::Unauthorized.into())
        }
    }

    pub async fn change_channel_description(&self, id: &str, description: &str) -> Result<(), RoleWrappedError> {
        let auth = self.auth()?;
        let (role, channel) = resolve_user_role(self.db_pool.clone(), id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);
        if validator.can_change_description() {
            self.db_pool.change_channel_description(id, description).await?;
            self.live_channel.receive_message(id, &LiveMessage::DescriptionChanged);

            self.activity_logger.log(Activity::ChannelDescriptionChanged { id: id.to_string(), by: auth.name.clone() });
            Ok(())
        } else {
            Err(GeneralError::Unauthorized.into())
        }
    }

    pub async fn change_channel_labels(&self, id: &str, labels: &[String]) -> Result<(), RoleWrappedError> {
        let auth = self.auth()?;
        let (role, channel) = resolve_user_role(self.db_pool.clone(), id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);
        if validator.can_set_labels() {
            self.db_pool.change_channel_labels(id, labels).await?;
            self.live_channel.receive_message(id, &LiveMessage::LabelsChanged);
            self.activity_logger.log(Activity::ChannelLabelsChanged { id: id.to_string(), by: auth.name.clone() });
            Ok(())
        } else {
            Err(GeneralError::Unauthorized.into())
        }
    }

    pub async fn get_channel_blocks(&self, id: &str, limit: &Option<i64>, offset: &Option<u64>) -> Result<(Vec<Block>, Vec<mongodb::error::Error>), RoleWrappedError> {
        let auth = self.auth()?;
        let (role, channel) = resolve_user_role(self.db_pool.clone(), id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);

        if validator.can_view_blocks() {
            let (db_blocks, blocks_errors) = self.db_pool.get_channel_blocks(id, limit, offset).await?;
            let mut blocks = Vec::new();
            for block in db_blocks {
                blocks.push(Block::from(block));
            }
            Ok((blocks, blocks_errors))
            
        } else {
            Err(GeneralError::Unauthorized.into())
        }
    }
}