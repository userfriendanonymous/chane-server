use actix_web::{HttpResponse, HttpResponseBuilder};
use serde::Serialize;
use ts_rs::TS;
use crate::session_pool;

use super::{AsBuilder, general::GeneralError};

#[derive(Serialize, TS)]
pub enum RoleError {
    Recursion(String),
}
impl AsBuilder for RoleError {
    fn builder(&self) -> HttpResponseBuilder {
        match self {
            Self::Recursion(_) => HttpResponse::LoopDetected()
        }
    }
}
impl From<session_pool::RoleError> for RoleError {
    fn from(value: session_pool::RoleError) -> Self {
        match value {
            session_pool::RoleError::Recursion(id) => Self::Recursion(id)
        }
    }
}

#[derive(Serialize, TS)]
pub enum RoleWrappedError {
    Role(RoleError),
    General(GeneralError)
}
impl AsBuilder for RoleWrappedError {
    fn builder(&self) -> HttpResponseBuilder {
        match self {
            Self::Role(error) => error.builder(),
            Self::General(error) => error.builder()
        }
    }
}
impl From<session_pool::RoleWrappedError> for RoleWrappedError {
    fn from(value: session_pool::RoleWrappedError) -> Self {
        match value {
            session_pool::RoleWrappedError::General(error) => Self::General(error.into()),
            session_pool::RoleWrappedError::Role(error) => Self::Role(error.into())
        }
    }
}

#[derive(Serialize, TS)]
pub enum CreateRoleError {
    General(GeneralError),
    DoesNotExist(String)
}
impl AsBuilder for CreateRoleError {
    fn builder(&self) -> HttpResponseBuilder {
        match self {
            Self::General(error) => error.builder(),
            Self::DoesNotExist(_) => HttpResponse::NotAcceptable()
        }
    }
}
impl From<session_pool::CreateRoleError> for CreateRoleError {
    fn from(value: session_pool::CreateRoleError) -> Self {
        match value {
            session_pool::CreateRoleError::General(error) => Self::General(error.into()),
            session_pool::CreateRoleError::RoleDoesNotExist(id, _) => Self::DoesNotExist(id)
        }
    }
}