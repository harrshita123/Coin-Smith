use vercel_runtime::{run, service_fn, Body, Error, Request, Response, StatusCode};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(service_fn(handler)).await
}

pub async fn handler(_req: Request) -> Result<Response<Body>, Error> {
    let response_body = json!({ "ok": true }).to_string();

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(response_body.into())?)
}
