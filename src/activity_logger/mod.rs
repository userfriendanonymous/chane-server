use std::sync::Arc;
use crate::db_pool::{DbPool, Activity};
use tokio::sync::mpsc;

type Message = (String, Vec<Activity>);

pub struct ActivityLogger {
    db_pool: Arc<DbPool>,
    sender: mpsc::UnboundedSender<Message>,
    receiver: mpsc::UnboundedReceiver<Message>
}

impl ActivityLogger {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            db_pool,
            sender,
            receiver
        }
    }

    pub async fn log(&self, id: &str, items: &[Activity]){
        self.sender.send((id.to_string(), items.to_vec()));
    }

    pub async fn run(&self){
        while let Some((id, items)) = self.receiver.recv().await {
            self.db_pool.push_to_activity_table(id.as_str(), &items).await;
        }
    }
}