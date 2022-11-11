extern crate core;

use rus_core::{
    sea_orm::{Database, DatabaseConnection},
    Mutation, Query,
};
use actix_files::Files as Fs;
use actix_web::{
    error, get, middleware, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Result,
};

use entity::redirection;
use listenfd::ListenFd;
use migration::{Migrator, MigratorTrait};
use serde::{Deserialize, Serialize};
use std::env;
use tera::Tera;
use url::Url;

const DEFAULT_REDIRECTIONS_PER_PAGE: u64 = 5;

#[derive(Debug, Clone)]
struct AppState {
    templates: tera::Tera,
    conn: DatabaseConnection,
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
    redirection_form: web::Form<CreateForm>,
) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let form = redirection_form.into_inner();
    let url_parsing = Url::parse(form.long_url.as_str());

    if let Err(_err) = url_parsing {
        return Ok(HttpResponse::Found().append_header(("location", "/new")).finish());
    }

    Mutation::create_redirection(conn, form.long_url)
        .await
        .expect("could not insert redirection");

    Ok(HttpResponse::Found()
        .append_header(("location", "/"))
        .finish())
}

#[get("/{id}")]
async fn redirect(data: web::Data<AppState>, id: web::Path<String>) -> Result<HttpResponse, Error> {
    let conn = &data.conn;

    let redirection = Query::find_redirection_by_short_url(conn, id.into_inner())
        .await
        .expect("could not find redirection")
        .unwrap_or_else(|| panic!("Could not find redirection"));

    Ok(HttpResponse::Found().append_header(("location", redirection.long_url.to_string())).finish())
}

#[get("/edit/{id}")]
async fn edit(data: web::Data<AppState>, id: web::Path<i32>) -> Result<HttpResponse, Error> {
    let conn = &data.conn;
    let template = &data.templates;
    let id = id.into_inner();

    let redirection: redirection::Model = Query::find_redirection_by_id(conn, id)
        .await
        .expect("could not find redirection")
        .unwrap_or_else(|| panic!("could not find redirection with id {}", id));

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

    Mutation::update_redirection_by_id(conn, id, form.long_url)
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

async fn not_found(data: web::Data<AppState>, request: HttpRequest) -> Result<HttpResponse, Error> {
    let mut ctx = tera::Context::new();
    ctx.insert("uri", request.uri().path());

    let template = &data.templates;
    let body = template
        .render("error/404.html.tera", &ctx)
        .map_err(|_| error::ErrorInternalServerError("Template error"))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}

#[actix_web::main]
async fn start() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    // get env vars
    dotenvy::dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");
    let server_url = format!("{}:{}", host, port);

    // establish connection to database and apply migrations
    let conn = Database::connect(&db_url).await.unwrap();
    Migrator::up(&conn, None).await.unwrap();

    // load tera templates and build app state
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let state = AppState { templates, conn };

    // create server and try to serve over socket if possible
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .service(Fs::new("/static", "./api/static"))
            .app_data(web::Data::new(state.clone()))
            .wrap(middleware::Logger::default()) // enable logger
            .default_service(web::route().to(not_found))
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
