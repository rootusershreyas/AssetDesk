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
