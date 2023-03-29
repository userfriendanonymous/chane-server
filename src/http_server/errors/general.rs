use actix_web::{HttpResponse, HttpResponseBuilder};
use serde::Serialize;
use ts_rs::TS;
use crate::session_pool;
use super::AsBuilder;

#[derive(Serialize, TS)]
#[ts(export, rename = "GeneralError")]
#[serde(tag = "is", content = "data")]
pub enum GeneralError {
    Internal,
    Unauthorized,
}
impl AsBuilder for GeneralError {
    fn builder(&self) -> HttpResponseBuilder {
        match self {
            Self::Internal => HttpResponse::InternalServerError(),
            Self::Unauthorized => HttpResponse::Forbidden()
        }
    }
}
impl From<session_pool::Error> for GeneralError {
    fn from(value: session_pool::Error) -> Self {
        match value {
            session_pool::Error::Db(error) => {
                println!("{error}");
                Self::Internal
            },
            session_pool::Error::Unauthorized => Self::Unauthorized
        }
    }
}
