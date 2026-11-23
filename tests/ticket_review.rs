use assetdesk::features::tickets::model::Priority;
use assetdesk::features::tickets::{
    model::ReviewError,
    review::{PersistReviewError, review_and_assign},
};
use sqlx::PgPool;
use uuid::Uuid;

async fn user(pool: &PgPool, role: &str) -> Uuid {
    sqlx::query_scalar("INSERT INTO users(role) VALUES($1) RETURNING id")
        .bind(role)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn ticket(pool: &PgPool, priority: &str) -> (Uuid, Uuid) {
    let owner = user(pool, "employee").await;
    sqlx::query_as("INSERT INTO tickets(requester_id,category_id,ticket_type,ticket_number,priority,subject,description)
        SELECT $1,id,ticket_type,'INC1001',$2,'VPN','Disconnected' FROM categories WHERE name='Others' RETURNING id,category_id")
        .bind(owner).bind(priority).fetch_one(pool).await.unwrap()
}

#[sqlx::test(fixtures("ticket_creation", "ticket_review"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn competing_managers_commit_one_review(pool: PgPool) {
    let (id, _) = ticket(&pool, "p1").await;
    let a = user(&pool, "it_manager").await;
    let b = user(&pool, "it_manager").await;
    let (first, second) = tokio::join!(
        review_and_assign(&pool, id, a, None, 1),
        review_and_assign(&pool, id, b, None, 1)
    );
    let winner = match (first, second) {
        (Ok(2), Err(PersistReviewError::Review(ReviewError::VersionConflict))) => a,
        (Err(PersistReviewError::Review(ReviewError::VersionConflict)), Ok(2)) => b,
        results => panic!("unexpected review outcomes: {results:?}"),
    };
    let state: (String,i64,Uuid,Uuid,bool) = sqlx::query_as("SELECT status,version,reviewed_by,assignee_id,reviewed_at IS NOT NULL FROM tickets WHERE id=$1")
        .bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(state, ("open".into(), 2, winner, winner, true));
    let events: Vec<(Uuid,Uuid,i64,String)> = sqlx::query_as("SELECT actor_id,new_assignee_id,resulting_version,new_priority FROM ticket_events WHERE ticket_id=$1 AND event_type='reviewed_and_assigned'")
        .bind(id).fetch_all(&pool).await.unwrap();
    assert_eq!(events, vec![(winner, winner, 2, "p1".into())]);
}

#[sqlx::test(fixtures("ticket_creation", "ticket_review"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn member_eligibility_and_event_rollback(pool: PgPool) {
    let (id, category) = ticket(&pool, "p2").await;
    let manager = user(&pool, "it_manager").await;
    let member = user(&pool, "it_member").await;
    assert!(matches!(
        review_and_assign(&pool, id, member, Some(member), 1).await,
        Err(PersistReviewError::ManagerRequired)
    ));
    assert!(matches!(
        review_and_assign(&pool, id, manager, Some(member), 1).await,
        Err(PersistReviewError::Review(ReviewError::Assignment(_)))
    ));
    sqlx::query("INSERT INTO category_agents(category_id,user_id) VALUES($1,$2)")
        .bind(category)
        .bind(member)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("ALTER TABLE ticket_events ADD CONSTRAINT reject_review CHECK(event_type <> 'reviewed_and_assigned')").execute(&pool).await.unwrap();
    assert!(
        matches!(review_and_assign(&pool,id,manager,Some(member),1).await, Err(PersistReviewError::Database(sqlx::Error::Database(e))) if e.constraint()==Some("reject_review"))
    );
    let pending: (i64,bool,bool,bool) = sqlx::query_as("SELECT version, assignee_id IS NULL, reviewed_by IS NULL, reviewed_at IS NULL FROM tickets WHERE id=$1").bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(pending, (1, true, true, true));
    sqlx::query("ALTER TABLE ticket_events DROP CONSTRAINT reject_review")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        review_and_assign(&pool, id, manager, Some(member), 1)
            .await
            .unwrap(),
        2
    );
    let assigned: Uuid = sqlx::query_scalar("SELECT assignee_id FROM tickets WHERE id=$1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(assigned, member);
}

#[sqlx::test(fixtures("ticket_creation", "ticket_review"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn concurrent_starts_record_one_transition(pool: PgPool) {
    use assetdesk::features::tickets::{
        lifecycle::{PersistStartError, start_assigned_ticket},
        model::StartWorkError,
    };
    let (id, _) = ticket(&pool, "p1").await;
    let manager = user(&pool, "it_manager").await;
    let other = user(&pool, "it_manager").await;
    review_and_assign(&pool, id, manager, None, 1)
        .await
        .unwrap();
    assert!(matches!(
        start_assigned_ticket(&pool, id, other, 2).await,
        Err(PersistStartError::Rule(StartWorkError::NotAssignedHandler))
    ));
    let start = || start_assigned_ticket(&pool, id, manager, 2);
    let (a, b) = tokio::join!(start(), start());
    assert!(matches!(
        (a, b),
        (
            Ok(3),
            Err(PersistStartError::Rule(StartWorkError::VersionConflict))
        ) | (
            Err(PersistStartError::Rule(StartWorkError::VersionConflict)),
            Ok(3)
        )
    ));
    let rows = sqlx::query_as::<_,(String,i64,Uuid,String,String,i64)>(
        "SELECT t.status,t.version,e.actor_id,e.old_status,e.new_status,e.resulting_version
         FROM tickets t JOIN ticket_events e ON t.id=e.ticket_id WHERE t.id=$1 AND e.event_type='started'")
        .bind(id).fetch_all(&pool).await.unwrap();
    assert_eq!(
        rows,
        vec![(
            "in_progress".into(),
            3,
            manager,
            "open".into(),
            "in_progress".into(),
            3
        )]
    );
}

#[sqlx::test(fixtures("ticket_creation", "ticket_review"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn start_rechecks_membership_and_rolls_back_on_event_failure(pool: PgPool) {
    use assetdesk::features::tickets::lifecycle::{PersistStartError, start_assigned_ticket};
    let (id, category) = ticket(&pool, "p3").await;
    let manager = user(&pool, "it_manager").await;
    let member = user(&pool, "it_member").await;
    let membership = "INSERT INTO category_agents(category_id,user_id) VALUES($1,$2)";
    sqlx::query(membership)
        .bind(category)
        .bind(member)
        .execute(&pool)
        .await
        .unwrap();
    review_and_assign(&pool, id, manager, Some(member), 1)
        .await
        .unwrap();
    sqlx::query("DELETE FROM category_agents WHERE category_id=$1 AND user_id=$2")
        .bind(category)
        .bind(member)
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        start_assigned_ticket(&pool, id, member, 2).await,
        Err(PersistStartError::IneligibleHandler)
    ));
    sqlx::query(membership)
        .bind(category)
        .bind(member)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "ALTER TABLE ticket_events ADD CONSTRAINT reject_start CHECK(event_type <> 'started')",
    )
    .execute(&pool)
    .await
    .unwrap();
    assert!(
        matches!(start_assigned_ticket(&pool,id,member,2).await,Err(PersistStartError::Database(sqlx::Error::Database(e))) if e.constraint()==Some("reject_start"))
    );
    let state: (String,i64,i64) = sqlx::query_as("SELECT status,version,(SELECT count(*) FROM ticket_events WHERE ticket_id=$1 AND event_type='started') FROM tickets WHERE id=$1")
        .bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(state, ("open".into(), 2, 0));
    sqlx::query("ALTER TABLE ticket_events DROP CONSTRAINT reject_start")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        start_assigned_ticket(&pool, id, member, 2).await.unwrap(),
        3
    );
}

#[sqlx::test(fixtures("ticket_creation", "ticket_review"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn resolution_requires_current_handler_and_commits_with_event(pool: PgPool) {
    use assetdesk::features::tickets::{
        lifecycle::{PersistResolutionError, resolve_assigned_ticket, start_assigned_ticket},
        service::SubmitResolutionError,
    };
    let (id, category) = ticket(&pool, "p2").await;
    let manager = user(&pool, "it_manager").await;
    let member = user(&pool, "it_member").await;
    sqlx::query("INSERT INTO category_agents(category_id,user_id) VALUES ($1,$2)")
        .bind(category)
        .bind(member)
        .execute(&pool)
        .await
        .unwrap();
    review_and_assign(&pool, id, manager, Some(member), 1)
        .await
        .unwrap();
    start_assigned_ticket(&pool, id, member, 2).await.unwrap();
    let submit =
        |actor, version| resolve_assigned_ticket(&pool, id, actor, version, "  Restored VPN  ");
    assert!(matches!(
        submit(manager, 3).await,
        Err(PersistResolutionError::Rule(
            SubmitResolutionError::NotAssignedHandler
        ))
    ));
    assert!(matches!(
        submit(member, 2).await,
        Err(PersistResolutionError::Rule(
            SubmitResolutionError::VersionConflict
        ))
    ));
    sqlx::query("UPDATE users SET account_status='inactive' WHERE id=$1")
        .bind(member)
        .execute(&pool)
        .await
        .unwrap();
    assert!(matches!(
        submit(member, 3).await,
        Err(PersistResolutionError::TicketNotFound)
    ));
    sqlx::query("UPDATE users SET account_status='active' WHERE id=$1")
        .bind(member)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("ALTER TABLE ticket_events ADD CONSTRAINT reject_resolution CHECK(event_type <> 'resolution_submitted')")
        .execute(&pool).await.unwrap();
    assert!(
        matches!(submit(member,3).await, Err(PersistResolutionError::Database(sqlx::Error::Database(e))) if e.constraint()==Some("reject_resolution"))
    );
    let before: (String, i64, Option<String>) =
        sqlx::query_as("SELECT status,version,resolution FROM tickets WHERE id=$1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(before, ("in_progress".into(), 3, None));
    sqlx::query("ALTER TABLE ticket_events DROP CONSTRAINT reject_resolution")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(submit(member, 3).await.unwrap(), 4);
    let records: Vec<(String,String,String,Uuid)> = sqlx::query_as(
        "SELECT t.status,t.resolution,e.resolution,e.actor_id FROM tickets t JOIN ticket_events e
         ON t.id=e.ticket_id AND t.version=e.resulting_version WHERE t.id=$1 AND e.event_type='resolution_submitted'
         AND e.old_status='in_progress' AND e.new_status=t.status")
        .bind(id).fetch_all(&pool).await.unwrap();
    assert_eq!(
        records,
        vec![(
            "awaiting_employee_confirmation".into(),
            "Restored VPN".into(),
            "Restored VPN".into(),
            member
        )]
    );
    assert!(matches!(
        submit(member, 4).await,
        Err(PersistResolutionError::Rule(
            SubmitResolutionError::InvalidStatus
        ))
    ));
}

#[sqlx::test(fixtures("ticket_creation", "ticket_review"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn requester_confirmation_commits_once_and_preserves_resolution(pool: PgPool) {
    use assetdesk::features::tickets::{
        lifecycle::{
            PersistConfirmationError, confirm_ticket_resolution, resolve_assigned_ticket,
            start_assigned_ticket,
        },
        service::ConfirmResolutionError,
    };
    let (id, _) = ticket(&pool, "p1").await;
    let owner: Uuid = sqlx::query_scalar("SELECT requester_id FROM tickets WHERE id=$1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let manager = user(&pool, "it_manager").await;
    review_and_assign(&pool, id, manager, None, 1)
        .await
        .unwrap();
    start_assigned_ticket(&pool, id, manager, 2).await.unwrap();
    resolve_assigned_ticket(&pool, id, manager, 3, "Restored VPN")
        .await
        .unwrap();
    let confirm = |actor, version| confirm_ticket_resolution(&pool, id, actor, version);
    assert!(matches!(
        confirm(manager, 4).await,
        Err(PersistConfirmationError::Rule(
            ConfirmResolutionError::NotRequester
        ))
    ));
    assert!(matches!(
        confirm(owner, 3).await,
        Err(PersistConfirmationError::Rule(
            ConfirmResolutionError::VersionConflict
        ))
    ));
    sqlx::query("ALTER TABLE ticket_events ADD CONSTRAINT reject_confirm CHECK(event_type <> 'resolution_confirmed')")
        .execute(&pool).await.unwrap();
    assert!(
        matches!(confirm(owner,4).await, Err(PersistConfirmationError::Database(sqlx::Error::Database(e))) if e.constraint()==Some("reject_confirm"))
    );
    let before: (String, i64) = sqlx::query_as("SELECT status,version FROM tickets WHERE id=$1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(before, ("awaiting_employee_confirmation".into(), 4));
    sqlx::query("ALTER TABLE ticket_events DROP CONSTRAINT reject_confirm")
        .execute(&pool)
        .await
        .unwrap();
    let results = tokio::join!(confirm(owner, 4), confirm(owner, 4));
    assert!(matches!(
        results,
        (
            Ok(5),
            Err(PersistConfirmationError::Rule(
                ConfirmResolutionError::VersionConflict
            ))
        ) | (
            Err(PersistConfirmationError::Rule(
                ConfirmResolutionError::VersionConflict
            )),
            Ok(5)
        )
    ));
    assert!(matches!(
        confirm(owner, 5).await,
        Err(PersistConfirmationError::Rule(
            ConfirmResolutionError::InvalidStatus
        ))
    ));
    let rows: Vec<(String,i64,String,Uuid,String,String,i64)> = sqlx::query_as(
        "SELECT t.status,t.version,t.resolution,e.actor_id,e.old_status,e.new_status,e.resulting_version
         FROM tickets t JOIN ticket_events e ON t.id=e.ticket_id WHERE t.id=$1 AND e.event_type='resolution_confirmed'")
        .bind(id).fetch_all(&pool).await.unwrap();
    assert_eq!(
        rows,
        vec![(
            "resolved".into(),
            5,
            "Restored VPN".into(),
            owner,
            "awaiting_employee_confirmation".into(),
            "resolved".into(),
            5
        )]
    );
}

#[sqlx::test(fixtures("ticket_creation", "ticket_review"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn complete_ticket_lifecycle_records_five_ordered_events(pool: PgPool) {
    verify_complete_lifecycle(pool, Priority::P2).await;
}

#[sqlx::test(fixtures("ticket_creation", "ticket_review"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn p1_manager_handles_without_membership_and_requester_confirms(pool: PgPool) {
    verify_complete_lifecycle(pool, Priority::P1).await;
}

async fn verify_complete_lifecycle(pool: PgPool, priority: Priority) {
    use assetdesk::features::{
        categories::model::TicketType,
        tickets::{
            dto::CreateTicketInput,
            lifecycle::{
                confirm_ticket_resolution, resolve_assigned_ticket, start_assigned_ticket,
            },
            repository::{create_ticket, find_ticket_for_requester, list_events_for_requester},
        },
    };
    let requester = user(&pool, "employee").await;
    let manager = user(&pool, "it_manager").await;
    let category_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM categories WHERE ticket_type='incident' AND name='Others'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let requested = if priority == Priority::P1 {
        None
    } else {
        let member = user(&pool, "it_member").await;
        sqlx::query("INSERT INTO category_agents(category_id,user_id) VALUES($1,$2)")
            .bind(category_id)
            .bind(member)
            .execute(&pool)
            .await
            .unwrap();

        Some(member)
    };
    let handler = requested.unwrap_or(manager);
    let input = CreateTicketInput {
        category_id,
        ticket_type: TicketType::Incident,
        priority,
        subject: "VPN reconnect".into(),
        description: "The VPN disconnects after sleep.".into(),
        custom_concern: Some("Reconnecting requires a restart.".into()),
    };
    let id = create_ticket(&pool, requester, &input).await.unwrap();
    let reviewed = review_and_assign(&pool, id, manager, requested, 1)
        .await
        .unwrap();
    let started = start_assigned_ticket(&pool, id, handler, reviewed)
        .await
        .unwrap();
    let awaiting = resolve_assigned_ticket(&pool, id, handler, started, "Updated VPN client.")
        .await
        .unwrap();
    assert!(matches!(
        confirm_ticket_resolution(&pool, id, manager, awaiting).await,
        Err(
            assetdesk::features::tickets::lifecycle::PersistConfirmationError::Rule(
                assetdesk::features::tickets::service::ConfirmResolutionError::NotRequester
            )
        )
    ));
    let confirmed = confirm_ticket_resolution(&pool, id, requester, awaiting)
        .await
        .unwrap();

    let saved = find_ticket_for_requester(&pool, id, requester)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.status, "resolved");
    assert_eq!((confirmed, saved.version), (5, 5));
    let history = list_events_for_requester(&pool, id, requester)
        .await
        .unwrap();
    let events: Vec<_> = history
        .iter()
        .map(|e| (e.resulting_version, e.event_type.as_str(), e.actor_id))
        .collect();
    assert_eq!(
        events,
        vec![
            (1, "created", requester),
            (2, "reviewed_and_assigned", manager),
            (3, "started", handler),
            (4, "resolution_submitted", handler),
            (5, "resolution_confirmed", requester),
        ]
    );
}

#[sqlx::test(fixtures("ticket_creation", "ticket_review"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn manager_review_response_reports_committed_version_and_conflicts(pool: PgPool) {
    use assetdesk::features::tickets::{dto::ReviewTicketInput, routes::review_ticket_response};
    use axum::{body::to_bytes, http::StatusCode};
    use serde_json::{Value, json};

    let (id, _) = ticket(&pool, "p1").await;
    let manager = user(&pool, "it_manager").await;
    let member = user(&pool, "it_member").await;
    let review = |ticket_id, actor, expected_version, assignee_id| {
        review_ticket_response(&pool, actor, ticket_id, ReviewTicketInput { expected_version, assignee_id })
    };
    for (ticket_id, actor, version, assignee, status, code) in [
        (id, manager, 0, None, StatusCode::UNPROCESSABLE_ENTITY, "validation_error"),
        (id, member, 1, None, StatusCode::FORBIDDEN, "manager_required"),
        (Uuid::nil(), manager, 1, None, StatusCode::NOT_FOUND, "ticket_not_found"),
        (id, manager, 1, Some(member), StatusCode::CONFLICT, "ineligible_assignee"),
    ] {
        let response = review(ticket_id, actor, version, assignee).await;
        assert_eq!(response.status(), status);
        let body: Value = serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["error"]["code"], code);
    }
    let response = review(id, manager, 1, None).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(body, json!({"id": id, "version": 2}));
    for (version, code) in [(1, "version_conflict"), (2, "not_pending_review")] {
        let response = review(id, manager, version, None).await;
        assert_eq!(response.status(), StatusCode::CONFLICT);
        let body: Value = serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["error"]["code"], code);
    }
}

#[sqlx::test(fixtures("ticket_creation", "ticket_review"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn start_response_scopes_access_before_reporting_ticket_conflicts(pool: PgPool) {
    use assetdesk::features::tickets::{dto::TicketStatusInput, routes::ticket_status_response};
    use axum::{body::to_bytes, http::StatusCode};
    use serde_json::{Value, json};

    let (id, _) = ticket(&pool, "p1").await;
    let handler = user(&pool, "it_manager").await;
    let manager = user(&pool, "it_manager").await;
    let outsider = user(&pool, "it_member").await;
    let owner: Uuid = sqlx::query_scalar("SELECT requester_id FROM tickets WHERE id=$1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    review_and_assign(&pool, id, handler, None, 1)
        .await
        .unwrap();

    let call = |ticket_id, actor, version| {
        let input = serde_json::from_value::<TicketStatusInput>(
            json!({"status": "in_progress", "expected_version": version}),
        )
        .unwrap();
        ticket_status_response(&pool, actor, ticket_id, input)
    };
    for (ticket_id, actor, version, status, code) in [
        (id, outsider, 1, StatusCode::NOT_FOUND, "ticket_not_found"),
        (
            Uuid::nil(),
            outsider,
            1,
            StatusCode::NOT_FOUND,
            "ticket_not_found",
        ),
        (id, owner, 1, StatusCode::FORBIDDEN, "not_assigned_handler"),
        (
            id,
            manager,
            1,
            StatusCode::FORBIDDEN,
            "not_assigned_handler",
        ),
        (id, handler, 1, StatusCode::CONFLICT, "version_conflict"),
        (
            id,
            handler,
            0,
            StatusCode::UNPROCESSABLE_ENTITY,
            "validation_error",
        ),
    ] {
        let response = call(ticket_id, actor, version).await;
        assert_eq!(response.status(), status);
        let body: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["error"]["code"], code);
    }
    let response = call(id, handler, 2).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 4096).await.unwrap()).unwrap();
    assert_eq!(
        body,
        json!({"id": id, "status": "in_progress", "version": 3})
    );
    assert_eq!(call(id, handler, 3).await.status(), StatusCode::CONFLICT);
}

#[sqlx::test(fixtures("ticket_creation", "ticket_review"))]
#[ignore = "requires disposable PostgreSQL databases"]
async fn resolution_response_validates_access_text_and_committed_result(pool: PgPool) {
    use assetdesk::features::tickets::{
        dto::TicketStatusInput, routes::ticket_status_response, lifecycle::start_assigned_ticket,
    };
    use axum::{body::to_bytes, http::StatusCode};
    let (id, _) = ticket(&pool, "p1").await;
    let manager = user(&pool, "it_manager").await;
    let outsider = user(&pool, "employee").await;
    review_and_assign(&pool, id, manager, None, 1).await.unwrap();
    start_assigned_ticket(&pool, id, manager, 2).await.unwrap();
    for (actor, version, resolution, status) in [
        (outsider, 1, "Fixed", StatusCode::NOT_FOUND),
        (manager, 2, "Fixed", StatusCode::CONFLICT),
        (manager, 3, "  ", StatusCode::UNPROCESSABLE_ENTITY),
        (manager, 3, "Updated client", StatusCode::OK),
    ] {
        let input = serde_json::from_value::<TicketStatusInput>(serde_json::json!({
            "status": "awaiting_employee_confirmation",
            "expected_version": version,
            "resolution": resolution
        })).unwrap();
        let response = ticket_status_response(&pool, actor, id, input).await;
        assert_eq!(response.status(), status);
        if status == StatusCode::OK {
            let body: serde_json::Value = serde_json::from_slice(
                &to_bytes(response.into_body(), 4096).await.unwrap()
            ).unwrap();
            assert_eq!(body, serde_json::json!({
                "id": id, "status": "awaiting_employee_confirmation", "version": 4
            }));
        }
    }
    let saved: (String, i64) = sqlx::query_as("SELECT resolution,version FROM tickets WHERE id=$1")
        .bind(id).fetch_one(&pool).await.unwrap();
    assert_eq!(saved, ("Updated client".into(), 4));
}
