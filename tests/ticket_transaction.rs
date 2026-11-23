use assetdesk::features::{
    categories::model::TicketType,
    tickets::{
        dto::CreateTicketInput,
        model::Priority,
        repository::{CreateTicketError, create_ticket},
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
