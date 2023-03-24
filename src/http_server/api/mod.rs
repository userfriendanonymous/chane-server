use actix_web::{Scope, web};

mod blocks;
mod channels;
mod users;
mod roles;
mod live;
mod auth;

pub fn service() -> Scope {
    web::scope("/api")
    .service(blocks::service())
    .service(channels::service())
    .service(users::service())
    .service(roles::service())
    .service(auth::service())
    .service(live::service)
}