use std::{env, str::FromStr};

use assetdesk::build_app;
use axum::{Router, body::Body, http::Request, http::StatusCode};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use tower::ServiceExt;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

struct TestDatabase {
    name: String,
    admin: PgPool,
    pool: PgPool,
    app: Router,
}

impl TestDatabase {
    async fn create() -> Self {
        let url = env::var("TEST_DATABASE_URL").expect(
            "Set TEST_DATABASE_URL to a disposable PostgreSQL maintenance database (for example, the Compose postgres database)",
        );
        let options = PgConnectOptions::from_str(&url)
            .expect("TEST_DATABASE_URL must be a valid PostgreSQL URL");
        let admin = PgPoolOptions::new()
            .max_connections(1)
            .connect_with(options.clone().database("postgres"))
            .await
            .expect("connect to PostgreSQL maintenance database");

        let name = format!("assetdesk_users_test_{}", Uuid::new_v4().simple());
        sqlx::query(&format!("CREATE DATABASE \"{name}\""))
            .execute(&admin)
            .await
            .expect("create isolated integration-test database");

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect_with(options.database(&name))
            .await
            .expect("connect to isolated integration-test database");
        MIGRATOR
            .run(&pool)
            .await
            .expect("apply project migrations to fresh test database");
        let app = build_app(pool.clone());

        Self {
            name,
            admin,
            pool,
            app,
        }
    }

    async fn cleanup(self) {
        drop(self.app);
        self.pool.close().await;
        sqlx::query(&format!(
            "DROP DATABASE IF EXISTS \"{}\" WITH (FORCE)",
            self.name
        ))
        .execute(&self.admin)
        .await
        .expect("drop isolated integration-test database");
        self.admin.close().await;
    }
}

async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    body: Option<&str>,
) -> axum::response::Response {
    let mut request = Request::builder().method(method).uri(uri);
    if body.is_some() {
        request = request.header("content-type", "application/json");
    }
    let body = body.map(str::to_owned).unwrap_or_default();
    app.clone()
        .oneshot(request.body(Body::from(body)).unwrap())
        .await
        .unwrap()
}

async fn response_json(response: axum::response::Response) -> Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

fn create_body(name: &str, email: &str) -> String {
    json!({"name": name, "email": email, "role": "employee"}).to_string()
}

#[tokio::test]
async fn create_get_and_list_return_same_normalized_user() {
    let db = TestDatabase::create().await;
    let response = send(
        &db.app,
        "POST",
        "/api/v1/users",
        Some(&create_body("Test Employee", "Test.Employee@Example.COM")),
    )
    .await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let location = response
        .headers()
        .get("location")
        .expect("201 response should include Location")
        .to_str()
        .unwrap()
        .to_owned();
    let created = response_json(response).await;
    let id = created["id"].as_str().unwrap();
    assert_eq!(location, format!("/api/v1/users/{id}"));
    assert_eq!(created["email"], "test.employee@example.com");

    let fetched = send(&db.app, "GET", &location, None).await;
    assert_eq!(fetched.status(), StatusCode::OK);
    let fetched = response_json(fetched).await;
    assert_eq!(fetched["id"], created["id"]);
    assert_eq!(fetched["email"], "test.employee@example.com");

    let listed = send(&db.app, "GET", "/api/v1/users", None).await;
    assert_eq!(listed.status(), StatusCode::OK);
    let listed = response_json(listed).await;
    assert_eq!(listed["items"][0]["id"], created["id"]);
    assert_eq!(listed["items"][0]["email"], "test.employee@example.com");
    db.cleanup().await;
}

#[tokio::test]
async fn duplicate_email_with_different_casing_conflicts_without_inserting() {
    let db = TestDatabase::create().await;
    let first = send(
        &db.app,
        "POST",
        "/api/v1/users",
        Some(&create_body("First", "Employee@Example.com")),
    )
    .await;
    assert_eq!(first.status(), StatusCode::CREATED);

    let duplicate = send(
        &db.app,
        "POST",
        "/api/v1/users",
        Some(&create_body("Second", "EMPLOYEE@example.COM")),
    )
    .await;
    assert_eq!(duplicate.status(), StatusCode::CONFLICT);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn invalid_fields_and_malformed_json_have_expected_statuses() {
    let db = TestDatabase::create().await;
    for body in [
        json!({"name": "  ", "email": "valid@example.com", "role": "employee"}),
        json!({"name": "Employee", "email": "not-an-email", "role": "employee"}),
        json!({"name": "Employee", "email": "valid@example.com", "role": "admin"}),
    ] {
        let response = send(&db.app, "POST", "/api/v1/users", Some(&body.to_string())).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    let malformed = send(&db.app, "POST", "/api/v1/users", Some("{not json")).await;
    assert_eq!(malformed.status(), StatusCode::BAD_REQUEST);
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn missing_valid_uuid_returns_not_found() {
    let db = TestDatabase::create().await;
    let id = Uuid::new_v4();
    let response = send(&db.app, "GET", &format!("/api/v1/users/{id}"), None).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    db.cleanup().await;
}

#[tokio::test]
async fn list_pagination_is_stable_disjoint_and_bounds_checked() {
    let db = TestDatabase::create().await;
    for index in 0..4 {
        let response = send(
            &db.app,
            "POST",
            "/api/v1/users",
            Some(&create_body(
                &format!("Employee {index}"),
                &format!("employee{index}@example.com"),
            )),
        )
        .await;
        assert_eq!(response.status(), StatusCode::CREATED);
    }
    sqlx::query("UPDATE users SET created_at = '2020-01-01T00:00:00Z'")
        .execute(&db.pool)
        .await
        .unwrap();
    let expected: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM users ORDER BY id ASC")
        .fetch_all(&db.pool)
        .await
        .unwrap();

    let first = send(&db.app, "GET", "/api/v1/users?limit=2&offset=0", None).await;
    assert_eq!(first.status(), StatusCode::OK);
    let first = response_json(first).await;
    assert_eq!(first["limit"], 2);
    assert_eq!(first["offset"], 0);
    let first_ids = page_ids(&first);
    assert_eq!(
        first_ids,
        expected[0..2]
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );

    let repeated = send(&db.app, "GET", "/api/v1/users?limit=2&offset=0", None).await;
    assert_eq!(page_ids(&response_json(repeated).await), first_ids);

    let second = send(&db.app, "GET", "/api/v1/users?limit=2&offset=2", None).await;
    assert_eq!(second.status(), StatusCode::OK);
    let second = response_json(second).await;
    assert_eq!(second["limit"], 2);
    assert_eq!(second["offset"], 2);
    let second_ids = page_ids(&second);
    assert_eq!(
        second_ids,
        expected[2..4]
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert!(first_ids.iter().all(|id| !second_ids.contains(id)));

    let empty = send(&db.app, "GET", "/api/v1/users?limit=2&offset=4", None).await;
    assert_eq!(empty.status(), StatusCode::OK);
    assert!(
        response_json(empty).await["items"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    for uri in ["/api/v1/users?limit=0", "/api/v1/users?offset=-1"] {
        let response = send(&db.app, "GET", uri, None).await;
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
    let malformed_limit = send(&db.app, "GET", "/api/v1/users?limit=abc", None).await;
    assert_eq!(malformed_limit.status(), StatusCode::BAD_REQUEST);
    db.cleanup().await;
}

fn page_ids(page: &Value) -> Vec<String> {
    page["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap().to_owned())
        .collect()
}
