use axum::{routing::{get, post}, Router, Json};
use tower_http::services::ServeDir;
use crate::{validate::validate_fixture, builder::build};
use serde_json::Value;

pub async fn run() {
    let port: u16 = std::env::var("PORT")
        .ok().and_then(|p| p.parse().ok()).unwrap_or(3000);
    
    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/build", post(build_handler))
        .fallback_service(ServeDir::new("public"));

    let addr = format!("127.0.0.1:{}", port);
    println!("http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(serde_json::json!({ "ok": true }))
}

async fn build_handler(Json(body): Json<Value>) -> Json<Value> {
    use crate::types::{Report, ErrorReport, ErrorDetail};
    match validate_fixture(body).map_err(|e| (e.code().to_string(), e.to_string()))
        .and_then(|f| build(f).map_err(|e| ("BUILD_ERROR".to_string(), e.to_string())))
    {
        Ok(report) => Json(serde_json::to_value(Report::Ok(report)).unwrap()),
        Err((code, msg)) => Json(serde_json::to_value(Report::Err(ErrorReport {
            ok: false,
            error: ErrorDetail { code, message: msg },
        })).unwrap()),
    }
}
