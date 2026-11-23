use assetdesk::features::tickets::http::{AuthenticatedActor, router};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use sqlx::PgPool;
use tower::ServiceExt;

#[sqlx::test]
#[ignore = "requires disposable PostgreSQL databases"]
async fn http_creation_checks_identity_and_returns_committed_ticket(pool: PgPool) {
    let app = router().with_state(pool.clone());
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/v1/tickets")
                .header("x-user-id", "00000000-0000-0000-0000-000000000001")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let user = sqlx::query_scalar("INSERT INTO users(name,email,role) VALUES('Employee','employee@example.test','employee') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let actor = AuthenticatedActor { user_id: user };
    let category: uuid::Uuid = sqlx::query_scalar("SELECT id FROM categories WHERE name='Others'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let body = serde_json::json!({
        "ticket_type":"incident", "category_id":category, "priority":"p2",
        "subject":"VPN", "description":"Disconnected", "custom_concern":"VPN disconnects"
    });
    let mut request = Request::builder()
        .method("POST")
        .uri("/api/v1/tickets")
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    request.extensions_mut().insert(actor);
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let location = response.headers()["location"].to_str().unwrap().to_owned();
    let mut request = Request::builder()
        .uri(location)
        .body(Body::empty())
        .unwrap();
    request.extensions_mut().insert(actor);
    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let ticket: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 8192).await.unwrap()).unwrap();
    assert_eq!(ticket["subject"], "VPN");
    assert_eq!(ticket["requester_id"], user.to_string());
    sqlx::query("UPDATE users SET account_status='inactive' WHERE id=$1")
        .bind(user)
        .execute(&pool)
        .await
        .unwrap();
    let mut request = Request::builder()
        .uri("/api/v1/tickets")
        .body(Body::empty())
        .unwrap();
    request.extensions_mut().insert(actor);
    assert_eq!(
        app.oneshot(request).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
}
