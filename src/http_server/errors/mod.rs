use actix_web::{HttpResponseBuilder, HttpResponse, Responder, body::BoxBody};
use serde::Serialize;
use ts_rs::TS;

pub mod auth;
pub mod general;
pub mod roles;

pub trait AsBuilder {
    fn builder(&self) -> HttpResponseBuilder;
}

#[derive(Serialize, TS)]
#[ts(export)]
#[serde(tag = "is", content = "data")]
pub enum ResultResponse<T: Serialize, E: Serialize> {
    Ok(T),
    Err(E)
}

impl<T: Serialize, E: Serialize> From<Result<T, E>> for ResultResponse<T, E> {
    fn from(value: Result<T, E>) -> Self {
        match value {
            Ok(data) => Self::Ok(data),
            Err(data) => Self::Err(data)
        }
    }
}

// pub enum TransResultResponse<T: Serialize, E: Serialize> {
//     Ok(T),
//     Err(E),
//     TransErr(TransError)
// }

// pub enum TransError {
//     Serialization(serde_json::Error)
// }
// impl From<serde_json::Error> for TransError {
//     fn from(value: serde_json::Error) -> Self {
//         Self::Serialization(value)
//     }
// }

// pub struct TransResponse<T: Serialize, F> where F: FnOnce(TransError) -> Response<T> {
//     data: T,
//     response: HttpResponse,
//     fallback: F
// }

// impl<T: Serialize, F> TransResponse<T, F> where F: FnOnce(TransError) -> Response<T> {
//     fn new(mut builder: HttpResponseBuilder, data: T, fallback: F) -> Self {
//         Self {
//             data,
//             response: builder.finish(),
//             fallback
//         }
//     }

//     fn from_response(response: HttpResponse, data: T, fallback: F) -> Self {
//         Self {
//             data,
//             response,
//             fallback
//         }
//     }

//     fn build(&self) -> HttpResponse {
//         match serde_json::to_string(&self.data) {
//             Ok(data) => self.response.set_body(BoxBody::new(data)),
//             Err(error) => (self.fallback)(error.into()).build()
//         }
//     }
// }

// impl<T: Serialize, F> Responder for TransResponse<T, F> where F: FnOnce(TransError) -> Response<T> {
//     type Body = BoxBody;
//     fn respond_to(self, req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
//         self.build()
//     }
// }

pub struct Response<T: Serialize> {
    data: T,
    builder: HttpResponseBuilder,
}

impl<T: Serialize> Responder for Response<T> {
    type Body = BoxBody;
    fn respond_to(self, _: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        self.build()
    }
}

impl<T: Serialize> Response<T> {
    pub fn new(builder: HttpResponseBuilder, data: T) -> Self {
        Self {
            builder,
            data
        }
    }
    pub fn ok(data: T) -> Self {
        Self::new(HttpResponse::Ok(), data)
    }
    fn build(mut self) -> HttpResponse {
        self.builder.json(self.data)
    }
}

impl<T: Serialize, E: Serialize> Response<ResultResponse<T, E>> {
    pub fn ok_ok(data: T) -> Self {
        Self::new(HttpResponse::Ok(), ResultResponse::Ok(data))
    }
    pub fn err(builder: HttpResponseBuilder, data: E) -> Self {
        Self::new(builder, ResultResponse::Err(data))
    }
}

impl<T: Serialize, E: AsBuilder + Serialize> Response<ResultResponse<T, E>> {
    pub fn err_err(data: E) -> Self {
        Self::new(data.builder(), ResultResponse::Err(data))
    }
}