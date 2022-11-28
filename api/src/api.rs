use crate::{errors, AppCache, AppState, CreateForm, Params, DEFAULT_REDIRECTIONS_PER_PAGE};
use actix_web::web::Json;
use actix_web::{error, web, Error, HttpRequest, HttpResponse, Responder};
use entity::redirection::Model;
use log::warn;
use rus_core::{CreateMutation, Mutation, Query, UpdateMutation};
use serde::Serialize;
use std::sync::Mutex;
use url::Url;

#[derive(Serialize)]
struct ListResponse {
    redirections: Vec<Model>,
    page: u64,
    redirections_per_page: u64,
    pages_count: u64,
}

#[derive(Serialize)]
struct CreateResponse {
    error: bool,
    message: String,
}

#[derive(Serialize)]
struct DeletedResponse {
    error: bool,
    message: String,
    id: i32,
}

pub async fn home(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let ctx = tera::Context::new();

    let body = template
        .render("index.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub async fn list(req: HttpRequest, data: web::Data<AppState>) -> Result<impl Responder, Error> {
    let conn = &data.conn;

    // get params
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let redirections_per_page = params
        .redirections_per_page
        .unwrap_or(DEFAULT_REDIRECTIONS_PER_PAGE);

    let (redirections, _pages_count) =
        Query::find_redirections_in_page(conn, page, redirections_per_page)
            .await
            .expect("Cannot find redirections in page");

    Ok(Json(redirections))
}

pub async fn create(
    data: web::Data<AppState>,
    request: HttpRequest,
    redirection_form: web::Form<CreateForm>,
) -> Result<impl Responder, Error> {
    let conn = &data.conn;
    let form = redirection_form.into_inner();
    let url_parsing = Url::parse(form.long_url.as_str());

    if let Err(_err) = url_parsing {
        return Ok(Json(CreateResponse {
            error: true,
            message: _err.to_string(),
        }));
    }

    Ok(Mutation::create_redirection(
        conn,
        CreateMutation::new(
            form.long_url,
            request
                .peer_addr()
                .map(|addr| addr.ip().to_string())
                .unwrap_or_else(|| "".to_string()),
            data.link_lifetime,
        ),
    )
    .await
    .map(|res| {
        Json(CreateResponse {
            error: false,
            message: format!("Url {} created", res.short_url.unwrap()),
        })
    })
    .unwrap_or_else(|err| {
        Json(CreateResponse {
            error: true,
            message: err.to_string(),
        })
    }))
}

pub async fn redirect(
    data: web::Data<AppState>,
    cache: web::Data<Mutex<AppCache>>,
    id: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let short = id.into_inner();

    {
        let cache = cache.lock().unwrap();

        let redirection_opt = cache.cache.try_get(&short);

        if let Some(redirection) = redirection_opt {
            actix_rt::spawn(async move {
                Query::update_access_date(&data.conn, short.to_string())
                    .await
                    .map_err(|e| {
                        warn!(
                            "Failed to update last access date of {}, cause : {}",
                            short, e
                        );
                    })
                    .unwrap();
            });

            return Ok(HttpResponse::Found()
                .append_header(("location", redirection))
                .finish());
        }
    }

    let from_database = Query::find_redirection_by_short_url(&data.conn, short.to_string())
        .await
        .map_err(errors::ApiError::from)?;

    if let Some(model) = from_database {
        let final_url = model.long_url.to_owned();

        actix_rt::spawn(async move {
            let saved = cache.lock().unwrap().cache.add_entry(
                short.to_string(),
                model.long_url.to_string(),
                model.expiration_date.unwrap_or_default(),
            );
            if let Err(e) = saved {
                warn!("Failed to save short url {} to cache : {}", short, e)
            }
            Query::update_access_date(&data.conn, short.to_string())
                .await
                .map_err(|e| {
                    warn!(
                        "Failed to update last access date of {}, cause : {}",
                        short, e
                    );
                })
                .unwrap();
        });
        Ok(HttpResponse::Found()
            .append_header(("location", final_url))
            .finish())
    } else {
        home(data).await
    }
}

pub async fn update(
    data: web::Data<AppState>,
    id: web::Path<i32>,
    redirection_form: web::Form<CreateForm>,
) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let form = redirection_form.into_inner();
    let id = id.into_inner();

    Mutation::update_redirection_by_id(conn, UpdateMutation::new(id, form.long_url))
        .await
        .expect("could not edit redirection");

    Ok(HttpResponse::Found()
        .append_header(("location", "/"))
        .finish())
}

pub async fn delete(
    data: web::Data<AppState>,
    id: web::Path<i32>,
) -> Result<impl Responder, Error> {
    let conn = &data.conn;
    let id = id.into_inner();

    Ok(Mutation::delete_redirection(conn, id)
        .await
        .map(|_| {
            Json(DeletedResponse {
                error: false,
                message: "Deleted".to_owned(),
                id,
            })
        })
        .unwrap_or_else(|err| {
            Json(DeletedResponse {
                error: true,
                message: err.to_string(),
                id,
            })
        }))
}
