mod common;

use std::time::Duration;

use assetdesk::app::router;
use axum::http::{Method, StatusCode};
use common::{
    assert_error, create_asset, create_assignment, create_employee, id, raw_request, request,
    request_with_content_type,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn creates_normalizes_retrieves_and_rejects_duplicate_tags(pool: PgPool) {
    let app = router(pool);
    let response = request(
        &app,
        Method::POST,
        "/api/v1/assets",
        Some(json!({
            "asset_tag": "  lap-021  ",
            "name": "  Development laptop  ",
            "category": "  laptop  "
        })),
    )
    .await;
    assert_eq!(response.status, StatusCode::CREATED, "{}", response.json);
    let asset_id = id(&response.json);
    assert_eq!(
        response.headers["location"]
            .to_str()
            .expect("Location header"),
        format!("/api/v1/assets/{asset_id}")
    );
    assert_eq!(response.json["asset_tag"], "LAP-021");
    assert_eq!(response.json["name"], "Development laptop");
    assert_eq!(response.json["category"], "laptop");
    assert_eq!(response.json["operational_status"], "in_service");
    assert_eq!(response.json["available"], true);
    for timestamp in ["created_at", "updated_at"] {
        assert!(
            chrono::DateTime::parse_from_rfc3339(
                response.json[timestamp].as_str().expect("timestamp string")
            )
            .is_ok()
        );
    }
    let retrieved = request(
        &app,
        Method::GET,
        &format!("/api/v1/assets/{asset_id}"),
        None,
    )
    .await;
    assert_eq!(retrieved.status, StatusCode::OK);
    assert_eq!(retrieved.json, response.json);

    let duplicate = request(
        &app,
        Method::POST,
        "/api/v1/assets",
        Some(json!({"asset_tag":" Lap-021 ", "name":"Other laptop", "category":"laptop"})),
    )
    .await;
    assert_error(&duplicate, StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn validates_create_and_patch_without_losing_omitted_values(pool: PgPool) {
    let app = router(pool);
    for body in [
        json!({"asset_tag":" ", "name":"Laptop", "category":"laptop"}),
        json!({"asset_tag":"X".repeat(65), "name":"Laptop", "category":"laptop"}),
        json!({"asset_tag":"VALID", "name":" ", "category":"laptop"}),
        json!({"asset_tag":"VALID", "name":"X".repeat(121), "category":"laptop"}),
        json!({"asset_tag":"VALID", "name":"Laptop", "category":"X".repeat(61)}),
        json!({"asset_tag":"VALID", "name":"Laptop", "category":" "}),
        json!({"asset_tag":"VALID", "name":"Laptop", "category":"laptop", "operational_status":"broken"}),
        json!({"asset_tag":"VALID", "name":"Laptop", "category":"laptop", "available":true}),
        json!({"asset_tag":"VALID", "category":"laptop"}),
    ] {
        let response = request(&app, Method::POST, "/api/v1/assets", Some(body)).await;
        assert_error(&response, StatusCode::UNPROCESSABLE_ENTITY);
    }
    let original = create_asset(&app, "PATCH-001", "Original", "laptop").await;
    let uri = format!("/api/v1/assets/{}", id(&original));
    for body in [
        json!({"name": null}),
        json!({"category": null}),
        json!({"operational_status": null}),
        json!({"asset_tag": "NEW-TAG"}),
        json!({"unexpected": true}),
        json!({"name": " "}),
        json!({"operational_status": "broken"}),
    ] {
        let response = request(&app, Method::PATCH, &uri, Some(body)).await;
        assert_error(&response, StatusCode::UNPROCESSABLE_ENTITY);
    }
    let updated = request(
        &app,
        Method::PATCH,
        &uri,
        Some(json!({"name": "  Renamed laptop  "})),
    )
    .await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.json);
    assert_eq!(updated.json["name"], "Renamed laptop");
    assert_eq!(updated.json["category"], original["category"]);
    assert_eq!(updated.json["asset_tag"], original["asset_tag"]);
    assert_eq!(updated.json["created_at"], original["created_at"]);
    assert_eq!(updated.json["operational_status"], "in_service");
    assert_eq!(updated.json["available"], true);
}

#[sqlx::test(migrations = "./migrations")]
async fn returns_consistent_errors_for_missing_and_malformed_requests(pool: PgPool) {
    let app = router(pool);
    let missing = format!("/api/v1/assets/{}", Uuid::new_v4());
    for (method, body) in [
        (Method::GET, None),
        (Method::PATCH, Some(json!({"name":"Missing"}))),
    ] {
        assert_error(
            &request(&app, method, &missing, body).await,
            StatusCode::NOT_FOUND,
        );
    }
    assert_error(
        &request(&app, Method::GET, "/api/v1/assets/not-a-uuid", None).await,
        StatusCode::BAD_REQUEST,
    );
    assert_error(
        &raw_request(&app, Method::POST, "/api/v1/assets", Some("{")).await,
        StatusCode::BAD_REQUEST,
    );
    for content_type in [None, Some("text/plain")] {
        let response = request_with_content_type(
            &app,
            Method::POST,
            "/api/v1/assets",
            Some(r#"{"asset_tag":"WRONG-TYPE","name":"Laptop","category":"laptop"}"#),
            content_type,
        )
        .await;
        assert_error(&response, StatusCode::UNSUPPORTED_MEDIA_TYPE);
        assert_eq!(response.headers["content-type"], "application/json");
    }
    let oversized = json!({
        "asset_tag": "TOO-LARGE",
        "name": "x".repeat(2 * 1024 * 1024),
        "category": "laptop"
    })
    .to_string();
    let response = raw_request(&app, Method::POST, "/api/v1/assets", Some(&oversized)).await;
    assert_error(&response, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(response.headers["content-type"], "application/json");
    for query in [
        "limit=abc",
        "sort=password",
        "order=sideways",
        "availability=maybe",
        "operational_status=broken",
    ] {
        assert_error(
            &request(&app, Method::GET, &format!("/api/v1/assets?{query}"), None).await,
            StatusCode::BAD_REQUEST,
        );
    }
    for query in ["limit=0", "limit=101", "offset=-1"] {
        assert_error(
            &request(&app, Method::GET, &format!("/api/v1/assets?{query}"), None).await,
            StatusCode::UNPROCESSABLE_ENTITY,
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn derives_availability_from_current_custody_and_filters_assets(pool: PgPool) {
    let app = router(pool.clone());
    let employee_id = create_employee(&pool).await;
    let held = create_asset(&app, "HELD", "Developer laptop", "laptop").await;
    let returned = create_asset(&app, "RETURNED", "Returned laptop", "laptop").await;
    let maintenance = create_asset(&app, "REPAIR", "Spare monitor", "monitor").await;
    let retired = create_asset(&app, "RETIRED", "Old monitor", "monitor").await;
    create_assignment(&pool, employee_id, id(&held)).await;
    let historical = create_assignment(&pool, employee_id, id(&returned)).await;
    sqlx::query(
        "UPDATE assignments SET returned_at=clock_timestamp(), condition_in='good', \
         version=version+1 WHERE id=$1",
    )
    .bind(historical)
    .execute(&pool)
    .await
    .expect("return fixture");
    for (asset, status) in [(&maintenance, "maintenance"), (&retired, "retired")] {
        let updated = request(
            &app,
            Method::PATCH,
            &format!("/api/v1/assets/{}", id(asset)),
            Some(json!({"operational_status":status})),
        )
        .await;
        assert_eq!(updated.status, StatusCode::OK);
        assert_eq!(updated.json["available"], false);
    }
    for (query, expected_ids) in [
        ("availability=available", vec![id(&returned)]),
        ("availability=assigned", vec![id(&held)]),
        (
            "availability=unavailable&sort=asset_tag&order=asc",
            vec![id(&maintenance), id(&retired)],
        ),
        ("operational_status=maintenance", vec![id(&maintenance)]),
        (
            "category=laptop&availability=available",
            vec![id(&returned)],
        ),
        ("q=developer", vec![id(&held)]),
    ] {
        let response = request(&app, Method::GET, &format!("/api/v1/assets?{query}"), None).await;
        assert_eq!(response.status, StatusCode::OK, "{}", response.json);
        let actual: Vec<_> = response.json["items"]
            .as_array()
            .expect("items")
            .iter()
            .map(id)
            .collect();
        assert_eq!(actual, expected_ids, "{query}");
    }
    let held_response = request(
        &app,
        Method::GET,
        &format!("/api/v1/assets/{}", id(&held)),
        None,
    )
    .await;
    assert_eq!(held_response.json["available"], false);
    assert_eq!(held_response.json["operational_status"], "in_service");
}

#[sqlx::test(migrations = "./migrations")]
async fn searches_literal_wildcards_and_backslashes(pool: PgPool) {
    let app = router(pool);
    let underscore = create_asset(&app, "LAP_100", "Underscore device", "laptop").await;
    let percent = create_asset(&app, "LAP%100", "Percent device", "laptop").await;
    let backslash = create_asset(&app, r"USB\C", "Cable", "accessory").await;
    create_asset(&app, "LAPX100", "Ordinary device", "laptop").await;
    for (query, expected) in [
        ("%5F", id(&underscore)),
        ("%25", id(&percent)),
        ("%5C", id(&backslash)),
    ] {
        let response = request(
            &app,
            Method::GET,
            &format!("/api/v1/assets?q={query}"),
            None,
        )
        .await;
        assert_eq!(response.status, StatusCode::OK, "{}", response.json);
        let items = response.json["items"].as_array().expect("items");
        assert_eq!(items.len(), 1, "{query}: {}", response.json);
        assert_eq!(id(&items[0]), expected);
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn sorts_and_paginates_with_stable_id_tie_breakers(pool: PgPool) {
    let app = router(pool.clone());
    let mut asset_ids = Vec::new();
    for tag in ["SORT-C", "SORT-A", "SORT-B"] {
        asset_ids.push(id(&create_asset(&app, tag, "Shared name", "laptop").await));
    }
    sqlx::query("UPDATE assets SET created_at='2026-01-01T00:00:00Z'")
        .execute(&pool)
        .await
        .expect("tie timestamps");
    asset_ids.sort();
    for (query, expected) in [
        (
            "sort=name&order=asc&limit=2&offset=1",
            asset_ids[1..].to_vec(),
        ),
        ("sort=created_at&order=asc", asset_ids.clone()),
        ("", asset_ids.iter().rev().copied().collect()),
    ] {
        let response = request(&app, Method::GET, &format!("/api/v1/assets?{query}"), None).await;
        assert_eq!(response.status, StatusCode::OK, "{}", response.json);
        let actual: Vec<_> = response.json["items"]
            .as_array()
            .expect("items")
            .iter()
            .map(id)
            .collect();
        assert_eq!(actual, expected, "{query}");
        if query.is_empty() {
            assert_eq!(response.json["limit"], 20);
            assert_eq!(response.json["offset"], 0);
        }
    }
    let by_tag = request(
        &app,
        Method::GET,
        "/api/v1/assets?sort=asset_tag&order=asc&limit=1&offset=1",
        None,
    )
    .await;
    assert_eq!(by_tag.json["items"][0]["asset_tag"], "SORT-B");
    assert_eq!(by_tag.json["limit"], 1);
    assert_eq!(by_tag.json["offset"], 1);
    let empty = request(&app, Method::GET, "/api/v1/assets?offset=100", None).await;
    assert_eq!(empty.status, StatusCode::OK);
    assert_eq!(empty.json["items"], json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn rejects_assigned_asset_status_changes_atomically(pool: PgPool) {
    let app = router(pool.clone());
    let asset = create_asset(&app, "GUARDED", "Original", "laptop").await;
    let employee_id = create_employee(&pool).await;
    create_assignment(&pool, employee_id, id(&asset)).await;
    let uri = format!("/api/v1/assets/{}", id(&asset));
    for status in ["maintenance", "retired"] {
        let response = request(
            &app,
            Method::PATCH,
            &uri,
            Some(json!({"name":"Must not persist", "operational_status":status})),
        )
        .await;
        assert_error(&response, StatusCode::CONFLICT);
        let current = request(&app, Method::GET, &uri, None).await;
        assert_eq!(current.json["name"], "Original");
        assert_eq!(current.json["operational_status"], "in_service");
        assert_eq!(current.json["available"], false);
    }
    let rename = request(
        &app,
        Method::PATCH,
        &uri,
        Some(json!({"name":"Assigned but renamed"})),
    )
    .await;
    assert_eq!(rename.status, StatusCode::OK);
    assert_eq!(rename.json["name"], "Assigned but renamed");
    assert_eq!(rename.json["available"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn status_change_waits_for_issue_lock_then_rechecks_custody(pool: PgPool) {
    let app = router(pool.clone());
    let employee_id = create_employee(&pool).await;
    let asset = create_asset(&app, "RACE", "Before race", "laptop").await;
    let asset_id = id(&asset);
    let mut issue = pool.begin().await.expect("begin issue fixture");
    sqlx::query("SELECT id FROM employees WHERE id=$1 FOR UPDATE")
        .bind(employee_id)
        .fetch_one(&mut *issue)
        .await
        .expect("lock employee");
    sqlx::query("SELECT id FROM assets WHERE id=$1 FOR UPDATE")
        .bind(asset_id)
        .fetch_one(&mut *issue)
        .await
        .expect("lock asset");

    let uri = format!("/api/v1/assets/{asset_id}");
    let concurrent_uri = uri.clone();
    let concurrent_app = app.clone();
    let pending_update = tokio::spawn(async move {
        request(
            &concurrent_app,
            Method::PATCH,
            &concurrent_uri,
            Some(json!({"name":"Must not persist", "operational_status":"maintenance"})),
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let waiting: bool = sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM pg_stat_activity \
                 WHERE datname=current_database() AND pid<>pg_backend_pid() \
                 AND wait_event_type='Lock' AND query ILIKE '%assets%' \
                 AND query ILIKE '%FOR UPDATE%')",
            )
            .fetch_one(&pool)
            .await
            .expect("inspect waiting asset transaction");
            if waiting {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("asset PATCH should block behind the in-flight issue's asset lock");

    sqlx::query(
        "INSERT INTO assignments (id, employee_id, asset_id, condition_out) \
         VALUES ($1, $2, $3, 'good')",
    )
    .bind(Uuid::new_v4())
    .bind(employee_id)
    .bind(asset_id)
    .execute(&mut *issue)
    .await
    .expect("issue before releasing lock");
    issue.commit().await.expect("commit issue fixture");
    let response = tokio::time::timeout(Duration::from_secs(5), pending_update)
        .await
        .expect("asset PATCH completes after issue commits")
        .expect("asset PATCH task");
    assert_error(&response, StatusCode::CONFLICT);
    let current = request(&app, Method::GET, &uri, None).await;
    assert_eq!(current.json["name"], "Before race");
    assert_eq!(current.json["operational_status"], "in_service");
    assert_eq!(current.json["available"], false);
    let open: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM assignments WHERE asset_id=$1 AND returned_at IS NULL",
    )
    .bind(asset_id)
    .fetch_one(&pool)
    .await
    .expect("count outstanding custody");
    assert_eq!(open, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn migration_enforces_custody_history_and_return_constraints(pool: PgPool) {
    let app = router(pool.clone());
    let employee_id = create_employee(&pool).await;
    let asset = create_asset(&app, "CONSTRAINTS", "Laptop", "laptop").await;
    let asset_id = id(&asset);
    let first_assignment = create_assignment(&pool, employee_id, asset_id).await;
    let duplicate = sqlx::query(
        "INSERT INTO assignments (id, employee_id, asset_id, condition_out) \
         VALUES ($1, $2, $3, 'good')",
    )
    .bind(Uuid::new_v4())
    .bind(employee_id)
    .bind(asset_id)
    .execute(&pool)
    .await
    .expect_err("second outstanding assignment must fail");
    let database_error = duplicate.as_database_error().expect("database error");
    assert_eq!(database_error.code().as_deref(), Some("23505"));
    assert_eq!(
        database_error.constraint(),
        Some("one_open_assignment_per_asset")
    );

    let free_asset = create_asset(&app, "INVALID-RETURNS", "Monitor", "monitor").await;
    for extra_columns_and_values in [
        ("condition_in", "'good'"),
        ("returned_at", "clock_timestamp()"),
        ("returned_at, condition_in", "clock_timestamp(), 'broken'"),
        (
            "returned_at, condition_in",
            "clock_timestamp() - interval '1 day', 'good'",
        ),
        ("expected_return_at", "clock_timestamp() - interval '1 day'"),
        ("version", "0"),
    ] {
        let (column, value) = extra_columns_and_values;
        let statement = format!(
            "INSERT INTO assignments (id, employee_id, asset_id, condition_out, {column}) \
             VALUES ($1, $2, $3, 'good', {value})"
        );
        let error = sqlx::query(&statement)
            .bind(Uuid::new_v4())
            .bind(employee_id)
            .bind(id(&free_asset))
            .execute(&pool)
            .await
            .expect_err("invalid assignment must violate a CHECK constraint");
        assert_eq!(
            error
                .as_database_error()
                .and_then(|error| error.code())
                .as_deref(),
            Some("23514"),
            "{column}"
        );
    }
    sqlx::query(
        "UPDATE assignments SET returned_at=clock_timestamp(), condition_in='good', \
         version=version+1 WHERE id=$1",
    )
    .bind(first_assignment)
    .execute(&pool)
    .await
    .expect("return first fixture");
    create_assignment(&pool, employee_id, asset_id).await;
    let historical_count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM assignments WHERE asset_id=$1")
            .bind(asset_id)
            .fetch_one(&pool)
            .await
            .expect("count custody history");
    assert_eq!(historical_count, 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn readiness_tracks_database_while_health_stays_live(pool: PgPool) {
    let app = router(pool.clone());
    assert_eq!(
        request(&app, Method::GET, "/health", None).await.status,
        StatusCode::OK
    );
    assert_eq!(
        request(&app, Method::GET, "/ready", None).await.status,
        StatusCode::OK
    );
    pool.close().await;
    let not_ready = request(&app, Method::GET, "/ready", None).await;
    assert_error(&not_ready, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        request(&app, Method::GET, "/health", None).await.status,
        StatusCode::OK
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn openapi_documents_assets_filters_and_conflict_responses(pool: PgPool) {
    let app = router(pool);
    let response = request(&app, Method::GET, "/api-docs/openapi.json", None).await;
    assert_eq!(response.status, StatusCode::OK);
    let paths = &response.json["paths"];
    for (path, method, statuses) in [
        (
            "/api/v1/assets",
            "post",
            vec!["201", "409", "413", "415", "422"],
        ),
        ("/api/v1/assets", "get", vec!["200", "400", "422"]),
        ("/api/v1/assets/{id}", "get", vec!["200", "400", "404"]),
        (
            "/api/v1/assets/{id}",
            "patch",
            vec!["200", "404", "409", "413", "415", "422"],
        ),
    ] {
        let operation = &paths[path][method];
        assert!(operation.is_object(), "{method} {path} missing");
        for status in statuses {
            assert!(
                operation["responses"][status].is_object(),
                "{method} {path} missing documented {status}"
            );
        }
    }
    let parameters = paths["/api/v1/assets"]["get"]["parameters"]
        .as_array()
        .expect("collection query parameter documentation");
    for expected in [
        "q",
        "category",
        "operational_status",
        "availability",
        "sort",
        "order",
        "limit",
        "offset",
    ] {
        assert!(
            parameters
                .iter()
                .any(|parameter| parameter["name"] == expected),
            "missing query parameter {expected}"
        );
    }
    let schemas = response.json["components"]["schemas"]
        .as_object()
        .expect("component schemas");
    assert!(schemas.values().any(|schema| {
        let properties = &schema["properties"];
        properties.get("asset_tag").is_some() && properties.get("available").is_some()
    }));
}
