use std::future::{ready, Ready};
use actix_web::{Error, dev::{Transform, forward_ready, Service, ServiceRequest, ServiceResponse}};
use futures_util::future::LocalBoxFuture;
use crate::{session::{AuthKeys, AuthTokens}};
use crate::{session::{Session, LiveChannel}, shared::Shared, db_pool::DbPool};

type SessionShared<LC> = Shared<Option<Session<LC>>>;

pub struct MiddlewareFactory<LC: LiveChannel> {
    pub session: SessionShared<LC>,
    pub db_pool: Shared<DbPool>,
    pub auth_keys: AuthKeys,
    pub live_channel: Shared<LC>
}

impl<S, B, LC: 'static + LiveChannel> Transform<S, ServiceRequest> for MiddlewareFactory<LC>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = Middleware<S, LC>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(Middleware {
            service,
            session: self.session.clone(),
            auth_keys: self.auth_keys.clone(),
            live_channel: self.live_channel.clone(),
            db_pool: self.db_pool.clone()
        }))
    }
}

pub struct Middleware<S, LC: LiveChannel> {
    service: S,
    session: SessionShared<LC>,
    db_pool: Shared<DbPool>,
    auth_keys: AuthKeys,
    live_channel: Shared<LC>
}

impl<S, B, LC: 'static + LiveChannel> Service<ServiceRequest> for Middleware<S, LC> // ugh need to fix thoose static
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;
    forward_ready!(service);

    fn call(&self, request: ServiceRequest) -> Self::Future {
        println!("middleware: {}", request.path());
        let future = self.service.call(request);

        let session = self.session.clone();
        let auth_keys = self.auth_keys.clone();
        let db_pool = self.db_pool.clone();
        let live_channel = self.live_channel.clone();

        Box::pin(async move {
            let mut session = session.lock().await;
            *session = Some(Session::new(db_pool, auth_keys, AuthTokens::new("".to_owned(), "".to_owned()), live_channel));
            
            let response = future.await?;
            println!("response");
            Ok(response)
        })
    }
}