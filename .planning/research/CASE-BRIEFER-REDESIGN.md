# Case Briefer Redesign: Text-Based Operation Extraction

**Researched:** 2026-03-24
**Domain:** Pre-implementation behavioral case discovery from planning documents
**Confidence:** HIGH

---

## 1. Problem Statement

The current `case-briefer` agent (`.claude/agents/case-briefer.md`) is entirely code-dependent. Its methodology scans source files for handlers, validators, test coverage, and domain constraints. This works when code exists but fails completely for new phases where no implementation code exists yet.

The /case skill runs AFTER `gsd:discuss-phase` but BEFORE `gsd:plan-phase`. At this point in the workflow:

| Exists | Does NOT Exist |
|--------|----------------|
| CONTEXT.md (locked design decisions) | Service implementation code |
| ROADMAP.md (phase description, requirements, success criteria) | Proto files (for new services) |
| REQUIREMENTS.md (requirement IDs and descriptions) | Database schemas |
| RESEARCH.md (technology patterns, sometimes) | Test files |
| PROJECT.md (architecture, service topology) | Validation logic |

The briefer must be redesigned to extract operations from **text** (planning documents) rather than **code** (source files).

---

## 2. Current Briefer Analysis

### What the Current Briefer Does (6 Steps)

| Step | Action | Source | Keep/Remove |
|------|--------|--------|-------------|
| 1: Understand phase scope | Read phase context | CONTEXT.md, ROADMAP.md | **KEEP** -- same documents available pre-implementation |
| 2: Discover operations | Scan callable interfaces | Source code (handlers, traits, routes) | **REMOVE** -- no source code exists |
| 3: Analyze existing validation | Find input validation logic | Source code (guard clauses, error types) | **REMOVE** -- no validation code exists |
| 4: Survey existing tests | Search for test files | Source code (test directories) | **REMOVE** -- no tests exist |
| 5: Extract domain constraints | Find business rules in code | Source code (constraints, enums, state machines) | **REMOVE** -- but CONTEXT.md captures these explicitly |
| 6: Group and write | Organize by category | Derived from steps 2-5 | **KEEP** -- grouping methodology applies to text-derived operations too |

### Current Output Contract Fields

| Field | Current Source | New Source |
|-------|---------------|-----------|
| Name | Function/method/endpoint name | Derived from CONTEXT.md decision descriptions |
| Interface | HTTP method + path, RPC name | Explicitly listed in CONTEXT.md endpoint tables |
| Inputs | Function parameters, request types | Inferred from CONTEXT.md decision descriptions + ROADMAP success criteria |
| Outputs | Return types, response types | Inferred from CONTEXT.md decision descriptions |
| Auth requirement | Middleware, guard logic | Explicitly listed in CONTEXT.md route tier tables |
| Existing validation | Guard clauses, validators | **REPLACED BY:** Decided constraints (from CONTEXT.md decisions) |
| Existing tests | Test file coverage | **REPLACED BY:** N/A -- no tests exist pre-implementation |
| Domain constraints | Code-level constraints | Explicitly listed in CONTEXT.md decisions (uniqueness, enums, limits) |

### What to Keep from the Current Design

1. **Output format structure** -- CASE-BRIEFING.md format (operation groups, per-operation details) is sound regardless of source
2. **Quality gate concept** -- cross-check operations against phase requirements
3. **Category grouping** -- natural operation grouping by domain cluster
4. **Downstream consumer contract** -- the Protester needs operations with interfaces, inputs, outputs, auth, grouped by category

### What Must Change

1. **Data extraction methodology** -- from code scanning to text parsing
2. **Per-operation detail fields** -- "Existing validation" and "Existing tests" become "Decided constraints" and "Open decisions"
3. **Source references** -- from `file:line` to `D-XX decision ID`
4. **Confidence model** -- distinguish between explicit (stated in CONTEXT.md) and inferred (derived from descriptions)

---

## 3. Text-Based Extraction Methodology

### Overview

The redesigned briefer extracts operations from three primary documents, each contributing different information layers:

```
CONTEXT.md              ROADMAP.md              REQUIREMENTS.md
 |                       |                       |
 +-> Endpoint tables     +-> Success criteria     +-> Requirement descriptions
 +-> Decision details    +-> Phase goal            (REQ-ID -> behavior)
 +-> Proto RPC lists     +-> Dependencies
 +-> Auth tiers
 +-> Domain constraints
 +-> Cross-service flows
```

### Step-by-Step Extraction

#### Step 1: Parse Endpoint Tables from CONTEXT.md

CONTEXT.md files contain explicit REST endpoint tables. These are the primary source of operations.

**Pattern observed across all three existing CONTEXT.md files:**

Phase 1 does not have endpoint tables (infrastructure only). Phase 2 has endpoint descriptions in decisions D-42 through D-47. Phase 3 has full endpoint tables with columns: Tier, Method, Path, Description.

**Extraction rule:** Scan `<decisions>` section for:
1. Markdown tables with HTTP method columns (GET, POST, PATCH, DELETE, PUT)
2. Decision items describing endpoints (e.g., "D-42: All user endpoints under `/v1/users`")
3. Proto RPC lists (e.g., "D-24: User service RPCs: CreateUser, GetUser, ...")

Each row in an endpoint table becomes one operation. Each RPC in a proto RPC list becomes one operation (some RPCs map 1:1 to REST endpoints; some are internal-only).

**Example from Phase 3 CONTEXT.md:**

```
| admin | POST | /v1/admin/invites | Auth svc | Issue invite (with role) |
```

Extracts to:
- **Name:** IssueInvite
- **Interface:** POST /v1/admin/invites
- **Auth:** admin tier
- **Routes to:** Auth service
- **Description:** Issue invite with role

#### Step 2: Parse Proto RPC Lists from CONTEXT.md

CONTEXT.md often lists gRPC RPCs separately from REST endpoints. These capture internal service-to-service operations.

**Pattern from Phase 3:**
```
D-59: Auth gRPC RPCs: RegisterBegin/Finish, LoginBegin/Finish, Recover,
      VerifyBegin/Finish, ValidateSession, RefreshToken, VerifyApiKey,
      InvalidateAllSessions, ListPasskeys, RenamePasskey, DeletePasskey,
      RegenerateRecoveryCodes
```

**Pattern from Phase 2:**
```
D-24: User service RPCs: CreateUser, GetUser, GetUserByHandle, ListUsers,
      UpdateUser, DeactivateUser, ActivateUser, ChangeRole
```

**Extraction rule:** Each RPC becomes an operation. Cross-reference with endpoint tables to determine which RPCs are exposed via REST (gateway-routed) vs internal-only (service-to-service).

**Classification:**
- RPC with matching REST endpoint: primary operation (the REST endpoint is the operation, RPC is the transport)
- RPC with no REST endpoint but mentioned as cross-service call: internal operation (e.g., Auth -> User service `CreateUser`)
- RPC that is purely infrastructure (Health, ValidateSession, RefreshToken): skip unless phase specifically targets it

#### Step 3: Extract Inputs and Outputs from Decision Details

CONTEXT.md decisions describe what data flows through each operation, though not always in a structured format.

**Patterns to scan for:**

1. **Explicit field lists in decisions:**
   ```
   D-112: Registration success response: 201 Created with
   { "user_id", "handle", "name", "role", "recovery_codes" }
   ```
   Extracts output fields: user_id, handle, name, role, recovery_codes.

2. **Request body descriptions:**
   ```
   D-23: Recovery flow: POST /v1/auth/recover (name + recovery_code in body)
   ```
   Extracts input fields: name (String), recovery_code (String).

3. **Schema decisions that imply fields:**
   ```
   D-01: Users table: id (UUIDv4 PK), handle (unique), name (display name),
   role (owner/admin/user enum), is_active (boolean), created_at, updated_at
   ```
   Implies output fields for GetUser/ListUsers operations.

4. **Cookie/header decisions:**
   ```
   D-44: Cookie: HttpOnly=true, Secure=true, SameSite=Strict
   D-111: Login success: 204 No Content. JWT via HttpOnly cookie only
   ```
   Implies output transport (Set-Cookie header, not response body).

**When fields are not explicit:** Mark as "inferred" and state what is known. For example, if CONTEXT.md says "admin can create API key" but does not list the request fields, the briefing should state:

```
- **Inputs:**
  - Inferred from D-28: `expires_at` (optional DateTime) -- expiry date
  - Inferred from D-29: scope is scraper-only (may be implicit, not a field)
  - [Other fields not specified in CONTEXT.md]
```

#### Step 4: Extract Auth Requirements from Route Tier Tables

CONTEXT.md Phase 3 introduces a tier system (D-46): public, protected, verified, admin, scraper. Each endpoint table row includes its tier.

**Extraction rule:** Copy the tier directly from the endpoint table. If no table exists (earlier phases), check for decision items describing access control.

**Tier mapping for briefing:**

| Tier | Briefing Representation |
|------|------------------------|
| public | Auth: none |
| protected | Auth: authenticated (JWT required) |
| verified | Auth: authenticated + recent step-up verification |
| admin | Auth: authenticated + admin+ role |
| scraper | Auth: API key |
| recovery | Auth: recovery JWT (restricted access) |

#### Step 5: Extract Domain Constraints from Decisions

Domain constraints in code become visible as explicit decision items in CONTEXT.md.

**Patterns to scan for:**

1. **Uniqueness constraints:** "D-08: Handle uniqueness: case-insensitive matching"
2. **Enum variants:** "D-07: Role system: owner/admin/user 3-tier hierarchy"
3. **Numeric limits:** "D-06: Handle length: 4~15 characters"
4. **State preconditions:** "D-38: User deactivation: immediately invalidates all sessions"
5. **Business rules:** "D-15: Last passkey cannot be deleted"
6. **Cross-field rules:** "D-18: Role change validation: caller_role > target_current_role AND caller_role > new_role"
7. **Single-use constraints:** "D-03: Invite token: single-use, 30-minute expiry"

**Extraction rule:** Attach each constraint to the operation(s) it governs. A constraint like "handle uniqueness" applies to CreateUser and UpdateUser. A constraint like "last passkey cannot be deleted" applies to DeletePasskey only.

#### Step 6: Identify Cross-Service Operations

CONTEXT.md documents cross-service flows (e.g., D-117 through D-120 in Phase 3).

**Extraction rule:** Flag operations that involve multiple services. Note the orchestration pattern and failure handling.

**Example:**
```
D-120: Deactivate orchestration: Gateway -> User (DeactivateUser) -> Auth
       (InvalidateAllSessions). Auth failure -> compensate by calling User (ActivateUser)
```

This means the "Deactivate User" operation in the briefing should note:
- Routes to: User service + Auth service (sequential, compensating)
- Failure mode: Auth failure triggers User reactivation as compensation

#### Step 7: Group and Write

Group operations by natural category, same as the current briefer. Categories emerge from the document structure:

- CONTEXT.md sections (User Onboarding, Passkey Flows, Recovery, API Key Management, etc.)
- ROADMAP success criteria clusters
- REST endpoint path prefixes (/v1/auth/*, /v1/admin/*, /v1/user/*)

---

## 4. Revised Output Contract

### CASE-BRIEFING.md Format (Text-Based)

```markdown
# Case Briefing: Phase [XX] - [Name]

**Generated:** [date]
**Source:** CONTEXT.md, ROADMAP.md, REQUIREMENTS.md (pre-implementation)
**Operations found:** [count]
**Categories:** [count]

---

## [Category Name]

### [OperationName]

- **Interface:** [how it is called -- REST endpoint or gRPC RPC]
- **Auth:** [none / authenticated / admin+ / verified / API key]
- **Inputs:**
  - `field_name`: `Type` -- [description] (source: D-XX)
  - [Inferred: field_name -- description, not explicitly specified]
- **Outputs:**
  - `field_name`: `Type` -- [description] (source: D-XX)
  - [Transport note: e.g., "JWT via Set-Cookie header, not response body"]
- **Decided constraints:** [what CONTEXT.md has already locked]
  - [constraint with decision ID, e.g., "single-use token (D-03)"]
  - [constraint, e.g., "case-insensitive handle uniqueness (D-08)"]
- **Open decisions:** [what is marked as Claude's Discretion in CONTEXT.md]
  - [decision area, e.g., "exact request/response message structure"]
- **Cross-service:** [if applicable -- which services, orchestration pattern]
- **Requirements:** [which requirement IDs this operation fulfills]

---

## Observations

[Cross-cutting patterns visible from CONTEXT.md decisions:
shared validation rules, common error handling strategy,
pagination conventions, authentication tiers,
anything the Protester should know that does not fit per-operation.]

## Extraction Confidence

| Operation | Confidence | Reason |
|-----------|-----------|--------|
| [name] | EXPLICIT | Interface and fields stated directly in CONTEXT.md |
| [name] | INFERRED | Interface derived from RPC list, fields inferred from schema decisions |
| [name] | PARTIAL | Some fields unknown -- marked as inferred |
```

### Key Differences from Code-Based Format

| Code-Based Field | Text-Based Replacement | Why |
|-----------------|----------------------|-----|
| `Existing validation: [from code]` | `Decided constraints: [from CONTEXT.md]` | No validation code exists yet. CONTEXT.md decisions ARE the specification for what validation will be implemented |
| `Existing tests: [from code]` | *Removed entirely* | No tests exist. The /case skill's entire purpose is to discover what tests WILL be needed |
| `Domain constraints: [with file:line refs]` | `Decided constraints: [with D-XX refs]` | Source references change from code locations to decision IDs |
| *Not present* | `Open decisions: [from Claude's Discretion]` | NEW -- tells the Protester which areas are still flexible for case discussion |
| *Not present* | `Requirements: [REQ-ID list]` | NEW -- links operations to requirements for traceability |
| *Not present* | `Extraction Confidence: [table]` | NEW -- honest reporting of what was explicit vs inferred |

---

## 5. Detailed Example: Phase 3 Authentication

Given Phase 3 CONTEXT.md (which exists), here is how the text-based extraction would produce a CASE-BRIEFING.md.

### Source: CONTEXT.md Endpoint Tables

The Phase 3 CONTEXT.md contains two endpoint tables: auth endpoints (14 rows) and admin endpoints (11 rows), plus a self-service endpoint (1 row). It also lists Proto RPCs in D-59.

### Extracted Operations (Grouped)

**Category: Registration Flow**

Operations extracted from the public-tier auth endpoint table rows for `/v1/auth/register/*`:

```markdown
### RegisterBegin

- **Interface:** POST /v1/auth/register/begin
- **Auth:** none (public)
- **Inputs:**
  - `token`: String -- invite token (source: D-16 "invite token verified in begin step")
- **Outputs:**
  - Ceremony cookie set (source: D-100 "begin response sets ceremony_id cookie")
  - WebAuthn PublicKeyCredentialCreationOptions in response body (inferred from D-16 "2-step begin+finish" + WebAuthn standard)
- **Decided constraints:**
  - Token must be valid, unused, unexpired -- 30 min expiry (D-03)
  - Token is single-use (D-03)
  - Generic error: invite_invalid for all token problems (D-52)
- **Open decisions:**
  - Exact request/response message structure (Claude's Discretion)
  - Ceremony cookie name (Claude's Discretion)
- **Requirements:** AUTH-01

### RegisterFinish

- **Interface:** POST /v1/auth/register/finish
- **Auth:** none (public, but requires valid ceremony cookie)
- **Inputs:**
  - WebAuthn AuthenticatorAttestationResponse (inferred from WebAuthn standard)
  - `name`: String -- display name (source: D-05)
  - `handle`: String -- unique identifier (source: D-05)
  - Ceremony cookie (automatic, source: D-100)
- **Outputs:**
  - 201 Created with { user_id, handle, name, role, recovery_codes } (source: D-112)
  - JWT set as HttpOnly cookie (source: D-44)
  - Session created in Redis (inferred from D-32)
- **Decided constraints:**
  - Handle: 4-15 chars, alphanumeric + underscore, case-insensitive unique (D-06, D-07, D-08)
  - Name: 1-20 chars by char count, Unicode allowed (D-10, D-11)
  - Reserved handles blocked (D-09)
  - 10 recovery codes generated, 8-char alphanumeric, stored as hashes (D-21, D-103)
  - AAGUID extracted from attestation CBOR (D-67)
  - Creates user in User service via gRPC (D-118)
- **Open decisions:**
  - Recovery code hash algorithm -- SHA-256 or similar (D-103 Claude's Discretion)
  - Proto message structure (Claude's Discretion)
- **Cross-service:** Auth -> User service CreateUser RPC (D-118)
- **Requirements:** AUTH-01, AUTH-03, AUTH-04
```

**Category: Admin - Invite Management**

Operations extracted from admin-tier endpoint table rows for `/v1/admin/invites`:

```markdown
### IssueInvite

- **Interface:** POST /v1/admin/invites
- **Auth:** admin+ (admin tier)
- **Inputs:**
  - `role`: String/Enum -- role for the invited user (source: D-02 "Invite table has role column")
  - [Other fields not specified -- e.g., description/notes are not mentioned]
- **Outputs:**
  - Invite record with token string (source: D-03 "token string returned via API")
  - [Exact response fields not specified]
- **Decided constraints:**
  - Role hierarchy: owner invite is DB seed only, admin invite requires owner, user invite requires admin+ (D-02)
  - Token: single-use, 30-minute expiry (D-03)
  - Owner role not available in invite creation API (D-115)
  - Invite history recorded (D-09)
- **Open decisions:**
  - Proto message structure (Claude's Discretion)
- **Requirements:** AUTH-01 (partial -- enables registration)
```

### What the Protester Learns from This

The briefing tells the Protester:

1. **RegisterFinish is the most complex operation** -- cross-service call, 6+ decided constraints, CBOR parsing, recovery code generation. High probing intensity warranted.
2. **IssueInvite has role hierarchy constraints** that need systematic probing: who can invite whom? What happens when a user tries to create an admin invite?
3. **Several operations share the ceremony cookie pattern** (RegisterBegin/Finish, LoginBegin/Finish, VerifyBegin/Finish) -- cross-operation consistency concern.
4. **Input fields for some operations are not fully specified** (marked as open decisions) -- the Protester should discover these during discussion, not assume.

---

## 6. Handling Ambiguity and Inference

### Classification: Explicit vs Inferred vs Unknown

Every piece of information in the briefing should be classified:

| Level | Definition | How to Mark | Example |
|-------|-----------|-------------|---------|
| **EXPLICIT** | Directly stated in CONTEXT.md with a decision ID | `(source: D-XX)` | "Handle: 4-15 chars (D-06)" |
| **INFERRED** | Logically derived from decisions + domain knowledge | `[Inferred: description]` | "WebAuthn challenge in response body" (inferred from D-16 + WebAuthn standard) |
| **UNKNOWN** | Not mentioned in any planning document | `[Not specified]` | Request body fields for IssueInvite beyond role |

### Rules for Inference

The briefer MAY infer:

1. **Standard protocol behavior** -- WebAuthn begin/finish always involves a challenge/response cycle. This is not a guess; it is a protocol specification.
2. **Field implications from schema** -- if CONTEXT.md says the users table has `created_at`, then GetUser likely returns `created_at`. This is a reasonable inference.
3. **HTTP semantics** -- if CONTEXT.md says "DELETE /v1/auth/passkeys/:id", the `:id` path parameter is an input. This is REST convention.

The briefer MUST NOT infer:

1. **Business rules not stated** -- do not invent validation rules that CONTEXT.md does not mention. If CONTEXT.md says nothing about invite description length, the briefer should not assume a limit.
2. **Implementation details** -- do not infer database behavior, caching strategy, or internal architecture from endpoint descriptions.
3. **Operations not mentioned** -- do not invent operations that CONTEXT.md does not describe. If there is no "GetInvite by ID" endpoint, do not create one.

### When to Flag as "Needs Clarification" vs Skip

| Situation | Action |
|-----------|--------|
| Operation mentioned in ROADMAP success criteria but no CONTEXT.md details | Include with PARTIAL confidence, note "minimal specification" |
| Decision marked "Claude's Discretion" | Include in "Open decisions" -- the Protester may want to discuss |
| Decision explicitly deferred | Do NOT include -- it is out of scope |
| Conflicting information between CONTEXT.md and ROADMAP.md | Include with note about conflict -- let Protester surface to developer |

---

## 7. The Briefer Should NOT Do

1. **Should not guess implementation details** -- no "probably uses middleware X" or "likely stored in table Y"
2. **Should not invent operations** -- only extract operations that are explicitly or clearly implied by CONTEXT.md endpoint tables, proto RPC lists, or ROADMAP success criteria
3. **Should not recommend validation rules** -- that is the Protester's job during case discussion
4. **Should not assess readiness** -- just present what exists in the documents
5. **Should not speculate about test strategy** -- no code exists to test yet
6. **Should not read code files** -- the redesigned briefer should explicitly NOT be given code paths (it would find nothing useful since code does not exist yet)

---

## 8. Input Contract Changes

### Current Input Contract

```
<files_to_read> contains source code paths: proto/*.proto, services/*/src/, etc.
```

### Revised Input Contract

```xml
<files_to_read>
- .planning/ROADMAP.md (phase description, success criteria, requirement IDs)
- {phase_dir}/*-CONTEXT.md (locked decisions, endpoint tables, proto RPC lists, constraints)
- .planning/REQUIREMENTS.md (requirement descriptions for context)
- .planning/PROJECT.md (architecture patterns, service topology -- reference only)
- {phase_dir}/*-RESEARCH.md (technology patterns -- reference only, if exists)
</files_to_read>
```

The key shift: from source code paths to planning document paths. The briefer's `<files_to_read>` should contain only planning documents, never source code.

### When Code Does Exist (Incremental Phases)

For phases that build on existing code (e.g., Phase 3 adds auth middleware to the existing gateway), there IS relevant existing code. However, the briefer should still prioritize planning documents for operation extraction. Existing code is useful only for:

- Confirming endpoint patterns already established (how routes are registered)
- Understanding existing error handling conventions
- Identifying integration points noted in CONTEXT.md `<code_context>` section

CONTEXT.md already has a `<code_context>` section that summarizes relevant existing code. The briefer should use this summary rather than scanning code directly, reducing the need for code-reading tools.

---

## 9. Operation Extraction Decision Tree

For each potential operation, the briefer should apply this decision tree:

```
1. Is it listed in a CONTEXT.md endpoint table?
   YES -> Include as EXPLICIT operation
   NO  -> Go to 2

2. Is it listed in a CONTEXT.md proto RPC list?
   YES -> Is it also mapped to a REST endpoint elsewhere?
     YES -> Already captured in step 1, note RPC name
     NO  -> Is it called by another service (cross-service)?
       YES -> Include as INTERNAL operation
       NO  -> Is it infrastructure-only (Health, ValidateSession)?
         YES -> Skip (unless phase targets it)
         NO  -> Include as EXPLICIT operation
   NO  -> Go to 3

3. Is it required by a ROADMAP success criterion?
   YES -> Include with PARTIAL confidence, note "specified in ROADMAP
          but not detailed in CONTEXT.md"
   NO  -> Go to 4

4. Is it logically required by other included operations?
   (e.g., "RegisterFinish" implies a user creation side-effect)
   YES -> Note as side-effect of the parent operation, NOT a separate operation
   NO  -> Do not include
```

---

## 10. Quality Gate (Revised)

Before returning, the briefer should verify:

- [ ] All endpoint table rows from CONTEXT.md are captured as operations
- [ ] All proto RPCs from CONTEXT.md are accounted for (either as operations or noted as infrastructure-skip)
- [ ] Each ROADMAP success criterion maps to at least one briefed operation
- [ ] Each operation has interface, auth tier, and at least partial inputs/outputs
- [ ] Decided constraints reference decision IDs (D-XX)
- [ ] Open decisions reference Claude's Discretion items
- [ ] Inferred fields are marked as `[Inferred: ...]`
- [ ] Unknown fields are marked as `[Not specified]`
- [ ] Operations are grouped by natural category
- [ ] Cross-service operations are flagged with orchestration notes
- [ ] No operations were invented beyond what CONTEXT.md describes
- [ ] No implementation recommendations (briefing describes what IS DECIDED, not what SHOULD BE)
- [ ] Extraction confidence table included

---

## 11. Dual-Mode Consideration

The briefer agent definition should support BOTH modes:

| Mode | Trigger | Methodology |
|------|---------|-------------|
| **Text-based** (primary) | No relevant source code exists for the phase | Extract from CONTEXT.md, ROADMAP.md, REQUIREMENTS.md |
| **Code-based** (original) | Source code exists and is the primary artifact | Scan source files for handlers, validators, tests |
| **Hybrid** | Phase modifies existing code AND has new planning decisions | Text-based for new operations, code-based for existing operations being modified |

**Mode detection logic** for the orchestrator (case.md):

The `/case` command (case.md) determines which files to pass in `<files_to_read>`. It should check:
1. Does relevant source code exist for this phase's services? (e.g., does `services/auth/src/` contain more than just a stub?)
2. If YES and substantive code exists -> code-based or hybrid mode
3. If NO or only stubs -> text-based mode

The briefer agent itself does not decide the mode. The orchestrator passes the appropriate file paths and annotates them. The briefer's methodology section should handle both: "If `<files_to_read>` contains planning documents, use text-based extraction. If it contains source code paths, use code-based extraction."

However, for the immediate need (pre-implementation phases), the briefer should be written primarily for text-based extraction, with the code-based methodology preserved as a secondary path.

---

## 12. Impact on case.md Dispatch

### Current Dispatch (Step 1d)

```
<files_to_read>
- proto/*.proto (service definitions)
- services/{relevant_service}/src/ (handlers, domain, adapters)
</files_to_read>
```

### Revised Dispatch for Pre-Implementation Phase

```
<files_to_read>
- .planning/ROADMAP.md -- phase description, success criteria, requirement IDs
- {phase_dir}/*-CONTEXT.md -- locked decisions, endpoint tables, constraints
- .planning/REQUIREMENTS.md -- requirement ID descriptions
- .planning/PROJECT.md -- architecture reference (service topology, patterns)
</files_to_read>
```

The case.md orchestrator needs logic to detect which mode to use. A simple heuristic: check if CONTEXT.md exists AND relevant service code has more than stub files. If CONTEXT.md exists but no substantive code -> text-based. If both exist -> hybrid (pass both planning docs and code paths).

---

## 13. Edge Cases and Limitations

### Edge Case 1: Phase with No CONTEXT.md

If the user runs `/case` before `gsd:discuss-phase`, there is no CONTEXT.md. Only ROADMAP.md and REQUIREMENTS.md exist.

**Handling:** The briefer can still extract operations from ROADMAP.md success criteria, but with very low confidence. Most operations will be PARTIAL or UNKNOWN. The briefing should include a prominent warning: "No CONTEXT.md found. Operations extracted from ROADMAP.md success criteria only. Details are minimal."

This is a degraded mode. The recommended workflow is: discuss-phase -> /case -> plan-phase.

### Edge Case 2: CONTEXT.md Without Endpoint Tables

Phase 1 CONTEXT.md has no endpoint tables -- it describes infrastructure setup. Operations like "Gateway health check" are mentioned in decisions but not tabulated.

**Handling:** The briefer falls back to decision-level scanning: look for decisions that mention HTTP methods, paths, or RPC names in their text. Mark these as INFERRED.

### Edge Case 3: Operations Spanning Multiple Phases

Phase 3's "Deactivate User" operation involves both User service (Phase 2 code) and Auth service (Phase 3 code). The briefer should capture the full flow from CONTEXT.md without requiring Phase 2's code.

**Handling:** CONTEXT.md D-120 explicitly describes the cross-service orchestration. The briefer extracts from this decision text. No code scanning needed.

### Edge Case 4: Overlapping REST Endpoints with Different Tiers

Phase 3 has endpoints that appear at multiple tiers (e.g., `GET /v1/auth/passkeys` appears as both "protected" and "recovery" tier). These are the SAME operation with different access contexts.

**Handling:** Present as a single operation with multiple auth contexts noted:
```
Auth: authenticated (protected tier) OR recovery JWT (restricted access -- D-104)
```

### Edge Case 5: Very Large CONTEXT.md

Phase 3's CONTEXT.md is ~350 lines with 120+ decisions. The briefer must not produce an overwhelming briefing.

**Handling:** The briefer should group aggressively. Operations within the same flow (RegisterBegin + RegisterFinish) should be grouped under "Registration Flow" as a single discussion unit. The Protester discusses them together since they share ceremony state.

### Limitation 1: Cannot Discover Implicit Operations

If a behavior is needed but not mentioned anywhere in planning documents, the text-based briefer will miss it. For example, if the team needs a "List Active Sessions" endpoint but nobody mentioned it in discuss-phase, it will not appear in the briefing.

**Mitigation:** Step 2 of the /case flow ("Which operations? Any I missed?") allows the developer to add operations the briefer did not find.

### Limitation 2: Input/Output Types Are Approximate

Without proto definitions or Rust types, the briefer can only describe fields in natural language. "user_id" is described as "UUID" based on PROJECT.md, not as a concrete Rust or protobuf type.

**Mitigation:** The Protester does not need exact types for case discussion. Cases operate at the behavioral level: "valid user_id", "non-existent user_id", "malformed user_id". The exact type matters for implementation, not for case discovery.

### Limitation 3: Cannot Validate Consistency with Existing Code

If a decision in CONTEXT.md contradicts existing code (e.g., CONTEXT.md says "POST /v1/users" but existing code uses "PUT /v1/users"), the text-based briefer will not catch this.

**Mitigation:** For hybrid mode (existing code + new decisions), the briefer should read the `<code_context>` section of CONTEXT.md, which already summarizes integration points and patterns.

---

## 14. Revised Agent Definition Outline

The agent file `.claude/agents/case-briefer.md` should be restructured:

```
---
name: case-briefer
description: >
  Extracts operations, constraints, and decision context from phase planning
  documents. Produces CASE-BRIEFING.md consumed by /case discussion.
  Supports text-based (planning docs) and code-based (source files) modes.
tools:
  - Read
  - Grep
  - Glob
  - Write
model: sonnet
---

# Case Briefer

[Purpose: extract operations from planning documents or source code]

## Methodology

### Mode Detection
[Check file types in <files_to_read> to determine text-based vs code-based]

### Text-Based Extraction (Primary)
[Steps 1-7 from Section 3 of this research]

### Code-Based Extraction (Legacy)
[Steps 1-6 from current briefer -- preserved for phases with existing code]

## Input Contract
[Same tags, different file paths depending on mode]

## Output Contract
[Revised CASE-BRIEFING.md format from Section 4]

## Quality Gate
[Revised checklist from Section 10]

## Guidelines
[Updated for dual-mode, with emphasis on text-based for new phases]
```

---

## 15. Summary

### What Changes

1. **Primary data source:** Planning documents (CONTEXT.md, ROADMAP.md, REQUIREMENTS.md) instead of source code
2. **Operation discovery:** Parse endpoint tables and proto RPC lists from CONTEXT.md instead of scanning handlers
3. **Constraint sourcing:** Decision IDs (D-XX) replace file:line references
4. **New fields:** "Open decisions" (from Claude's Discretion), "Requirements" (from REQUIREMENTS.md), "Extraction Confidence"
5. **Removed fields:** "Existing tests" (meaningless pre-implementation)
6. **Renamed fields:** "Existing validation" becomes "Decided constraints"
7. **Confidence model:** EXPLICIT/INFERRED/UNKNOWN classification per data point

### What Stays the Same

1. **Output structure:** CASE-BRIEFING.md format with grouped operations
2. **Downstream consumer:** The Protester reads operations with interfaces, inputs, outputs, auth
3. **Grouping logic:** Natural category grouping by domain cluster
4. **Quality gate concept:** Cross-check operations against phase requirements
5. **Return protocol:** `## BRIEFING COMPLETE` with counts

### Implementation Path

1. Rewrite `.claude/agents/case-briefer.md` with text-based methodology as primary, code-based as secondary
2. Update `case.md` Step 1d dispatch to pass planning document paths instead of source code paths
3. Add mode detection logic to case.md: check if substantive code exists before deciding which file paths to pass
4. Test with Phase 3 (Authentication) -- CONTEXT.md exists, no implementation code exists

---

## Confidence Assessment

| Area | Confidence | Reasoning |
|------|-----------|-----------|
| CONTEXT.md structure analysis | HIGH | Read all 3 existing CONTEXT.md files. Clear, consistent patterns observed |
| Operation extraction methodology | HIGH | Endpoint tables and RPC lists are explicit and machine-parseable |
| Input/output inference rules | MEDIUM | Some fields require inference from decision descriptions. Not all decisions are structured consistently |
| Dual-mode design | MEDIUM | Text-based mode is well-defined. Hybrid mode (text + code) needs practical validation |
| Edge case handling | HIGH | All edge cases identified from actual project data |

**Overall confidence: HIGH.** The methodology is grounded in observed document patterns from three existing phases.

---

## Sources

### Primary (HIGH confidence)
- `.claude/agents/case-briefer.md` -- current briefer agent definition (direct reading)
- `.planning/phases/03-authentication/03-CONTEXT.md` -- most detailed CONTEXT.md, source for endpoint table pattern analysis
- `.planning/phases/02-user-profile/02-CONTEXT.md` -- second CONTEXT.md, confirms pattern consistency
- `.planning/phases/01-foundation-and-gateway-infrastructure/01-CONTEXT.md` -- first CONTEXT.md, edge case for no endpoint tables
- `.planning/ROADMAP.md` -- phase description format, success criteria structure
- `.planning/REQUIREMENTS.md` -- requirement ID format and descriptions
- `.planning/research/CASE-SKILL-SYNTHESIS.md` -- overall /case skill design and layers
- `.planning/research/CASE-AGENT-SYNTHESIS.md` -- agent architecture decisions (briefer as subagent)
- `.planning/research/CASE-AGENT-NEEDS.md` -- per-step subagent analysis
- `.claude/commands/case.md` -- current /case skill definition with dispatch patterns

### Secondary (MEDIUM confidence)
- `.planning/research/CASE-CATEGORY-COMPLETENESS.md` -- S/F/E category system validation (relevant for output format)

---

*Research completed: 2026-03-24*
*Valid until: 2026-06-24 (stable domain -- planning document formats are project-internal)*
