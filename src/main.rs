use std::sync::Arc;

use activity_logger::ActivityLogger;
use auth_validator::AuthValidator;
use db_pool::DbPool;
use http_server::HttpServer;
use live_channel::LiveChannel;
use logger::Logger;
use session_pool::SessionPool;

mod db_pool;
mod http_server;
mod session_pool;
mod shared;
mod app;
mod live_channel;
mod logger;
mod auth_validator;
mod activity_logger;


// #[actix_web::main]
// async fn main() {
//     dotenv::dotenv().ok();
//     http_server::launch(
//         AuthKeys {
//             access: std::env::var("ACCESS_KEY").unwrap(),
//             key: std::env::var("KEY_KEY").unwrap()
//         }
//     ).await.unwrap();
// }

#[tokio::main]
async fn main(){
    dotenv::dotenv().ok();

    let auth_keys = auth_validator::Keys {
        access: std::env::var("ACCESS_KEY").unwrap(),
        key: std::env::var("KEY_KEY").unwrap()
    };
    let auth_validator = Arc::new(AuthValidator::new(&auth_keys));
    let db_pool = Arc::new(DbPool::new().await.unwrap());
    let live_channel = Arc::new(LiveChannel::default());
    let activity_logger = Arc::new(ActivityLogger::new(db_pool));
    let session_pool = Arc::new(SessionPool::new(db_pool.clone(), auth_validator.clone(), live_channel.clone(), activity_logger.clone()));
    let logger = Arc::new(Logger::default());
    let http_server = HttpServer::new(session_pool.clone(), logger.clone());

    let app_task = tokio::join!(
        http_server.run(),
        logger.run(),
        activity_logger.run()
    );

    app_task.await;
}