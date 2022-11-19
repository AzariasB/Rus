extern crate core;

use std::collections::HashMap;
use std::env;
use std::fmt::Debug;
use std::sync::Mutex;

use actix_files::Files as Fs;
use actix_web::{middleware, web, App, HttpServer};
use listenfd::ListenFd;
use log::LevelFilter;
use serde::Deserialize;
use tera::Tera;

use crate::routes::init;
use migration::{Migrator, MigratorTrait};
use rus_core::sea_orm::ConnectOptions;
use rus_core::{
    redis,
    sea_orm::{Database, DatabaseConnection},
    Cache,
};

mod api;
mod conf;
mod errors;
mod routes;

const DEFAULT_REDIRECTIONS_PER_PAGE: u64 = 5;
const DEFAULT_WEB_HOST: &str = "0.0.0.0";
const DEFAULT_WEB_PORT: &str = "8000";

#[derive(Debug, Clone)]
pub struct AppState {
    templates: Tera,
    conn: DatabaseConnection,
}

#[derive(Debug, Clone)]
pub struct AppCache {
    cache: Cache,
}

#[derive(Debug, Deserialize)]
pub struct Params {
    page: Option<u64>,
    redirections_per_page: Option<u64>,
}

#[derive(Deserialize)]
pub struct CreateForm {
    long_url: String,
}

fn create_cache() -> Cache {
    let redis_url = conf::RusEnv::RedisUrl.get();

    if let Some(url) = redis_url {
        if let Ok(client) = redis::Client::open(url) {
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
    env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt::init();

    // get env vars
    dotenvy::dotenv().ok();
    let db_url = conf::RusEnv::DatabaseUrl
        .get()
        .expect("Must set RUS_DATABASE_URL env variable");
    let host = conf::RusEnv::WebHost.get_or(DEFAULT_WEB_HOST.to_owned());
    let port = conf::RusEnv::WebPort.get_or(DEFAULT_WEB_PORT.to_owned());
    let server_url = format!("{}:{}", host, port);
    let mut connect_options = ConnectOptions::new(db_url);
    connect_options.sqlx_logging_level(LevelFilter::Debug);

    let conn = Database::connect(connect_options)
        .await
        .expect("Failed to connect to the database");

    Migrator::up(&conn, None).await.unwrap();

    // load tera templates and build app state
    let templates = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let state = AppState { templates, conn };
    let cache = AppCache {
        cache: create_cache(),
    };

    // create server and try to serve over socket if possible
    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        App::new()
            .service(Fs::new("/static", "./api/static"))
            .app_data(web::Data::new(state.clone()))
            .app_data(web::Data::new(Mutex::new(cache.clone())))
            .wrap(middleware::Logger::default()) // enable logger
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

pub fn main() {
    let result = start();

    if let Some(err) = result.err() {
        println!("Error: {}", err);
    }
}
