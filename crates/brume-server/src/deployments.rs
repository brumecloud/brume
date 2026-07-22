use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::Path as FilePath,
};

use axum::{
    Json, Router,
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, Query, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use brume_core::{
    DEPLOYMENT_MAX_FILE_BYTES, DeploySiteResponse, DeploymentFile, DeploymentManifest,
    DeploymentSummary, validate_relative_path,
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::ApiError,
    state::AppState,
    util::{public_label, random_public_id},
};

const MAX_ARCHIVE_BYTES: usize = 110 * 1024 * 1024;
const MAX_EXPANDED_BYTES: usize = 100 * 1024 * 1024;
const MAX_FILES: usize = 5_000;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .route("/api/v1/deployments", post(deploy))
        .layer(DefaultBodyLimit::max(MAX_ARCHIVE_BYTES))
}

#[derive(Deserialize)]
struct DeployParameters {
    slug: Option<String>,
    #[serde(default)]
    spa: bool,
    #[serde(default)]
    pinned: bool,
}

#[derive(Debug)]
struct DeploymentObject {
    bytes: Bytes,
    content_type: String,
}

#[derive(Debug)]
struct ValidatedDeployment {
    manifest: DeploymentManifest,
    files: HashMap<String, DeploymentObject>,
}

async fn deploy(
    State(state): State<AppState>,
    user: AuthUser,
    Query(parameters): Query<DeployParameters>,
    body: Bytes,
) -> Result<Json<DeploySiteResponse>, ApiError> {
    let requested_slug = parameters.slug;
    if let Some(slug) = &requested_slug {
        validate_slug(slug)?;
    }
    let _endpoint_guard = state.public_endpoints.lock().await;
    let generated = requested_slug.is_none();
    let mut candidate = requested_slug.unwrap_or_else(random_public_id);
    let (slug, public_label) = loop {
        let label = public_label(&candidate, &user.handle).ok_or_else(|| {
            ApiError::bad_request("Deployment URL slug is too long for this user handle")
        })?;
        let tunnel_conflict = state.tunnels.contains_label(&label);
        let label_owner =
            sqlx::query("SELECT user_id, slug FROM deployments WHERE public_label = $1")
                .bind(&label)
                .fetch_optional(&state.database)
                .await?;
        let deployment_conflict = label_owner.as_ref().is_some_and(|row| {
            generated
                || row.try_get::<Uuid, _>("user_id").ok() != Some(user.id)
                || row.try_get::<String, _>("slug").ok().as_deref() != Some(candidate.as_str())
        });
        if !tunnel_conflict && !deployment_conflict {
            break (candidate, label);
        }
        if !generated {
            return Err(ApiError::public_url_conflict(
                "This public URL is already used by another tunnel or deployment",
            ));
        }
        candidate = random_public_id();
    };
    let deployment = tokio::task::spawn_blocking(move || validate_archive(body, parameters.spa))
        .await
        .map_err(ApiError::internal)??;
    let existing = sqlx::query(
        "SELECT id, active_bundle_id
         FROM deployments WHERE user_id = $1 AND slug = $2",
    )
    .bind(user.id)
    .bind(&slug)
    .fetch_optional(&state.database)
    .await?;
    let deployment_id = existing
        .as_ref()
        .map(|row| row.try_get("id"))
        .transpose()?
        .unwrap_or_else(Uuid::now_v7);
    let old_bundle_id: Option<Uuid> = existing
        .as_ref()
        .map(|row| row.try_get("active_bundle_id"))
        .transpose()?
        .flatten();
    let bundle_id = Uuid::now_v7();
    let prefix = format!(
        "users/{}/deployments/{deployment_id}/bundles/{bundle_id}",
        user.id
    );

    for (path, file) in &deployment.files {
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
        if existing.is_none() {
            sqlx::query(
                "INSERT INTO deployments (
                    id, user_id, slug, public_label, spa, pinned_at
                 ) VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(deployment_id)
            .bind(user.id)
            .bind(&slug)
            .bind(&public_label)
            .bind(parameters.spa)
            .bind(parameters.pinned.then(Utc::now))
            .execute(&mut *transaction)
            .await?;
        } else {
            sqlx::query(
                "UPDATE deployments SET
                    spa = $1,
                    pinned_at = CASE WHEN $2 THEN COALESCE(pinned_at, now()) ELSE NULL END,
                    published_at = now(),
                    last_read_at = NULL,
                    deletion_attempted_at = NULL,
                    updated_at = now(),
                    status = 'active'
                 WHERE id = $3",
            )
            .bind(parameters.spa)
            .bind(parameters.pinned)
            .bind(deployment_id)
            .execute(&mut *transaction)
            .await?;
        }
        sqlx::query(
            "INSERT INTO deployment_bundles (
                id, deployment_id, object_prefix, manifest, status
             ) VALUES ($1, $2, $3, $4, 'active')",
        )
        .bind(bundle_id)
        .bind(deployment_id)
        .bind(&prefix)
        .bind(serde_json::to_value(&deployment.manifest).map_err(ApiError::internal)?)
        .execute(&mut *transaction)
        .await?;
        sqlx::query("UPDATE deployments SET active_bundle_id = $1 WHERE id = $2")
            .bind(bundle_id)
            .bind(deployment_id)
            .execute(&mut *transaction)
            .await?;
        if let Some(old_bundle_id) = old_bundle_id {
            sqlx::query("UPDATE deployment_bundles SET status = 'superseded' WHERE id = $1")
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
            tracing::warn!(%bundle_id, %cleanup_error, "could not clean failed deployment upload");
        }
        return Err(error);
    }

    if let Some(old_bundle_id) = old_bundle_id {
        cleanup_old_bundle(&state, old_bundle_id).await;
    }
    let row = sqlx::query(
        "SELECT deployments.id, deployments.slug, deployments.public_label, deployments.spa,
                deployments.published_at, deployments.last_read_at,
                deployments.pinned_at, users.handle
         FROM deployments
         JOIN users ON users.id = deployments.user_id
         WHERE deployments.id = $1",
    )
    .bind(deployment_id)
    .fetch_one(&state.database)
    .await?;
    Ok(Json(DeploySiteResponse {
        deployment: summary(&state, &row)?,
    }))
}

async fn cleanup_old_bundle(state: &AppState, bundle_id: Uuid) {
    let result = async {
        let prefix: Option<String> =
            sqlx::query_scalar("SELECT object_prefix FROM deployment_bundles WHERE id = $1")
                .bind(bundle_id)
                .fetch_optional(&state.database)
                .await?;
        if let Some(prefix) = prefix {
            state.storage.delete_prefix(&prefix).await?;
            sqlx::query("DELETE FROM deployment_bundles WHERE id = $1 AND status = 'superseded'")
                .bind(bundle_id)
                .execute(&state.database)
                .await?;
        }
        Ok::<_, anyhow::Error>(())
    }
    .await;
    if let Err(error) = result {
        tracing::warn!(%bundle_id, %error, "could not clean superseded deployment bundle");
    }
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
        return Err(ApiError::bad_request("Invalid deployment URL slug"));
    }
    Ok(())
}

fn validate_archive(body: Bytes, spa: bool) -> Result<ValidatedDeployment, ApiError> {
    let decoder = zstd::Decoder::new(Cursor::new(body)).map_err(ApiError::bad_request)?;
    let mut archive = tar::Archive::new(decoder);
    let mut files = HashMap::new();
    let mut total_size = 0_usize;
    for entry in archive.entries().map_err(ApiError::bad_request)? {
        let mut entry = entry.map_err(ApiError::bad_request)?;
        if entry.header().entry_type().is_dir() {
            continue;
        }
        if !entry.header().entry_type().is_file() {
            return Err(ApiError::bad_request(
                "Deployment contains a link or unsupported filesystem entry",
            ));
        }
        if files.len() >= MAX_FILES {
            return Err(ApiError::bad_request("Deployment contains too many files"));
        }
        let path = entry
            .path()
            .map_err(ApiError::bad_request)?
            .to_string_lossy()
            .trim_start_matches("./")
            .to_owned();
        if path.contains('\\') {
            return Err(ApiError::bad_request(
                "Deployment path contains a backslash",
            ));
        }
        validate_relative_path(&path).map_err(ApiError::bad_request)?;
        let remaining_total = MAX_EXPANDED_BYTES.saturating_sub(total_size);
        let read_limit = remaining_total
            .min(DEPLOYMENT_MAX_FILE_BYTES as usize)
            .saturating_add(1);
        let mut bytes = Vec::new();
        entry
            .by_ref()
            .take(read_limit as u64)
            .read_to_end(&mut bytes)
            .map_err(ApiError::bad_request)?;
        if bytes.len() as u64 > DEPLOYMENT_MAX_FILE_BYTES {
            return Err(ApiError::bad_request(format!(
                "Deployment object `{path}` exceeds 20 MiB"
            )));
        }
        total_size += bytes.len();
        if total_size > MAX_EXPANDED_BYTES {
            return Err(ApiError::bad_request("Expanded deployment exceeds 100 MiB"));
        }
        let content_type = content_type_for(&path);
        if files
            .insert(
                path.clone(),
                DeploymentObject {
                    bytes: Bytes::from(bytes),
                    content_type,
                },
            )
            .is_some()
        {
            return Err(ApiError::bad_request(format!(
                "Duplicate deployment path `{path}`"
            )));
        }
    }
    if !files.contains_key("index.html") {
        return Err(ApiError::bad_request(
            "Deployment must contain index.html at its root",
        ));
    }
    let mut manifest_files = files
        .iter()
        .map(|(path, file)| DeploymentFile {
            path: path.clone(),
            size: file.bytes.len() as u64,
            content_type: file.content_type.clone(),
        })
        .collect::<Vec<_>>();
    manifest_files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(ValidatedDeployment {
        manifest: DeploymentManifest {
            spa,
            files: manifest_files,
        },
        files,
    })
}

fn content_type_for(path: &str) -> String {
    match FilePath::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("html") => "text/html; charset=utf-8".to_owned(),
        Some("css") => "text/css; charset=utf-8".to_owned(),
        Some("js" | "mjs") => "text/javascript; charset=utf-8".to_owned(),
        Some("json" | "map") => "application/json".to_owned(),
        Some("svg") => "image/svg+xml".to_owned(),
        Some("wasm") => "application/wasm".to_owned(),
        _ => mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string(),
    }
}

struct PublicDeployment {
    id: Uuid,
    object_prefix: String,
    manifest: DeploymentManifest,
}

pub async fn serve_public(
    state: AppState,
    public_label: String,
    method: Method,
    request_uri: axum::http::Uri,
) -> Response {
    if !matches!(method, Method::GET | Method::HEAD) {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let result = async {
        let deployment = load_public(&state, &public_label).await?;
        let request_path = request_uri.path().trim_start_matches('/');
        validate_relative_request_path(request_path)?;
        let file = resolve_file(&deployment.manifest, request_path)
            .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "not_found", "File not found"))?;
        let object = state
            .storage
            .get(&format!("{}/{}", deployment.object_prefix, file.path))
            .await
            .map_err(ApiError::internal)?;
        if file.content_type.starts_with("text/html") {
            let _ = sqlx::query(
                "UPDATE deployments SET last_read_at = now()
                 WHERE id = $1
                   AND (last_read_at IS NULL OR last_read_at < now() - interval '1 hour')",
            )
            .bind(deployment.id)
            .execute(&state.database)
            .await;
        }
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str(&file.content_type).map_err(ApiError::internal)?,
        );
        headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&file.size.to_string()).map_err(ApiError::internal)?,
        );
        headers.insert(
            header::CACHE_CONTROL,
            if file.content_type.starts_with("text/html") {
                HeaderValue::from_static("public, no-cache")
            } else {
                HeaderValue::from_static("public, max-age=3600")
            },
        );
        headers.insert(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        );
        headers.insert(
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        );
        let body = if method == Method::HEAD {
            Body::empty()
        } else {
            Body::from(object.bytes)
        };
        Ok::<_, ApiError>((headers, body).into_response())
    }
    .await;
    result.unwrap_or_else(IntoResponse::into_response)
}

fn validate_relative_request_path(path: &str) -> Result<(), ApiError> {
    if path.is_empty() {
        return Ok(());
    }
    if path.contains('\\') {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "not_found",
            "File not found",
        ));
    }
    validate_relative_path(path)
        .map_err(|_| ApiError::new(StatusCode::NOT_FOUND, "not_found", "File not found"))
}

async fn load_public(state: &AppState, public_label: &str) -> Result<PublicDeployment, ApiError> {
    let row = sqlx::query(
        "SELECT deployments.id, deployment_bundles.object_prefix,
                deployment_bundles.manifest
         FROM deployments
         JOIN deployment_bundles ON deployment_bundles.id = deployments.active_bundle_id
         WHERE deployments.public_label = $1
           AND deployments.status = 'active'",
    )
    .bind(public_label)
    .fetch_optional(&state.database)
    .await?
    .ok_or_else(|| ApiError::new(StatusCode::NOT_FOUND, "not_found", "Deployment not found"))?;
    Ok(PublicDeployment {
        id: row.try_get("id")?,
        object_prefix: row.try_get("object_prefix")?,
        manifest: serde_json::from_value(row.try_get("manifest")?).map_err(ApiError::internal)?,
    })
}

fn resolve_file<'a>(
    manifest: &'a DeploymentManifest,
    request_path: &str,
) -> Option<&'a DeploymentFile> {
    let exact = if request_path.is_empty() {
        "index.html".to_owned()
    } else {
        request_path.to_owned()
    };
    find_file(manifest, &exact)
        .or_else(|| {
            let directory_index = format!("{}/index.html", exact.trim_end_matches('/'));
            find_file(manifest, &directory_index)
        })
        .or_else(|| {
            (manifest.spa && FilePath::new(request_path).extension().is_none())
                .then(|| find_file(manifest, "index.html"))
                .flatten()
        })
}

fn find_file<'a>(manifest: &'a DeploymentManifest, path: &str) -> Option<&'a DeploymentFile> {
    manifest.files.iter().find(|file| file.path == path)
}

fn summary(state: &AppState, row: &sqlx::postgres::PgRow) -> Result<DeploymentSummary, ApiError> {
    let published_at: DateTime<Utc> = row.try_get("published_at")?;
    let last_read_at: Option<DateTime<Utc>> = row.try_get("last_read_at")?;
    let pinned_at: Option<DateTime<Utc>> = row.try_get("pinned_at")?;
    let handle: String = row.try_get("handle")?;
    let slug: String = row.try_get("slug")?;
    let public_label: String = row.try_get("public_label")?;
    Ok(DeploymentSummary {
        id: row.try_get("id")?,
        owner_handle: handle.clone(),
        slug: slug.clone(),
        url: format!("{}/", state.config.public_url(&public_label)),
        spa: row.try_get("spa")?,
        published_at,
        expires_at: pinned_at
            .is_none()
            .then(|| last_read_at.unwrap_or(published_at) + Duration::days(15)),
        pinned: pinned_at.is_some(),
    })
}

pub async fn claim_expired(
    transaction: &mut Transaction<'_, Postgres>,
    limit: i64,
) -> Result<Vec<(Uuid, Uuid)>, ApiError> {
    let rows = sqlx::query(
        "WITH candidates AS (
            SELECT id FROM deployments
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
         UPDATE deployments
         SET status = 'deleting', deletion_attempted_at = now()
         FROM candidates
         WHERE deployments.id = candidates.id
         RETURNING deployments.user_id, deployments.id",
    )
    .bind(limit)
    .fetch_all(&mut **transaction)
    .await?;
    rows.into_iter()
        .map(|row| Ok((row.try_get("user_id")?, row.try_get("id")?)))
        .collect()
}

pub async fn delete_objects_and_row(
    state: &AppState,
    user_id: Uuid,
    deployment_id: Uuid,
) -> Result<(), ApiError> {
    let prefixes = sqlx::query_scalar::<_, String>(
        "SELECT object_prefix FROM deployment_bundles WHERE deployment_id = $1",
    )
    .bind(deployment_id)
    .fetch_all(&state.database)
    .await?;
    for prefix in prefixes {
        state
            .storage
            .delete_prefix(&prefix)
            .await
            .map_err(ApiError::internal)?;
    }
    sqlx::query("DELETE FROM deployments WHERE id = $1 AND user_id = $2")
        .bind(deployment_id)
        .bind(user_id)
        .execute(&state.database)
        .await?;
    Ok(())
}

pub async fn cleanup_superseded(state: &AppState) -> Result<(), ApiError> {
    let rows = sqlx::query(
        "SELECT id, object_prefix FROM deployment_bundles
         WHERE status = 'superseded'
         ORDER BY created_at
         LIMIT 100",
    )
    .fetch_all(&state.database)
    .await?;
    for row in rows {
        let id: Uuid = row.try_get("id")?;
        let prefix: String = row.try_get("object_prefix")?;
        state
            .storage
            .delete_prefix(&prefix)
            .await
            .map_err(ApiError::internal)?;
        sqlx::query("DELETE FROM deployment_bundles WHERE id = $1 AND status = 'superseded'")
            .bind(id)
            .execute(&state.database)
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn archive(entries: &[(&str, usize)]) -> Bytes {
        let encoder = zstd::Encoder::new(Vec::new(), 1).unwrap();
        let mut archive = tar::Builder::new(encoder);
        for (path, size) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(*size as u64);
            header.set_mode(0o644);
            header.set_cksum();
            archive
                .append_data(&mut header, path, std::io::repeat(0).take(*size as u64))
                .unwrap();
        }
        let encoder = archive.into_inner().unwrap();
        Bytes::from(encoder.finish().unwrap())
    }

    fn manifest(spa: bool, paths: &[&str]) -> DeploymentManifest {
        DeploymentManifest {
            spa,
            files: paths
                .iter()
                .map(|path| DeploymentFile {
                    path: (*path).to_owned(),
                    size: 1,
                    content_type: content_type_for(path),
                })
                .collect(),
        }
    }

    #[test]
    fn resolves_exact_files_and_directory_indexes() {
        let manifest = manifest(false, &["index.html", "assets/app.js", "docs/index.html"]);
        assert_eq!(resolve_file(&manifest, "").unwrap().path, "index.html");
        assert_eq!(
            resolve_file(&manifest, "assets/app.js").unwrap().path,
            "assets/app.js"
        );
        assert_eq!(
            resolve_file(&manifest, "docs").unwrap().path,
            "docs/index.html"
        );
    }

    #[test]
    fn spa_falls_back_only_for_extensionless_paths() {
        let manifest = manifest(true, &["index.html", "assets/app.js"]);
        assert_eq!(
            resolve_file(&manifest, "settings/profile").unwrap().path,
            "index.html"
        );
        assert!(resolve_file(&manifest, "assets/missing.js").is_none());
    }

    #[test]
    fn assigns_browser_asset_content_types() {
        assert_eq!(content_type_for("index.html"), "text/html; charset=utf-8");
        assert_eq!(
            content_type_for("assets/app.js"),
            "text/javascript; charset=utf-8"
        );
        assert_eq!(content_type_for("assets/app.wasm"), "application/wasm");
    }

    #[test]
    fn deployment_requires_a_root_index() {
        let error = validate_archive(archive(&[("page.html", 10)]), false).unwrap_err();
        assert!(error.to_string().contains("index.html"));
    }

    #[test]
    fn deployment_rejects_objects_larger_than_twenty_mibibytes() {
        let error = validate_archive(
            archive(&[
                ("index.html", 10),
                ("large.bin", DEPLOYMENT_MAX_FILE_BYTES as usize + 1),
            ]),
            false,
        )
        .unwrap_err();
        assert!(error.to_string().contains("exceeds 20 MiB"));
    }
}
