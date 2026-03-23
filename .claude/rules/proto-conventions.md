---
paths: ["proto/**/*.proto"]
---

# Proto Conventions

## UUID Fields

Use `bytes` for all UUID fields. Never use `string` for UUIDs.

```protobuf
// Good
bytes id = 1;
bytes user_id = 2;

// Bad
string id = 1;
string user_id = 2;
```
