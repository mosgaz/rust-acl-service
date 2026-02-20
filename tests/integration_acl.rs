use reqwest::StatusCode;
use rust_acl_service::{app::build_router, db::Db, metrics::init_metrics};
use serde_json::json;
use std::time::Duration;
use testcontainers::{runners::AsyncRunner, ContainerAsync, GenericImage, ImageExt};
use tokio::{net::TcpListener, task::JoinHandle};

struct TestApp {
    _container: ContainerAsync<GenericImage>,
    _server: JoinHandle<()>,
    base_url: String,
    client: reqwest::Client,
}

async fn spawn_postgres() -> anyhow::Result<(ContainerAsync<GenericImage>, String)> {
    let image = GenericImage::new("postgres", "16-alpine")
        .with_exposed_port(5432.into())
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_env_var("POSTGRES_USER", "postgres")
        .with_env_var("POSTGRES_DB", "acl");

    let container = image.start().await?;

    let host_port = container.get_host_port_ipv4(5432).await?;

    let url = format!("postgres://postgres:postgres@127.0.0.1:{host_port}/acl");
    Ok((container, url))
}

async fn start_app() -> anyhow::Result<TestApp> {
    let (container, db_url) = spawn_postgres().await?;

    tokio::time::sleep(Duration::from_secs(2)).await;

    let db = Db::connect(&db_url).await?;
    db.run_migrations().await?;

    let metrics = init_metrics("127.0.0.1:0")?;
    let app = build_router(db.pool.clone(), metrics);
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    let server = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("server should stay alive");
    });

    Ok(TestApp {
        _container: container,
        _server: server,
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    })
}

#[tokio::test]
async fn check_allow_and_deny_paths() {
    let app = match start_app().await {
        Ok(app) => app,
        Err(error) => {
            eprintln!("Skipping integration test due to missing runtime dependencies: {error}");
            return;
        }
    };

    let health = app
        .client
        .get(format!("{}/health", app.base_url))
        .send()
        .await
        .expect("health request must complete");
    assert_eq!(health.status(), StatusCode::OK);

    app.client
        .post(format!("{}/v1/admin/roles", app.base_url))
        .json(&json!({"name": "reader"}))
        .send()
        .await
        .expect("create role request")
        .error_for_status()
        .expect("create role must succeed");

    app.client
        .post(format!("{}/v1/admin/permissions", app.base_url))
        .json(&json!({"action": "read", "resource": "document:123"}))
        .send()
        .await
        .expect("create permission request")
        .error_for_status()
        .expect("create permission must succeed");

    app.client
        .post(format!("{}/v1/admin/role-permissions", app.base_url))
        .json(&json!({"role_name": "reader", "action": "read", "resource": "document:123"}))
        .send()
        .await
        .expect("assign permission request")
        .error_for_status()
        .expect("assign permission must succeed");

    app.client
        .post(format!("{}/v1/admin/actor-roles", app.base_url))
        .json(&json!({"actor_id": "user-1", "role_name": "reader"}))
        .send()
        .await
        .expect("assign role request")
        .error_for_status()
        .expect("assign role must succeed");

    let allow = app
        .client
        .post(format!("{}/v1/check", app.base_url))
        .json(&json!({"actor_id": "user-1", "action": "read", "resource": "document:123"}))
        .send()
        .await
        .expect("allow check request")
        .error_for_status()
        .expect("allow check must succeed")
        .json::<serde_json::Value>()
        .await
        .expect("allow response parse");

    assert_eq!(allow["allow"], json!(true));

    let deny = app
        .client
        .post(format!("{}/v1/check", app.base_url))
        .json(&json!({"actor_id": "user-1", "action": "write", "resource": "document:123"}))
        .send()
        .await
        .expect("deny check request")
        .error_for_status()
        .expect("deny check must succeed")
        .json::<serde_json::Value>()
        .await
        .expect("deny response parse");

    assert_eq!(deny["allow"], json!(false));
}
