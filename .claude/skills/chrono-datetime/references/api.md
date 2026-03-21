# Chrono API Reference

> chrono 0.4 | Source: https://docs.rs/chrono/0.4

## Type Overview

| Type | Has Timezone | Use for |
|------|-------------|---------|
| `DateTime<Utc>` | Yes (UTC) | Storage, APIs, cross-system timestamps |
| `DateTime<Local>` | Yes (local) | Display to user in local time |
| `DateTime<FixedOffset>` | Yes (fixed) | Specific timezone offset |
| `NaiveDateTime` | No | DB columns without tz, intermediate |
| `NaiveDate` | No | Date-only values |
| `NaiveTime` | No | Time-only values |

## Creation

### Current Time

```rust
use chrono::{Utc, Local};

let now = Utc::now();           // DateTime<Utc>
let local = Local::now();       // DateTime<Local>
```

### From Components

```rust
use chrono::{NaiveDate, NaiveTime, NaiveDateTime};

// Date
let date = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();

// Time
let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();
let time_ms = NaiveTime::from_hms_milli_opt(14, 30, 0, 500).unwrap();

// DateTime
let dt = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap()
    .and_hms_opt(14, 30, 0).unwrap();  // NaiveDateTime

// With timezone
let utc = dt.and_utc();  // DateTime<Utc>
```

### From Timestamps

```rust
use chrono::DateTime;

// Unix timestamp → DateTime<Utc>
let dt = DateTime::from_timestamp(1709251200, 0).unwrap();

// With milliseconds
let dt = DateTime::from_timestamp_millis(1709251200000).unwrap();

// With nanoseconds
let dt = DateTime::from_timestamp_nanos(1709251200_000_000_000);
```

### Deprecated Patterns

| Deprecated | Use Instead |
|------------|-------------|
| `Utc.timestamp(secs, nsecs)` | `DateTime::from_timestamp(secs, nsecs)` |
| `Utc.timestamp_millis(ms)` | `DateTime::from_timestamp_millis(ms)` |
| `NaiveDate::from_ymd(y, m, d)` | `NaiveDate::from_ymd_opt(y, m, d).unwrap()` |
| `NaiveTime::from_hms(h, m, s)` | `NaiveTime::from_hms_opt(h, m, s).unwrap()` |
| `date.and_hms(h, m, s)` | `date.and_hms_opt(h, m, s).unwrap()` |

## Parsing

### From String

```rust
// RFC 3339 / ISO 8601 (via FromStr)
let dt: DateTime<Utc> = "2026-03-01T14:30:00Z".parse().unwrap();
let dt: DateTime<FixedOffset> = "2026-03-01T14:30:00+09:00".parse().unwrap();

// RFC 2822
let dt = DateTime::parse_from_rfc2822("Sat, 1 Mar 2026 14:30:00 +0000")?;

// RFC 3339
let dt = DateTime::parse_from_rfc3339("2026-03-01T14:30:00Z")?;
```

### Custom Format

```rust
// NaiveDateTime
let dt = NaiveDateTime::parse_from_str(
    "2026-03-01 14:30:00", "%Y-%m-%d %H:%M:%S"
)?;

// NaiveDate
let d = NaiveDate::parse_from_str("2026-03-01", "%Y-%m-%d")?;

// DateTime with timezone
let dt = DateTime::parse_from_str(
    "2026-03-01 14:30:00 +0900", "%Y-%m-%d %H:%M:%S %z"
)?;
```

## Formatting

```rust
// Standard formats
let rfc3339 = dt.to_rfc3339();    // "2026-03-01T14:30:00+00:00"
let rfc2822 = dt.to_rfc2822();    // "Sat, 1 Mar 2026 14:30:00 +0000"

// Custom format
let s = dt.format("%Y-%m-%d %H:%M:%S").to_string();
// "2026-03-01 14:30:00"

// Display trait uses RFC 3339
println!("{}", dt);  // "2026-03-01 14:30:00 UTC"
```

### Format Specifiers

| Spec | Description | Example |
|------|-------------|---------|
| `%Y` | Year 4-digit | `2026` |
| `%y` | Year 2-digit | `26` |
| `%m` | Month (01-12) | `03` |
| `%b` | Month abbrev | `Mar` |
| `%B` | Month full | `March` |
| `%d` | Day (01-31) | `01` |
| `%e` | Day (space-padded) | ` 1` |
| `%H` | Hour 24h (00-23) | `14` |
| `%I` | Hour 12h (01-12) | `02` |
| `%M` | Minute (00-59) | `30` |
| `%S` | Second (00-59) | `00` |
| `%f` | Nanoseconds | `000000000` |
| `%3f` | Milliseconds | `000` |
| `%P` | am/pm | `pm` |
| `%p` | AM/PM | `PM` |
| `%Z` | Timezone name | `UTC` |
| `%z` | Timezone offset | `+0000` |
| `%:z` | Timezone offset | `+00:00` |
| `%A` | Weekday full | `Saturday` |
| `%a` | Weekday abbrev | `Sat` |
| `%+` | RFC 3339 | `2026-03-01T14:30:00+00:00` |

## Arithmetic

```rust
use chrono::Duration;

// Add/subtract duration
let tomorrow = Utc::now() + Duration::days(1);
let yesterday = Utc::now() - Duration::days(1);
let later = Utc::now() + Duration::hours(6);

// Difference between two DateTimes
let diff: TimeDelta = end - start;
diff.num_seconds();      // i64
diff.num_milliseconds(); // i64
diff.num_minutes();      // i64
diff.num_hours();        // i64
diff.num_days();         // i64
```

### Duration Constructors

| Constructor | Description |
|-------------|-------------|
| `Duration::weeks(n)` | n weeks |
| `Duration::days(n)` | n days |
| `Duration::hours(n)` | n hours |
| `Duration::minutes(n)` | n minutes |
| `Duration::seconds(n)` | n seconds |
| `Duration::milliseconds(n)` | n milliseconds |
| `Duration::microseconds(n)` | n microseconds |
| `Duration::nanoseconds(n)` | n nanoseconds |

## Conversions

```rust
// DateTime<Utc> → NaiveDateTime
let naive = dt.naive_utc();

// NaiveDateTime → DateTime<Utc>
let utc = naive.and_utc();

// DateTime<Utc> → DateTime<Local>
let local: DateTime<Local> = dt.with_timezone(&Local);

// DateTime<Utc> → DateTime<FixedOffset>
let offset = FixedOffset::east_opt(9 * 3600).unwrap(); // +09:00
let tokyo: DateTime<FixedOffset> = dt.with_timezone(&offset);

// DateTime → Unix timestamp
let secs: i64 = dt.timestamp();
let millis: i64 = dt.timestamp_millis();

// Extract components
let date: NaiveDate = dt.date_naive();
let time: NaiveTime = dt.time();
let year: i32 = dt.year();
let month: u32 = dt.month();
let day: u32 = dt.day();
let weekday: Weekday = dt.weekday();
```

## Serde Integration

```rust
// Default: RFC 3339 string
#[derive(Serialize, Deserialize)]
struct Event {
    created_at: DateTime<Utc>,  // "2026-03-01T14:30:00Z"
}

// Unix timestamp (seconds)
use chrono::serde::ts_seconds;
#[derive(Serialize, Deserialize)]
struct Event {
    #[serde(with = "ts_seconds")]
    created_at: DateTime<Utc>,  // 1709251200
}

// Unix timestamp (milliseconds)
use chrono::serde::ts_milliseconds;
#[derive(Serialize, Deserialize)]
struct Event {
    #[serde(with = "ts_milliseconds")]
    created_at: DateTime<Utc>,  // 1709251200000
}

// Optional timestamp
use chrono::serde::ts_seconds_option;
#[derive(Serialize, Deserialize)]
struct Event {
    #[serde(with = "ts_seconds_option")]
    deleted_at: Option<DateTime<Utc>>,
}
```

## Cargo Features

```toml
chrono = { version = "0.4", features = ["serde"] }
```

| Feature | Description |
|---------|-------------|
| `serde` | Serialize/Deserialize support |
| `clock` | `Utc::now()`, `Local::now()` (default) |
| `std` | Standard library (default) |
