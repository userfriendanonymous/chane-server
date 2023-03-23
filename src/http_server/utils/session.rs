use std::ops::Deref;

use actix_web::HttpRequest;
use crate::{session_pool::{AuthTokens, Session}, db_pool::DbPool, http_server::AppStateData};

fn extract_cookie_as_string(request: &actix_web::HttpRequest, name: &str) -> String {
    match request.cookie(name) {
        Some(cookie) => cookie.value().to_owned(),
        None => "".to_owned()
    }
}

pub struct HttpSession(Session);

impl HttpSession {
    pub fn from_request(request: &HttpRequest, app_state: AppStateData) -> Self {
        let tokens = AuthTokens {
            access: extract_cookie_as_string(request, "access-token"),
            key: extract_cookie_as_string(request, "key-token")
        };
        Self(Session::new(app_state.db_pool.clone(), app_state.auth_keys.clone(), tokens, app_state.live_channel.clone()))
    }
}

impl Deref for HttpSession {
    type Target = Session;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}