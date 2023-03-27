use actix_web::{HttpResponse, HttpResponseBuilder};
use serde::Serialize;
use ts_rs::TS;
use crate::session_pool;
use super::{general::GeneralError, AsBuilder};

#[derive(Serialize, TS)]
#[ts(export, rename = "AuthJoinError")]
pub enum JoinError {
    InvaildNameChars,
    BadNameLength,
    TooShortPassword,
    TooLongPassword,
    NameTaken,
    EmailTaken,
    General (GeneralError),
}
impl AsBuilder for JoinError {
    fn builder(&self) -> HttpResponseBuilder {
        match self {
            Self::General(error) => error.builder(),
            _ => HttpResponse::BadRequest()
        }
    }
}
impl From<session_pool::RegisterError> for JoinError {
    fn from(value: session_pool::RegisterError) -> Self {
        match value {
            session_pool::RegisterError::BadNameLength => Self::BadNameLength,
            session_pool::RegisterError::EmailTaken => Self::EmailTaken,
            session_pool::RegisterError::General(error) => Self::General(error.into()),
            session_pool::RegisterError::Hashing(_) => Self::General(GeneralError::Internal),
            session_pool::RegisterError::InfoAsTokens(_) => Self::General(GeneralError::Internal),
            session_pool::RegisterError::InvaildNameChars => Self::InvaildNameChars,
            session_pool::RegisterError::NameTaken => Self::NameTaken,
            session_pool::RegisterError::TooLongPassword => Self::TooLongPassword,
            session_pool::RegisterError::TooShortPassword => Self::TooShortPassword
        }
    }
}

#[derive(Serialize, TS)]
#[ts(export, rename = "AuthLoginError")]
pub enum LoginError {
    General(GeneralError),
    InvalidCredentials
}
impl AsBuilder for LoginError {
    fn builder(&self) -> HttpResponseBuilder {
        match self {
            Self::General(error) => error.builder(),
            Self::InvalidCredentials => HttpResponse::Forbidden()
        }
    }
}
impl From<session_pool::LoginError> for LoginError {
    fn from(value: session_pool::LoginError) -> Self {
        match value {
            session_pool::LoginError::General(error) => Self::General(error.into()),
            session_pool::LoginError::InfoAsTokens(_) => Self::General(GeneralError::Internal),
            session_pool::LoginError::InvalidCredentials => Self::InvalidCredentials
        }
    }
}