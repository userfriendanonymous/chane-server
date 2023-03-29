use actix_web::{web::{self, Path}, Scope, get, HttpRequest};
use crate::session_pool::ActivityTable;
use super::{Response, AppStateData, errors::{ResultResponse, general::GeneralError}};

pub fn service() -> Scope {
    web::scope("/activity-table")
    .service(get_one)
}

type GetOneResponse = ResultResponse<ActivityTable, GeneralError>;
#[get("/{id}")]
pub async fn get_one(app_state: AppStateData, id: Path<String>, req: HttpRequest) -> Response<GetOneResponse> {
    let session = app_state.session_from_request(&req);
    match session.get_activity_table(&id).await {
        Ok(table) => Response::ok_ok(table),
        Err(error) => Response::err_err(error.into())
    }
}