use actix_web::web;
use actix_web::web::{get, post};

use crate::api;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/")
            .route("", get().to(api::list))
            .route("new", get().to(api::new))
            .route("", post().to(api::create))
            .route("{id}", get().to(api::redirect))
            .route("edit/{id}", get().to(api::edit))
            .route("{id}", post().to(api::update))
            .route("delete/{id}", post().to(api::delete)),
    );
}
