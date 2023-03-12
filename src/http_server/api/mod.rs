use actix_web::{Scope, web::{self, Data}};

mod blocks;
mod channels;
mod users;
mod groups;
mod roles;

pub fn service() -> Scope {
    web::scope("/api")
    .service(blocks::service())
    .service(users::service())
    .service(users::service())
    .service(groups::serivce())
    .service(roles::service())
}