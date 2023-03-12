use std::future::{ready, Ready};
use actix_web::{Error, dev::{Transform, forward_ready, Service, ServiceRequest, ServiceResponse}};
use futures_util::future::LocalBoxFuture;

use crate::session::SessionShared;

pub struct MiddlewareFactory {
    session: SessionShared
}

impl<S, B> Transform<S, ServiceRequest> for MiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = Middleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(Middleware {
            service,
            session: self.session.clone()
        }))
    }
}

pub struct Middleware<S> {
    service: S,
    session: SessionShared
}

impl<S, B> Service<ServiceRequest> for Middleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
    forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        println!("middleware: {}", request.path());
        let future = self.service.call(request);

        let session = self.session.clone();

        Box::pin(async move {
            let session = session.lock().await;
            
            let response = future.await?;
            println!("response");
            Ok(response)
        })
    }
}