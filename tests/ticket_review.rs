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
