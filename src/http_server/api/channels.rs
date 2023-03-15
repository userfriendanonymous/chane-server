use actix_web::{Scope, web::{self, Path, Json}, post, get, put, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::db_pool::ChannelType;
use super::super::{AppStateData, extract_session, extract_session_gen};

pub fn service() -> Scope {
    web::scope("/channels")
    .service(get_one)
    .service(create)
    .service(connect_block)
    .service(disconnect_block)
    .service(pin_block)
    .service(change_description)
    .service(change_labels)
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
pub async fn create(app_state: AppStateData, body: Json<CreateBoby>) -> HttpResponse {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.create_channel(&body._type, &body.description, &body.default_role, &body.labels).await {
        Ok(id) => HttpResponse::Created().json(json!({"id": id})),
        Err(error) => HttpResponse::from(error)
    }
}

#[derive(Deserialize)]
pub struct ConnectBlockBody {
    pub id: String
}

#[put("/{id}")]
pub async fn connect_block(app_state: AppStateData, id: Path<String>, body: Json<ConnectBlockBody>) -> HttpResponse {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.connect_block_to_channel(id.as_str(), &body.id).await {
        Ok(()) => HttpResponse::Ok().json(json!({
            "message": "success"
        })),
        Err(error) => error.into()
    }
}

#[derive(Deserialize)]
pub struct DisconnectBlockBody {
    pub id: String
}

#[put("/{id}")]
pub async fn disconnect_block(app_state: AppStateData, id: Path<String>, body: Json<DisconnectBlockBody>) -> HttpResponse {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.disconnect_block_from_channel(id.as_str(), &body.id).await {
        Ok(()) => HttpResponse::Ok().json(json!({
            "message": "success"
        })),
        Err(error) => error.into()
    }
}

#[derive(Deserialize)]
pub struct PinBlockBody {
    pub id: Option<String>
}

#[put("/{id}")]
pub async fn pin_block(app_state: AppStateData, id: Path<String>, body: Json<PinBlockBody>) -> HttpResponse {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.pin_channel_block(id.as_str(), &body.id).await {
        Ok(()) => HttpResponse::Ok().json(json!({
            "message": "success"
        })),
        Err(error) => error.into()
    }
}


#[derive(Deserialize)]
pub struct ChangeDescriptionBody {
    pub content: String
}

#[put("/{id}")]
pub async fn change_description(app_state: AppStateData, id: Path<String>, body: Json<ChangeDescriptionBody>) -> HttpResponse {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.change_channel_description(id.as_str(), body.content.as_str()).await {
        Ok(()) => HttpResponse::Ok().json(json!({
            "message": "success"
        })),
        Err(error) => error.into()
    }
}

#[derive(Deserialize)]
pub struct ChangeLabelsBody {
    pub labels: Vec<String>
}

#[put("/{id}")]
pub async fn change_labels(app_state: AppStateData, id: Path<String>, body: Json<ChangeLabelsBody>) -> HttpResponse {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.change_channel_labels(id.as_str(), &body.labels).await {
        Ok(()) => HttpResponse::Ok().json(json!({
            "message": "success"
        })),
        Err(error) => error.into()
    }
} // hmm... a lot of copy-paste-s???