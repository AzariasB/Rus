use actix_web::web::{delete, get, post, put, route, scope, ServiceConfig};

use crate::api;

pub fn init(cfg: &mut ServiceConfig) {
    cfg.service(
        scope("")
            .service(scope("/").route("", get().to(api::home)))
            .service(
                scope("/api/v1").service(
                    scope("/redirections")
                        .route("", get().to(api::list))
                        .route("", post().to(api::create))
                        .route("/{id}", delete().to(api::delete))
                        .route("/{id}", put().to(api::update)),
                ),
            )
            .route("/{id}", get().to(api::redirect)),
    )
    .default_service(route().to(api::home));
}
