use axum::{
    Router,
    body::Body,
    http::{HeaderMap, Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

pub struct ApiResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub json: Value,
}

pub async fn request(app: &Router, method: Method, uri: &str, body: Option<Value>) -> ApiResponse {
    let serialized = body.map(|value| serde_json::to_string(&value).expect("serialize request"));
    raw_request(app, method, uri, serialized.as_deref()).await
}

pub async fn raw_request(
    app: &Router,
    method: Method,
    uri: &str,
    body: Option<&str>,
) -> ApiResponse {
    request_with_content_type(app, method, uri, body, body.map(|_| "application/json")).await
}

pub async fn request_with_content_type(
    app: &Router,
    method: Method,
    uri: &str,
    body: Option<&str>,
    content_type: Option<&str>,
) -> ApiResponse {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(content_type) = content_type {
        builder = builder.header("content-type", content_type);
    }
    let response = app
        .clone()
        .oneshot(
            builder
                .body(Body::from(body.unwrap_or_default().to_owned()))
                .expect("build request"),
        )
        .await
        .expect("serve request");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("collect response")
        .to_bytes();
    let json = serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "expected JSON for {uri}, status {status}: {error}; body={}",
            String::from_utf8_lossy(&bytes)
        )
    });
    ApiResponse {
        status,
        headers,
        json,
    }
}

pub fn assert_error(response: &ApiResponse, status: StatusCode) {
    assert_eq!(response.status, status, "{}", response.json);
    for field in ["code", "message"] {
        assert!(
            response.json["error"][field]
                .as_str()
                .is_some_and(|value| !value.is_empty()),
            "error response lacks {field}: {}",
            response.json
        );
    }
}

pub fn id(value: &Value) -> Uuid {
    value["id"]
        .as_str()
        .expect("response id")
        .parse()
        .expect("UUID response id")
}

pub async fn create_asset(app: &Router, tag: &str, name: &str, category: &str) -> Value {
    let response = request(
        app,
        Method::POST,
        "/api/v1/assets",
        Some(json!({"asset_tag": tag, "name": name, "category": category})),
    )
    .await;
    assert_eq!(response.status, StatusCode::CREATED, "{}", response.json);
    response.json
}

pub async fn create_employee(pool: &PgPool) -> Uuid {
    let employee_id = Uuid::new_v4();
    sqlx::query("INSERT INTO employees (id, name, email) VALUES ($1, $2, $3)")
        .bind(employee_id)
        .bind("Test employee")
        .bind(format!("{employee_id}@example.test"))
        .execute(pool)
        .await
        .expect("insert employee fixture");
    employee_id
}

pub async fn create_assignment(pool: &PgPool, employee_id: Uuid, asset_id: Uuid) -> Uuid {
    let assignment_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO assignments (id, employee_id, asset_id, condition_out) \
         VALUES ($1, $2, $3, 'good')",
    )
    .bind(assignment_id)
    .bind(employee_id)
    .bind(asset_id)
    .execute(pool)
    .await
    .expect("insert assignment fixture");
    assignment_id
}
