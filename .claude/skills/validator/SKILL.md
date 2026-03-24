---
name: validator
description: "CRITICAL: Use for validator derive-based validation in Rust. Triggers on: validator, Validate, #[validate], validation rules, email, url, length, range, custom, nested, schema_validation, must_match, contains, required, ValidationErrors"
version: "0.20"
---

# validator Crate Skill (v0.20)

## Quick Reference

```rust
use validator::{Validate, ValidationError, ValidationErrors};

#[derive(Validate)]
struct SignupData {
    #[validate(email)]
    email: String,
    #[validate(length(min = 8, max = 128))]
    password: String,
}

// Usage
let data = SignupData { ... };
data.validate()?; // Returns Result<(), ValidationErrors>
```

## Field Validation Rules

### String validators
```rust
#[validate(email)]                              // RFC 5321
#[validate(url)]                                // Valid URL
#[validate(length(min = 1))]                    // min, max, equal
#[validate(contains(pattern = "hello"))]
#[validate(does_not_contain(pattern = "bad"))]
#[validate(regex(path = *RE))]                  // lazy_static or OnceLock regex
```

### Numeric validators
```rust
#[validate(range(min = 0, max = 100))]          // inclusive
#[validate(range(min = 0.0, exclusive_max = 1.0))] // exclusive max
#[validate(range(exclusive_min = 0))]           // exclusive min
```

### Comparison validators
```rust
#[validate(must_match(other = "password_confirm"))]  // field equality
```

### Required (Option fields)
```rust
#[validate(required)]    // Option<T> must be Some
```

### Nested validation
```rust
#[validate(nested)]      // Validate inner struct (must also derive Validate)
nested: InnerStruct,
```

## Custom Validation

### Field-level custom
```rust
#[validate(custom(function = "validate_username"))]
username: String,

fn validate_username(username: &str) -> Result<(), ValidationError> {
    if username.contains("admin") {
        let mut err = ValidationError::new("forbidden_username");
        err.message = Some("Username cannot contain 'admin'".into());
        return Err(err);
    }
    Ok(())
}
```

### Schema-level validation (cross-field)
```rust
#[derive(Validate)]
#[validate(schema(function = "validate_date_range"))]
struct DateRange {
    start: NaiveDate,
    end: NaiveDate,
}

fn validate_date_range(range: &DateRange) -> Result<(), ValidationError> {
    if range.start >= range.end {
        return Err(ValidationError::new("invalid_date_range"));
    }
    Ok(())
}
```

## Error Handling

```rust
match data.validate() {
    Ok(()) => { /* valid */ }
    Err(errors) => {
        // errors: ValidationErrors
        // errors.field_errors()  -> HashMap<&str, Vec<ValidationError>>
        // errors.into_errors()   -> HashMap<&str, ValidationErrorsKind>

        // Access specific field errors
        if let Some(email_errors) = errors.field_errors().get("email") {
            for e in email_errors {
                println!("code: {}, message: {:?}", e.code, e.message);
            }
        }
    }
}
```

### ValidationError fields
```rust
ValidationError {
    code: Cow<'static, str>,      // e.g., "email", "length", custom codes
    message: Option<Cow<'static, str>>,
    params: HashMap<Cow<'static, str>, Value>,  // serde_json::Value
}
```

## Integration with axum

```rust
use axum::{extract::Json, response::IntoResponse, http::StatusCode};
use validator::Validate;

// Option A: Validate in handler
async fn create_user(Json(payload): Json<CreateUserRequest>) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    payload.validate().map_err(|e| {
        (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({ "errors": e.to_string() })))
    })?;
    // ...
}

// Option B: Custom extractor (ValidatedJson)
struct ValidatedJson<T>(pub T);

#[axum::async_trait]
impl<S, T> axum::extract::FromRequest<S> for ValidatedJson<T>
where
    T: serde::de::DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    async fn from_request(req: axum::extract::Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": e.to_string()}))))?;
        value.validate().map_err(|e| {
            (StatusCode::UNPROCESSABLE_ENTITY, Json(serde_json::json!({"errors": e.to_string()})))
        })?;
        Ok(ValidatedJson(value))
    }
}
```

## Multiple Rules on One Field

```rust
#[validate(length(min = 3, max = 50), custom(function = "validate_no_spaces"))]
username: String,
```

## Common Mistakes

1. **Forgetting `#[validate(nested)]`** on struct fields — inner struct won't be validated
2. **Using `required` on non-Option fields** — only meaningful for `Option<T>`
3. **Regex path must be a static reference** — use `std::sync::LazyLock` or `once_cell::sync::Lazy`
4. **`custom` function signature** — must return `Result<(), ValidationError>`, not `Result<(), ValidationErrors>`
5. **Schema validation vs field validation** — schema-level sees the whole struct, field-level sees one field
