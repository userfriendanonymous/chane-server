use crate::db_pool::{Activity as DbActivity, UserActivity, ChannelActivity};

pub enum ActivityTablesOf {
    User {name: String},
    Channel {id: String}
}

#[derive(Clone)]
pub enum Activity {
    BlockConnectedToChannel {
        block_id: String,
        id: String,
        by: String,
    },
    BlockDisconnectedFromChannel {
        block_id: String,
        id: String,
        by: String,
    },
    BlockPinnedOnChannel {
        block_id: Option<String>,
        id: String,
        by: String,
    },
    ChannelDescriptionChanged {
        id: String,
        by: String,
    },
    Joined {
        by: String,
    },
    RoleCreated {
        by: String,
        id: String,
    },
    ChannelLabelsChanged {
        id: String,
        by: String,
    },
    BlockCreated {
        id: String,
        by: String,
    }
}

impl Activity {
    pub fn process_into(self) -> Vec<(ActivityTablesOf, Vec<DbActivity>)> {
        match self {
            Self::BlockConnectedToChannel { block_id, id, by } => vec![
                (
                    ActivityTablesOf::User {name: by.clone()},
                    vec![DbActivity::User { activity: UserActivity::BlockConnectedToChannel { id: id.clone(), block_id: block_id.clone() } }]
                ), (
                    ActivityTablesOf::Channel {id},
                    vec![DbActivity::Channel { activity: ChannelActivity::BlockConnected { id: block_id, by } }]
                )
            ],
            Self::BlockDisconnectedFromChannel { block_id, id, by } => vec![
                (
                    ActivityTablesOf::User {name: by.clone()},
                    vec![DbActivity::User { activity: UserActivity::BlockDisconnectedFromChannel { id: id.clone(), block_id: block_id.clone() } }]
                ), (
                    ActivityTablesOf::Channel {id},
                    vec![DbActivity::Channel { activity: ChannelActivity::BlockDisconnected { id: block_id, by } }]
                )
            ],
            Self::BlockPinnedOnChannel { block_id, id, by } => vec![
                (
                    ActivityTablesOf::User {name: by.clone()},
                    vec![DbActivity::User { activity: UserActivity::ChannelBlockPinned { id: id.clone(), block_id: block_id.clone() } }]
                ), (
                    ActivityTablesOf::Channel {id},
                    vec![DbActivity::Channel { activity: ChannelActivity::BlockPinned { id: block_id, by } }]
                )
            ],
            _ => vec![]
        }
    }
}
