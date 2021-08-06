use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;
// use std::net::SocketAddr;
use crate::{
    read_seed,
    wallet::{Address, Multisig, Segwit},
};
use anyhow::Result;
use std::convert::TryInto;

async fn new_addresses(_req: Request<Body>) -> Result<Response<Body>> {
    Ok(Response::new(
        format!(
            "{}\n{}",
            Segwit::from_seed(&read_seed("seed.txt").expect("Readd seed.txt error"))?,
            Multisig::new(
                3,
                3,
                vec![
                    hex::decode(
                        "03d728ad6757d4784effea04d47baafa216cf474866c2d4dc99b1e8e3eb936e730"
                    )?
                    .try_into()
                    .expect("slice with incorrect length"),
                    hex::decode(
                        "03aeb681df5ac19e449a872b9e9347f1db5a0394d2ec5caf2a9c143f86e232b0d9"
                    )?
                    .try_into()
                    .expect("slice with incorrect length"),
                    hex::decode(
                        "02d83bba35a8022c247b645eed6f81ac41b7c1580de550e7e82c75ad63ee9ac2fd"
                    )?
                    .try_into()
                    .expect("slice with incorrect length"),
                ]
            )?
        )
        .into(),
    ))
}

pub async fn start_http_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(|_conn| {
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async { Ok::<_, Infallible>(service_fn(new_addresses)) }
    });

    let addr = ([127, 0, 0, 1], 8080).into();

    let server = Server::bind(&addr).serve(make_svc);

    log::info!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
