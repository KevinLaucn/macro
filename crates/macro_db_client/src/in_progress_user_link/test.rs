use super::*;
use sqlx::{Pool, Postgres};

async fn insert_macro_user(pool: &Pool<Postgres>, id: Uuid) -> anyhow::Result<()> {
    let suffix = id.to_string();
    let username = format!("tester-{suffix}");
    let email = format!("tester-{suffix}@example.com");
    let stripe_customer_id = format!("cus_{suffix}");
    sqlx::query!(
        r#"
        INSERT INTO macro_user (id, username, email, stripe_customer_id)
        VALUES ($1, $2, $3, $4)
        "#,
        &id,
        username,
        email,
        stripe_customer_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[sqlx::test]
async fn set_linked_email_then_get(pool: Pool<Postgres>) -> anyhow::Result<()> {
    let macro_user_id = macro_uuid::generate_uuid_v7();
    insert_macro_user(&pool, macro_user_id).await?;

    let link_id = create_in_progress_user_link(&pool, &macro_user_id.to_string()).await?;

    let pre = get_in_progress_user_link(&pool, &link_id).await?;
    assert_eq!(pre.macro_user_id, macro_user_id);
    assert!(pre.linked_email.is_none());
    assert!(pre.requested_google_scopes.is_empty());
    assert!(pre.granted_google_scopes.is_empty());

    set_linked_email(&pool, &link_id, "linked@example.com").await?;

    let post = get_in_progress_user_link(&pool, &link_id).await?;
    assert_eq!(post.macro_user_id, macro_user_id);
    assert_eq!(post.linked_email.as_deref(), Some("linked@example.com"));

    Ok(())
}

#[sqlx::test]
async fn google_link_records_requested_and_granted_scopes(
    pool: Pool<Postgres>,
) -> anyhow::Result<()> {
    let macro_user_id = macro_uuid::generate_uuid_v7();
    insert_macro_user(&pool, macro_user_id).await?;
    let requested = vec!["gmail".to_string(), "calendar".to_string()];
    let granted = vec!["gmail".to_string()];

    let link_id =
        create_in_progress_google_link(&pool, &macro_user_id.to_string(), &requested).await?;
    set_linked_google_grant(&pool, &link_id, "linked@example.com", &granted).await?;

    let link = get_in_progress_user_link(&pool, &link_id).await?;
    assert_eq!(link.requested_google_scopes, requested);
    assert_eq!(link.granted_google_scopes, granted);
    assert_eq!(link.linked_email.as_deref(), Some("linked@example.com"));

    Ok(())
}

#[sqlx::test]
async fn count_excludes_expired_links(pool: Pool<Postgres>) -> anyhow::Result<()> {
    let macro_user_id = macro_uuid::generate_uuid_v7();
    insert_macro_user(&pool, macro_user_id).await?;

    // A link created just over 24 hours ago should no longer count toward the cap.
    let expired_link_id = create_in_progress_user_link(&pool, &macro_user_id.to_string()).await?;
    let stale_created_at = chrono::Utc::now().naive_utc() - chrono::Duration::hours(25);
    sqlx::query!(
        r#"
            UPDATE in_progress_user_link
            SET created_at = $1
            WHERE id = $2
        "#,
        stale_created_at,
        expired_link_id
    )
    .execute(&pool)
    .await?;

    // A freshly created link should still count.
    create_in_progress_user_link(&pool, &macro_user_id.to_string()).await?;

    let count =
        count_existing_in_progress_user_links_for_user(&pool, &macro_user_id.to_string()).await?;
    assert_eq!(
        count, 1,
        "in-progress links older than 24 hours should not count toward the cap"
    );

    Ok(())
}

#[sqlx::test]
async fn deletes_only_expired_uncompleted_links_for_user(
    pool: Pool<Postgres>,
) -> anyhow::Result<()> {
    let macro_user_id = macro_uuid::generate_uuid_v7();
    let other_macro_user_id = macro_uuid::generate_uuid_v7();
    insert_macro_user(&pool, macro_user_id).await?;
    insert_macro_user(&pool, other_macro_user_id).await?;

    let expired_uncompleted =
        create_in_progress_user_link(&pool, &macro_user_id.to_string()).await?;
    let expired_completed = create_in_progress_user_link(&pool, &macro_user_id.to_string()).await?;
    set_linked_email(&pool, &expired_completed, "linked@example.com").await?;
    let fresh_uncompleted = create_in_progress_user_link(&pool, &macro_user_id.to_string()).await?;
    let other_user_expired =
        create_in_progress_user_link(&pool, &other_macro_user_id.to_string()).await?;

    let stale_created_at = chrono::Utc::now().naive_utc() - chrono::Duration::hours(25);
    for link_id in [expired_uncompleted, expired_completed, other_user_expired] {
        sqlx::query!(
            r#"
                UPDATE in_progress_user_link
                SET created_at = $1
                WHERE id = $2
            "#,
            stale_created_at,
            link_id
        )
        .execute(&pool)
        .await?;
    }

    delete_expired_uncompleted_in_progress_user_links_for_user(&pool, &macro_user_id.to_string())
        .await?;

    assert!(
        get_in_progress_user_link(&pool, &expired_uncompleted)
            .await
            .is_err()
    );
    assert!(
        get_in_progress_user_link(&pool, &expired_completed)
            .await
            .is_ok()
    );
    assert!(
        get_in_progress_user_link(&pool, &fresh_uncompleted)
            .await
            .is_ok()
    );
    assert!(
        get_in_progress_user_link(&pool, &other_user_expired)
            .await
            .is_ok()
    );

    Ok(())
}

#[sqlx::test]
async fn delete_clears_row(pool: Pool<Postgres>) -> anyhow::Result<()> {
    let macro_user_id = macro_uuid::generate_uuid_v7();
    insert_macro_user(&pool, macro_user_id).await?;

    let link_id = create_in_progress_user_link(&pool, &macro_user_id.to_string()).await?;
    set_linked_email(&pool, &link_id, "linked@example.com").await?;
    delete_in_progress_user_link(&pool, &link_id).await?;

    let err = get_in_progress_user_link(&pool, &link_id).await;
    assert!(err.is_err(), "row should be gone after delete");

    Ok(())
}
