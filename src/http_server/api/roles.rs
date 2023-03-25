use actix_web::{Scope, web::{self, Path, Json}, get, post, put, HttpResponse, HttpRequest};
use serde::Deserialize;
use serde_json::json;
use crate::{db_pool::RolePermissions, session_pool::CreateRoleError};

use super::super::AppStateData;

pub fn service() -> Scope {
    web::scope("/roles")
    .service(get_one)
    .service(create)
    .service(change)
}

#[get("/{id}")]
pub async fn get_one(app_state: AppStateData, id: Path<String>, req: HttpRequest) -> HttpResponse {
    let session = app_state.session_from_request(&req);
    match session.get_role(&id).await {
        Ok(role) => HttpResponse::Ok().json(role),
        Err(error) => error.into()
    }
}

#[derive(Deserialize)]
pub struct CreateBoby {
    name: String,
    extends: Vec<String>,
    editors: Vec<String>,
    permissions: RolePermissions
}

#[post("/")]
pub async fn create(app_state: AppStateData, body: Json<CreateBoby>, req: HttpRequest) -> HttpResponse {
    let session = app_state.session_from_request(&req);
    match session.create_role(&body.name, &body.extends, &body.editors, &body.permissions).await {
        Ok(id) => HttpResponse::Created().json(json!({
            "id": id
        })),
        Err(error) => match error {
            CreateRoleError::General(error) => error.into(),
            CreateRoleError::RoleDoesNotExist(id, error) => HttpResponse::Forbidden().json(json!({
                "message": format!("can't extend role that doesn't exist: {id}"),
                "db_error_DEBUG_ONLY": error.to_string()
            }))
        }
    }
}

#[derive(Deserialize)]
pub struct ChangeBody {
    name: String,
    extends: Vec<String>,
    editors: Vec<String>,
    permissions: RolePermissions
}

#[put("/{id}")]
pub async fn change(app_state: AppStateData, id: Path<String>, body: Json<ChangeBody>, req: HttpRequest) -> HttpResponse {
    let session = app_state.session_from_request(&req);
    match session.change_role(id.as_str(), &body.name, &body.extends, &body.editors, body.permissions.clone()).await {
        Ok(()) => HttpResponse::Ok().json(json!({
            "message": "success"
        })),
        Err(error) => error.into()
    }
}