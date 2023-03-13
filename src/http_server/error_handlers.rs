use actix_web::HttpResponse;
use crate::session::{self, Error as SessionError, RoleWrappedError};
use crate::db_pool::Error as DbError;
use serde_json::json;

impl From<session::Error> for HttpResponse {
    fn from(error: session::Error) -> Self {
        match error {
            SessionError::Db(error) => match error {
                DbError::InvalidObjectId(message) => HttpResponse::BadRequest().json(json!({
                    "message": "invalid object id"
                })),
                DbError::NotFound => HttpResponse::NotFound().json(json!({
                    "message": "not found"
                })),
                DbError::Query(error) => HttpResponse::InternalServerError().json(json!({
                    "db query error": error.to_string()
                })),
            },
            SessionError::Unauthorized(message) => HttpResponse::Unauthorized().json(json!({
                "unauthorized": message
            }))
        }
    }
}

impl From<RoleWrappedError> for HttpResponse {
    fn from(error: RoleWrappedError) -> Self {
        match error {
            RoleWrappedError::Recursion(message) => HttpResponse::LoopDetected().json(json!({
                "error": "Role recursion detected. Please contact us, this error should never happen..."
            })),
            RoleWrappedError::General(error) => error.into()
        }
    }
}