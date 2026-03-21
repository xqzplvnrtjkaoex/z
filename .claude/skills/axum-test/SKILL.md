---
name: axum-test
description: "CRITICAL: Use for integration testing of axum web applications. Triggers on:\naxum-test, TestServer, TestRequest, TestResponse, axum integration test, axum test server, assert_status, assert_json, assert_text, expect_json, test axum handler, axum e2e test, axum route test, TestServerConfig"
---

> **Version:** axum-test 17.3 | **Last Updated:** 2026-03-03

# axum-test — Quick Reference

Source: https://docs.rs/axum-test

## Key Patterns

### Minimal Test Setup

```rust
use axum::{Router, routing::get};
use axum_test::TestServer;

#[tokio::test]
async fn it_should_ping() {
    let app = Router::new()
        .route("/ping", get(|| async { "pong!" }));

    let server = TestServer::new(app).unwrap();

    let response = server.get("/ping").await;

    response.assert_status_ok();
    response.assert_text("pong!");
}
```

### JSON Request and Response

```rust
use axum::{Router, routing::post, Json};
use serde::{Deserialize, Serialize};
use axum_test::TestServer;
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct CreateUser { name: String }

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct User { id: u64, name: String }

#[tokio::test]
async fn test_create_user() {
    let app = Router::new()
        .route("/users", post(create_user_handler));

    let server = TestServer::new(app).unwrap();

    let response = server
        .post("/users")
        .json(&CreateUser { name: "Alice".into() })
        .await;

    response.assert_status(StatusCode::CREATED);

    // Deserialize and assert:
    let user = response.json::<User>();
    assert_eq!(user.name, "Alice");

    // Or assert against a JSON literal:
    response.assert_json(&json!({ "name": "Alice" }));
}
```

### Shape-Based JSON Assertions with expect_json

```rust
use axum_test::TestServer;
use axum_test::expect_json;
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn test_user_shape() {
    let server = TestServer::new(app).unwrap();

    server.get("/users/1")
        .await
        .assert_json(&json!({
            "id":         expect_json::uuid(),
            "name":       "Alice",
            "age":        expect_json::integer().in_range(18..=120),
            "created_at": expect_json::iso_date_time()
                            .within_past(Duration::from_secs(60))
                            .utc(),
        }));
}
```

### Query Parameters

```rust
use serde::Serialize;

#[derive(Serialize)]
struct Filters { page: u32, per_page: u32 }

let response = server
    .get("/items")
    .add_query_params(Filters { page: 1, per_page: 25 })
    .await;

// Or one at a time:
let response = server
    .get("/items")
    .add_query_param("page", "1")
    .add_query_param("per_page", "25")
    .await;
```

### Headers and Authorization

```rust
use axum::http::HeaderName;

// Bearer token:
let response = server
    .get("/protected")
    .authorization_bearer("my-jwt-token")
    .await;

// Custom header:
let response = server
    .post("/resource")
    .add_header(HeaderName::from_static("x-api-key"), "secret".parse().unwrap())
    .json(&body)
    .await;
```

### Cookie Handling

```rust
use axum_test::TestServer;
use axum_extra::extract::cookie::Cookie;

#[tokio::test]
async fn test_session_flow() {
    // Save cookies from login response for subsequent requests:
    let server = TestServer::new(app).unwrap();

    // Option A: save_cookies() on the request — persists to TestServer
    let login_resp = server
        .post("/auth/login")
        .json(&credentials)
        .save_cookies()   // cookies from this response are stored in server
        .await;

    // Subsequent requests automatically send saved cookies:
    let profile = server.get("/profile").await;
    profile.assert_status_ok();

    // Option B: global cookie persistence for all requests
    let server = TestServer::builder()
        .save_cookies()
        .build(app)
        .unwrap();

    // Option C: manually extract and add a specific cookie
    let cookie = login_resp.cookie("session");
    server.get("/profile")
        .add_cookie(cookie)
        .await
        .assert_status_ok();
}
```

### Form Data

```rust
use serde::Serialize;

#[derive(Serialize)]
struct LoginForm { username: String, password: String }

let response = server
    .post("/login")
    .form(&LoginForm { username: "alice".into(), password: "secret".into() })
    .await;
```

### Multipart Upload

```rust
use axum_test::multipart::MultipartForm;
use axum_test::multipart::Part;

let form = MultipartForm::new()
    .add_text("title", "My Document")
    .add_part("file", Part::bytes(file_bytes).file_name("doc.pdf").mime_type("application/pdf"));

let response = server
    .post("/upload")
    .multipart(form)
    .await;
```

### Expect Success/Failure (panic on wrong status)

```rust
// Per-request:
server.get("/must-succeed").expect_success().await;
server.post("/must-fail").json(&bad).expect_failure().await;

// Global — all requests on this server must return 2xx:
let server = TestServer::builder()
    .expect_success_by_default()
    .build(app)
    .unwrap();
```

### TestServerConfig / Builder

```rust
use axum_test::{TestServer, TestServerConfig, Transport};

// Full config struct:
let config = TestServerConfig {
    transport: Some(Transport::HttpRandomPort),  // real HTTP on random port
    save_cookies: true,         // persist cookies across requests
    expect_success_by_default: true,
    default_content_type: Some("application/json".into()),
    ..TestServerConfig::default()
};
let server = TestServer::new_with_config(app, config).unwrap();

// Or builder API:
let server = TestServer::builder()
    .http_transport()           // real TCP; needed for ConnectInfo extractors
    .save_cookies()
    .expect_success_by_default()
    .default_content_type("application/json")
    .build(app)
    .unwrap();
```

### WebSocket Testing (feature = "ws")

```rust
// Cargo.toml: axum-test = { features = ["ws"] }
use axum_test::TestServer;

#[tokio::test]
async fn test_websocket() {
    let server = TestServer::new(app).unwrap();

    let mut ws = server.get_websocket("/ws").await.into_websocket().await;

    ws.send_text("hello").await;
    let msg = ws.receive_text().await;
    assert_eq!(msg, "hello");

    ws.close().await;
}
```

### App State in Tests

```rust
use axum::{Router, extract::State};
use std::sync::Arc;

struct AppState { db_url: String }

let state = Arc::new(AppState { db_url: "postgres://...".into() });
let app = Router::new()
    .route("/health", get(health_handler))
    .with_state(state);

let server = TestServer::new(app).unwrap();
```

## API Reference

### `TestServer`
| Method | Purpose |
|--------|---------|
| `TestServer::new(app)` | Create with default config |
| `TestServer::new_with_config(app, config)` | Create with custom config |
| `TestServer::builder()` | Get a `TestServerBuilder` |
| `server.get(path)` | Build a GET `TestRequest` |
| `server.post(path)` | Build a POST `TestRequest` |
| `server.put(path)` | Build a PUT `TestRequest` |
| `server.patch(path)` | Build a PATCH `TestRequest` |
| `server.delete(path)` | Build a DELETE `TestRequest` |
| `server.method(method, path)` | Build any HTTP method |
| `server.get_websocket(path)` | Build WebSocket upgrade request |
| `server.add_cookie(cookie)` | Add cookie to all future requests |
| `server.add_header(name, value)` | Add header to all future requests |
| `server.save_cookies()` | Enable global cookie persistence |
| `server.do_not_save_cookies()` | Disable global cookie persistence |
| `server.expect_success()` | Assert 2xx by default globally |
| `server.clear_cookies()` | Remove all stored cookies |
| `server.server_address()` | Get bound address (real transport) |

### `TestRequest` (builder, call `.await` to execute)
| Method | Purpose |
|--------|---------|
| `.json(&T)` | JSON body + `Content-Type: application/json` |
| `.form(&T)` | URL-encoded body |
| `.multipart(form)` | Multipart body |
| `.text(str)` | Plain text body + `text/plain` |
| `.bytes(bytes)` | Raw bytes body |
| `.content_type(str)` | Override content type header |
| `.add_header(name, value)` | Add request header |
| `.authorization_bearer(token)` | `Authorization: Bearer {token}` |
| `.authorization(value)` | `Authorization: {value}` |
| `.add_query_param(key, val)` | Append a query parameter |
| `.add_query_params(&T)` | Append multiple query parameters |
| `.add_raw_query_param(str)` | Append raw (unencoded) query string |
| `.clear_query_params()` | Remove all query params |
| `.add_cookie(cookie)` | Add a cookie to this request |
| `.add_cookies(jar)` | Add cookies from a `CookieJar` |
| `.save_cookies()` | Persist response cookies to server |
| `.do_not_save_cookies()` | Do not persist response cookies |
| `.expect_success()` | Panic if response is not 2xx |
| `.expect_failure()` | Panic if response is 2xx |

### `TestResponse` — Status Assertions
| Method | Purpose |
|--------|---------|
| `.assert_status(code)` | Exact status code match |
| `.assert_status_ok()` | 200 OK |
| `.assert_status_success()` | Any 2xx |
| `.assert_status_not_found()` | 404 |
| `.assert_status_bad_request()` | 400 |
| `.assert_status_unauthorized()` | 401 |
| `.assert_status_forbidden()` | 403 |
| `.assert_status_conflict()` | 409 |
| `.assert_status_internal_server_error()` | 500 |
| `.assert_not_status(code)` | Status must NOT match |
| `.assert_status_in_range(lo..=hi)` | Status in numeric range |

### `TestResponse` — Content Assertions
| Method | Purpose |
|--------|---------|
| `.assert_text(expected)` | Exact text body match |
| `.assert_text_contains(sub)` | Body contains substring |
| `.assert_json(&value)` | JSON matches (exact or with expect_json matchers) |
| `.assert_json_contains(&value)` | Partial JSON match |
| `.assert_form(&T)` | Form-encoded body match |
| `.assert_header(name, value)` | Exact header value match |
| `.assert_contains_header(name)` | Header is present |

### `TestResponse` — Data Extraction
| Method | Purpose |
|--------|---------|
| `.status_code()` | `StatusCode` |
| `.text()` | Body as `String` |
| `.json::<T>()` | Deserialize body as JSON |
| `.bytes()` / `.into_bytes()` | Raw body bytes |
| `.form::<T>()` | Deserialize URL-encoded body |
| `.headers()` | All response headers |
| `.header(name)` | Single header value (panics if absent) |
| `.maybe_header(name)` | `Option<HeaderValue>` |
| `.content_type()` | Content-Type header |
| `.cookie(name)` | Cookie by name (panics if absent) |
| `.maybe_cookie(name)` | `Option<Cookie>` |
| `.cookies()` | All cookies as `CookieJar` |
| `.iter_cookies()` | Iterator over cookies |

### `expect_json` Matchers
| Matcher | Purpose |
|---------|---------|
| `expect_json::uuid()` | Value is a valid UUID string |
| `expect_json::integer()` | Value is a JSON integer |
| `expect_json::integer().in_range(lo..=hi)` | Integer within range |
| `expect_json::float()` | Value is a JSON float |
| `expect_json::string()` | Value is a JSON string |
| `expect_json::iso_date_time()` | Valid ISO 8601 datetime string |
| `expect_json::iso_date_time().within_past(Duration)` | Datetime is recent |
| `expect_json::iso_date_time().utc()` | Datetime is UTC |

### `TestServerConfig` Fields
| Field | Default | Purpose |
|-------|---------|---------|
| `transport` | `None` (mocked) | `Transport::HttpRandomPort` for real TCP |
| `save_cookies` | `false` | Persist response cookies automatically |
| `expect_success_by_default` | `false` | Panic on non-2xx by default |
| `default_content_type` | `None` | Override content type for all requests |
| `default_scheme` | `None` | URI scheme (defaults to `http`) |
| `restrict_requests_with_http_schema` | `false` | Route `http://` URLs through test server |

## Gotchas

1. **Default transport is in-process (mocked).** `TestServer::new()` uses `tower::util::Oneshot` — no actual TCP socket. Handlers that use `ConnectInfo<SocketAddr>` will fail unless you use `http_transport()` / `Transport::HttpRandomPort`.

2. **Cookies are NOT saved by default.** Each request is cookieless unless you call `.save_cookies()` on the request (persists to server) or set `save_cookies: true` in `TestServerConfig`. This trips up stateful login flows.

3. **`.await` on `TestRequest` executes it.** `TestRequest` implements `IntoFuture`. Call `.await` to send; do not call a separate `.send()`.

4. **`assert_json()` requires exact match unless expect_json matchers are used.** If a field is present in the response but not in your expected JSON, the assertion succeeds (partial matching). If a field is in your expected JSON but absent in the response, it fails.

5. **Feature flags.** `ws` for WebSocket, `yaml` for YAML body/assertions, `msgpack` for MessagePack, `typed-routing` for axum-extra TypedPath. None are on by default except `pretty-assertions`.

6. **Version alignment is critical.** axum-test 17.x requires axum 0.8.x. Mismatched versions cause compile errors. Check the crates.io page for the compatibility matrix.

7. **`expect_success()` / `expect_failure()` panic, not return error.** These are test assertions, not Result-returning checks. They are meant to catch bugs in the test harness itself (e.g., accidentally hitting the wrong endpoint).

8. **`multipart` module path.** Import from `axum_test::multipart`, not from `axum`. The types are specific to the testing library.

9. **Headers are case-insensitive in HTTP but some assert methods are case-sensitive in string comparison.** Use the canonical lowercase form when asserting header names.

10. **Parallel test isolation.** With mocked transport (default), tests are fully isolated. With `http_transport()`, each test gets a random port, so parallel execution is safe.

## Tips

- Use `expect_success_by_default()` at the server level to catch accidental non-2xx responses early, then override with `.expect_failure()` on tests that explicitly check error paths.
- Store a shared `TestServer` in a helper function to avoid repeating app setup boilerplate. Since `TestServer` is `Clone`, you can reuse it across tests if state is clean.
- Use `assert_json_contains()` instead of `assert_json()` when testing only a subset of response fields — avoids brittle tests that break when new fields are added.
- Combine `.save_cookies()` on the login request with subsequent requests on the same `server` instance to simulate a logged-in session without manually threading cookies.
- For tests that create a real DB connection, wrap the test setup in a transaction and roll it back in the teardown — this keeps the DB clean without dropping/recreating tables.
- When asserting headers, use `axum::http::header::CONTENT_TYPE` etc. constants rather than raw strings to avoid case-sensitivity issues.
- `response.text()` is useful for debugging failing `assert_json()` calls — print the raw text to see what the handler actually returned.
