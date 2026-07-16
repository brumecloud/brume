use anyhow::Result;
use sqlx::Row;
use uuid::Uuid;

use crate::{deployments, plans, state::AppState};

pub async fn run(state: &AppState) -> Result<()> {
    loop {
        let mut transaction = state.database.begin().await?;
        let plans = plans::claim_expired_plans(&mut transaction, 50).await?;
        transaction.commit().await?;
        if plans.is_empty() {
            break;
        }
        for (user_id, plan_id) in plans {
            if let Err(error) = plans::delete_plan_objects_and_row(state, user_id, plan_id).await {
                tracing::error!(%user_id, %plan_id, error = %error.status(), "retention deletion failed");
            } else {
                tracing::info!(%user_id, %plan_id, "deleted expired plan");
            }
        }
    }
    loop {
        let mut transaction = state.database.begin().await?;
        let expired = deployments::claim_expired(&mut transaction, 50).await?;
        transaction.commit().await?;
        if expired.is_empty() {
            break;
        }
        for (user_id, deployment_id) in expired {
            if let Err(error) =
                deployments::delete_objects_and_row(state, user_id, deployment_id).await
            {
                tracing::error!(
                    %user_id,
                    %deployment_id,
                    error = %error.status(),
                    "deployment retention deletion failed"
                );
            } else {
                tracing::info!(%user_id, %deployment_id, "deleted expired deployment");
            }
        }
    }
    cleanup_superseded_bundles(state).await?;
    deployments::cleanup_superseded(state).await?;
    cleanup_expired_auth(state).await?;
    Ok(())
}

async fn cleanup_superseded_bundles(state: &AppState) -> Result<()> {
    let rows = sqlx::query(
        "SELECT id, object_prefix FROM plan_bundles
         WHERE status = 'superseded'
         ORDER BY created_at
         LIMIT 100",
    )
    .fetch_all(&state.database)
    .await?;
    for row in rows {
        let id: Uuid = row.try_get("id")?;
        let prefix: String = row.try_get("object_prefix")?;
        state.storage.delete_prefix(&prefix).await?;
        sqlx::query("DELETE FROM plan_bundles WHERE id = $1 AND status = 'superseded'")
            .bind(id)
            .execute(&state.database)
            .await?;
    }
    Ok(())
}

async fn cleanup_expired_auth(state: &AppState) -> Result<()> {
    sqlx::query("DELETE FROM oauth_states WHERE expires_at <= now()")
        .execute(&state.database)
        .await?;
    sqlx::query("DELETE FROM cli_login_sessions WHERE expires_at <= now()")
        .execute(&state.database)
        .await?;
    sqlx::query("DELETE FROM web_sessions WHERE expires_at <= now()")
        .execute(&state.database)
        .await?;
    sqlx::query("DELETE FROM deletion_challenges WHERE expires_at <= now()")
        .execute(&state.database)
        .await?;
    Ok(())
}
