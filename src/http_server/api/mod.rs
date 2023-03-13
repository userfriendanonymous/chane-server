use actix_web::{Scope, web::{self, Data}};

mod blocks;
mod channels;
mod users;
mod roles;
mod live_channel;
pub use live_channel::State as LiveChannelState;

pub fn service() -> Scope {
    web::scope("/api")
    .service(blocks::service())
    .service(channels::service())
    .service(users::service())
    .service(roles::service())
}