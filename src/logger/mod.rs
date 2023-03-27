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

    pub fn log(&self, log: String){
        if let Err(error) = self.sender.send(log.clone()) {
            println!("[failed to send log message]:\n{log}\n[error]:\n{error}")
        }
    }

    pub async fn log_many(&self, logs: Vec<String>){
        for log in logs {
            self.log(log);
        }
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