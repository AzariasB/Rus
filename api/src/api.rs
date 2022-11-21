use crate::{errors, AppCache, AppState, CreateForm, Params, DEFAULT_REDIRECTIONS_PER_PAGE};
use actix_web::{error, web, Error, HttpRequest, HttpResponse};
use log::warn;
use rus_core::{CreateMutation, Mutation, Query, UpdateMutation};
use std::sync::Mutex;
use tera::Tera;
use url::Url;

pub async fn list(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let conn = &data.conn;

    // get params
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let redirections_per_page = params
        .redirections_per_page
        .unwrap_or(DEFAULT_REDIRECTIONS_PER_PAGE);

    let (redirections, num_pages) =
        Query::find_redirections_in_page(conn, page, redirections_per_page)
            .await
            .expect("Cannot find redirections in page");

    let mut ctx = tera::Context::new();
    ctx.insert("redirections", &redirections);
    ctx.insert("page", &page);
    ctx.insert("redirections_per_page", &redirections_per_page);
    ctx.insert("num_pages", &num_pages);

    let body = template
        .render("index.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub async fn new(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let ctx = tera::Context::new();
    let body = template
        .render("new.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

pub async fn create(
    data: web::Data<AppState>,
    request: HttpRequest,
    redirection_form: web::Form<CreateForm>,
) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let form = redirection_form.into_inner();
    let url_parsing = Url::parse(form.long_url.as_str());

    if let Err(_err) = url_parsing {
        return Ok(HttpResponse::Found()
            .append_header(("location", "/new"))
            .finish());
    }

    Mutation::create_redirection(
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
    .expect("could not insert redirection");

    Ok(HttpResponse::Found()
        .append_header(("location", "/"))
        .finish())
}

pub async fn redirect(
    data: web::Data<AppState>,
    cache: web::Data<Mutex<AppCache>>,
    request: HttpRequest,
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
        not_found(&data.templates, request)
    }
}

pub async fn edit(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let template = &data.templates;
    let id = id.into_inner();

    let redirection = Query::find_redirection_by_id(conn, id)
        .await
        .map_err(errors::ApiError::from)?
        .ok_or(errors::ApiError::NotFound)?;

    let mut ctx = tera::Context::new();
    ctx.insert("redirection", &redirection);

    let body = template
        .render("edit.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
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

pub async fn delete(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let id = id.into_inner();

    Mutation::delete_redirection(conn, id)
        .await
        .expect("could not delete redirection");

    Ok(HttpResponse::Found()
        .append_header(("location", "/"))
        .finish())
}

fn not_found(templates: &Tera, request: HttpRequest) -> Result<HttpResponse, Error> {
    let mut ctx = tera::Context::new();
    ctx.insert("uri", request.uri().path());

    let body = templates
        .render("error/404.html.tera", &ctx)
        .map_err(|err| error::ErrorInternalServerError(format!("Template error : {}", err)))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
