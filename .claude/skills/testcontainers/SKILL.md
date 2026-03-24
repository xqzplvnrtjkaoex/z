---
name: testcontainers
description: "CRITICAL: Use for testcontainers integration testing with real containers. Triggers on: testcontainers, testcontainers-modules, ContainerAsync, ImageExt, GenericImage, runners, Postgres container, integration test, real database, DOCKER_HOST, container lifecycle"
version: "0.27"
---

# testcontainers Crate Skill (v0.27)

## Quick Reference

```rust
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::postgres::Postgres;

#[tokio::test]
async fn test_with_postgres() {
    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    // Use url to connect sea-orm, sqlx, etc.
}
```

## Core Types

```rust
// Key imports
use testcontainers::{
    ContainerAsync,          // Running container handle
    GenericImage,            // Build custom images
    ImageExt,                // Extension trait for configuration (.with_env_var, etc.)
    runners::AsyncRunner,    // .start() on images
};
```

## testcontainers-modules (v0.15)

Pre-built images. Enable via feature flags in Cargo.toml:

```toml
[dev-dependencies]
testcontainers-modules = { version = "0.15", features = ["postgres"] }
```

### Postgres
```rust
use testcontainers_modules::postgres::Postgres;

// Default: postgres:postgres@localhost/postgres
let container = Postgres::default().start().await.unwrap();

// Custom config
let pg = Postgres::default()
    .with_db_name("testdb")
    .with_user("myuser")
    .with_password("mypass");
let container = pg.start().await.unwrap();
```

### Other modules
- `redis` → `Redis::default()`
- `minio` → `MinIO::default()`
- `kafka` → `Kafka::default()`
- Enable each via its feature flag

## Container Configuration (ImageExt)

```rust
use testcontainers::ImageExt;

let container = Postgres::default()
    .with_env_var("POSTGRES_INITDB_ARGS", "--encoding=UTF8")
    .with_mapped_port(5432, 5432.into())   // host_port, container_port
    .with_network("my-network")
    .with_container_name("test-pg")
    .start()
    .await
    .unwrap();
```

## Getting Connection Info

```rust
// Get mapped host port (containers use random ports by default)
let port = container.get_host_port_ipv4(5432).await.unwrap();

// Get host address
let host = container.get_host().await.unwrap();

// Build connection string
let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
```

## Integration with sea-orm

```rust
use sea_orm::{Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

async fn setup_db() -> (ContainerAsync<Postgres>, DatabaseConnection) {
    let container = Postgres::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

    let db = Database::connect(&url).await.unwrap();

    // Run migrations
    Migrator::up(&db, None).await.unwrap();

    (container, db)  // IMPORTANT: return container to keep it alive
}

#[tokio::test]
async fn test_user_repo() {
    let (_container, db) = setup_db().await;
    // _container must live for the duration of the test
    // ...
}
```

## GenericImage (Custom Images)

```rust
use testcontainers::GenericImage;

let image = GenericImage::new("my-registry/my-image", "latest")
    .with_exposed_port(8080.into())
    .with_wait_for(testcontainers::core::WaitFor::message_on_stdout("Ready"));

let container = image.start().await.unwrap();
```

## DOCKER_HOST (Remote Docker)

When using remote Docker (e.g., colima, Docker on another machine):

```bash
# .env
DOCKER_HOST=unix:///Users/syr/.colima/default/docker.sock
```

testcontainers reads `DOCKER_HOST` automatically. No code changes needed.

## Common Patterns

### Shared container across tests (test module)
```rust
use std::sync::OnceLock;
use tokio::sync::OnceCell;

static DB: OnceCell<(ContainerAsync<Postgres>, DatabaseConnection)> = OnceCell::const_new();

async fn get_db() -> &'static (ContainerAsync<Postgres>, DatabaseConnection) {
    DB.get_or_init(|| async { setup_db().await }).await
}
```

### Exec commands in container
```rust
use testcontainers::core::ExecCommand;

container
    .exec(ExecCommand::new(vec!["psql", "-U", "postgres", "-c", "CREATE DATABASE mydb;"]))
    .await
    .unwrap();
```

## Common Mistakes

1. **Container dropped too early** — container stops when `ContainerAsync` drops. Always hold the handle (use `_container` prefix to suppress warnings).
2. **Using fixed ports** — default is random port mapping. Use `get_host_port_ipv4()` to discover the mapped port. Fixed ports cause test parallelism issues.
3. **Missing tokio runtime** — `start().await` requires tokio. Use `#[tokio::test]`.
4. **Docker not running** — tests panic if Docker daemon is unreachable. Check `DOCKER_HOST` env var.
5. **Feature flags** — each module needs its feature enabled: `features = ["postgres"]`.
6. **Port type** — `get_host_port_ipv4(5432)` takes `u16`, but `with_mapped_port` takes `ContainerPort`.
