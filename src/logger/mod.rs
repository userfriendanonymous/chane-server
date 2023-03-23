use std::fmt;
use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use tokio::fs;

type Message = dyn fmt::Display;

#[derive(Default)]
pub struct Logger {
    sender: UnboundedSender<Message>,
    receiver: UnboundedReceiver<Message>
}

impl Logger {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            sender,
            receiver
        }
    }

    pub async fn log<E: fmt::Display>(&self, log: &E){
        self.sender.send(log);
    }

    pub async fn log_many<E: fmt::Display>(&self, logs: &[E]){
        logs.iter.for_each(|log| self.log(log));
    }

    pub async fn run(&self){
        while let Some(log) = self.receiver.recv().await {
            let log = log.to_string();
            let Ok(file) = fs::File::open("logs.txt").await else {
                println!("[failed to open logs-file]: {log}");
            };

            let Ok(()) = file.write_all(log).await else {
                println!("[failed to write logs-file]: {log}");
            };
        }
    }
}