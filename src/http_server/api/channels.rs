use actix_web::{Scope, web::{self, Path, Json, Query}, post, get, put, HttpRequest};
use serde::Deserialize;
use ts_rs::TS;
use crate::{db_pool::ChannelType, session_pool::{self, Block}};
use super::{AppStateData, Response, errors::{ResultResponse, general::GeneralError, roles::RoleWrappedError}};

pub fn service() -> Scope {
    web::scope("/channels")
    .service(get_one)
    .service(create)
    .service(connect_block)
    .service(disconnect_block)
    .service(pin_block)
    .service(change_description)
    .service(change_labels)
    .service(get_channel_blocks)
}

pub type GetOneResponse = ResultResponse<session_pool::Channel, GeneralError>;
#[get("/{id}")]
pub async fn get_one(app_state: AppStateData, id: Path<String>, req: HttpRequest) -> Response<GetOneResponse> {
    let session = app_state.session_from_request(&req);
    match session.get_channel(id.as_str()).await {
        Ok(channel) => Response::ok_ok(channel),
        Err(error) => Response::err_err(error.into())
    }
}

#[derive(Deserialize, TS)]
#[ts(export, rename = "CreateChannelBody")]
pub struct CreateBoby {
    #[serde(rename = "type")]
    pub _type: ChannelType,
    pub description: String,
    pub title: String,
    pub default_role: String,
    pub labels: Vec<String>,
}

pub type CreateResponse = ResultResponse<String, GeneralError>;
#[post("/create")]
pub async fn create(app_state: AppStateData, body: Json<CreateBoby>, req: HttpRequest) -> Response<CreateResponse> {
    let session = app_state.session_from_request(&req);
    match session.create_channel(&body._type, &body.title, &body.description, &body.default_role, &body.labels).await {
        Ok(id) => Response::ok_ok(id),
        Err(error) => Response::err_err(error.into())
    }
}

#[derive(Deserialize, TS)]
#[ts(rename = "ConnectBlockToChannelBody", export)]
pub struct ConnectBlockBody {
    pub block_id: String,
    pub id: String
}

pub type ConnectBlockResponse = ResultResponse<(), RoleWrappedError>;
#[put("/connect-block")]
pub async fn connect_block(app_state: AppStateData, body: Json<ConnectBlockBody>, req: HttpRequest) -> Response<ConnectBlockResponse> {
    let session = app_state.session_from_request(&req);
    match session.connect_block_to_channel(&body.id, &body.block_id).await {
        Ok(()) => Response::ok_ok(()),
        Err(error) => Response::err_err(error.into())
    }
}

#[derive(Deserialize, TS)]
#[ts(rename = "DisconnectBlockFromChannelBody", export)]
pub struct DisconnectBlockBody {
    pub id: String,
    pub block_id: String
}

pub type DisconnectBlockResponse = ResultResponse<(), RoleWrappedError>;
#[put("/disconnect-block")]
pub async fn disconnect_block(app_state: AppStateData, body: Json<DisconnectBlockBody>, req: HttpRequest) -> Response<DisconnectBlockResponse> {
    let session = app_state.session_from_request(&req);
    match session.disconnect_block_from_channel(&body.id, &body.block_id).await {
        Ok(()) => Response::ok_ok(()),
        Err(error) => Response::err_err(error.into())
    }
}

#[derive(Deserialize, TS)]
#[ts(rename = "PinChannelBlockBody", export)]
pub struct PinBlockBody {
    pub id: String,
    pub block_id: Option<String>
}

pub type PinBlockResponse = ResultResponse<(), RoleWrappedError>;
#[put("/pin")]
pub async fn pin_block(app_state: AppStateData, body: Json<PinBlockBody>, req: HttpRequest) -> Response<PinBlockResponse> {
    let session = app_state.session_from_request(&req);
    match session.pin_channel_block(&body.id, &body.block_id).await {
        Ok(()) => Response::ok_ok(()),
        Err(error) => Response::err_err(error.into())
    }
}


#[derive(Deserialize, TS)]
#[ts(rename = "ChangeChannelDescriptionBody", export)]
pub struct ChangeDescriptionBody {
    pub id: String,
    pub content: String
}

pub type ChangeDescriptionResponse = ResultResponse<(), RoleWrappedError>;
#[put("/description")]
pub async fn change_description(app_state: AppStateData, body: Json<ChangeDescriptionBody>, req: HttpRequest) -> Response<ChangeDescriptionResponse> {
    let session = app_state.session_from_request(&req);
    match session.change_channel_description(&body.id, body.content.as_str()).await {
        Ok(()) => Response::ok_ok(()),
        Err(error) => Response::err_err(error.into())
    }
}

#[derive(Deserialize, TS)]
#[ts(rename = "ChangeChannelLabelsBody", export)]
pub struct ChangeLabelsBody {
    pub id: String,
    pub labels: Vec<String>
}

pub type ChangeLabelsResponse = ResultResponse<(), RoleWrappedError>;
#[put("/labels")]
pub async fn change_labels(app_state: AppStateData, body: Json<ChangeLabelsBody>, req: HttpRequest) -> Response<ChangeLabelsResponse> {
    let session = app_state.session_from_request(&req);
    match session.change_channel_labels(&body.id, &body.labels).await {
        Ok(()) => Response::ok_ok(()),
        Err(error) => Response::err_err(error.into())
    }
} // hmm... a lot of copy-paste-s???

#[derive(Deserialize, TS)]
#[ts(rename = "GetChannelBlocksQuery", export)]
pub struct GetBlocksQuery {
    pub limit: Option<i64>,
    pub offset: Option<u64>
}

type GetBlocksResponse = ResultResponse<Vec<Block>, RoleWrappedError>;
#[get("/{id}/blocks")]
pub async fn get_channel_blocks(app_state: AppStateData, id: Path<String>, query: Query<GetBlocksQuery>, req: HttpRequest) -> Response<GetBlocksResponse> {
    let session = app_state.session_from_request(&req);
    match session.get_channel_blocks(&id, &query.limit, &query.offset).await {
        Ok((blocks, errors)) => {
            println!("ERRORS: {errors:?}");
            Response::ok_ok(blocks)
        },
        Err(error) => Response::err_err(error.into())
    }
}