use actix_web::{Scope, web::{self, Path}, get, Responder, HttpResponse};
use super::super::{AppStateData, extract_session_gen, extract_session};

pub fn service() -> Scope {
    web::scope("/users")
    .service(get_one)
}

#[get("/{name}")]
pub async fn get_one(app_state: AppStateData, name: Path<String>) -> HttpResponse {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.get_user(&*name).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(error) => error.into()
    }
}