use std::sync::Arc;

use activity_logger::ActivityLogger;
use auth_validator::AuthValidator;
use db_pool::DbPool;
use http_server::HttpServer;
use live_channel::LiveChannel;
use logger::Logger;
use session_pool::SessionPool;
use ts_rs::TS;

mod db_pool;
mod http_server;
mod session_pool;
mod shared;
mod live_channel;
mod logger;
mod auth_validator;
mod activity_logger;

#[tokio::main]
async fn main(){
    dotenv::dotenv().ok();

    let auth_keys = auth_validator::Keys {
        access: std::env::var("ACCESS_KEY").unwrap(),
        key: std::env::var("KEY_KEY").unwrap()
    };
    let auth_validator = Arc::new(AuthValidator::new(&auth_keys));
    let logger = Arc::new(Logger::new());
    let db_pool = Arc::new(DbPool::new().await.unwrap());
    let live_channel = Arc::new(LiveChannel::new(logger.clone()));
    let activity_logger = Arc::new(ActivityLogger::new(db_pool.clone(), logger.clone()));
    let session_pool = Arc::new(SessionPool::new(db_pool, auth_validator.clone(), live_channel.clone(), activity_logger.clone(), logger.clone())); // everything.clone()
    let http_server = Arc::new(HttpServer::new(session_pool, logger.clone()));

    let handle = std::thread::spawn(|| {
        let system = actix::System::new();
        system.block_on(async move {
            http_server.run().await;
        });
    });
    println!("run http server");

    tokio_scoped::scope(|scope| {
        scope.spawn(logger.run());
        scope.spawn(activity_logger.run());
        scope.spawn(live_channel.run());
    });

    handle.join().unwrap();
    dbg!("run all");
}