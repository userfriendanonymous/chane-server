use actix_web::{Scope, web::{self, Json}, post, get, HttpResponse, cookie::{CookieBuilder, Cookie}, HttpRequest};
use serde::Deserialize;
use serde_json::json;
use crate::http_server::AppStateData;

pub fn service() -> Scope {
    web::scope("/auth")
    .service(join)
    .service(login)
    .service(me)
}

#[derive(Deserialize)]
pub struct JoinBody {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[get("/me")]
pub async fn me(app_state: AppStateData, req: HttpRequest) -> HttpResponse {
    let session = app_state.session_from_request(&req);
    HttpResponse::Ok().json(session.me().await)
}

#[post("/join")]
pub async fn join(app_state: AppStateData, body: Json<JoinBody>, req: HttpRequest) -> HttpResponse {
    let session = app_state.session_from_request(&req);
    match session.register(&body.name, &body.email, &body.password).await {
        Ok(tokens) => HttpResponse::Created()
        .cookie(
            Cookie::build("access-token", tokens.access)
            .http_only(true)
            .same_site(actix_web::cookie::SameSite::None)
            .finish()
        )
        .cookie(
            Cookie::build("key-token", tokens.key)
            .same_site(actix_web::cookie::SameSite::None)
            .finish()
        )
        .json(json!({"message": "success"})),
        Err(error) => error.into()
    }
}

#[derive(Deserialize)]
pub struct LoginBody {
    pub name: String,
    pub password: String,
}

#[post("/login")]
pub async fn login(app_state: AppStateData, body: Json<LoginBody>, req: HttpRequest) -> HttpResponse {
    let session = app_state.session_from_request(&req);
    match session.login(&body.name, &body.password).await {
        Ok(tokens) => HttpResponse::Ok()
        .cookie(
            CookieBuilder::new("access-token", tokens.access)
            .http_only(true)
            .finish()
        )
        .cookie(
            CookieBuilder::new("key-token", tokens.key)
            .finish()
        )
        .json(json!({"message": "success"})),
        
        Err(error) => error.into()
    }
}