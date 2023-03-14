use actix_web::{Scope, web::{self, Path, Json}, post, get, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use crate::db_pool::ChannelType;
use super::super::{AppStateData, extract_session, extract_session_gen};

pub fn service() -> Scope {
    web::scope("/channels")
    .service(get_one)
    .service(create)
}

#[get("/{id}")]
pub async fn get_one(app_state: AppStateData, id: Path<String>) -> impl Responder {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.get_channel(id.as_str()).await {
        Ok(channel) => HttpResponse::Ok().json(channel),
        Err(error) => error.into()
    }
}

#[derive(Deserialize)]
pub struct CreateBoby {
    pub _type: ChannelType,
    pub description: String,
    pub default_role: String,
    pub labels: Vec<String>,
}

#[post("/")]
pub async fn create(app_state: AppStateData, body: Json<CreateBoby>) -> impl Responder {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.create_channel(&body._type, &body.description, &body.default_role, &body.labels).await {
        Ok(id) => HttpResponse::Created().json(json!({"id": id})),
        Err(error) => HttpResponse::from(error)
    }
}