extern crate core;

use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::sync::Mutex;

use actix_files::Files as Fs;
use actix_web::{App, error, Error, get, HttpRequest, HttpResponse, HttpServer, post, Result, web};
use listenfd::ListenFd;
use serde::{Deserialize, Serialize};
use tera::Tera;
use url::Url;

use migration::{Migrator, MigratorTrait};
use rus_core::{Cache, CreateMutation, Mutation, Query, redis, sea_orm::{Database, DatabaseConnection}, UpdateMutation};

mod errors;

const DEFAULT_REDIRECTIONS_PER_PAGE: u64 = 5;

#[derive(Debug, Clone)]
struct AppState {
    templates: Tera,
    conn: DatabaseConnection,
}

#[derive(Debug, Clone)]
struct AppCache {
    cache: Cache,
}

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<u64>,
    redirections_per_page: Option<u64>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
struct FlashData {
    kind: String,
    message: String,
}

#[derive(Deserialize)]
struct CreateForm {
    long_url: String,
}


#[get("/")]
async fn list(req: HttpRequest, data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let conn = &data.conn;


    // get params
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();

    let page = params.page.unwrap_or(1);
    let redirections_per_page = params.redirections_per_page.unwrap_or(DEFAULT_REDIRECTIONS_PER_PAGE);

    let (redirections, num_pages) = Query::find_redirections_in_page(conn, page, redirections_per_page)
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

#[get("/new")]
async fn new(data: web::Data<AppState>) -> Result<HttpResponse, Error> {
    let template = &data.templates;
    let ctx = tera::Context::new();
    let body = template
        .render("new.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[post("/")]
async fn create(
    data: web::Data<AppState>,
    request: HttpRequest,
    redirection_form: web::Form<CreateForm>,
) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let form = redirection_form.into_inner();
    let url_parsing = Url::parse(form.long_url.as_str());

    if let Err(_err) = url_parsing {
        return Ok(HttpResponse::Found().append_header(("location", "/new")).finish());
    }

    Mutation::create_redirection(conn, CreateMutation::new(form.long_url,
                                                           request
                                                               .peer_addr()
                                                               .map(|addr| addr.ip().to_string())
                                                               .unwrap_or("".to_string()),
    ))
        .await
        .expect("could not insert redirection");

    Ok(HttpResponse::Found()
        .append_header(("location", "/"))
        .finish())
}

#[get("/{id}")]
async fn redirect(data: web::Data<AppState>, cache: web::Data<Mutex<AppCache>>, request: HttpRequest, id: web::Path<String>) -> Result<HttpResponse, Error> {
    let mut cache = cache.lock().unwrap();

    let short = id.into_inner();

    let redirection_opt = cache.cache.try_get(&short);

    if let Some(redirection) = redirection_opt {
        return Ok(HttpResponse::Found().append_header(("location", redirection.to_string())).finish());
    }

    let from_database = Query::find_redirection_by_short_url(&data.conn, short.to_string())
        .await
        .map_err(errors::ApiError::from)?;

    if let Some(model) = from_database {
        let saved = cache.cache.add_entry(short.to_string(), model.long_url.to_string());
        if let Err(e) = saved {
            println!("Failed to save short url {} to cache : {}", short, e);
        }
        Ok(HttpResponse::Found().append_header(("location", model.long_url.to_string())).finish())
    } else {
        not_found(&data.templates, request)
    }
}

#[get("/edit/{id}")]
async fn edit(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
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

#[post("/{id}")]
async fn update(
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

#[post("/delete/{id}")]
async fn delete(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
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

fn create_cache() -> Cache {
    let redis_url = env::var("REDIS_URL").ok();

    if let Some(url) = redis_url {
        if let Some(client) = redis::Client::open(url).ok() {
            println!("Using redis as cache");
            Cache::Redis(client)
        } else {
            println!("Failed to open redis connection, fallback to in-memory cache");
            Cache::InMemory(HashMap::new())
        }
    } else {
        println!("No redis url found, using in-memory cache");
        Cache::InMemory(HashMap::new())
    }
}

#[actix_web::main]
async fn start() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "error");
    tracing_subscriber::fmt::init();

    // get env vars
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{}:{}", host, port);

    let conn = Database::connect(&db_url).await.expect("Failed to connet to the database");

    Migrator::up(&conn, None).await.unwrap();

    // load tera templates and build app state
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let state = AppState { templates, conn: conn };

    // create server and try to serve over socket if possible
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .service(Fs::new("/static", "./api/static"))
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(Mutex::new(AppCache { cache: create_cache() })))
            // .wrap(middleware::Logger::default()) // enable logger
            .configure(init)
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => server.bind(&server_url)?,
    };

    println!("Starting server at {}", server_url);
    server.run().await?;

    Ok(())
}

fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(list);
    cfg.service(new);
    cfg.service(create);
    cfg.service(edit);
    cfg.service(update);
    cfg.service(delete);
    cfg.service(redirect);
}

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {}", err);
    }
}
