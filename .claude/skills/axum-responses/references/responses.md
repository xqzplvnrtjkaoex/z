# Axum Responses Reference

> axum 0.8 | Source: https://docs.rs/axum/0.8

## IntoResponse Trait

Any handler return type must implement `IntoResponse`.

### Built-in Implementations

```rust
// Status code only (empty body)
async fn handler() -> StatusCode {
    StatusCode::NO_CONTENT
}

// String (text/plain)
async fn handler() -> String {
    "hello".to_string()
}

// Json
async fn handler() -> Json<User> {
    Json(User { id: 1, name: "Alice".into() })
}

// Tuple: (StatusCode, body)
async fn handler() -> (StatusCode, String) {
    (StatusCode::CREATED, "created".into())
}

// Tuple: (StatusCode, headers, body)
async fn handler() -> (StatusCode, [(HeaderName, HeaderValue); 1], String) {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"))],
        "hello".into(),
    )
}

// Full Response
async fn handler() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("x-custom", "value")
        .body(Body::from("hello"))
        .unwrap()
}
```

## Custom IntoResponse

```rust
use axum::response::{IntoResponse, Response};
use axum::http::StatusCode;

struct AppError {
    status: StatusCode,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = serde_json::json!({
            "kind": "ERROR",
            "message": self.message,
        });
        (self.status, Json(body)).into_response()
    }
}
```

## Result<T, E> Pattern

When both `T` and `E` implement `IntoResponse`:

```rust
async fn handler() -> Result<Json<User>, AppError> {
    let user = find_user().await.map_err(|e| AppError {
        status: StatusCode::NOT_FOUND,
        message: e.to_string(),
    })?;
    Ok(Json(user))
}
```

## Redirect

```rust
use axum::response::Redirect;

async fn handler() -> Redirect {
    Redirect::to("/new-location")
}

// Variants
Redirect::to(uri)           // 303 See Other
Redirect::temporary(uri)    // 307 Temporary
Redirect::permanent(uri)    // 308 Permanent
```

## HTML

```rust
use axum::response::Html;

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello</h1>")
}
```

## Headers in Response

```rust
use axum::http::{HeaderMap, HeaderValue, header};

async fn handler() -> (HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, "session=abc".parse().unwrap());
    (headers, "ok".into())
}
```

## Cookies (axum-extra)

```rust
use axum_extra::extract::CookieJar;
use cookie::Cookie;

async fn login(jar: CookieJar) -> (CookieJar, StatusCode) {
    let jar = jar.add(
        Cookie::build(("session", "value"))
            .path("/")
            .http_only(true)
            .secure(true)
            .same_site(cookie::SameSite::Lax)
            .max_age(cookie::time::Duration::days(7))
    );
    (jar, StatusCode::OK)
}

async fn logout(jar: CookieJar) -> (CookieJar, StatusCode) {
    let jar = jar.remove(Cookie::from("session"));
    (jar, StatusCode::OK)
}
```
