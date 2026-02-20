use axum::{extract::State, Json};
use metrics::{counter, histogram};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::time::Instant;

use crate::{
    db::{
        assign_permission_to_role, assign_role_to_actor, check_access, create_permission,
        create_role, PermissionInput, RoleInput,
    },
    error::AppError,
};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool<Postgres>,
}

#[derive(Debug, Deserialize)]
pub struct CheckRequest {
    pub actor_id: String,
    pub action: String,
    pub resource: String,
}

#[derive(Debug, Serialize)]
pub struct CheckResponse {
    pub allow: bool,
}

#[derive(Debug, Deserialize)]
pub struct AssignPermissionRequest {
    pub role_name: String,
    pub action: String,
    pub resource: String,
}

#[derive(Debug, Deserialize)]
pub struct AssignActorRoleRequest {
    pub actor_id: String,
    pub role_name: String,
}

pub async fn health() -> &'static str {
    "ok"
}

pub async fn ready(State(state): State<AppState>) -> Result<&'static str, AppError> {
    sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&state.db_pool)
        .await?;
    Ok("ready")
}

pub async fn check(
    State(state): State<AppState>,
    Json(payload): Json<CheckRequest>,
) -> Result<Json<CheckResponse>, AppError> {
    let started = Instant::now();
    let allow = check_access(
        &state.db_pool,
        &payload.actor_id,
        &payload.action,
        &payload.resource,
    )
    .await
    .unwrap_or(false);

    counter!("request_count", "endpoint" => "/v1/check").increment(1);
    histogram!("request_latency_seconds", "endpoint" => "/v1/check")
        .record(started.elapsed().as_secs_f64());

    Ok(Json(CheckResponse { allow }))
}

pub async fn create_role_handler(
    State(state): State<AppState>,
    Json(payload): Json<RoleInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    create_role(&state.db_pool, &payload.name).await?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

pub async fn create_permission_handler(
    State(state): State<AppState>,
    Json(payload): Json<PermissionInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    create_permission(&state.db_pool, &payload.action, &payload.resource).await?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

pub async fn assign_permission_handler(
    State(state): State<AppState>,
    Json(payload): Json<AssignPermissionRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    assign_permission_to_role(
        &state.db_pool,
        &payload.role_name,
        &payload.action,
        &payload.resource,
    )
    .await?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}

pub async fn assign_actor_role_handler(
    State(state): State<AppState>,
    Json(payload): Json<AssignActorRoleRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    assign_role_to_actor(&state.db_pool, &payload.actor_id, &payload.role_name).await?;
    Ok(Json(serde_json::json!({"status": "ok"})))
}
