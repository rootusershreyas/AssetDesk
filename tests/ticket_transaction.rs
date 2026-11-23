use assetdesk::features::{
    categories::model::TicketType,
    tickets::{
        dto::CreateTicketInput,
        model::Priority,
        repository::{CreateTicketError, create_ticket, find_ticket_for_requester},
    },
};
use sqlx::PgPool;
use uuid::Uuid;

async fn sample(pool: &PgPool) -> (Uuid, CreateTicketInput<Uuid>) {
    let actor = sqlx::query_scalar("INSERT INTO users DEFAULT VALUES RETURNING id")
        .fetch_one(pool)
        .await
        .unwrap();
    let category_id = sqlx::query_scalar(
        "SELECT id FROM categories WHERE ticket_type='incident' AND name='Others'",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    (
        actor,
        CreateTicketInput {
            ticket_type: TicketType::Incident,
            category_id,
            priority: Priority::P2,
            subject: "VPN".into(),
            description: "Tunnel disconnects".into(),
            custom_concern: Some("Disconnect after sleep".into()),
        },
    )
}

#[sqlx::test(fixtures("ticket_creation"))]
#[ignore = "requires PostgreSQL with permission to create disposable databases"]
async fn commits_ticket_and_created_event(pool: PgPool) {
    let (actor, input) = sample(&pool).await;
    let id = create_ticket(&pool, actor, &input).await.unwrap();
    let rows = sqlx::query_as::<_, (String, String, String, i64, Uuid, String, i64)>(
        "SELECT t.ticket_number, t.status, t.priority, t.version, e.actor_id, e.event_type, e.resulting_version
         FROM tickets t JOIN ticket_events e ON t.id=e.ticket_id WHERE t.id=$1 AND t.requester_id=$2"
    ).bind(id).bind(actor).fetch_all(&pool).await.unwrap();
    assert_eq!(
        rows,
        vec![(
            "INC1001".into(),
            "open".into(),
            "p2".into(),
            1,
            actor,
            "created".into(),
            1
        )]
    );
}

#[sqlx::test(fixtures("ticket_creation"))]
#[ignore = "requires PostgreSQL with permission to create disposable databases"]
async fn failed_event_rolls_back_ticket(pool: PgPool) {
    let (actor, input) = sample(&pool).await;
    sqlx::query("ALTER TABLE ticket_events ADD CONSTRAINT reject_test_event CHECK (event_type <> 'created')")
        .execute(&pool).await.unwrap();
    let error = create_ticket(&pool, actor, &input).await.unwrap_err();
    assert!(
        matches!(error, CreateTicketError::Database(sqlx::Error::Database(ref e))
        if e.constraint() == Some("reject_test_event"))
    );
    let counts: (i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM tickets), (SELECT count(*) FROM ticket_events)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(counts, (0, 0));
}

#[sqlx::test(fixtures("ticket_creation"))]
#[ignore = "requires PostgreSQL with permission to create disposable databases"]
async fn requester_reads_only_their_own_selected_ticket(pool: PgPool) {
    let (requester, input) = sample(&pool).await;
    let (other_requester, other_input) = sample(&pool).await;
    let id = create_ticket(&pool, requester, &input).await.unwrap();
    let other_id = create_ticket(&pool, other_requester, &other_input)
        .await
        .unwrap();
    let ticket = find_ticket_for_requester(&pool, id, requester)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ticket.id, id);
    assert_eq!(ticket.requester_id, requester);
    assert_eq!(ticket.category_id, input.category_id);
    assert_eq!(ticket.subject, input.subject);
    assert_eq!(ticket.description, input.description);
    assert_eq!(ticket.custom_concern, input.custom_concern);
    assert_eq!(ticket.ticket_type, input.ticket_type);
    assert_eq!(ticket.priority, "p2");
    assert_eq!(ticket.status, "open");
    assert_eq!(ticket.version, 1);
    assert_eq!(ticket.ticket_number, "INC1001");
    for invisible_id in [other_id, Uuid::nil()] {
        assert!(
            find_ticket_for_requester(&pool, invisible_id, requester)
                .await
                .unwrap()
                .is_none()
        );
    }
}

#[sqlx::test(fixtures("ticket_creation"))]
#[ignore = "requires PostgreSQL with permission to create disposable databases"]
async fn lists_requester_tickets_in_stable_pages(pool: PgPool) {
    use assetdesk::features::tickets::repository::{ListTicketsError, list_tickets_for_requester};
    let (owner, input) = sample(&pool).await;
    let (other, _) = sample(&pool).await;
    let older = create_ticket(&pool, owner, &input).await.unwrap();
    let a = create_ticket(&pool, owner, &input).await.unwrap();
    let b = create_ticket(&pool, owner, &input).await.unwrap();
    create_ticket(&pool, other, &input).await.unwrap();
    sqlx::query("UPDATE tickets SET created_at = CASE WHEN id = $1 THEN timestamptz '2000-01-01 UTC' ELSE timestamptz '2001-01-01 UTC' END WHERE requester_id = $2")
        .bind(older).bind(owner).execute(&pool).await.unwrap();
    let expected = [a.min(b), a.max(b), older];
    for (offset, expected_page) in [
        (0, &expected[..2]),
        (2, &expected[2..]),
        (3, &expected[3..]),
    ] {
        let rows = list_tickets_for_requester(&pool, owner, Some(2), Some(offset))
            .await
            .unwrap();
        assert_eq!(
            rows.iter().map(|row| row.id).collect::<Vec<_>>(),
            expected_page
        );
    }
    assert_eq!(
        list_tickets_for_requester(&pool, other, None, None)
            .await
            .unwrap()
            .len(),
        1
    );
    for limit in [0, 101] {
        assert!(matches!(
            list_tickets_for_requester(&pool, owner, Some(limit), None).await,
            Err(ListTicketsError::InvalidLimit)
        ));
    }
    assert!(matches!(
        list_tickets_for_requester(&pool, owner, None, Some(-1)).await,
        Err(ListTicketsError::InvalidOffset)
    ));
}

#[sqlx::test(fixtures("ticket_creation"))]
#[ignore = "requires PostgreSQL with permission to create disposable databases"]
async fn requester_filters_precede_pagination(pool: PgPool) {
    use assetdesk::features::tickets::{
        model::TicketStatus, repository::list_filtered_tickets_for_requester,
    };

    let (owner, mut input) = sample(&pool).await;
    let (other, _) = sample(&pool).await;
    let waiting = create_ticket(&pool, owner, &input).await.unwrap();
    sqlx::query("UPDATE tickets SET status = 'awaiting_employee_confirmation' WHERE id = $1")
        .bind(waiting)
        .execute(&pool)
        .await
        .unwrap();
    let open_p2 = create_ticket(&pool, owner, &input).await.unwrap();
    create_ticket(&pool, other, &input).await.unwrap();
    input.priority = Priority::P4;
    let open_p4 = create_ticket(&pool, owner, &input).await.unwrap();
    for (status, priority, limit, mut expected) in [
        (Some(TicketStatus::Open), None, 20, vec![open_p2, open_p4]),
        (None, Some(Priority::P2), 20, vec![waiting, open_p2]),
        (
            Some(TicketStatus::Open),
            Some(Priority::P2),
            1,
            vec![open_p2],
        ),
        (
            Some(TicketStatus::AwaitingEmployeeConfirmation),
            Some(Priority::P2),
            1,
            vec![waiting],
        ),
        (Some(TicketStatus::Resolved), None, 20, vec![]),
    ] {
        let rows =
            list_filtered_tickets_for_requester(&pool, owner, status, priority, Some(limit), None)
                .await
                .unwrap();
        let mut ids: Vec<_> = rows.iter().map(|r| r.id).collect();
        ids.sort();
        expected.sort();
        assert_eq!(ids, expected);
    }
}

#[sqlx::test(fixtures("ticket_creation"))]
#[ignore = "requires PostgreSQL with permission to create disposable databases"]
async fn requester_filters_by_type_and_category(pool: PgPool) {
    use assetdesk::features::tickets::{
        model::TicketStatus,
        repository::{TicketFilters, query_requester_tickets},
    };
    let (owner, mut input) = sample(&pool).await;
    let (other, _) = sample(&pool).await;
    let incident = create_ticket(&pool, owner, &input).await.unwrap();
    input.ticket_type = TicketType::ServiceRequest;
    input.category_id = sqlx::query_scalar("INSERT INTO categories (ticket_type, name) VALUES ('service_request', 'Software') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let software = input.category_id;
    let target = create_ticket(&pool, owner, &input).await.unwrap();
    create_ticket(&pool, other, &input).await.unwrap();
    input.category_id = sqlx::query_scalar("INSERT INTO categories (ticket_type, name) VALUES ('service_request', 'Access') RETURNING id")
        .fetch_one(&pool).await.unwrap();
    let access = create_ticket(&pool, owner, &input).await.unwrap();
    for (filters, mut expected) in [
        (
            TicketFilters {
                ticket_type: Some(TicketType::Incident),
                ..Default::default()
            },
            vec![incident],
        ),
        (
            TicketFilters {
                ticket_type: Some(TicketType::ServiceRequest),
                ..Default::default()
            },
            vec![target, access],
        ),
        (
            TicketFilters {
                category_id: Some(software),
                ..Default::default()
            },
            vec![target],
        ),
        (
            TicketFilters {
                status: Some(TicketStatus::Open),
                priority: Some(Priority::P2),
                ticket_type: Some(TicketType::ServiceRequest),
                category_id: Some(software),
            },
            vec![target],
        ),
        (
            TicketFilters {
                ticket_type: Some(TicketType::Incident),
                category_id: Some(software),
                ..Default::default()
            },
            vec![],
        ),
    ] {
        let rows = query_requester_tickets(&pool, owner, filters, None, None)
            .await
            .unwrap();
        let mut ids: Vec<_> = rows.into_iter().map(|r| r.id).collect();
        ids.sort();
        expected.sort();
        assert_eq!(ids, expected);
    }
}

#[sqlx::test(fixtures("ticket_creation"))]
#[ignore = "requires PostgreSQL with permission to create disposable databases"]
async fn list_query_returns_scoped_json_page(pool: PgPool) {
    use assetdesk::features::tickets::{dto::ListTicketsQuery, service::requester_ticket_page};
    use axum::extract::Query;
    let (owner, input) = sample(&pool).await;
    let (other, _) = sample(&pool).await;
    let id = create_ticket(&pool, owner, &input).await.unwrap();
    create_ticket(&pool, other, &input).await.unwrap();
    let query = Query::<ListTicketsQuery>::try_from_uri(&"/?priority=p2&limit=1".parse().unwrap())
        .unwrap()
        .0;
    let page = requester_ticket_page(&pool, owner, query).await.unwrap();
    let json = serde_json::to_value(page).unwrap();
    assert_eq!(json["items"].as_array().unwrap().len(), 1);
    assert_eq!(json["items"][0]["id"], id.to_string());
    assert_eq!(json["limit"], 1);
    assert_eq!(json["offset"], 0);
}

#[sqlx::test(fixtures("ticket_creation"))]
#[ignore = "requires PostgreSQL with permission to create disposable databases"]
async fn history_is_ordered_and_visible_only_to_requester(pool: PgPool) {
    use assetdesk::features::tickets::repository::list_events_for_requester;
    let (owner, input) = sample(&pool).await;
    let (other, _) = sample(&pool).await;
    let id = create_ticket(&pool, owner, &input).await.unwrap();
    create_ticket(&pool, owner, &input).await.unwrap();
    sqlx::query(
        "INSERT INTO ticket_events (ticket_id, actor_id, event_type, resulting_version)
        VALUES ($1, $2, 'started', 3), ($1, $2, 'reviewed_and_assigned', 2)",
    )
    .bind(id)
    .bind(other)
    .execute(&pool)
    .await
    .unwrap();
    let events = list_events_for_requester(&pool, id, owner).await.unwrap();
    assert_eq!(
        events
            .iter()
            .map(|e| (e.resulting_version, e.event_type.as_str(), e.actor_id))
            .collect::<Vec<_>>(),
        vec![
            (1, "created", owner),
            (2, "reviewed_and_assigned", other),
            (3, "started", other)
        ]
    );
    for (ticket, requester) in [(id, other), (Uuid::nil(), owner)] {
        assert!(
            list_events_for_requester(&pool, ticket, requester)
                .await
                .unwrap()
                .is_empty()
        );
    }
}

#[sqlx::test(fixtures("ticket_creation"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn creation_response_identifies_committed_ticket_and_reports_validation_errors(pool: PgPool) {
    use assetdesk::features::tickets::routes::create_ticket_response;
    use axum::{
        body::to_bytes,
        http::{StatusCode, header},
    };
    let (actor, mut input) = sample(&pool).await;
    let response = create_ticket_response(&pool, actor, &input).await;
    assert_eq!(response.status(), StatusCode::CREATED);
    let location = response.headers()[header::LOCATION]
        .to_str()
        .unwrap()
        .to_owned();
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    let id = Uuid::parse_str(body["id"].as_str().unwrap()).unwrap();
    assert_eq!(location, format!("/api/v1/tickets/{id}"));
    assert!(
        find_ticket_for_requester(&pool, id, actor)
            .await
            .unwrap()
            .is_some()
    );

    input.custom_concern = None;
    let response = create_ticket_response(&pool, actor, &input).await;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body["error"]["code"], "validation_error");
    input.category_id = Uuid::nil();
    assert_eq!(
        create_ticket_response(&pool, actor, &input).await.status(),
        StatusCode::NOT_FOUND
    );
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM tickets")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[sqlx::test(fixtures("ticket_creation"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn ticket_detail_response_is_scoped_to_requester(pool: PgPool) {
    use assetdesk::features::tickets::routes::get_ticket_response;
    use axum::{body::to_bytes, http::StatusCode};
    let (owner, input) = sample(&pool).await;
    let (other, _) = sample(&pool).await;
    let id = create_ticket(&pool, owner, &input).await.unwrap();

    let response = get_ticket_response(&pool, owner, id).await;
    assert_eq!(response.status(), StatusCode::OK);
    let details: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 8192).await.unwrap()).unwrap();
    assert_eq!(details["id"], id.to_string());
    assert_eq!(details["requester_id"], owner.to_string());
    assert_eq!(details["description"], input.description);
    for (actor, ticket_id) in [(other, id), (owner, Uuid::nil())] {
        let response = get_ticket_response(&pool, actor, ticket_id).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 8192).await.unwrap()).unwrap();
        assert_eq!(
            body,
            serde_json::json!({"error": {"code": "ticket_not_found", "message": "Ticket was not found."}})
        );
    }
}

#[sqlx::test(fixtures("ticket_creation"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn history_response_returns_ordered_events_and_hides_other_tickets(pool: PgPool) {
    use assetdesk::features::tickets::routes::{get_ticket_events_response, get_ticket_response};
    use axum::{body::to_bytes, http::StatusCode};
    let (owner, input) = sample(&pool).await;
    let (other, _) = sample(&pool).await;
    let id = create_ticket(&pool, owner, &input).await.unwrap();
    sqlx::query(
        "INSERT INTO ticket_events(ticket_id,actor_id,event_type,resulting_version)
        VALUES($1,$2,'started',3),($1,$2,'reviewed_and_assigned',2)",
    )
    .bind(id)
    .bind(other)
    .execute(&pool)
    .await
    .unwrap();
    let response = get_ticket_events_response(&pool, owner, id).await;
    assert_eq!(response.status(), StatusCode::OK);
    let history: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 8192).await.unwrap()).unwrap();
    assert_eq!(
        history,
        serde_json::json!([
            {"actor_id":owner,"event_type":"created","resulting_version":1,"resolution":null,"root_cause":null,"explanation":null},
            {"actor_id":other,"event_type":"reviewed_and_assigned","resulting_version":2,"resolution":null,"root_cause":null,"explanation":null},
            {"actor_id":other,"event_type":"started","resulting_version":3,"resolution":null,"root_cause":null,"explanation":null}
        ])
    );
    let detail = get_ticket_response(&pool, owner, Uuid::nil()).await;
    let missing_body = to_bytes(detail.into_body(), 8192).await.unwrap();
    for (actor, ticket) in [(other, id), (owner, Uuid::nil())] {
        let response = get_ticket_events_response(&pool, actor, ticket).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            to_bytes(response.into_body(), 8192).await.unwrap(),
            missing_body
        );
    }
}

#[sqlx::test(fixtures("ticket_creation"))]
#[ignore = "requires PostgreSQL with permission to create disposable databases"]
async fn ticket_list_response_scopes_pages_and_reports_invalid_queries(pool: PgPool) {
    use assetdesk::features::tickets::{dto::ListTicketsQuery, routes::list_tickets_response};
    use axum::{body::to_bytes, extract::Query, http::StatusCode};

    let (owner, input) = sample(&pool).await;
    let (other, _) = sample(&pool).await;
    let id = create_ticket(&pool, owner, &input).await.unwrap();
    create_ticket(&pool, other, &input).await.unwrap();
    let parse = |uri: &str| Query::<ListTicketsQuery>::try_from_uri(&uri.parse().unwrap());

    let response = list_tickets_response(&pool, owner, parse("/?priority=p2&limit=10")).await;
    assert_eq!(response.status(), StatusCode::OK);
    let page: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 8192).await.unwrap()).unwrap();
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert_eq!(page["items"][0]["id"], id.to_string());
    assert_eq!(page["limit"], 10);
    assert_eq!(page["offset"], 0);

    let response = list_tickets_response(&pool, owner, parse("/?offset=1")).await;
    assert_eq!(response.status(), StatusCode::OK);
    let page: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 8192).await.unwrap()).unwrap();
    assert_eq!(page, serde_json::json!({"items": [], "limit": 20, "offset": 1}));

    for (uri, status, code) in [
        ("/?priority=unknown", StatusCode::BAD_REQUEST, "invalid_query"),
        ("/?requester_id=other", StatusCode::BAD_REQUEST, "invalid_query"),
        ("/?limit=101", StatusCode::UNPROCESSABLE_ENTITY, "validation_error"),
        ("/?offset=-1", StatusCode::UNPROCESSABLE_ENTITY, "validation_error"),
    ] {
        let response = list_tickets_response(&pool, owner, parse(uri)).await;
        assert_eq!(response.status(), status, "{uri}");
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 8192).await.unwrap()).unwrap();
        assert_eq!(body["error"]["code"], code, "{uri}");
    }
}
