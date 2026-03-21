---
name: tokio-runtime
description: |
  CRITICAL: Use for tokio async runtime. Triggers on:
  tokio, spawn, select, JoinHandle, JoinSet, signal,
  tokio::sync, mpsc, oneshot, broadcast, watch, Notify,
  tokio::time, sleep, timeout, interval, runtime builder
---

# Tokio Runtime Skill

> **Version:** tokio 1.49 | **Last Updated:** 2026-03-01
>
> Check for updates: https://crates.io/crates/tokio

You are an expert at the Rust `tokio` crate. Help users by:
- **Writing code**: Generate async runtime setup, task spawning, channel usage, signal handling
- **Answering questions**: Explain task scheduling, cancellation, sync primitives

## Documentation

- `./references/api.md` — Runtime, spawning, channels, timers, signals, I/O

## Key Patterns

### Runtime Setup

```rust
#[tokio::main]
async fn main() {
    // full multi-threaded runtime (default)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // single-threaded runtime
}
```

### Task Spawning

```rust
// Fire-and-forget
tokio::spawn(async move {
    do_work().await;
});

// With result
let handle: JoinHandle<u32> = tokio::spawn(async { 42 });
let result = handle.await?; // Result<u32, JoinError>

// Blocking work on dedicated thread pool
let result = tokio::task::spawn_blocking(move || {
    expensive_sync_computation()
}).await?;
```

### Channels

```rust
use tokio::sync::{mpsc, oneshot};

// mpsc (multi-producer, single-consumer)
let (tx, mut rx) = mpsc::channel::<String>(32); // bounded
tx.send("hello".into()).await?;
let msg = rx.recv().await; // Option<String>

// oneshot (single-use response)
let (tx, rx) = oneshot::channel::<u32>();
tx.send(42).unwrap(); // no await needed
let val = rx.await?;
```

### Select

```rust
tokio::select! {
    val = rx.recv() => { /* channel message */ }
    _ = tokio::time::sleep(Duration::from_secs(5)) => { /* timeout */ }
    _ = shutdown.recv() => { /* shutdown signal */ }
}
```

## API Reference Table

| Function / Type | Description |
|-----------------|-------------|
| `tokio::spawn(fut)` | Spawn task on runtime (requires `Send`) |
| `tokio::spawn_blocking(fn)` | Run blocking code on dedicated pool |
| `tokio::select!` | Wait on multiple futures, first wins |
| `JoinHandle<T>` | Handle to spawned task |
| `JoinSet<T>` | Manage group of spawned tasks |
| `mpsc::channel(cap)` | Bounded multi-producer channel |
| `mpsc::unbounded_channel()` | Unbounded channel |
| `oneshot::channel()` | Single-use channel |
| `broadcast::channel(cap)` | Multi-consumer broadcast |
| `watch::channel(init)` | Single-value watch (latest only) |
| `Notify` | Task notification (no data) |
| `Mutex<T>` | Async-aware mutex |
| `RwLock<T>` | Async-aware read-write lock |
| `Semaphore` | Async-aware semaphore |
| `tokio::time::sleep(dur)` | Async sleep |
| `tokio::time::timeout(dur, fut)` | Future with timeout |
| `tokio::time::interval(dur)` | Repeating timer |
| `tokio::signal::ctrl_c()` | Wait for Ctrl+C |

## When Writing Code

1. `tokio::spawn` requires `Send + 'static` — no borrowed references across `.await`
2. Use `spawn_blocking` for CPU-heavy or synchronous I/O (file system, etc.)
3. Prefer `mpsc` over `Mutex` for cross-task communication
4. `select!` cancels unfinished branches — ensure cancel-safety
5. `JoinSet` over manual `Vec<JoinHandle>` when managing dynamic task groups
6. `tokio::sync::Mutex` only needed when holding lock across `.await` — otherwise use `std::sync::Mutex`

## When Answering Questions

1. `#[tokio::main]` expands to `Runtime::new().block_on(main())`
2. `tokio::spawn` returns immediately — the task runs concurrently
3. `JoinHandle.abort()` cancels the task at the next `.await` point
4. `select!` is cancel-safe only if the futures are cancel-safe (most channel ops are)
5. `watch` keeps only the latest value — receivers see the most recent, not a queue
