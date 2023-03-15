use actix_web::{Scope, web::{self, Json}, post, HttpResponse};
use serde::Deserialize;
use crate::http_server::{AppStateData, extract_session, extract_session_gen};

pub fn service() -> Scope {
    web::scope("/auth")
    .service(register)
    .service(login)
}

#[derive(Deserialize)]
pub struct RegisterBody {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[post("/register")]
pub async fn register(app_state: AppStateData, body: Json<RegisterBody>) -> HttpResponse {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.register(&body.name, &body.email, &body.password).await {
        Ok(tokens) => HttpResponse::Created().json(tokens),
        Err(error) => error.into()
    }
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub name: String,
    pub password: String,
}

#[post("/login")]
pub async fn login(app_state: AppStateData, body: Json<LoginBody>) -> HttpResponse {
    extract_session!(app_state, session, session_shared, extract_session_gen);
    match session.login(&body.name, &body.password).await {
        Ok(tokens) => HttpResponse::Ok().json(tokens),
        Err(error) => error.into()
    }
}