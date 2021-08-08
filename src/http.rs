use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
// use std::convert::Infallible;
// use std::net::SocketAddr;
use crate::wallet::{Address, MofN, Multisig, PubKey, Seed, Segwit, Wallet};
use anyhow::Result;
use std::convert::TryInto;

static NOTFOUND: &[u8] = b"Oops! Not Found";

async fn router(req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(
            format!("Welcome to bitcoin address server",).into(),
        )),
        (&Method::POST, "/seed") => {
            let body = hyper::body::to_bytes(req.into_body()).await?;
            let body = String::from_utf8(body.to_vec())?;
            log::info!("{}", body);
            match serde_json::from_str::<Seed>(&body) {
                Ok(v) => {
                    let segwit = Segwit::from_seed(&v.to_string());
                    let wallet = Wallet::new(segwit.to_address());
                    Ok(Response::new(
                        serde_json::to_string(&wallet).unwrap().into(),
                    ))
                }
                Err(e) => Ok(Response::new(e.to_string().into())),
            }
        }
        (&Method::POST, "/mofn") => {
            let body = hyper::body::to_bytes(req.into_body()).await?;
            let body = String::from_utf8(body.to_vec())?;
            log::info!("{}", body);
            match serde_json::from_str::<MofN>(&body) {
                Ok(v) => {
                    let public_keys = v
                        .public_keys
                        .iter()
                        .map(|key| {
                            let key: PubKey = hex::decode(key)?
                                .try_into()
                                .map_err(|e| anyhow::Error::msg(format!("{:?}", e)))?;
                            Ok(key)
                        })
                        .collect::<Result<Vec<PubKey>>>();
                    match public_keys {
                        Ok(keys) => {
                            let multisig = Multisig::new(v.m, v.n, keys);
                            match multisig {
                                Ok(multisig) => {
                                    let wallet = Wallet::new(multisig.to_address());
                                    Ok(Response::new(
                                        serde_json::to_string(&wallet).unwrap().into(),
                                    ))
                                }
                                Err(e) => Ok(Response::new(e.to_string().into())),
                            }
                        }
                        Err(e) => Ok(Response::new(e.to_string().into())),
                    }
                }
                Err(e) => Ok(Response::new(e.to_string().into())),
            }
        }
        _ => four_oh_four(),
    }
}

fn four_oh_four() -> Result<Response<Body>> {
    let body = Body::from(NOTFOUND);
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(body)
        .map_err(|e| anyhow::Error::msg(e))
}

pub async fn start_http_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, anyhow::Error>(service_fn(router)) }
    });

    let addr = ([127, 0, 0, 1], 8080).into();

    let server = Server::bind(&addr).serve(make_svc);

    log::info!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
