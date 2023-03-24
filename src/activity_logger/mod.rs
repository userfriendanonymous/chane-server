use std::sync::Arc;
use crate::{db_pool::{DbPool, Activity}, logger::Logger};
use tokio::sync::{mpsc, Mutex};

type Message = (String, Vec<Activity>);

pub struct ActivityLogger {
    db_pool: Arc<DbPool>,
    sender: mpsc::UnboundedSender<Message>,
    receiver: Mutex<mpsc::UnboundedReceiver<Message>>,
    logger: Arc<Logger>
}

impl ActivityLogger {
    pub fn new(db_pool: Arc<DbPool>, logger: Arc<Logger>) -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            db_pool,
            sender,
            receiver: Mutex::new(receiver),
            logger
        }
    }

    pub async fn log(&self, id: &str, items: &[Activity]){
        if let Err(error) = self.sender.send((id.to_string(), items.to_vec())) {
            self.logger.log(error.to_string().as_str());
        }
    }

    pub async fn run(&self){
        let mut receiver = self.receiver.lock().await;
        while let Some((id, items)) = receiver.recv().await {
            if let Err(error) = self.db_pool.push_to_activity_table(id.as_str(), &items).await {
                self.logger.log(error.to_string().as_str());
            }
        }
    }
}