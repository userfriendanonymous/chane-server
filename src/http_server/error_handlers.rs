use actix_web::HttpResponse;
use crate::session_pool::{self, Error as SessionError, RoleWrappedError, RegisterError, LoginError};
use crate::db_pool::Error as DbError;
use serde_json::json;

pub enum Error {
    Db,
    Unauthorized (String),
    Role (RoleError),
}

pub enum RoleError {
    Recursion
}

impl From<session_pool::Error> for HttpResponse {
    fn from(error: session_pool::Error) -> Self {
        match error {
            SessionError::Db(error) => match error {
                DbError::InvalidObjectId(message) => HttpResponse::BadRequest().json(json!({
                    "message": format!("invalid object id: {message}")
                })),
                DbError::NotFound => HttpResponse::NotFound().json(json!({
                    "message": "not found"
                })),
                DbError::Query(error) => HttpResponse::InternalServerError().json(json!({
                    "message": error.to_string()
                })),
                DbError::BsonSerialization(error) => HttpResponse::InternalServerError().json(json!({
                    "message": error.to_string()
                }))
            },
            SessionError::Unauthorized(message) => HttpResponse::Unauthorized().json(json!({
                "message": message
            }))
        }
    }
}

impl From<RoleWrappedError> for HttpResponse {
    fn from(error: RoleWrappedError) -> Self {
        match error {
            RoleWrappedError::Recursion(message) => HttpResponse::LoopDetected().json(json!({
                "message": format!("Role recursion detected. Please contact us, this error should never happen. {message}")
            })),
            RoleWrappedError::General(error) => error.into()
        }
    }
}

impl From<RegisterError> for HttpResponse {
    fn from(error: RegisterError) -> Self {
        match error {
            RegisterError::BadNameLength => HttpResponse::Forbidden().json(json!({"message": "username length must be 3 - 20 characters"})),
            RegisterError::EmailTaken => HttpResponse::Conflict().json(json!({"message": "email already taken"})),
            RegisterError::NameTaken => HttpResponse::Conflict().json(json!({"message": "username already taken"})),
            RegisterError::InvaildNameChars => HttpResponse::Forbidden().json(json!({"message": "username contains invalid characters"})),
            RegisterError::TooLongPassword => HttpResponse::Conflict().json(json!({"message": "password is too long"})),
            RegisterError::TooShortPassword => HttpResponse::Conflict().json(json!({"message": "password is too short"})),
            RegisterError::InfoAsTokens(error) => HttpResponse::InternalServerError().json(json!({"message": format!("failed to convert generate tokens: {error}")})),
            RegisterError::Hashing(message) => HttpResponse::InternalServerError().json(json!({"message": format!("failed to hash: {message}")})),
            RegisterError::General(error) => error.into(),
        }
    }
}

impl From<LoginError> for HttpResponse {
    fn from(error: LoginError) -> Self {
        match error {
            LoginError::InvalidCredentials => HttpResponse::Forbidden().json(json!({"message": "invalid credentials"})),
            LoginError::InfoAsTokens(error) => HttpResponse::InternalServerError().json(json!({"message": format!("failed to convert generate tokens: {error}")})),
            LoginError::General(error) => error.into()
        }
    }
}