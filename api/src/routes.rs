use actix_web::web;
use actix_web::web::{delete, get, post};

use crate::api;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .service(web::scope("/").route("", get().to(api::home)))
            .service(
                web::scope("/api/v1").service(
                    web::scope("/redirections")
                        .route("", get().to(api::list))
                        .route("", post().to(api::create))
                        .route("/{id}", delete().to(api::delete))
                        .route("/{id}", get().to(api::redirect))
                        .route("/{id}", post().to(api::update)),
                ),
            ),
    );
}
