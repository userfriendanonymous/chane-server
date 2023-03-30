use actix_web::{Scope, web::{self, Path, Json}, get, post, put, HttpRequest};
use serde::Deserialize;
use ts_rs::TS;
use crate::{db_pool::RolePermissions, session_pool::Role};

use super::{AppStateData, Response, errors::{ResultResponse, general::GeneralError, roles::CreateRoleError}};

pub fn service() -> Scope {
    web::scope("/roles")
    .service(get_one)
    .service(create)
    .service(change)
}

type GetOneResponse = ResultResponse<Role, GeneralError>;
#[get("/{id}")]
pub async fn get_one(app_state: AppStateData, id: Path<String>, req: HttpRequest) -> Response<GetOneResponse> {
    let session = app_state.session_from_request(&req);
    match session.get_role(&id).await {
        Ok(role) => Response::ok_ok(role),
        Err(error) => Response::err_err(error.into())
    }
}

#[derive(Deserialize, TS)]
#[ts(rename = "CreateRoleBody", export)]
pub struct CreateBoby {
    name: String,
    extends: Vec<String>,
    editors: Vec<String>,
    permissions: RolePermissions
}

type CreateResponse = ResultResponse<String, CreateRoleError>;
#[post("/create")]
pub async fn create(app_state: AppStateData, body: Json<CreateBoby>, req: HttpRequest) -> Response<CreateResponse> {
    let session = app_state.session_from_request(&req);
    match session.create_role(&body.name, &body.extends, &body.editors, &body.permissions).await {
        Ok(id) => Response::ok_ok(id),
        Err(error) => Response::err_err(error.into())
    }
}

#[derive(Deserialize, TS)]
#[ts(rename = "ChangeRoleBody", export)]
pub struct ChangeBody {
    name: String,
    extends: Vec<String>,
    editors: Vec<String>,
    permissions: RolePermissions
}

type ChangeRoleResponse = ResultResponse<(), GeneralError>;
#[put("/change/{id}")]
pub async fn change(app_state: AppStateData, id: Path<String>, body: Json<ChangeBody>, req: HttpRequest) -> Response<ChangeRoleResponse> {
    let session = app_state.session_from_request(&req);
    match session.change_role(id.as_str(), &body.name, &body.extends, &body.editors, body.permissions.clone()).await {
        Ok(()) => Response::ok_ok(()),
        Err(error) => Response::err_err(error.into())
    }
}