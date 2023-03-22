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
        println!("request: {}", request.path());

        let session = self.session.clone();
        let auth_keys = self.auth_keys.clone();
        let db_pool = self.db_pool.clone();
        let live_channel = self.live_channel.clone();

        let tokens = AuthTokens::new(
            extract_cookie_as_string(&request, "access-token"),
            extract_cookie_as_string(&request, "key-token")
        );
        
        let future = self.service.call(request);

        Box::pin(async move {
            let mut session = session.lock().await;
            *session = Some(Session::new(db_pool, auth_keys, tokens, live_channel));
            drop(session); // oh wow! required to explicity drop rwlockguard or else it will never release and program will be stuck!
            
            let response = future.await?;
            Ok(response)
        })
    }
}

fn extract_cookie_as_string(request: &ServiceRequest, name: &str) -> String {
    match request.cookie(name) {
        Some(cookie) => cookie.value().to_owned(),
        None => "".to_owned()
    }
}