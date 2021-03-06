use actix_web::{error, get, post, web, App, HttpResponse, HttpServer, Responder, Result};

#[cfg(feature = "postgres")]
use crate::{handlers, models};
#[cfg(feature = "postgres")]
use diesel::{
    prelude::*,
    r2d2::{self, ConnectionManager},
};

use crate::auth;
use crate::wallet::{MofN, Multisig, PubKey, Seed, Segwit};

use std::convert::TryInto;

#[cfg(feature = "postgres")]
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

use actix_web_httpauth::middleware::HttpAuthentication;

pub async fn start_http_server() -> Result<(), std::io::Error> {
    // Get the port number to listen on (required for heroku deployment).
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let addr = format!("0.0.0.0:{}", port);

    #[cfg(feature = "postgres")]
    {
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:ming@localhost/btc_address_server?sslmode=disable".to_string()
        });

        // create db connection pool
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool: Pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");

        HttpServer::new(move || {
            App::new()
                .wrap(HttpAuthentication::bearer(auth::validator))
                .data(pool.clone())
                .service(index)
                .service(post_seed)
                .service(post_mofn)
                .service(handlers::get_user_by_id)
                .service(handlers::add_user)
                .service(handlers::delete_user)
            // .route("/users", web::get().to(handlers::get_users))
        })
        .bind(addr)?
        .run()
        .await
    }
    #[cfg(not(feature = "postgres"))]
    {
        HttpServer::new(move || {
            App::new()
                .wrap(HttpAuthentication::bearer(auth::validator))
                .service(index)
                .service(post_seed)
                .service(post_mofn)
        })
        .bind(addr)?
        .run()
        .await
    }
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
