/*
use actix_web::{Scope, web::{self, Path, Json}, post, get, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use crate::{http_server::{AppStateData, extract_session, extract_session_gen, handle_session_error}, db_pool::ChannelType};

pub fn service() -> Scope {
    web::scope("/groups")
    .service(get_one)
    .service(create)
}

#[get("/{id}")]
pub async fn get_one(app_state: AppStateData, id: Path<String>) -> impl Responder {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.get_channel(id.as_str()).await {
        Ok(channel) => HttpResponse::Ok().json(channel),
        Err(error) => handle_session_error(error)
    }
}

#[derive(Deserialize)]
struct CreateBoby {
    _type: ChannelType
}

#[post("/")]
pub async fn create(app_state: AppStateData, body: Json<CreateBoby>) -> impl Responder {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.create(body._type.clone()).await {
        Ok(id) => HttpResponse::Created().json(json!({"id": id})),
        Err(error) => handle_session_error(error)
    }
}
*/