use actix_web::{get, web, post, App, HttpServer, Responder, HttpRequest, HttpResponse};

use actix_web::http::{header, Method, StatusCode};
use actix_web::web::Bytes;


// use std::convert::Infallible;
// use std::net::SocketAddr;
use crate::wallet::{MofN, Multisig, PubKey, Seed, Segwit};
// use crate::handlers;
use anyhow::Result;
use std::convert::TryInto;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

// static NOTFOUND: &[u8] = b"Oops! Not Found";

// fn four_oh_four() -> Result<Response<Body>> {
//     let body = Body::from(NOTFOUND);
//     Response::builder()
//         .status(StatusCode::NOT_FOUND)
//         .body(body)
//         .map_err(|e| anyhow::Error::msg(e))
// }

// fn bad_request(v: String) -> Result<Response<Body>> {
//     let body = Body::from(serde_json::json!({ "error": v }).to_string());
//     Response::builder()
//         .status(StatusCode::BAD_REQUEST)
//         .body(body)
//         .map_err(|e| anyhow::Error::msg(e))
// }

pub async fn start_http_server() -> Result<(), std::io::Error> {
    // Get the port number to listen on (required for heroku deployment).
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let addr = format!("0.0.0.0:{}", port);

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://postgres:ming@localhost/btc_address_server?sslmode=disable".to_string());


    // create db connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(index)
            // .service(post_seed)
            .route("/seed", web::post().to(post_seed))
            // .route("/users", web::get().to(handlers::get_users))
            // .route("/users/{id}", web::get().to(handlers::get_user_by_id))
            // .route("/users", web::post().to(handlers::add_user))
            // .route("/users/{id}", web::delete().to(handlers::delete_user))
            // .route("/seed", web::post().to(handlers::post_seed))
            // .route("/mofn", web::post().to(handlers::post_mofn))

    }).bind(addr)?.run().await


}

#[get("/")]
async fn index() -> impl Responder {
    format!("Hello world")
}


#[post("/seed")]
async fn post_seed(bytes: Bytes) -> Result<String, HttpResponse> {
    // let body = hyper::body::to_bytes(req.into_body()).await?;
    // let body = String::from_utf8(bytes.to_vec()).map_err(|_| HttpResponse::BadRequest().into())?;
    // log::debug!("{}", body);
    // let wallet = Segwit::seed(&serde_json::from_str::<Seed>(&body)?.to_string()).finalize();
    // serde_json::to_string(&wallet).unwrap()
    // Ok(HttpResponse::Ok().json(&wallet).finish())
    match String::from_utf8(bytes.to_vec()) {
        Ok(text) => Ok(format!("Hello, {}!\n", text)),
        Err(_) => Err(HttpResponse::BadRequest().into())
    }

    // Ok(HttpResponse::Ok().json(&body))
}

// async fn post_mofn(req: Request<Body>) -> Result<Response<Body>> {
//     let body = hyper::body::to_bytes(req.into_body()).await?;
//     let body = String::from_utf8(body.to_vec())?;
//     log::debug!("{}", body);
//     let mofn = serde_json::from_str::<MofN>(&body)?;
//     let keys = mofn
//         .public_keys
//         .iter()
//         .map(|key| {
//             let key: PubKey = hex::decode(key)?
//             .try_into()
//             .map_err(|e| anyhow::Error::msg(format!("{:?}", e)))?;
//             Ok(key)
//         })
//         .collect::<Result<Vec<PubKey>>>()?;
//     let wallet = Multisig::m(mofn.m)
//         .n(mofn.n)
//         .public_keys(keys)
//         .generate_address()?
//     .finalize();
//     Ok(Response::new(
//         serde_json::to_string(&wallet).unwrap().into(),
//     ))
// }
