use anyhow::Context;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};

#[derive(Clone)]
pub struct Db {
    pub pool: Pool<Postgres>,
}

impl Db {
    pub async fn connect(database_url: &str) -> anyhow::Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .context("failed to create postgres pool")?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::migrate!("./db/migrations")
            .run(&self.pool)
            .await
            .context("migration run failed")?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleInput {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionInput {
    pub action: String,
    pub resource: String,
}

pub async fn create_role(pool: &Pool<Postgres>, name: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO roles (name) VALUES ($1) ON CONFLICT (name) DO NOTHING")
        .bind(name)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn create_permission(
    pool: &Pool<Postgres>,
    action: &str,
    resource: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO permissions (action, resource) VALUES ($1, $2) ON CONFLICT (action, resource) DO NOTHING",
    )
    .bind(action)
    .bind(resource)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn assign_permission_to_role(
    pool: &Pool<Postgres>,
    role_name: &str,
    action: &str,
    resource: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO role_permissions (role_id, permission_id)
        SELECT r.id, p.id
        FROM roles r
        JOIN permissions p ON p.action = $2 AND p.resource = $3
        WHERE r.name = $1
        ON CONFLICT (role_id, permission_id) DO NOTHING
        "#,
    )
    .bind(role_name)
    .bind(action)
    .bind(resource)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn assign_role_to_actor(
    pool: &Pool<Postgres>,
    actor_id: &str,
    role_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO actor_roles (actor_id, role_id)
        SELECT $1, r.id
        FROM roles r
        WHERE r.name = $2
        ON CONFLICT (actor_id, role_id) DO NOTHING
        "#,
    )
    .bind(actor_id)
    .bind(role_name)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn check_access(
    pool: &Pool<Postgres>,
    actor_id: &str,
    action: &str,
    resource: &str,
) -> Result<bool, sqlx::Error> {
    let allowed = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM actor_roles ar
            JOIN role_permissions rp ON rp.role_id = ar.role_id
            JOIN permissions p ON p.id = rp.permission_id
            WHERE ar.actor_id = $1
              AND p.action = $2
              AND p.resource = $3
        )
        "#,
    )
    .bind(actor_id)
    .bind(action)
    .bind(resource)
    .fetch_one(pool)
    .await?;

    Ok(allowed)
}
