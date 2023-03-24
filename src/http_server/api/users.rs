use actix_web::{Scope, web::{self, Path}, get, HttpResponse, HttpRequest};
use super::super::AppStateData;

pub fn service() -> Scope {
    web::scope("/users")
    .service(get_one)
}

#[get("/{name}")]
pub async fn get_one(app_state: AppStateData, name: Path<String>, req: HttpRequest) -> HttpResponse {
    let session = app_state.session_from_request(&req);
    match session.get_user(&name).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(error) => error.into()
    }
}