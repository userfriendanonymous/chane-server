use actix_web::{Scope, web::{self, Data}};

mod blocks;
mod channels;
mod users;

pub fn service() -> Scope {
    web::scope("/api")
    .service(blocks::service())
    .service(users::service())
    .service(users::service())
}