---
phase: quick
plan: 260324-rfl
type: execute
wave: 1
depends_on: []
files_modified:
  - services/user/src/domain/input/handle_input.rs
  - services/user/Cargo.toml
  - Cargo.toml
autonomous: true
requirements: []

must_haves:
  truths:
    - "Handle validation accepts only ASCII alphanumeric and underscore characters"
    - "No regex dependency remains in the user service"
    - "All existing handle_input tests pass unchanged"
  artifacts:
    - path: "services/user/src/domain/input/handle_input.rs"
      provides: "char-based handle validation without regex"
      contains: "is_ascii_alphanumeric"
  key_links: []
---

<objective>
Replace `HANDLE_REGEX` (`LazyLock<Regex>`) in `handle_input.rs` with Rust std char methods. The regex pattern `^[a-zA-Z0-9_]+$` is trivially expressible as `c.is_ascii_alphanumeric() || c == '_'`, eliminating the `regex` crate dependency from the user service entirely (it is the sole consumer).

Purpose: Remove unnecessary heavyweight dependency; simplify validation to idiomatic Rust.
Output: Updated `handle_input.rs`, removed `regex` from user and workspace Cargo.toml.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@services/user/src/domain/input/handle_input.rs
@services/user/Cargo.toml
@Cargo.toml
</context>

<tasks>

<task type="auto">
  <name>Task 1: Replace regex with char methods and remove regex dependency</name>
  <files>services/user/src/domain/input/handle_input.rs, services/user/Cargo.toml, Cargo.toml</files>
  <action>
In `handle_input.rs`:
1. Remove `use std::sync::LazyLock;`
2. Remove the `static HANDLE_REGEX: LazyLock<regex::Regex>` declaration (lines 17-18)
3. Rewrite `check_handle_chars` to use std char methods:
   ```rust
   fn check_handle_chars(handle: &str) -> Result<(), validator::ValidationError> {
       if !handle.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
           return Err(validator::ValidationError::new("invalid_handle_chars"));
       }
       Ok(())
   }
   ```
   Use `.bytes().all()` with byte comparison — the length validator already ensures non-empty, and the handle domain is pure ASCII so byte-level iteration is correct and avoids char decode overhead.

In `services/user/Cargo.toml`:
4. Remove the line `regex = { workspace = true }` from `[dependencies]`

In workspace `Cargo.toml`:
5. Remove the line `regex = "1"` from the `[workspace.dependencies]` section (under `# Validation`)
  </action>
  <verify>
    <automated>cd /Users/syr/Developments/madome && cargo test -p user -- handle && cargo build -p user</automated>
  </verify>
  <done>All 5 existing handle_input tests pass. `regex` no longer appears in user Cargo.toml or workspace Cargo.toml. No `LazyLock` or `regex::` references remain in handle_input.rs.</done>
</task>

</tasks>

<verification>
- `cargo test -p user -- handle` — all 5 handle_input tests pass
- `cargo build -p user` — compiles without regex dependency
- `grep -r "regex" services/user/src/` returns no matches
- `grep "regex" services/user/Cargo.toml` returns no matches
- `grep "regex" Cargo.toml` returns no matches
</verification>

<success_criteria>
Handle validation behavior identical (all tests pass), regex crate fully removed from workspace.
</success_criteria>

<output>
After completion, create `.planning/quick/260324-rfl-replace-handle-regex-lazylock-regex-with/260324-rfl-SUMMARY.md`
</output>
