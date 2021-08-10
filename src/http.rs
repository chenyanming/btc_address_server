use actix_web::{
    error, get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};

use actix_web::http::{header, Method, StatusCode};
use actix_web::web::Bytes;

use crate::wallet::{MofN, Multisig, PubKey, Seed, Segwit};
// use crate::handlers;
use std::convert::TryInto;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub async fn start_http_server() -> Result<(), std::io::Error> {
    // Get the port number to listen on (required for heroku deployment).
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let addr = format!("0.0.0.0:{}", port);

    // let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:ming@localhost/btc_address_server?sslmode=disable".to_string());

    // create db connection pool
    // let manager = ConnectionManager::<PgConnection>::new(database_url);
    // let pool: Pool = r2d2::Pool::builder()
    //     .build(manager)
    //     .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            // .data(pool.clone())
            .service(index)
            .service(post_seed)
            .service(post_mofn)
        // .route("/users", web::get().to(handlers::get_users))
        // .route("/users/{id}", web::get().to(handlers::get_user_by_id))
        // .route("/users", web::post().to(handlers::add_user))
        // .route("/users/{id}", web::delete().to(handlers::delete_user))
        // .route("/seed", web::post().to(handlers::post_seed))
        // .route("/mofn", web::post().to(handlers::post_mofn))
    })
    .bind(addr)?
    .run()
    .await
}

#[get("/")]
async fn index() -> impl Responder {
    format!("Welcome to bitcoin address server")
}

#[post("/seed")]
async fn post_seed(seed: web::Json<Seed>) -> Result<HttpResponse> {
    let wallet = Segwit::seed(&seed.to_string()).finalize();
    Ok(HttpResponse::Ok().json(wallet))
}

#[post("/mofn")]
async fn post_mofn(mofn: web::Json<MofN>) -> Result<HttpResponse> {
    let keys = mofn
        .public_keys
        .iter()
        .map(|key| {
            let key: PubKey = hex::decode(key)
                .map_err(|e| error::ErrorBadRequest(e))?
                .try_into()
                .map_err(|e| error::ErrorBadRequest(format!("{:?}", e)))?;
            Ok(key)
        })
        .collect::<Result<Vec<PubKey>>>()?;
    let wallet = Multisig::m(mofn.m)
        .n(mofn.n)
        .public_keys(keys)
        .generate_address()
        .map_err(|e| error::ErrorBadRequest(format!("{:?}", e)))?
        .finalize();
    Ok(HttpResponse::Ok().json(wallet))
}
