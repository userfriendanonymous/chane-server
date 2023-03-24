use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};
use tokio::fs;

pub struct Logger {
    sender: UnboundedSender<String>,
    receiver: Mutex<UnboundedReceiver<String>>
}

impl Logger {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        Self {
            sender,
            receiver: Mutex::new(receiver)
        }
    }

    pub fn log(&self, log: &str){
        self.sender.send(log.to_string());
    }

    pub async fn log_many(&self, logs: &[String]){
        logs.iter().for_each(|log| self.log(log));
    }

    pub async fn run(&self){
        let mut receiver = self.receiver.lock().await;
        while let Some(log) = receiver.recv().await {
            let Ok(mut file) = fs::File::open("logs.txt").await else {
                println!("[failed to open logs-file]: {log}");
                continue;
            };

            let Ok(()) = file.write_all(log.as_bytes()).await else {
                println!("[failed to write logs-file]: {log}");
                continue;
            };
        }
    }
}