use actix_web::{Scope, web::{self, Json}, post, get, HttpResponse, cookie::{CookieBuilder, Cookie}, HttpRequest};
use serde::Deserialize;
use ts_rs::TS;
use crate::{http_server::{AppStateData, errors::ResultResponse}, session_pool::AuthMe};
use super::{Response, errors};

pub fn service() -> Scope {
    web::scope("/auth")
    .service(join)
    .service(login)
    .service(me)
}

#[get("/me")]
pub async fn me(app_state: AppStateData, req: HttpRequest) -> Response<AuthMe> {
    let session = app_state.session_from_request(&req);
    Response::ok(session.me().await)
}

#[derive(Deserialize, TS)]
#[ts(export, rename = "AuthJoinBody")]
pub struct JoinBody {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[post("/join")]
pub async fn join(app_state: AppStateData, body: Json<JoinBody>, req: HttpRequest) -> Response<ResultResponse<(), errors::auth::JoinError>> {
    let session = app_state.session_from_request(&req);

    match session.register(&body.name, &body.email, &body.password).await {
        Ok(tokens) => Response::new(
            HttpResponse::Ok()
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
            ).take(),
            ResultResponse::Ok(())
        ),

        Err(error) => Response::err_err(error.into()),
    }
}

#[derive(Deserialize, TS)]
#[ts(export, rename = "AuthLoginBody")]
pub struct LoginBody {
    pub name: String,
    pub password: String,
}

#[post("/login")]
pub async fn login(app_state: AppStateData, body: Json<LoginBody>, req: HttpRequest) -> Response<ResultResponse<(), errors::auth::LoginError>> {
    let session = app_state.session_from_request(&req);
    match session.login(&body.name, &body.password).await {
        Ok(tokens) => Response::new(
            HttpResponse::Ok()
            .cookie(
                CookieBuilder::new("access-token", tokens.access)
                .http_only(true)
                .finish()
            )
            .cookie(
                CookieBuilder::new("key-token", tokens.key)
                .finish()
            ).take(),
            ResultResponse::Ok(())
        ),
        Err(error) => Response::err_err(error.into())
    }
}