use actix_web::{Scope, web::{self, Json, Path}, Responder, HttpResponse, get, post, HttpRequest};
use serde::Deserialize;
use serde_json::json;
use super::super::{AppStateData, utils::session::HttpSession};

pub fn service() -> Scope {
    web::scope("/blocks")
    .service(get_one)
    .service(create)
    .service(change)
}

#[get("/{id}")]
async fn get_one(app_state: AppStateData, id: Path<String>, req: HttpRequest) -> impl Responder {
    let session = HttpSession::from_request(&req, app_state.clone());
    match session.get_block(id.as_str()).await {
        Ok(block) => HttpResponse::Ok().json(block),
        Err(error) => HttpResponse::from(error)
    }
}

#[derive(Deserialize)]
pub struct CreateBody {
    pub content: String,
}

#[post("/")]
async fn create(app_state: AppStateData, body: Json<CreateBody>) -> impl Responder {
    extract_session!(app_state, session, extract_session_gen);
    match session.create_block(body.content.as_str()).await {
        Ok(id) => HttpResponse::Ok().json(json!({"id": id})),
        Err(error) => HttpResponse::from(error)
    }
}

#[derive(Deserialize)]
pub struct ChangeBody {
    pub content: String,
}

#[post("/{id}")]
async fn change(app_state: AppStateData, id: Path<String>, body: Json<CreateBody>) -> impl Responder {
    extract_session!(app_state, session, extract_session_gen);
    match session.change_block(id.as_str(), body.content.as_str()).await {
        Ok(()) => HttpResponse::Ok().json(json!({"message": "success"})),
        Err(error) => HttpResponse::from(error)
    }
}