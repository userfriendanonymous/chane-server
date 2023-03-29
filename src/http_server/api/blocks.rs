use actix_web::{Scope, web::{self, Json, Path}, get, post, HttpRequest};
use serde::Deserialize;
use ts_rs::TS;
use crate::session_pool;
use super::{AppStateData, errors::{Response, ResultResponse, general::GeneralError}};

pub fn service() -> Scope {
    web::scope("/blocks")
    .service(get_one)
    .service(create)
    .service(change)
}

type GetOneResponse = ResultResponse<session_pool::Block, GeneralError>;
#[get("/{id}")]
async fn get_one(app_state: AppStateData, id: Path<String>, req: HttpRequest) -> Response<GetOneResponse> {
    let session = app_state.session_from_request(&req);
    match session.get_block(id.as_str()).await {
        Ok(block) => Response::ok_ok(block),
        Err(error) => {
            dbg!(&error);
            Response::err_err(error.into())
        }
    }
}

#[derive(Deserialize, TS)]
#[ts(export, rename = "CreateBlockBody")]
pub struct CreateBody {
    pub content: String,
}

type CreateResponse = ResultResponse<String, GeneralError>;
#[post("/create")]
async fn create(app_state: AppStateData, body: Json<CreateBody>, req: HttpRequest) -> Response<CreateResponse> {
    let session = app_state.session_from_request(&req);
    match session.create_block(body.content.as_str()).await {
        Ok(id) => Response::ok_ok(id),
        Err(error) => Response::err_err(error.into())
    }
}

#[derive(Deserialize, TS)]
#[ts(export, rename = "ChangeBlockBody")]
pub struct ChangeBody {
    pub content: String,
    pub id: String
}

type ChangeResponse = ResultResponse<(), GeneralError>;
#[post("/change")]
async fn change(app_state: AppStateData, body: Json<ChangeBody>, req: HttpRequest) -> Response<ChangeResponse> {
    let session = app_state.session_from_request(&req);
    match session.change_block(body.id.as_str(), body.content.as_str()).await {
        Ok(()) => Response::ok_ok(()),
        Err(error) => Response::err_err(error.into())
    }
}