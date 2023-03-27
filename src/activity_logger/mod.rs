use std::sync::Arc;
use crate::{db_pool::{DbPool, self}, logger::Logger};
use tokio::sync::{mpsc, Mutex};
use activity::ActivityTablesOf;
pub use activity::Activity;

mod activity;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("db: {0}")]
    Db(db_pool::Error)
}
impl From<db_pool::Error> for Error {
    fn from(value: db_pool::Error) -> Self {
        Self::Db(value)
    }
}

type Message = Activity;

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

    pub fn log(&self, activity: Activity){
        if let Err(error) = self.sender.send(activity) {
            self.logger.log(error.to_string());
        }
    }

    async fn resolve_activity_tables_of(&self, activity_tables_of: &ActivityTablesOf) -> Result<Vec<String>, Error> {
        match activity_tables_of {
            ActivityTablesOf::User { name } => {
                let user = self.db_pool.get_user(name).await?;
                Ok(vec![user.activity_table])
            },
            ActivityTablesOf::Channel { id } => {
                let channel = self.db_pool.get_channel(id).await?;
                Ok(vec![channel.activity_table])
            }
        }
    }

    pub async fn run(&self){
        let mut receiver = self.receiver.lock().await;
        while let Some(activity) = receiver.recv().await {
            for (activity_tables_of, activities) in activity.process_into() {
                match self.resolve_activity_tables_of(&activity_tables_of).await {
                    Err(e) => self.logger.log(e.to_string()),
                    Ok(activity_tables_ids) => {
                        for activity_table_id in activity_tables_ids {
                            if let Err(e) = self.db_pool.push_to_activity_table(&activity_table_id, &activities).await {
                                self.logger.log(e.to_string());
                            }
                        }
                    }
                }
            }
        }
    }
}

