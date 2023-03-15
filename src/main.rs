mod db_pool;
mod http_server;
mod session;
mod shared;

#[actix_web::main]
async fn main() {
    http_server::launch().await.unwrap();
    println!("Hello, world!");
}
