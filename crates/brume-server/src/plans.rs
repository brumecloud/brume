use std::{
    collections::{HashMap, HashSet},
    io::{Cursor, Read},
};

use ammonia::Builder as HtmlCleaner;
use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
};
use brume_core::{
    AuthMode, BundleManifest, CreateDeletionChallengeResponse, DeployPlanResponse,
    ListPlansResponse, PlanDetails, PlanPatch, PlanSummary, validate_relative_path,
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

use crate::{
    access,
    auth::AuthUser,
    error::ApiError,
    state::AppState,
    util::{hash_secret, random_token},
};

const MAX_ARCHIVE_BYTES: usize = 25 * 1024 * 1024;
const MAX_EXPANDED_BYTES: usize = 50 * 1024 * 1024;
const MAX_FILES: usize = 500;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/plans", get(list_plans))
        .route(
            "/api/v1/plans/{selector}",
            get(get_plan).patch(patch_plan).delete(confirm_deletion),
        )
        .route("/api/v1/plans/{slug}/deploy", post(deploy_plan))
        .route(
            "/api/v1/plans/{selector}/deletion-challenges",
            post(create_deletion_challenge),
        )
        .layer(DefaultBodyLimit::max(MAX_ARCHIVE_BYTES))
}

#[derive(Deserialize)]
struct DeployParameters {
    #[serde(default)]
    auth: AuthMode,
    #[serde(default)]
    pinned: bool,
    #[serde(default = "default_true")]
    overlay: bool,
    #[serde(default)]
    review: bool,
}

fn default_true() -> bool {
    true
}

struct BundleFile {
    bytes: Bytes,
    content_type: String,
}

struct ValidatedBundle {
    manifest: BundleManifest,
    files: HashMap<String, BundleFile>,
}

async fn deploy_plan(
    State(state): State<AppState>,
    user: AuthUser,
    Path(slug): Path<String>,
    Query(parameters): Query<DeployParameters>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DeployPlanResponse>, ApiError> {
    validate_slug(&slug)?;
    let bundle = tokio::task::spawn_blocking(move || validate_bundle(body))
        .await
        .map_err(ApiError::internal)??;
    let existing = sqlx::query(
        "SELECT id, active_bundle_id, access_control_id
         FROM plans WHERE user_id = $1 AND slug = $2",
    )
    .bind(user.id)
    .bind(&slug)
    .fetch_optional(&state.database)
    .await?;
    let plan_id = existing
        .as_ref()
        .map(|row| row.try_get("id"))
        .transpose()?
        .unwrap_or_else(Uuid::now_v7);
    let old_bundle_id: Option<Uuid> = existing
        .as_ref()
        .map(|row| row.try_get("active_bundle_id"))
        .transpose()?
        .flatten();
    let existing_control_id: Option<Uuid> = existing
        .as_ref()
        .map(|row| row.try_get("access_control_id"))
        .transpose()?
        .flatten();
    let password = headers
        .get("x-brume-password")
        .and_then(|value| value.to_str().ok());
    let bundle_id = Uuid::now_v7();
    let prefix = format!("users/{}/plans/{plan_id}/bundles/{bundle_id}", user.id);

    for (path, file) in &bundle.files {
        if let Err(error) = state
            .storage
            .put(
                &format!("{prefix}/{path}"),
                file.bytes.clone(),
                &file.content_type,
            )
            .await
        {
            let _ = state.storage.delete_prefix(&prefix).await;
            return Err(ApiError::internal(error));
        }
    }

    let database_result: Result<(), ApiError> = async {
        let mut transaction = state.database.begin().await?;
        let access_control_id = if let Some(control_id) = existing_control_id {
            access::update_control(
                &mut transaction,
                control_id,
                user.id,
                parameters.auth,
                password,
                parameters.overlay,
            )
            .await?;
            control_id
        } else {
            access::insert_control(
                &mut transaction,
                Uuid::now_v7(),
                user.id,
                parameters.auth,
                password,
                parameters.overlay,
            )
            .await?
        };
        crate::review::supersede_open_round(&mut transaction, access_control_id).await?;
        if parameters.review {
            crate::review::start_round(&mut transaction, access_control_id).await?;
        }
        if existing.is_none() {
            sqlx::query(
                "INSERT INTO plans (
                    id, user_id, slug, title, access_control_id, pinned_at
                 ) VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(plan_id)
            .bind(user.id)
            .bind(&slug)
            .bind(&bundle.manifest.title)
            .bind(access_control_id)
            .bind(parameters.pinned.then(Utc::now))
            .execute(&mut *transaction)
            .await?;
        } else {
            sqlx::query(
                "UPDATE plans SET
                    title = $1,
                    pinned_at = CASE WHEN $2 THEN COALESCE(pinned_at, now()) ELSE NULL END,
                    published_at = now(),
                    last_read_at = NULL,
                    deletion_attempted_at = NULL,
                    updated_at = now(),
                    status = 'active'
                 WHERE id = $3",
            )
            .bind(&bundle.manifest.title)
            .bind(parameters.pinned)
            .bind(plan_id)
            .execute(&mut *transaction)
            .await?;
        }
        sqlx::query(
            "INSERT INTO plan_bundles (
                id, plan_id, object_prefix, renderer_version,
                html_contract_version, manifest, status
             ) VALUES ($1, $2, $3, $4, $5, $6, 'active')",
        )
        .bind(bundle_id)
        .bind(plan_id)
        .bind(&prefix)
        .bind(&bundle.manifest.renderer_version)
        .bind(bundle.manifest.html_contract_version as i32)
        .bind(serde_json::to_value(&bundle.manifest).map_err(ApiError::internal)?)
        .execute(&mut *transaction)
        .await?;
        sqlx::query("UPDATE plans SET active_bundle_id = $1 WHERE id = $2")
            .bind(bundle_id)
            .bind(plan_id)
            .execute(&mut *transaction)
            .await?;
        if let Some(old_bundle_id) = old_bundle_id {
            sqlx::query("UPDATE plan_bundles SET status = 'superseded' WHERE id = $1")
                .bind(old_bundle_id)
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(())
    }
    .await;
    if let Err(error) = database_result {
        if let Err(cleanup_error) = state.storage.delete_prefix(&prefix).await {
            tracing::warn!(%bundle_id, %cleanup_error, "could not clean failed bundle upload");
        }
        return Err(error);
    }

    if let Some(old_bundle_id) = old_bundle_id {
        cleanup_old_bundle(&state, old_bundle_id).await;
    }
    let record = find_owned_plan(&state, &user, &slug).await?;
    let plan = details(&state, &record)?;
    let review_round = if parameters.review {
        crate::review::latest_round(&state, record.access_control_id).await?
    } else {
        None
    };
    Ok(Json(DeployPlanResponse { plan, review_round }))
}

async fn cleanup_old_bundle(state: &AppState, bundle_id: Uuid) {
    let result = async {
        let prefix: Option<String> =
            sqlx::query_scalar("SELECT object_prefix FROM plan_bundles WHERE id = $1")
                .bind(bundle_id)
                .fetch_optional(&state.database)
                .await?;
        if let Some(prefix) = prefix {
            state.storage.delete_prefix(&prefix).await?;
            sqlx::query("DELETE FROM plan_bundles WHERE id = $1 AND status = 'superseded'")
                .bind(bundle_id)
                .execute(&state.database)
                .await?;
        }
        Ok::<_, anyhow::Error>(())
    }
    .await;
    if let Err(error) = result {
        tracing::warn!(%bundle_id, %error, "could not clean superseded plan bundle");
    }
}

async fn list_plans(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<ListPlansResponse>, ApiError> {
    let rows = sqlx::query(
        "SELECT plans.*, users.handle, plan_bundles.manifest,
                access_controls.auth_mode, access_controls.overlay_enabled,
                plan_bundles.renderer_version, plan_bundles.html_contract_version
         FROM plans
         JOIN users ON users.id = plans.user_id
         JOIN plan_bundles ON plan_bundles.id = plans.active_bundle_id
         JOIN access_controls ON access_controls.id = plans.access_control_id
         WHERE plans.user_id = $1 AND plans.status = 'active'
         ORDER BY plans.updated_at DESC",
    )
    .bind(user.id)
    .fetch_all(&state.database)
    .await?;
    let plans = rows
        .iter()
        .map(|row| record_from_row(row).and_then(|record| summary(&state, &record)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(ListPlansResponse { plans }))
}

async fn get_plan(
    State(state): State<AppState>,
    user: AuthUser,
    Path(selector): Path<String>,
) -> Result<Json<PlanDetails>, ApiError> {
    let record = find_owned_plan(&state, &user, &selector).await?;
    Ok(Json(details(&state, &record)?))
}

async fn patch_plan(
    State(state): State<AppState>,
    user: AuthUser,
    Path(selector): Path<String>,
    Json(patch): Json<PlanPatch>,
) -> Result<Json<PlanDetails>, ApiError> {
    let current = find_owned_plan(&state, &user, &selector).await?;
    let pinned = patch.pinned.unwrap_or(current.pinned_at.is_some());
    access::patch_control(
        &state,
        current.access_control_id,
        user.id,
        patch.auth,
        patch.password.as_deref(),
        patch.overlay_enabled,
    )
    .await?;
    sqlx::query(
        "UPDATE plans SET
            pinned_at = CASE WHEN $1 THEN COALESCE(pinned_at, now()) ELSE NULL END,
            updated_at = now()
         WHERE id = $2",
    )
    .bind(pinned)
    .bind(current.id)
    .execute(&state.database)
    .await?;
    let record = find_owned_plan(&state, &user, &current.id.to_string()).await?;
    Ok(Json(details(&state, &record)?))
}

async fn create_deletion_challenge(
    State(state): State<AppState>,
    user: AuthUser,
    Path(selector): Path<String>,
) -> Result<Json<CreateDeletionChallengeResponse>, ApiError> {
    let record = find_owned_plan(&state, &user, &selector).await?;
    let challenge = random_token("delete_");
    sqlx::query(
        "INSERT INTO deletion_challenges (
            id, plan_id, user_id, challenge_hash, expires_at
         ) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::now_v7())
    .bind(record.id)
    .bind(user.id)
    .bind(hash_secret(&challenge))
    .bind(Utc::now() + Duration::minutes(5))
    .execute(&state.database)
    .await?;
    Ok(Json(CreateDeletionChallengeResponse {
        challenge,
        expires_in_seconds: 300,
        plan: summary(&state, &record)?,
    }))
}

async fn confirm_deletion(
    State(state): State<AppState>,
    user: AuthUser,
    Path(selector): Path<String>,
    Json(request): Json<brume_core::ConfirmDeletionRequest>,
) -> Result<StatusCode, ApiError> {
    let record = find_owned_plan(&state, &user, &selector).await?;
    let mut transaction = state.database.begin().await?;
    let deleted = sqlx::query(
        "DELETE FROM deletion_challenges
         WHERE plan_id = $1 AND user_id = $2 AND challenge_hash = $3 AND expires_at > now()
         RETURNING id",
    )
    .bind(record.id)
    .bind(user.id)
    .bind(hash_secret(&request.challenge))
    .fetch_optional(&mut *transaction)
    .await?;
    if deleted.is_none() {
        return Err(ApiError::bad_request(
            "Deletion challenge is invalid or expired",
        ));
    }
    sqlx::query(
        "UPDATE plans
         SET status = 'deleting', deletion_attempted_at = now()
         WHERE id = $1",
    )
    .bind(record.id)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    delete_plan_objects_and_row(&state, user.id, record.id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_plan_objects_and_row(
    state: &AppState,
    user_id: Uuid,
    plan_id: Uuid,
) -> Result<(), ApiError> {
    state
        .storage
        .delete_prefix(&format!("users/{user_id}/plans/{plan_id}"))
        .await
        .map_err(ApiError::internal)?;
    let mut transaction = state.database.begin().await?;
    let control_id: Option<Uuid> =
        sqlx::query_scalar("SELECT access_control_id FROM plans WHERE id = $1")
            .bind(plan_id)
            .fetch_optional(&mut *transaction)
            .await?;
    sqlx::query("DELETE FROM plans WHERE id = $1")
        .bind(plan_id)
        .execute(&mut *transaction)
        .await?;
    if let Some(control_id) = control_id {
        sqlx::query("DELETE FROM access_controls WHERE id = $1")
            .bind(control_id)
            .execute(&mut *transaction)
            .await?;
    }
    transaction.commit().await?;
    Ok(())
}

fn validate_slug(slug: &str) -> Result<(), ApiError> {
    if slug.is_empty()
        || slug.len() > 80
        || slug.starts_with('-')
        || slug.ends_with('-')
        || !slug
            .chars()
            .all(|value| value.is_ascii_lowercase() || value.is_ascii_digit() || value == '-')
    {
        return Err(ApiError::bad_request("Invalid plan slug"));
    }
    Ok(())
}

fn validate_bundle(body: Bytes) -> Result<ValidatedBundle, ApiError> {
    let decoder = zstd::Decoder::new(Cursor::new(body)).map_err(ApiError::bad_request)?;
    let mut archive = tar::Archive::new(decoder);
    let mut files = HashMap::new();
    let mut total_size = 0_usize;
    for entry in archive.entries().map_err(ApiError::bad_request)? {
        let mut entry = entry.map_err(ApiError::bad_request)?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        if files.len() >= MAX_FILES {
            return Err(ApiError::bad_request("Bundle contains too many files"));
        }
        let path = entry
            .path()
            .map_err(ApiError::bad_request)?
            .to_string_lossy()
            .trim_start_matches("./")
            .to_owned();
        validate_relative_path(&path).map_err(ApiError::bad_request)?;
        let mut bytes = Vec::new();
        entry
            .by_ref()
            .take((MAX_EXPANDED_BYTES - total_size + 1) as u64)
            .read_to_end(&mut bytes)
            .map_err(ApiError::bad_request)?;
        total_size += bytes.len();
        if total_size > MAX_EXPANDED_BYTES {
            return Err(ApiError::bad_request("Expanded bundle exceeds 50 MiB"));
        }
        if files
            .insert(
                path.clone(),
                BundleFile {
                    content_type: content_type_for(&path),
                    bytes: Bytes::from(bytes),
                },
            )
            .is_some()
        {
            return Err(ApiError::bad_request(format!(
                "Duplicate bundle path `{path}`"
            )));
        }
    }
    let manifest_file = files
        .get("brume-manifest.json")
        .ok_or_else(|| ApiError::bad_request("Bundle manifest is missing"))?;
    let manifest: BundleManifest =
        serde_json::from_slice(&manifest_file.bytes).map_err(ApiError::bad_request)?;
    manifest.validate().map_err(ApiError::bad_request)?;

    let expected = expected_paths(&manifest);
    let actual = files.keys().cloned().collect::<HashSet<_>>();
    if actual != expected {
        return Err(ApiError::bad_request(
            "Bundle files do not match its manifest",
        ));
    }
    verify_hashes(&manifest, &files)?;
    let cleaner = html_cleaner();
    for page in &manifest.pages {
        let file = files
            .get_mut(&page.object_path)
            .ok_or_else(|| ApiError::bad_request("Manifest page is missing"))?;
        let html = std::str::from_utf8(&file.bytes).map_err(ApiError::bad_request)?;
        file.bytes = Bytes::from(cleaner.clean(html).to_string());
        file.content_type = "text/html; charset=utf-8".to_owned();
    }
    for asset in &manifest.assets {
        if let Some(file) = files.get_mut(&asset.path) {
            let expected_content_type = asset_content_type(&asset.path).ok_or_else(|| {
                ApiError::bad_request(format!("Unsupported asset type `{}`", asset.path))
            })?;
            if asset.content_type != expected_content_type {
                return Err(ApiError::bad_request(format!(
                    "Invalid content type for `{}`",
                    asset.path
                )));
            }
            file.content_type = expected_content_type.to_owned();
        }
    }
    Ok(ValidatedBundle { files, manifest })
}

fn expected_paths(manifest: &BundleManifest) -> HashSet<String> {
    let mut result = HashSet::from(["brume-manifest.json".to_owned()]);
    result.extend(manifest.pages.iter().map(|page| page.object_path.clone()));
    result.extend(manifest.assets.iter().map(|asset| asset.path.clone()));
    result.extend(manifest.sources.iter().map(|source| source.path.clone()));
    result
}

fn verify_hashes(
    manifest: &BundleManifest,
    files: &HashMap<String, BundleFile>,
) -> Result<(), ApiError> {
    for (path, expected_hash, expected_size) in manifest
        .assets
        .iter()
        .map(|value| (&value.path, &value.sha256, value.size))
        .chain(
            manifest
                .sources
                .iter()
                .map(|value| (&value.path, &value.sha256, value.size)),
        )
    {
        let file = files
            .get(path)
            .ok_or_else(|| ApiError::bad_request(format!("Missing `{path}`")))?;
        let digest = hex::encode(Sha256::digest(&file.bytes));
        if digest != *expected_hash || file.bytes.len() as u64 != expected_size {
            return Err(ApiError::bad_request(format!(
                "Checksum mismatch for `{path}`"
            )));
        }
    }
    Ok(())
}

fn html_cleaner() -> HtmlCleaner<'static> {
    let mut cleaner = HtmlCleaner::default();
    cleaner
        .add_tags(["aside", "button", "main", "nav", "section"])
        .add_generic_attributes([
            "aria-current",
            "aria-label",
            "aria-selected",
            "class",
            "data-brume-tab-button",
            "data-brume-tab-panel",
            "data-brume-tabs",
            "hidden",
            "id",
            "loading",
            "role",
            "style",
            "tabindex",
            "type",
        ]);
    cleaner
}

fn content_type_for(path: &str) -> String {
    if path.ends_with(".html") {
        "text/html; charset=utf-8".to_owned()
    } else if path.ends_with(".json") {
        "application/json".to_owned()
    } else if path.ends_with(".md") || path.ends_with(".mdx") {
        "text/markdown; charset=utf-8".to_owned()
    } else {
        mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string()
    }
}

fn asset_content_type(path: &str) -> Option<&'static str> {
    let extension = path.rsplit_once('.')?.1.to_ascii_lowercase();
    match extension.as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

struct PlanRecord {
    id: Uuid,
    access_control_id: Uuid,
    handle: String,
    slug: String,
    title: String,
    auth: AuthMode,
    overlay_enabled: bool,
    published_at: DateTime<Utc>,
    last_read_at: Option<DateTime<Utc>>,
    pinned_at: Option<DateTime<Utc>>,
    renderer_version: String,
    html_contract_version: i32,
    manifest: BundleManifest,
}

async fn find_owned_plan(
    state: &AppState,
    user: &AuthUser,
    selector: &str,
) -> Result<PlanRecord, ApiError> {
    let row = sqlx::query(
        "SELECT plans.*, users.handle, plan_bundles.manifest,
                access_controls.auth_mode, access_controls.overlay_enabled,
                plan_bundles.renderer_version, plan_bundles.html_contract_version
         FROM plans
         JOIN users ON users.id = plans.user_id
         JOIN plan_bundles ON plan_bundles.id = plans.active_bundle_id
         JOIN access_controls ON access_controls.id = plans.access_control_id
         WHERE plans.user_id = $1 AND plans.status = 'active'
           AND (plans.id::text = $2 OR plans.slug = $2)",
    )
    .bind(user.id)
    .bind(selector)
    .fetch_optional(&state.database)
    .await?
    .ok_or_else(ApiError::not_found)?;
    record_from_row(&row)
}

fn record_from_row(row: &sqlx::postgres::PgRow) -> Result<PlanRecord, ApiError> {
    let auth = row
        .try_get::<String, _>("auth_mode")?
        .parse()
        .map_err(ApiError::internal)?;
    let manifest = serde_json::from_value(row.try_get("manifest")?).map_err(ApiError::internal)?;
    Ok(PlanRecord {
        id: row.try_get("id")?,
        access_control_id: row.try_get("access_control_id")?,
        handle: row.try_get("handle")?,
        slug: row.try_get("slug")?,
        title: row.try_get("title")?,
        auth,
        overlay_enabled: row.try_get("overlay_enabled")?,
        published_at: row.try_get("published_at")?,
        last_read_at: row.try_get("last_read_at")?,
        pinned_at: row.try_get("pinned_at")?,
        renderer_version: row.try_get("renderer_version")?,
        html_contract_version: row.try_get("html_contract_version")?,
        manifest,
    })
}

fn record_url(state: &AppState, record: &PlanRecord) -> String {
    format!(
        "{}/{}/{}",
        state.config.plan_public_url, record.handle, record.slug
    )
}

fn summary(state: &AppState, record: &PlanRecord) -> Result<PlanSummary, ApiError> {
    Ok(PlanSummary {
        id: record.id,
        owner_handle: record.handle.clone(),
        slug: record.slug.clone(),
        title: record.title.clone(),
        auth: record.auth,
        overlay_enabled: record.overlay_enabled,
        url: record_url(state, record),
        published_at: record.published_at,
        last_read_at: record.last_read_at,
        expires_at: record
            .pinned_at
            .is_none()
            .then(|| record.last_read_at.unwrap_or(record.published_at) + Duration::days(15)),
        pinned: record.pinned_at.is_some(),
    })
}

fn details(state: &AppState, record: &PlanRecord) -> Result<PlanDetails, ApiError> {
    Ok(PlanDetails {
        summary: summary(state, record)?,
        renderer_version: record.renderer_version.clone(),
        html_contract_version: record.html_contract_version as u32,
        routes: record
            .manifest
            .pages
            .iter()
            .map(|page| page.route.clone())
            .collect(),
    })
}

pub async fn claim_expired_plans(
    transaction: &mut Transaction<'_, Postgres>,
    limit: i64,
) -> Result<Vec<(Uuid, Uuid)>, ApiError> {
    let rows = sqlx::query(
        "WITH candidates AS (
            SELECT id FROM plans
            WHERE (status = 'deleting'
                   AND (deletion_attempted_at IS NULL
                        OR deletion_attempted_at < now() - interval '1 hour'))
               OR (
                    status = 'active'
                    AND pinned_at IS NULL
                    AND COALESCE(last_read_at, published_at) < now() - interval '15 days'
               )
            ORDER BY COALESCE(last_read_at, published_at)
            FOR UPDATE SKIP LOCKED
            LIMIT $1
         )
         UPDATE plans
         SET status = 'deleting', deletion_attempted_at = now()
         FROM candidates
         WHERE plans.id = candidates.id
         RETURNING plans.user_id, plans.id",
    )
    .bind(limit)
    .fetch_all(&mut **transaction)
    .await?;
    rows.into_iter()
        .map(|row| Ok((row.try_get("user_id")?, row.try_get("id")?)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_renderer_image_asset_types() {
        assert_eq!(asset_content_type("assets/diagram.png"), Some("image/png"));
        assert_eq!(asset_content_type("assets/photo.JPEG"), Some("image/jpeg"));
        assert_eq!(asset_content_type("assets/page.html"), None);
        assert_eq!(asset_content_type("assets/script.js"), None);
    }

    #[test]
    fn validates_public_plan_slugs() {
        assert!(validate_slug("agent-plan-2").is_ok());
        assert!(validate_slug("../agent-plan").is_err());
        assert!(validate_slug("Agent Plan").is_err());
    }
}
