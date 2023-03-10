use actix_web::{Scope, web, Responder};

pub fn service() -> Scope {
    web::scope("/blocks")
    .service(get_one)
    .service(create)
}

async fn get_one(app_state: AppStateData, id: u64) -> impl Responder {

}