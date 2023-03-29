use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{live_channel::{self, LiveChannel}, logger::Logger};
use super::{Session, roles::{resolve_user_role, RolePermissionValidator}, Error as GeneralError, RoleWrappedError};

pub struct Handle {
    live_channel: Arc<LiveChannel>,
    channel_id: String,
    handle: Mutex<Option<live_channel::Handle>>,
    logger: Arc<Logger>
}

impl Handle {
    pub async fn connect(&self, peer: Arc<dyn live_channel::Peer + Send + Sync>){
        let handle = self.live_channel.connect(peer, &self.channel_id).await;
        *self.handle.lock().await = Some(handle);
    }

    pub async fn disconnect(self){
        if let Err(error) = match &*self.handle.lock().await {
            Some(handle) => self.live_channel.disconnect(handle.clone()).await,
            None => panic!("[live channel session failed to find handle, called disconnected without connecting]")
        } { self.logger.log(error.to_string()) }
    }
}

impl Session {
    pub async fn live(&self, channel_id: &str) -> Result<Handle, RoleWrappedError> {
        // TO BE REMOVED -------------------
        let handle = Handle {
            live_channel: self.live_channel.clone(),
            logger: self.logger.clone(),
            channel_id: channel_id.to_owned(),
            handle: Mutex::new(None)
        };
        return Ok(handle);
        // TO BE REMOVED -------------------

        let auth = self.auth()?;
        let (role, channel) = resolve_user_role(self.db_pool.clone(), channel_id, &auth.name).await?;
        let validator = RolePermissionValidator::new(&role.permissions, &channel.labels);

        if !validator.can_live() {
            return Err(GeneralError::Unauthorized.into());
        }
        
        let handle = Handle {
            live_channel: self.live_channel.clone(),
            logger: self.logger.clone(),
            channel_id: channel_id.to_owned(),
            handle: Mutex::new(None)
        };
        Ok(handle)
    }
}