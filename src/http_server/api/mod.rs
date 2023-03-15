use actix_web::{Scope, web};

mod blocks;
mod channels;
mod users;
mod roles;
mod live_channel;
mod auth;
pub use live_channel::State as LiveChannel;

pub fn service() -> Scope {
    web::scope("/api")
    .service(blocks::service())
    .service(channels::service())
    .service(users::service())
    .service(roles::service())
    .service(auth::service())
}