use assetdesk::features::{
    categories::model::TicketType,
    tickets::{
        dto::{CreateInputError, CreateTicketInput},
        model::Priority,
        service::{CreationError, validate_creation_in_db},
    },
};
use sqlx::PgPool;
use uuid::Uuid;

#[tokio::test]
#[ignore = "requires a migrated AssetDesk PostgreSQL database with the seeded Others category"]
async fn incident_others_uses_stored_category_for_concern_validation() {
    dotenvy::dotenv().ok();
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL required"))
        .await
        .unwrap();
    let category_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM categories WHERE ticket_type = 'incident' AND name = 'Others' AND is_active",
    ).fetch_one(&pool).await.unwrap();
    let mut input = CreateTicketInput {
        category_id,
        ticket_type: TicketType::Incident,
        priority: Priority::P3,
        subject: "Login interrupted".into(),
        description: "Login stops after pairing".into(),
        custom_concern: None,
    };
    assert_eq!(
        validate_creation_in_db(&pool, &input).await,
        Err(CreationError::Input(CreateInputError::MissingCustomConcern))
    );
    input.custom_concern = Some("Pairing a headset locks the session".into());
    assert_eq!(validate_creation_in_db(&pool, &input).await, Ok(()));
    input.category_id = Uuid::nil();
    assert_eq!(
        validate_creation_in_db(&pool, &input).await,
        Err(CreationError::CategoryNotFound)
    );
}
