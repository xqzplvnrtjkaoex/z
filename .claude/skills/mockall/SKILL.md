---
name: mockall
description: "CRITICAL: Use for mockall mock generation in Rust. Triggers on: mockall, #[automock], mock!, MockXxx, expect_xxx, returning, times, withf, with, in_sequence, Sequence, mock trait, test double"
version: "0.14"
---

# mockall Crate Skill (v0.14)

## Quick Reference

```rust
use mockall::automock;
use mockall::predicate::*;

#[automock]
trait UserRepository {
    fn find_by_id(&self, id: Uuid) -> Option<User>;
}

// In tests:
let mut mock = MockUserRepository::new();
mock.expect_find_by_id()
    .with(eq(user_id))
    .times(1)
    .returning(|_| Some(User { ... }));
```

## #[automock] on Traits

```rust
#[automock]
trait MyTrait {
    fn simple(&self, x: i32) -> String;
    fn by_ref(&self, x: &str) -> bool;
    fn mutable(&mut self, x: Vec<u8>);
}
// Generates: MockMyTrait with expect_simple(), expect_by_ref(), expect_mutable()
```

### Naming convention
- Trait `Foo` → Mock struct `MockFoo`
- Method `bar()` → Expectation `expect_bar()`

## #[automock] on Async Traits

```rust
#[automock]
#[async_trait]
trait AsyncRepo {
    async fn find(&self, id: Uuid) -> Result<User, Error>;
}

// Works the same way:
mock.expect_find()
    .returning(|_| Ok(user));
```

### With trait-variant (Send-bound async)
```rust
// For #[trait_variant::make(SendTrait: Send)] patterns,
// apply #[automock] to the Send variant trait
#[automock]
trait UserRepositorySend: Send {
    fn find_by_id(&self, id: Uuid) -> impl Future<Output = Result<User, Error>> + Send;
}
```

## mock! Macro (Manual Definition)

For structs, foreign traits, or complex cases:

```rust
mock! {
    pub MyStruct {
        fn do_something(&self, x: i32) -> String;
    }
    impl Clone for MyStruct {
        fn clone(&self) -> Self;
    }
}
// Generates: MockMyStruct
```

## Expectation API

### Setting return values
```rust
.returning(|arg| value)          // closure, called each time
.return_const(value)             // clone same value each time (T: Clone)
.return_once(|arg| value)        // FnOnce, single use (for non-Clone)
```

### Call count
```rust
.times(1)                        // exact count
.times(2..5)                     // range (2, 3, or 4 times)
.times(0)                        // must NOT be called
.never()                         // alias for times(0)
.once()                          // alias for times(1)
```

### Argument matching
```rust
use mockall::predicate::*;

.with(eq(42))                    // exact match
.with(ne(0))                     // not equal
.with(gt(10))                    // greater than (also ge, lt, le)
.with(always())                  // any value
.with(str::starts_with("pre"))   // string predicates
.with(function(|x: &i32| *x > 0)) // custom predicate
.withf(|x: &i32| *x > 0)        // shorthand for function predicate

// Multiple arguments: tuple of predicates
.with(eq(1), eq("hello"))
```

### Sequences (ordered expectations)
```rust
use mockall::Sequence;

let mut seq = Sequence::new();
mock.expect_first()
    .times(1)
    .in_sequence(&mut seq)
    .returning(|| 1);
mock.expect_second()
    .times(1)
    .in_sequence(&mut seq)
    .returning(|| 2);
```

## Common Patterns

### Returning different values on successive calls
```rust
let mut mock = MockRepo::new();
let mut count = 0;
mock.expect_next()
    .times(3)
    .returning(move |_| {
        count += 1;
        count
    });
```

### Returning Err for failure paths
```rust
mock.expect_find_by_id()
    .with(eq(bad_id))
    .returning(|_| Err(DomainError::NotFound));
```

### Mock with generic methods
```rust
#[automock]
trait Store {
    fn get<T: 'static>(&self, key: &str) -> Option<T>;
}
// Must set expectations per concrete type:
mock.expect_get::<String>()
    .returning(|_| Some("value".into()));
```

## Checkpoint

```rust
mock.checkpoint();  // Verify all current expectations, then clear them
// Useful for "phase" testing within a single test
```

## Common Mistakes

1. **Forgetting `.returning()`** — panics at runtime with "no matching expectation"
2. **Expectations are verified on drop** — if mock drops without meeting `times()`, test panics
3. **Order matters for multiple expectations on same method** — last matching expectation wins (LIFO)
4. **`&str` arguments use `&str` in predicates**, not `String`
5. **`returning` closure must be `Send + Sync`** for async mocks — use `Arc<Mutex<>>` if needed for shared state
6. **Don't test mock configuration** — test behavior (outputs), not that mock was called with specific args. Use `.returning()` for setup, assertions for output.
