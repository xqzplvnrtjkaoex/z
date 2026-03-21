# Tokio API Reference

> tokio 1.49 | Source: https://docs.rs/tokio/1.49

## Runtime Configuration

```rust
// Custom runtime
let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(4)
    .enable_all()
    .build()?;

rt.block_on(async { /* ... */ });

// Current-thread runtime
let rt = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()?;
```

## Task Spawning

### spawn

```rust
// Basic spawn — requires Send + 'static
let handle = tokio::spawn(async move {
    expensive_work().await
});

// Await result
let result: Result<T, JoinError> = handle.await;
```

### spawn_blocking

```rust
// For CPU-bound or synchronous I/O
let result = tokio::task::spawn_blocking(move || {
    std::fs::read_to_string("file.txt")
}).await??;
```

### JoinSet

```rust
use tokio::task::JoinSet;

let mut set = JoinSet::new();

for id in ids {
    set.spawn(async move {
        fetch(id).await
    });
}

// Collect results as they complete
while let Some(result) = set.join_next().await {
    let value = result?;
    // process value
}
```

### abort

```rust
let handle = tokio::spawn(async { /* ... */ });
handle.abort(); // cancel the task
// task is cancelled at the next .await point
```

## Channels

### mpsc (multi-producer, single-consumer)

```rust
use tokio::sync::mpsc;

// Bounded (backpressure)
let (tx, mut rx) = mpsc::channel::<String>(32);

// Send
tx.send("hello".into()).await?;  // blocks if full
tx.try_send("hello".into())?;   // returns Err if full

// Receive
match rx.recv().await {
    Some(msg) => { /* process */ }
    None => { /* all senders dropped */ }
}

// Clone sender for multiple producers
let tx2 = tx.clone();
```

### unbounded mpsc

```rust
let (tx, mut rx) = mpsc::unbounded_channel::<String>();
tx.send("hello".into())?; // never blocks (no await)
```

### oneshot (single-value response)

```rust
use tokio::sync::oneshot;

let (tx, rx) = oneshot::channel::<u32>();
tx.send(42).unwrap(); // no await — immediate
let val = rx.await?;
```

### broadcast (multi-consumer)

```rust
use tokio::sync::broadcast;

let (tx, _) = broadcast::channel::<String>(16);
let mut rx1 = tx.subscribe();
let mut rx2 = tx.subscribe();

tx.send("hello".into())?;
// both rx1 and rx2 receive "hello"
```

### watch (latest-value)

```rust
use tokio::sync::watch;

let (tx, mut rx) = watch::channel("initial".to_string());

// Update
tx.send("updated".into())?;

// Receive latest
let val = rx.borrow().clone();

// Wait for change
rx.changed().await?;
let new_val = rx.borrow().clone();
```

### Notify (task wake-up, no data)

```rust
use tokio::sync::Notify;
use std::sync::Arc;

let notify = Arc::new(Notify::new());

// Waiter
let n = notify.clone();
tokio::spawn(async move {
    n.notified().await;
    println!("woke up");
});

// Notifier
notify.notify_one();
// or notify.notify_waiters() for all
```

## Sync Primitives

### Mutex

```rust
use tokio::sync::Mutex;
use std::sync::Arc;

let data = Arc::new(Mutex::new(vec![]));

let d = data.clone();
tokio::spawn(async move {
    let mut guard = d.lock().await;
    guard.push(1);
    // guard dropped here, lock released
});
```

**When to use tokio::sync::Mutex vs std::sync::Mutex:**

| | `tokio::sync::Mutex` | `std::sync::Mutex` |
|---|---|---|
| Holds across `.await` | Yes | No (not `Send`) |
| Performance | Slightly slower | Faster |
| Use when | Lock held across async ops | Quick, synchronous access |

### RwLock

```rust
use tokio::sync::RwLock;

let data = Arc::new(RwLock::new(Config::default()));

// Read (multiple concurrent readers)
let config = data.read().await;

// Write (exclusive)
let mut config = data.write().await;
config.update();
```

### Semaphore

```rust
use tokio::sync::Semaphore;

let sem = Arc::new(Semaphore::new(10)); // 10 permits

let permit = sem.acquire().await?;
// do work with limited concurrency
drop(permit); // release
```

## Time

```rust
use tokio::time::{sleep, timeout, interval, Duration};

// Sleep
sleep(Duration::from_secs(1)).await;

// Timeout
match timeout(Duration::from_secs(5), slow_op()).await {
    Ok(result) => { /* completed in time */ }
    Err(_) => { /* timed out */ }
}

// Interval
let mut interval = interval(Duration::from_secs(60));
loop {
    interval.tick().await;
    do_periodic_work().await;
}
```

## Select

```rust
use tokio::select;

// First future to complete wins; others are cancelled
select! {
    result = async_op1() => { /* op1 finished first */ }
    result = async_op2() => { /* op2 finished first */ }
    _ = sleep(Duration::from_secs(10)) => { /* timeout */ }
}

// With biased (check in order, prevent starvation)
select! {
    biased;
    msg = priority_rx.recv() => { /* checked first */ }
    msg = normal_rx.recv() => { /* checked second */ }
}
```

## Signal Handling

```rust
use tokio::signal;

// Wait for Ctrl+C
signal::ctrl_c().await?;

// Unix signals
#[cfg(unix)]
{
    use tokio::signal::unix::{signal, SignalKind};
    let mut sigterm = signal(SignalKind::terminate())?;
    sigterm.recv().await;
}
```

## Graceful Shutdown Pattern

```rust
use tokio::sync::watch;

let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

// In worker tasks
tokio::spawn(async move {
    loop {
        select! {
            _ = shutdown_rx.changed() => break,
            msg = rx.recv() => { /* process */ }
        }
    }
});

// Trigger shutdown
shutdown_tx.send(true)?;
```

## Cargo Features

```toml
tokio = { version = "1", features = ["full"] }
# or pick specific features:
tokio = { version = "1", features = [
    "rt-multi-thread", "macros", "time", "sync",
    "signal", "io-util", "net", "fs",
] }
```

| Feature | Enables |
|---------|---------|
| `rt` | Runtime core |
| `rt-multi-thread` | Multi-threaded runtime |
| `macros` | `#[tokio::main]`, `#[tokio::test]` |
| `time` | `sleep`, `timeout`, `interval` |
| `sync` | Channels, Mutex, RwLock, Semaphore |
| `signal` | Signal handling |
| `io-util` | `AsyncReadExt`, `AsyncWriteExt` |
| `net` | TCP, UDP, Unix sockets |
| `fs` | Async filesystem operations |
| `full` | All features |
