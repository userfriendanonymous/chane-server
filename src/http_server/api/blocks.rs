use actix_web::{Scope, web::{self, Json, Path}, Responder, HttpResponse, get, post};
use serde::{Serialize, Deserialize};
use serde_json::json;
use super::super::{AppStateData, extract_session, extract_session_gen, handle_session_error};

pub fn service() -> Scope {
    web::scope("/blocks")
    .service(get_one)
    .service(create)
}

#[get("/{id}")]
async fn get_one(app_state: AppStateData, id: Path<String>) -> impl Responder {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.get_block(id.as_str()).await {
        Ok(block) => HttpResponse::Ok().json(block),
        Err(error) => handle_session_error(error)
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateBody {
    content: String,
}

#[post("/")]
async fn create(app_state: AppStateData, body: Json<CreateBody>) -> impl Responder {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.create_block(body.content.as_str()).await {
        Ok(id) => HttpResponse::Ok().json(json!({"id": id})),
        Err(error) => handle_session_error(error)
    }
}