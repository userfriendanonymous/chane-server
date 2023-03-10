use actix_web::{HttpServer, App, web::Data};

use crate::db_pool::{DbPoolShared, DbPool};

mod api;

struct AppState {
    db_pool: DbPoolShared
}

type AppStateData = Data<AppState>;

pub async fn launch() -> std::io::Result<()> {
    let app_state = Data::new(AppState {
        db_pool: DbPool::new_shared().await
    });

    HttpServer::new(move || {
        App::new()
        .app_data(app_state.clone())
        .service(api::service())
    })
    .bind(("127.0.0.1", 5000))?
    .run()
    .await
}