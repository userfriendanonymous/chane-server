use actix_web::{Scope, web::{self, Path}, get, HttpRequest};
use super::{AppStateData, Response, errors::{general::GeneralError, ResultResponse}};
use crate::session_pool::User;

pub fn service() -> Scope {
    web::scope("/users")
    .service(get_one)
}

type GetOneResponse = ResultResponse<User, GeneralError>;
#[get("/{name}")]
pub async fn get_one(app_state: AppStateData, name: Path<String>, req: HttpRequest) -> Response<GetOneResponse> {
    let session = app_state.session_from_request(&req);
    match session.get_user(&name).await {
        Ok(user) => Response::ok_ok(user),
        Err(error) => Response::err_err(error.into())
    }
}