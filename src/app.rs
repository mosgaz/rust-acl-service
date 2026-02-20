use axum::{routing::get, routing::post, Router};
use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::{Pool, Postgres};

use crate::handlers::{
    assign_actor_role_handler, assign_permission_handler, check, create_permission_handler,
    create_role_handler, health, ready, AppState,
};

pub fn build_router(pool: Pool<Postgres>, metrics: PrometheusHandle) -> Router {
    let state = AppState { db_pool: pool };

    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/v1/check", post(check))
        .route("/v1/admin/roles", post(create_role_handler))
        .route("/v1/admin/permissions", post(create_permission_handler))
        .route(
            "/v1/admin/role-permissions",
            post(assign_permission_handler),
        )
        .route("/v1/admin/actor-roles", post(assign_actor_role_handler))
        .route(
            "/metrics",
            get(move || {
                let rendered = metrics.render();
                async move { rendered }
            }),
        )
        .with_state(state)
}
