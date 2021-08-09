use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
// use std::convert::Infallible;
// use std::net::SocketAddr;
use crate::wallet::{MofN, Multisig, PubKey, Seed, Segwit};
use anyhow::Result;
use std::convert::TryInto;

static NOTFOUND: &[u8] = b"Oops! Not Found";

async fn router(req: Request<Body>) -> Result<Response<Body>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") => Ok(Response::new(
            format!("Welcome to bitcoin address server",).into(),
        )),
        (&Method::POST, "/seed") => match post_seed(req).await {
            Ok(response) => Ok(response),
            Err(e) => bad_request(e.to_string()),
        },
        (&Method::POST, "/mofn") => match post_mofn(req).await {
            Ok(response) => Ok(response),
            Err(e) => bad_request(e.to_string()),
        },
        _ => four_oh_four(),
    }
}

async fn post_seed(req: Request<Body>) -> Result<Response<Body>> {
    let body = hyper::body::to_bytes(req.into_body()).await?;
    let body = String::from_utf8(body.to_vec())?;
    log::debug!("{}", body);
    let wallet = Segwit::seed(&serde_json::from_str::<Seed>(&body)?.to_string()).finalize();
    Ok(Response::new(
        serde_json::to_string(&wallet).unwrap().into(),
    ))
}

async fn post_mofn(req: Request<Body>) -> Result<Response<Body>> {
    let body = hyper::body::to_bytes(req.into_body()).await?;
    let body = String::from_utf8(body.to_vec())?;
    log::debug!("{}", body);
    let mofn = serde_json::from_str::<MofN>(&body)?;
    let keys = mofn
        .public_keys
        .iter()
        .map(|key| {
            let key: PubKey = hex::decode(key)?
                .try_into()
                .map_err(|e| anyhow::Error::msg(format!("{:?}", e)))?;
            Ok(key)
        })
        .collect::<Result<Vec<PubKey>>>()?;
    let wallet = Multisig::m(mofn.m)
        .n(mofn.n)
        .public_keys(keys)
        .generate_address()?
        .finalize();
    Ok(Response::new(
        serde_json::to_string(&wallet).unwrap().into(),
    ))
}

fn four_oh_four() -> Result<Response<Body>> {
    let body = Body::from(NOTFOUND);
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(body)
        .map_err(|e| anyhow::Error::msg(e))
}

fn bad_request(v: String) -> Result<Response<Body>> {
    let body = Body::from(serde_json::json!({ "error": v }).to_string());
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
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

    let addr = ([127, 0, 0, 1], 80).into();

    let server = Server::bind(&addr).serve(make_svc);

    log::info!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
