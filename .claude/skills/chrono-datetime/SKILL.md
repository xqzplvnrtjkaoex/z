---
name: chrono-datetime
description: |
  CRITICAL: Use for date/time handling in Rust. Triggers on:
  chrono, DateTime, Utc, NaiveDateTime, NaiveDate, Duration,
  timestamp, date parsing, date formatting, timezone,
  date arithmetic, strftime, parse_from_str
---

# Chrono DateTime Skill

> **Version:** chrono 0.4 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/chrono

You are an expert at the Rust `chrono` crate. Help users by:
- **Writing code**: Generate date/time creation, parsing, formatting, arithmetic
- **Answering questions**: Explain Naive vs Aware types, timezone handling, serde integration

## Documentation

- `./references/api.md` — Types, creation, parsing, formatting, arithmetic, serde

## Key Patterns

### Current Time

```rust
use chrono::{Utc, Local};

let now_utc = Utc::now();       // DateTime<Utc>
let now_local = Local::now();   // DateTime<Local>
```

### Type Hierarchy

```rust
// Timezone-aware (preferred for storage/APIs)
DateTime<Utc>       // UTC timestamp
DateTime<Local>     // Local timezone
DateTime<FixedOffset>  // Fixed offset (+09:00)

// Naive (no timezone — use for display-only or DB columns without tz)
NaiveDateTime       // date + time, no tz
NaiveDate           // date only
NaiveTime           // time only
```

### Creation

```rust
use chrono::{NaiveDate, NaiveDateTime, Utc, TimeZone};

// From components
let dt = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
    .and_hms_opt(12, 0, 0).unwrap();  // NaiveDateTime

// NaiveDateTime → DateTime<Utc>
let utc = dt.and_utc();  // DateTime<Utc>

// From timestamp
let dt = DateTime::from_timestamp(1709251200, 0).unwrap(); // DateTime<Utc>
```

### Parsing

```rust
use chrono::{DateTime, Utc, NaiveDateTime};

// RFC 3339 / ISO 8601
let dt: DateTime<Utc> = "2026-03-01T12:00:00Z".parse().unwrap();

// Custom format
let naive = NaiveDateTime::parse_from_str(
    "2026-03-01 12:00:00", "%Y-%m-%d %H:%M:%S"
).unwrap();
```

### Formatting

```rust
let formatted = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
// "2026-03-01 12:00:00"
```

### Arithmetic

```rust
use chrono::Duration;

let tomorrow = Utc::now() + Duration::days(1);
let diff = end - start;  // TimeDelta
let seconds = diff.num_seconds();
```

## API Reference Table

| Type / Function | Description |
|-----------------|-------------|
| `Utc::now()` | Current UTC time |
| `Local::now()` | Current local time |
| `DateTime::from_timestamp(secs, nsecs)` | From Unix timestamp |
| `dt.timestamp()` | To Unix timestamp (seconds) |
| `dt.timestamp_millis()` | To Unix timestamp (milliseconds) |
| `dt.naive_utc()` | Strip timezone → NaiveDateTime |
| `naive.and_utc()` | NaiveDateTime → DateTime<Utc> |
| `dt.format(fmt)` | Format with strftime pattern |
| `NaiveDateTime::parse_from_str(s, fmt)` | Parse with format |
| `dt.date_naive()` | Extract NaiveDate |
| `dt.time()` | Extract NaiveTime |
| `Duration::days(n)` | Duration of n days |
| `Duration::hours(n)` | Duration of n hours |
| `Duration::seconds(n)` | Duration of n seconds |

### Common Format Specifiers

| Specifier | Description | Example |
|-----------|-------------|---------|
| `%Y` | Year (4 digit) | `2026` |
| `%m` | Month (01-12) | `03` |
| `%d` | Day (01-31) | `01` |
| `%H` | Hour 24h (00-23) | `14` |
| `%M` | Minute (00-59) | `30` |
| `%S` | Second (00-59) | `45` |
| `%Y-%m-%dT%H:%M:%SZ` | ISO 8601 | `2026-03-01T14:30:45Z` |
| `%+` | RFC 3339 | `2026-03-01T14:30:45+00:00` |

## When Writing Code

1. Use `DateTime<Utc>` for storage and API responses — not `NaiveDateTime`
2. `NaiveDateTime` → `DateTime<Utc>`: use `.and_utc()`
3. `DateTime<Utc>` → `NaiveDateTime`: use `.naive_utc()`
4. For serde: `#[serde(with = "chrono::serde::ts_seconds")]` for Unix timestamps
5. `Duration::days(1)` is `chrono::Duration`, not `std::time::Duration`
6. `from_ymd_opt` / `and_hms_opt` return `Option` — always handle `None`

## When Answering Questions

1. `DateTime<Utc>` vs `NaiveDateTime`: Utc carries timezone info, Naive does not
2. sea-orm maps `timestamptz` → `DateTime<Utc>`, `timestamp` → `NaiveDateTime`
3. `chrono::Duration` and `std::time::Duration` are different types — no auto-conversion
4. serde default: `DateTime<Utc>` serializes as RFC 3339 string (`"2026-03-01T12:00:00Z"`)
5. `Utc.timestamp_opt()` is deprecated — use `DateTime::from_timestamp()` instead
