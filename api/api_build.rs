use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};
use serde_json::Value;
use coin_smith::validate::validate_fixture;
use coin_smith::builder::build;
use coin_smith::types::{Report, ErrorReport, ErrorDetail};

#[tokio::main]
async fn main() -> Result<(), Error> {
    run(handler).await
}

pub async fn handler(req: Request) -> Result<Response<Body>, Error> {
    if req.method() != "POST" {
        return Ok(Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .body("Method Not Allowed".into())?);
    }

    let body = req.body();
    let body_bytes = match body {
        Body::Binary(bytes) => bytes.clone(),
        Body::Text(text) => text.as_bytes().to_vec(),
        Body::Empty => return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Empty Body".into())?),
    };

    let json_body: Value = serde_json::from_slice(&body_bytes)?;

    let result = match validate_fixture(json_body).map_err(|e| (e.code().to_string(), e.to_string()))
        .and_then(|f| build(f).map_err(|e| ("BUILD_ERROR".to_string(), e.to_string())))
    {
        Ok(report) => Report::Ok(report),
        Err((code, msg)) => Report::Err(ErrorReport {
            ok: false,
            error: ErrorDetail { code, message: msg },
        }),
    };

    let response_body = serde_json::to_string(&result)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .body(response_body.into())?)
}
