use session::AuthKeys;

mod db_pool;
mod http_server;
mod session;
mod shared;
mod app;
mod live_channel;



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
    let http_server = HttpServer::new();
    http_server.run().await;

    let app_task = tokio::join!(
        http_server.run(),

    );

    app_task.await;
}