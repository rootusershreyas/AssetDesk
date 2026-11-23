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

async fn request(
    app: &axum::Router,
    actor: uuid::Uuid,
    method: &str,
    path: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    request
        .extensions_mut()
        .insert(AuthenticatedActor { user_id: actor });
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    (status, body)
}

#[sqlx::test]
#[ignore = "requires disposable PostgreSQL databases"]
async fn http_lifecycle_preserves_history_and_checks_roles(pool: PgPool) {
    use serde_json::json;
    let app = router().with_state(pool.clone());
    let mut users = Vec::new();
    for role in ["employee", "it_manager", "it_member"] {
        let id: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO users(name,email,role) VALUES($1,$1 || '@example.test',$1) RETURNING id",
        )
        .bind(role)
        .fetch_one(&pool)
        .await
        .unwrap();
        users.push(id);
    }
    let (owner, manager, member) = (users[0], users[1], users[2]);
    let category: uuid::Uuid = sqlx::query_scalar("SELECT id FROM categories WHERE name='Others'")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO category_agents(category_id,user_id) VALUES($1,$2)")
        .bind(category)
        .bind(member)
        .execute(&pool)
        .await
        .unwrap();
    let (status, created) = request(
        &app,
        owner,
        "POST",
        "/api/v1/tickets",
        json!({
            "ticket_type":"incident", "category_id":category, "priority":"p2",
            "subject":"VPN", "description":"Disconnected", "custom_concern":"Connection drops"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let path = format!("/api/v1/tickets/{}", created["id"].as_str().unwrap());
    let review = format!("{path}/review");
    let update = format!("{path}/status");
    assert_eq!(
        request(
            &app,
            owner,
            "POST",
            &review,
            json!({"expected_version":1,"assignee_id":member})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let (status, reviewed) = request(
        &app,
        manager,
        "POST",
        &review,
        json!({"expected_version":1,"assignee_id":member}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(reviewed["version"], 2);
    for (actor, body, expected) in [
        (
            owner,
            json!({"status":"in_progress","expected_version":2}),
            StatusCode::FORBIDDEN,
        ),
        (
            member,
            json!({"status":"in_progress","expected_version":2}),
            StatusCode::OK,
        ),
        (
            member,
            json!({"status":"awaiting_employee_confirmation","expected_version":3,"resolution":"Updated client"}),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            member,
            json!({"status":"awaiting_employee_confirmation","expected_version":3,"resolution":"Updated client","root_cause":"Old client"}),
            StatusCode::OK,
        ),
        (
            member,
            json!({"status":"resolved","expected_version":4}),
            StatusCode::FORBIDDEN,
        ),
        (
            owner,
            json!({"status":"resolved","expected_version":4}),
            StatusCode::OK,
        ),
        (
            owner,
            json!({"status":"open","expected_version":5,"explanation":"Still disconnecting"}),
            StatusCode::OK,
        ),
    ] {
        assert_eq!(
            request(&app, actor, "PATCH", &update, body).await.0,
            expected
        );
    }
    let (status, detail) = request(&app, owner, "GET", &path, json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(detail["version"], 6);
    assert_eq!(detail["status"], "open");
    assert!(detail["resolution"].is_null());
    let (status, history) =
        request(&app, owner, "GET", &format!("{path}/events"), json!(null)).await;
    assert_eq!(status, StatusCode::OK);
    let events = history.as_array().unwrap();
    assert_eq!(events.len(), 6);
    assert_eq!(events[3]["root_cause"], "Old client");
    assert_eq!(events[5]["explanation"], "Still disconnecting");
    assert_eq!(
        request(&app, member, "GET", &path, json!(null)).await.0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        request(
            &app,
            manager,
            "POST",
            &review,
            json!({"expected_version":6,"assignee_id":member})
        )
        .await
        .0,
        StatusCode::OK
    );
}
