# Quick Task 260325-1um: Make payload fields private — Context

**Gathered:** 2026-03-25
**Status:** Ready for planning

<domain>
## Task Boundary

Make all payload struct fields private in `services/user/src/payload/user.rs`. Add `new()` constructors. Update all call sites (app/rpc/, tests) to use `new()` instead of struct literals.

</domain>

<decisions>
## Implementation Decisions

### Construction pattern
- Add `pub fn new(...)` to each payload struct. All fields become private.
- `From` impls (in app/rpc/) call `Payload::new(...)` internally.
- Tests also use `Payload::new(...)`.

### From impl location
- Stays in `app/rpc/` per existing convention (proto → payload conversion belongs to the layer that knows both types).
- Now calls `new()` instead of struct literal.

### Test construction
- Both payload module tests and usecase tests use `Payload::new(...)`.
- No test-only helpers needed — `new()` is the public API.

</decisions>

<specifics>
## Specific Ideas

No specific requirements — standard refactor following the decided pattern.

</specifics>
