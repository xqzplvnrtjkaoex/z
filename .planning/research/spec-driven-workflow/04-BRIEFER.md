# Briefer Intelligence & Dependency Resolution

**Researched:** 2026-03-26
**Domain:** case-briefer agent intelligence, cross-phase dependency discovery, spec awareness
**Confidence:** HIGH (analysis of real artifacts, implemented agent code, 3 completed phases + 1 in-progress)
**Inputs:** case-briefer.md, CASE-BRIEFING.md (3A), CASE-SCRATCH.md (3A), all CONTEXT.md files, ROADMAP.md, PROJECT.md, proto files, existing cross-phase forwarding research (4 documents)

---

## Executive Summary

The case-briefer agent has evolved from a code-scanning tool to a text-based extraction agent over three research iterations. Its current form (post cross-phase forwarding implementation) reads CONTEXT.md, ROADMAP.md, REQUIREMENTS.md, PROJECT.md, and dependency phases' CASES.md. However, several intelligence gaps remain that limit its ability to surface relevant context for the /case conversation.

This research analyzes seven specific questions about briefer intelligence, using concrete evidence from the madome project's 4 phases of artifacts, 3 proto files, and implemented code. The findings reveal that the biggest gap is not in what the briefer reads (it reads most relevant files), but in HOW it connects information across sources. The briefer performs single-pass extraction from each document independently rather than cross-referencing between documents to discover implicit dependencies.

**Primary recommendation:** Implement a layered intelligence model -- the briefer performs three passes: (1) document extraction (current behavior), (2) cross-reference resolution (new: connecting operation names, service names, and entity names across documents), and (3) PROJECT.md constraint injection (new: surfacing system-wide rules and architectural decisions relevant to the phase). Quick wins first: add PROJECT.md reading + operation name cross-referencing. Defer proto/code awareness and spec folder until the spec-driven workflow design is finalized.

---

## 1. Current State Analysis

### What the Briefer Currently Does

The case-briefer (`.claude/agents/case-briefer.md`) is a Sonnet subagent with 7 steps:

| Step | Action | Source | Quality |
|------|--------|--------|---------|
| 1 | Understand phase scope | CONTEXT.md, ROADMAP.md | Good |
| 2 | Discover operations | CONTEXT.md endpoint/RPC tables | Good |
| 3 | Extract inputs/outputs | CONTEXT.md decision details | Good |
| 4 | Extract decided constraints | CONTEXT.md decisions | Good |
| 4.5 | Classify cross-cutting constraints | All decisions, PROJECT.md SR section | Good |
| 4.7 | Scan dependency CASES.md | Dep phase CASES.md via ROADMAP `Depends on` | New, untested |
| 5-7 | Map requirements, identify open decisions, group and write | ROADMAP, CONTEXT.md, REQUIREMENTS.md | Good |

### What the Briefer Currently Reads

| Document | Reads? | What It Extracts |
|----------|--------|------------------|
| Current phase CONTEXT.md | YES | Operations, constraints, decisions |
| ROADMAP.md | YES | Phase description, success criteria, dependencies |
| REQUIREMENTS.md | YES | REQ-ID descriptions for mapping |
| PROJECT.md | YES (since Step 4.5) | System-Wide Rules section only |
| Dependency CASES.md | YES (since Step 4.7) | Open Questions, Forward tags, Phase Rules |
| Dependency CONTEXT.md | NO | -- |
| Proto files | NO | -- |
| Implementation code | NO (by design) | -- |
| Existing gateway routes | NO | -- |

### Confirmed Gaps

**Gap 1: PROJECT.md is underutilized.** The briefer reads PROJECT.md but only for the System-Wide Rules section (Step 4.5). It does NOT extract:
- Service topology (which services exist, their relationships)
- Auth design (JWT flow, session model, renewal patterns)
- Cross-service communication patterns (compensating transactions)
- Domain decisions (ID design, pagination conventions, API conventions)
- Internal service architecture (4-layer pattern, ports/config separation)

**Evidence:** The 3A CASE-BRIEFING.md contains no information from PROJECT.md's Auth Design section despite it being directly relevant to VerifyJwt, ValidateSession, and RefreshToken operations. The briefer inferred these from CONTEXT.md decisions alone.

**Gap 2: No cross-reference between CONTEXT.md operation mentions and existing specs/artifacts.** When 3A CONTEXT.md mentions "User.CreateUser" (D-118), the briefer does not look up Phase 2's CASES.md or the user.proto to understand what CreateUser's interface, constraints, and failure modes actually are.

**Evidence:** The 3A CASE-BRIEFING.md lists "Cross-service: Auth -> User service CreateUser RPC" but does not include CreateUser's input constraints (handle 4-15 chars), its error responses, or its existing proto signature. These were manually discovered during the /case discussion (Q1 about handle length was left as an Open Question until re-validation found it was already decided in user service).

**Gap 3: Implicit dependencies beyond ROADMAP `Depends on` are invisible.** ROADMAP.md says Phase 3A depends on Phase 1 and Phase 2. But Phase 4 (Catalog Core) depends on Phase 1 only, despite Phase 4's operations potentially needing auth middleware (Phase 3A) for the publish workflow. The ROADMAP dependency graph reflects build-order dependencies, not behavioral dependencies.

**Evidence:** Phase 6 (User Preferences) "Depends on: Phase 3, Phase 4" -- it depends on auth (Phase 3) for access control and catalog (Phase 4) for book entities. But Phase 5 (Catalog Queries) "Depends on: Phase 4" -- no auth dependency listed, even though all endpoints require authentication (PROJECT.md Authentication Policy: "all endpoints require authentication").

**Gap 4: No awareness of already-implemented decisions.** The Q1 handle length Open Question in 3A existed because the briefer had no way to know that `handle: 4-15 chars` was already validated in the user service implementation. This was resolved during re-validation by manually checking user service code -- not by the briefer.

---

## 2. PROJECT.md as Input: Extraction Strategy

### What PROJECT.md Contains That the Briefer Should Use

PROJECT.md is 493 lines with 9 major sections. The briefer should extract context from these categories:

| Section | Relevance to Briefer | How to Use |
|---------|---------------------|------------|
| Service Topology | Which services exist, their communication patterns | Identify cross-service operations; know which service handles what |
| Internal Service Architecture | 4-layer pattern, ports, error separation | Context for the Protester about implementation patterns (informational) |
| Communication Flow | Client -> Gateway -> gRPC chain | Identify where Gateway orchestration is needed |
| Authentication Policy | "All endpoints require authentication" | Flag if current phase introduces endpoints that need auth |
| Role Hierarchy | owner > admin > user, management rules | Extract role-based constraints for operations with role checks |
| Cross-Service Operations | Compensating transactions pattern | Flag multi-service operations needing compensation |
| Pagination | Cursor-based, X-Next-Cursor header | Apply to any list operation in the current phase |
| API Conventions | kebab-case query, snake_case body | Apply to all REST endpoints |
| System-Wide Rules (SR) | SR-01 through SR-04 | Already extracted in Step 4.5 |
| Key Decisions | Validated/pending decisions | Context for what has been confirmed vs assumed |

### Proposed Extraction Strategy

The briefer should NOT dump all of PROJECT.md into the briefing -- that would be noise. Instead, it should perform **selective extraction based on phase relevance**:

1. **Service identification:** From CONTEXT.md, identify which services this phase touches (e.g., 3A touches auth, user, gateway). From PROJECT.md Service Topology, extract the communication patterns for those services.

2. **Architectural constraints:** If the phase introduces a new service (e.g., auth in 3A), extract the Internal Service Architecture pattern. If the phase extends an existing service (e.g., gateway in 3A), note the established pattern (gateway exception: routes/ + middleware/ + state.rs).

3. **Policy constraints:** For any phase introducing REST endpoints, extract Authentication Policy and API Conventions. For any phase with list operations, extract Pagination conventions.

4. **Cross-service patterns:** If CONTEXT.md mentions multi-service operations, extract the Cross-Service Operations section (compensating transactions level).

5. **Domain decisions:** If the phase touches entities mentioned in Key Decisions, note which decisions are Validated vs Pending.

### Output Format for PROJECT.md Context

Add a new section between "Extraction Confidence" and "Cross-Cutting Constraints":

```markdown
## Architectural Context (from PROJECT.md)

### Services Involved
- **auth** (new in this phase): 4-layer architecture, tonic gRPC
- **user** (dependency): gRPC calls from auth for user creation/lookup
- **gateway** (extended): JWT middleware addition

### Relevant Policies
- Authentication: all endpoints require authentication (except public routes)
- Pagination: cursor-based with X-Next-Cursor header
- API: kebab-case query params, snake_case body fields

### Cross-Service Patterns
- Auth -> User: direct gRPC, 5s timeout (SR-01)
- Gateway -> Auth: JWT validation, session verification
- Compensation: Gateway orchestrates sequential calls with compensate-on-failure (Level 1)
```

### Risk: Context Overload

PROJECT.md is dense. The briefer (Sonnet) may spend too much context on architectural extraction at the expense of operation discovery quality.

**Mitigation:** Keep the architectural context section to ~20 lines. The briefer should extract, not summarize. Use bullet points, not paragraphs.

---

## 3. Cross-Reference Heuristics for Dependency Discovery

### The Problem

CONTEXT.md for Phase 3A contains textual references to operations from other phases/services:

- "D-118: Registration orchestration: Gateway calls User.CreateUser" -- references Phase 2
- "D-134: Login flow ordering: User.GetUser" -- references Phase 2
- "D-120: Deactivation: Gateway -> User(DeactivateUser) -> Auth(InvalidateAllSessions)" -- references Phase 2 + future auth
- "D-139: compensating User.DeleteUser" -- references Phase 2

These are NOT captured by ROADMAP `Depends on` scanning -- the briefer already knows about the dependency. The gap is that it does not USE the dependency to enrich the briefing with what those operations actually do.

### Proposed Cross-Reference Strategies

#### Strategy A: Operation Name Matching (Recommended -- Quick Win)

Scan current phase CONTEXT.md for operation name patterns:
- `{Service}.{Operation}` pattern (e.g., "User.CreateUser", "Auth.ValidateSession")
- Service RPC references (e.g., "calls User service", "gRPC to auth")

For each matched operation:
1. Check if a CASES.md exists for the phase that implements that operation
2. If found, extract that operation's Rules and failure modes
3. Include as "Referenced Operations" in the briefing

**Implementation:**
```
# Regex patterns to scan CONTEXT.md
User\.\w+           -> look in Phase 2 CASES.md
Auth\.\w+           -> look in Phase 3A CASES.md
Catalog\.\w+        -> look in Phase 4 CASES.md
```

**What this catches for 3A:**
- "User.CreateUser" -> finds Phase 2 CASES.md CreateUser operation -> extracts handle validation rules (4-15 chars, case-insensitive), existing error responses (409 Conflict for duplicate handle)
- "User.GetUser" -> finds Phase 2 CASES.md GetUser operation -> extracts deactivated user handling
- "User.DeleteUser" -> finds Phase 2 CASES.md (if it had been discussed) or proto definition

**Concrete example of the gap this closes:**

The 3A CASE-BRIEFING.md says:
```
- **Cross-service:** Auth -> User service CreateUser RPC (D-118)
```

With operation name matching, it would say:
```
- **Cross-service:** Auth -> User service CreateUser RPC (D-118)
  - Referenced operation: User.CreateUser (Phase 2)
    - Input constraints: handle (4-15 chars, alphanumeric+underscore, case-insensitive unique), name (1-20 chars), role (enum)
    - Failure modes: 409 handle_taken, 400 validation_error, 500 infra
    - Proto: CreateUserRequest { handle: String, name: String, role: Role }
```

This would have prevented the Q1 "handle length" Open Question -- the answer was already known.

#### Strategy B: Service Name Affinity

When CONTEXT.md mentions a service by name (e.g., "user service", "auth service"), scan all phases that touch that service for relevant specifications.

**How to determine which phases touch which service:**
1. ROADMAP phase descriptions mention service names
2. CONTEXT.md decision blocks mention services
3. Proto file names correspond to services (user.proto, auth.proto, catalog.proto)

This is broader than Strategy A but catches cases where no specific operation is named -- just "interacts with user service."

**Risk:** Too broad. If 3B mentions "auth service," it would scan ALL of 3A's cases, which is 9 operations. Most are irrelevant to the specific interaction.

**Recommendation:** Use as a secondary heuristic when Strategy A finds no specific operation names.

#### Strategy C: Proto Import Graph

Proto files declare service contracts explicitly. If `auth.proto` imports types from `user.proto` (or if auth service calls user service RPCs), there is a structural dependency.

**Current state in madome:**
- `auth.proto` currently has only Health RPC (stub). After 3A implementation, it will have 10+ RPCs.
- `user.proto` has 8 RPCs fully defined.
- No cross-file imports exist (each proto is self-contained).

**The proto dependency is captured differently:** auth service code imports `madome_proto::user` to call user service RPCs. This is a code-level dependency, not a proto-level dependency.

**Recommendation:** Defer. Proto files in this project do not have cross-file imports. The dependency is in code, not proto definitions. And code-awareness is deferred (see Section 7).

#### Strategy D: Shared Entity Matching

Scan CONTEXT.md and CASES.md for entity names that appear across phases:
- "user" entity -> Phase 2 (creation), Phase 3A (auth), Phase 6 (preferences)
- "book" entity -> Phase 4 (CRUD), Phase 5 (queries), Phase 6 (preferences)
- "session" entity -> Phase 3A (creation), Phase 3B (management)

**How it works:** Build an entity-to-phase index from ROADMAP descriptions and CONTEXT.md mentions. When briefing Phase 6, the entity index shows that "user" was defined in Phase 2 and "book" in Phase 4, surfacing those phases as relevant sources.

**Risk:** Too broad without operation-level specificity. Knowing that "user" is relevant to Phase 6 does not help unless the briefer knows WHICH user operations matter.

**Recommendation:** Use as a discovery heuristic to FIND relevant phases, then apply Strategy A within those phases to extract specific operations.

### Recommended Implementation Priority

1. **Strategy A (Operation Name Matching):** Implement first. Direct, specific, low false-positive rate. Closes the most impactful gap (the Q1 handle length problem).
2. **Strategy D (Shared Entity Matching):** Implement second as a discovery layer that feeds into Strategy A.
3. **Strategy B (Service Name Affinity):** Fallback for when A and D find nothing.
4. **Strategy C (Proto Import Graph):** Defer. Not applicable to current project structure.

---

## 4. Alternative Dependency Discovery Strategies

### Beyond ROADMAP `Depends on`

The ROADMAP `Depends on` field captures build-order dependencies, not behavioral dependencies. Other signals exist:

| Signal | What It Reveals | How to Detect | Reliability |
|--------|----------------|---------------|-------------|
| Shared domain entities | Phases touching same data | Entity names in CONTEXT.md and CASES.md | HIGH -- entities are explicit |
| Service overlap | Two phases modify same service | Service names in ROADMAP descriptions | HIGH -- service names are explicit |
| Error/type references | Phase B uses Phase A's error types | Not detectable pre-implementation | LOW -- requires code |
| Temporal ordering | All completed phases are potential dependencies | Phase completion status in ROADMAP | MEDIUM -- overly broad |
| Cross-service call references | CONTEXT.md mentions calling another service's RPCs | `{Service}.{Operation}` pattern in decisions | HIGH -- specific and explicit |
| Authentication requirement | Any phase with REST endpoints depends on auth phase | PROJECT.md Authentication Policy + endpoint tables | HIGH -- universal rule |

### How Other Systems Solve This

**Package managers** (npm, cargo): Explicit declaration in a manifest file (Cargo.toml `[dependencies]`). No heuristic discovery -- if you use it, you declare it. Analogous to ROADMAP `Depends on`.

**Build systems** (Bazel, Make): Dependency graph from file-level declarations (`deps = ["//proto:user"]`). Bazel uses transitive closure. Analogous to proto import graph (Strategy C), but requires explicit declarations.

**Knowledge graphs / requirements traceability** (recent research): Graph-RAG approaches construct graphs from requirement documents, extracting entities and relationships automatically. This is the most analogous to what the briefer needs -- but it requires an explicit graph construction step that does not currently exist in the GSD pipeline.

**IDE "Find References"**: Follows symbol references across files. Code-level only. Not applicable pre-implementation.

### Recommendation for madome

The project has a small number of phases (7) and services (6). The dependency graph is small enough that heuristic discovery is feasible without building an explicit graph structure.

**Layered approach:**
1. ROADMAP `Depends on` (existing) -- captures explicit build-order deps
2. Cross-service call references in CONTEXT.md (Strategy A) -- captures behavioral deps
3. Shared entity matching (Strategy D) -- catches broader associations
4. Authentication Policy rule (auto-inject auth dependency for any phase with REST endpoints)

The authentication policy rule is the lowest-hanging fruit: PROJECT.md says "all endpoints require authentication." Any phase introducing REST endpoints has an implicit dependency on Phase 3A. The briefer should auto-detect this.

---

## 5. Spec Folder Integration

### Current State

No `.planning/specs/` directory exists. CASES.md files live in phase directories (`.planning/phases/{phase}/`). Only one CASES.md has been finalized so far (3A's is still CASE-SCRATCH.md).

### Proposed Spec Folder Structure (from memory entry)

```
.planning/specs/
  INDEX.md          -- service-to-phase mapping, operation index
  auth/
    3a-cases.md     -- symlink or copy of 3A CASES.md
    3b-cases.md     -- symlink or copy of 3B CASES.md
  user/
    2-cases.md      -- symlink or copy of Phase 2 CASES.md
  catalog/
    4-cases.md
    5-cases.md
  gateway/
    1-cases.md
    3a-cases.md     -- gateway-relevant parts of 3A
```

### How the Briefer Would Use It

**INDEX.md grepping:** If the briefer needs to find specs for "CreateUser," it greps INDEX.md for the operation name and gets back `user/2-cases.md`. This is faster and more reliable than scanning all phase directories.

**Vs phase directory scanning:** The current approach scans dependency phase directories. This works but requires knowing which phases to scan (from ROADMAP). The INDEX.md approach is operation-centric rather than phase-centric -- the briefer asks "where is CreateUser specified?" instead of "what does Phase 2 specify?"

### Could the Briefer Read Specs INSTEAD of Phase Directories?

**Yes, eventually.** The spec folder consolidates per-service behavioral specifications. The briefer would read `specs/auth/` instead of scanning `.planning/phases/03a-*/` and `.planning/phases/03b-*/`.

**Benefits:**
- Service-centric view (all auth specs in one place)
- No need to resolve ROADMAP dependencies -- just grep the operation name
- INDEX.md provides a lookup table

**Drawbacks:**
- Another artifact to maintain (spec folder must be populated from phase CASES.md)
- Dual source of truth risk (phase CASES.md vs spec CASES.md)
- Migration cost (existing phases need their specs reorganized)

### Migration Path: Support Both During Transition

1. **Phase 1 (now):** Briefer reads phase directories (current behavior). No spec folder.
2. **Phase 2 (after spec folder design):** Briefer checks for INDEX.md in `.planning/specs/`. If found, use it. If not, fall back to phase directory scanning.
3. **Phase 3 (after all phases migrated):** Briefer reads spec folder exclusively.

The briefer should be designed with this migration in mind. The cross-reference heuristics (Section 3) work with both approaches: Strategy A grepping for `User.CreateUser` works whether the result is in `phases/02-*/02-CASES.md` or `specs/user/2-cases.md`.

### Recommendation

**Defer spec folder creation.** The spec-driven workflow idea needs its own research and design session before committing to a folder structure. The briefer improvements in this document (PROJECT.md reading, cross-referencing, dependency discovery) are independent of the spec folder and should be implemented first. They will work equally well with or without a spec folder.

---

## 6. False Positive/Negative Balance

### The Spectrum

```
Too few concerns (misses critical constraints)  <--->  Too many (noise, context overload)
```

The briefer's current behavior is biased toward the "too few" end: it extracts what is explicitly in CONTEXT.md and nothing more. The cross-phase forwarding research added dependency CASES.md scanning, but the operation cross-referencing gap (Section 3) means behavioral dependencies from other phases are still invisible.

### Filtering and Ranking Strategies

**Strategy 1: Relevance tiering** (Recommended)

Classify each piece of inherited/cross-referenced information as:

| Tier | Criteria | Presentation |
|------|----------|-------------|
| **Definite** | Explicitly mentioned in current CONTEXT.md decisions | Inline in per-operation briefing |
| **Likely relevant** | Operation referenced by name in current CONTEXT.md | "Referenced Operations" subsection |
| **Maybe relevant** | Same service or entity overlap | "Architectural Context" section (informational) |
| **Background** | System-wide rules, pagination conventions | "Policies" subsection (applied automatically) |

The Protester sees all tiers but knows which are high-confidence vs contextual.

**Strategy 2: Operation-scoped vs phase-scoped**

Some concerns are operation-specific (CreateUser's handle validation applies to SignupBegin). Others are phase-scoped (all auth endpoints return generic errors). The briefer should present operation-scoped concerns in the per-operation sections and phase-scoped concerns in Cross-Cutting Constraints.

**Strategy 3: Present "maybe relevant" items separately** (Recommended)

Add a section `## Additional Context` that separates items the briefer is less confident about:

```markdown
## Additional Context (may be relevant)

These items were discovered through cross-referencing but may not directly affect
this phase's operations. Review for relevance.

- User service handle validation: 4-15 chars, URL-safe (from Phase 2 CASES.md)
  -> Relevant if SignupBegin validates handle before User.CreateUser
- Pagination conventions from PROJECT.md
  -> Relevant if any list operations are introduced
```

This preserves the signal while separating it from the high-confidence operation extraction.

### Quantitative Guidance

Based on the 3A briefing (the only real example):

- **10 operations** briefed: appropriate level of detail
- **~25 lines per operation**: manageable for the Protester
- **1 Observations section**: captures cross-cutting patterns

With the proposed additions:
- **+1 Architectural Context section (~15 lines)**: acceptable
- **+1 Referenced Operations subsection per relevant operation (~5 lines each, ~3 operations)**: acceptable
- **+1 Inherited Concerns section (~10 lines)**: already implemented

Total briefing size increase: ~40 lines on a ~300-line briefing. This is within acceptable bounds.

### Risk: Sonnet Context Budget

The briefer is Sonnet. Adding more documents to read (PROJECT.md fully, dependency CASES.md, potentially dependency CONTEXT.md) increases input context. Current estimated input:
- CONTEXT.md: ~300 lines (3A)
- ROADMAP.md: ~160 lines
- REQUIREMENTS.md: ~140 lines
- PROJECT.md: ~490 lines
- Dependency CASES.md: ~470 lines (3A CASE-SCRATCH.md)

Total: ~1560 lines. Well within Sonnet's capacity but approaching the point where extraction quality may degrade.

**Mitigation:** The briefer should NOT read all of PROJECT.md and all dependency CASES.md in full. It should use targeted reads:
- PROJECT.md: read Service Topology, Auth Design, System-Wide Rules, and Key Decisions sections
- Dependency CASES.md: scan for operations referenced in current CONTEXT.md, extract only those operations' Rules and failure modes

---

## 7. Spec-Only vs Code-Aware

### The Fundamental Question

Should the briefer read implementation code or proto files to detect already-implemented decisions?

### Arguments For Code Awareness

1. **Catches already-decided constraints:** The handle 4-15 validation was implemented in user service before 3A's /case ran. Code awareness would have found it.
2. **Detects implementation patterns:** Existing gateway middleware patterns inform how new middleware should be structured.
3. **Proto files as contracts:** Proto definitions ARE specifications. Reading `user.proto` reveals the exact interface of User.CreateUser without needing to find Phase 2's CASES.md.
4. **Prevents redundant Open Questions:** If a constraint is already in code, the briefer can mark it as RESOLVED rather than leaving it as an open question.

### Arguments Against Code Awareness

1. **Breaks spec-first principle:** WORKFLOW.md says "No implementation code, proto files, or tests exist until gsd:execute. All preceding steps must work from planning documents only." Code awareness contradicts this for later phases where earlier phases' code exists.
2. **Code may not match spec intent:** Implementation details may diverge from the behavioral intent. The briefer should present the INTENDED behavior (from specs), not the IMPLEMENTED behavior (from code).
3. **Complexity:** Code parsing adds failure modes. Proto files are simple, but Rust source code is complex to extract constraints from.
4. **Context cost:** Reading code files adds significant context for marginal benefit.

### The Middle Ground: Proto Files Only

Proto files are contracts, not implementation. They define the interface between services. Reading proto files is closer to reading a specification than reading code.

**What proto awareness would provide for 3A:**

```
# From user.proto:
CreateUserRequest { handle: String, name: String, role: Role }
-> Tells the briefer: SignupBegin will need to construct a CreateUserRequest
   with exactly these fields.

# From user.proto:
UserResponse { id: bytes, handle: String, name: String, role: Role,
               is_active: bool, created_at: Timestamp, updated_at: Timestamp }
-> Tells the briefer: GetCurrentUser response fields are already defined.
```

**What proto awareness would NOT provide:**
- Handle validation rules (4-15 chars) -- these are in usecase code, not proto
- Error handling patterns -- proto defines success types, not error semantics
- Business rules -- these live in domain/usecase layers

**Conclusion:** Proto awareness provides interface-level information but NOT behavioral constraints. The handle length gap would NOT be closed by reading proto files alone.

### Recommendation

**Defer code awareness entirely.** The cross-reference heuristics (Section 3) solve the same problem through spec-to-spec references: if Phase 2's CASES.md documents handle validation as 4-15 chars, and the briefer cross-references "User.CreateUser" to find Phase 2's CASES.md, the constraint surfaces through specs, not code.

Proto awareness is a marginal improvement over spec-based cross-referencing. It adds interface types but not behavioral constraints. The cost (new file reading, parsing, context budget) outweighs the benefit.

**When to reconsider:** If the spec folder is implemented and maintained, proto awareness becomes redundant (specs capture everything proto does, plus behavior). If specs are NOT maintained and drift from code, then proto/code awareness becomes more valuable as a validation layer.

---

## 8. Proposed Briefer Architecture Improvements

### Priority 1: Quick Wins (Implement Now)

#### 1A: Enhanced PROJECT.md Extraction

**Current:** Reads PROJECT.md only for System-Wide Rules section.
**Proposed:** Extract service topology, auth policy, cross-service patterns, and API conventions relevant to the phase.

**Changes to case-briefer.md:**
- Step 1 extension: After understanding phase scope from CONTEXT.md, extract relevant architectural context from PROJECT.md
- New output section: `## Architectural Context` (between Extraction Confidence and Cross-Cutting Constraints)

**Effort:** ~15 lines added to briefer agent definition.
**Risk:** LOW -- PROJECT.md is already in `<files_to_read>`, just underutilized.

#### 1B: Operation Name Cross-Referencing (Strategy A)

**Current:** Briefer extracts cross-service call mentions but does not look up the referenced operation.
**Proposed:** When CONTEXT.md mentions `{Service}.{Operation}`, find that operation's spec in dependency phases and extract its constraints and failure modes.

**Changes to case-briefer.md:**
- New sub-step in Step 4 (after extracting decided constraints): scan for `{Service}.{Operation}` patterns in constraints, then look up referenced operations in dependency CASES.md
- Per-operation output addition: `Referenced Operations` subsection listing constraints and failure modes from the source spec

**Effort:** ~25 lines added to briefer agent definition.
**Risk:** MEDIUM -- depends on Sonnet's ability to parse operation names from CONTEXT.md decision text and match them to CASES.md operation headers. The patterns are consistent in this project (`User.CreateUser`, `Auth.ValidateSession`) but may vary in other projects.

### Priority 2: Medium-Term (After First Real Test)

#### 2A: Auto-Detect Authentication Dependency

**Current:** Briefer relies on ROADMAP `Depends on` for dependency phases.
**Proposed:** If current phase introduces REST endpoints and auth phase exists (completed), auto-include auth phase as a behavioral dependency even if not listed in ROADMAP.

**Logic:**
```
if phase has REST endpoints (CONTEXT.md endpoint tables exist)
  AND PROJECT.md says "all endpoints require authentication"
  AND auth phase is complete
THEN auto-include auth CASES.md as dependency source
```

**Effort:** ~10 lines.
**Risk:** LOW -- the rule is explicit in PROJECT.md.

#### 2B: Entity-Based Phase Discovery (Strategy D)

**Current:** Only ROADMAP direct dependencies are scanned.
**Proposed:** Build a lightweight entity-to-phase index from ROADMAP descriptions and scan phases that share entities with the current phase.

**Effort:** ~20 lines.
**Risk:** MEDIUM -- may produce false positives for phases that mention the same entity in different contexts.

### Priority 3: Deferred (Until Spec-Driven Workflow Design)

#### 3A: Spec Folder Integration

Wait for the spec-driven workflow design to finalize the folder structure. Then update the briefer to check for INDEX.md before falling back to phase directory scanning.

#### 3B: Proto/Code Awareness

Defer until either (a) spec folder is not adopted and specs drift from code, or (b) a specific gap is found that only proto awareness can close.

#### 3C: Dependency CONTEXT.md Reading

Scan dependency phases' CONTEXT.md for deferred items targeting the current phase. Currently, only CASES.md is scanned. CONTEXT.md `<deferred>` sections contain feature-level deferrals that may be relevant.

**Effort:** ~15 lines.
**Risk:** LOW -- CONTEXT.md deferred sections are small and well-structured.
**Why deferred:** The discuss-phase CLAUDE.md instruction (from cross-phase forwarding research) already handles design-level deferred concerns. Adding CONTEXT.md scanning to the briefer would create duplication. Wait to see if the discuss-phase path is sufficient.

---

## 9. Risk Analysis

| Improvement | Risk | Severity | Mitigation |
|-------------|------|----------|------------|
| PROJECT.md extraction | Over-extraction bloats briefing | LOW | Cap at ~20 lines; extract only sections matching phase services |
| Operation cross-referencing | Wrong operation matched | MEDIUM | Require exact `{Service}.{Operation}` pattern; flag fuzzy matches as "maybe relevant" |
| Auto-auth dependency | Phase introduces endpoints before auth is implemented | LOW | Check auth phase completion status before auto-including |
| Entity-based discovery | False positives from broad entity matching | MEDIUM | Use as discovery layer only; require Strategy A confirmation before including details |
| Context budget overflow | Too many documents degrade Sonnet extraction quality | MEDIUM | Use targeted reads (specific sections, not full files); monitor briefing quality |
| Cross-referencing accuracy | Sonnet misidentifies operation references | MEDIUM | Use structured patterns only; avoid natural-language inference for operation matching |

---

## 10. Recommendation: Minimal Viable Improvement vs Full Redesign

### Minimal Viable Improvement (Implement Before 3B's /case)

1. **1A: Enhanced PROJECT.md extraction** -- read service topology, auth design, API conventions
2. **1B: Operation name cross-referencing** -- when CONTEXT.md references `User.CreateUser`, look up Phase 2's CASES.md

These two changes address the two most impactful gaps discovered in the 3A experience:
- PROJECT.md underutilization (architectural context missing)
- Q1 handle length (already-decided constraint not surfaced)

Total effort: ~40 lines added to case-briefer.md. No new files. No changes to other agents/skills.

### Full Redesign (Future -- After Spec Folder Decision)

1. All Priority 1 and 2 items
2. Spec folder integration with INDEX.md-based lookup
3. Three-pass intelligence model (extraction -> cross-reference -> constraint injection)
4. Optional proto awareness as a validation layer

The full redesign depends on the spec-driven workflow design. Implementing it now would be premature -- the spec folder structure is not decided, and the cross-referencing heuristics need validation on a real /case session first.

### Recommendation

**Implement the Minimal Viable Improvement now.** Validate during 3B's /case session. If the operation cross-referencing catches the type of gap that Q1 represented, expand to Priority 2 items. If not, revisit the approach.

---

## Confidence Assessment

| Area | Level | Reason |
|------|-------|--------|
| Current state analysis | HIGH | Direct reading of agent code, real briefing output, confirmed gaps |
| PROJECT.md extraction strategy | HIGH | Concrete content analysis of PROJECT.md sections, relevance mapping |
| Operation cross-referencing (Strategy A) | HIGH | Concrete evidence from 3A Q1 gap; clear regex patterns from CONTEXT.md |
| Alternative dependency discovery | MEDIUM | Strategies proposed but untested; evidence from 3A is limited to one phase |
| Spec folder integration | MEDIUM | Design depends on unfinished spec-driven workflow research |
| Proto/code awareness recommendation | HIGH | Clear reasoning; spec-based approach covers the same gap without code complexity |
| False positive/negative balance | MEDIUM | Quantitative guidance based on one real example (3A briefing) |
| Risk analysis | HIGH | Risks identified from concrete agent behavior and context constraints |

**Overall: HIGH** for the current state analysis and quick wins. MEDIUM for the longer-term improvements that depend on untested heuristics and unfinished design decisions.

---

## Open Questions

### OQ-1: Briefer Dispatch -- Should step-init.md Resolve Dependencies?

Currently, step-init.md passes fixed file paths to the briefer. The cross-phase forwarding research added dependency CASES.md paths. Should step-init.md also resolve operation cross-references and pass the relevant files, or should the briefer discover them itself?

**Recommendation:** Let the briefer discover them. The briefer already reads ROADMAP.md and can resolve dependencies. Adding cross-reference resolution to step-init.md moves intelligence out of the agent and into the orchestrator, which contradicts the design principle of step-init being a thin dispatcher.

### OQ-2: How Does This Interact with the Forward Concerns System?

The cross-phase forwarding system (Step 4.7) scans dependency CASES.md for Forward tags and Phase Rules. The operation cross-referencing (Section 3, Strategy A) also reads dependency CASES.md but for a different purpose -- extracting behavioral constraints of referenced operations. These are complementary:
- Step 4.7: "What concerns did Phase A flag for Phase B?" (Forward direction)
- Strategy A: "What constraints does this referenced operation have?" (Reference direction)

Both read the same files. The briefer should perform both in a single pass to avoid redundant reads.

### OQ-3: Entity Index vs Operation Index

Strategy D proposes entity-based matching. The spec folder INDEX.md would provide operation-based matching. These are different granularities:
- Entity index: "user" -> Phase 2, Phase 3A, Phase 6
- Operation index: "User.CreateUser" -> Phase 2 CreateUser section

Both are useful but the operation index is more actionable. The entity index is a discovery heuristic that feeds into the operation index. If the spec folder provides an operation index, the entity index becomes less important.

**Recommendation:** If spec folder is adopted, use its INDEX.md as the operation index. If not, implement a lightweight entity-to-phase mapping from ROADMAP descriptions.

---

## Sources

### Primary (HIGH confidence)
- `.claude/agents/case-briefer.md` -- current briefer agent definition (direct reading, 247 lines)
- `.planning/phases/03a-authentication-core/CASE-BRIEFING.md` -- real briefer output (305 lines)
- `.planning/phases/03a-authentication-core/CASE-SCRATCH.md` -- real /case session data (471 lines)
- `.planning/phases/03a-authentication-core/03a-CONTEXT.md` -- real CONTEXT.md with cross-service references
- `.planning/PROJECT.md` -- architectural context (493 lines)
- `.planning/ROADMAP.md` -- phase dependency structure
- `proto/user.proto` -- real proto definition (79 lines)
- `proto/auth.proto` -- stub proto (8 lines, pre-3A implementation)
- `.planning/research/CASE-BRIEFER-REDESIGN.md` -- prior briefer redesign research
- `.planning/research/CROSS-PHASE-FORWARDING-MECHANISM.md` -- forwarding mechanism design
- `.planning/research/CROSS-PHASE-CONCERN-CLASSIFICATION.md` -- concern taxonomy
- `.planning/research/CROSS-PHASE-GSD-INTEGRATION.md` -- GSD integration points
- `.planning/research/CROSS-PHASE-FORWARDING-SUMMARY.md` -- forwarding synthesis

### Secondary (HIGH confidence)
- `.claude/skills/case/step-init.md` -- briefer dispatch mechanism
- `.claude/skills/case/step-finalize.md` -- forward concerns output format
- `.planning/WORKFLOW.md` -- artifact availability constraints
- Memory: `project_spec_driven_workflow.md` -- spec folder idea
- Memory: `project_unified_spec_idea.md` -- living spec concept

### Tertiary (MEDIUM confidence)
- [DependEval: Benchmarking LLMs for Repository Dependencies](https://aclanthology.org/2025.findings-acl.373.pdf) -- LLM dependency recognition patterns
- [Leveraging Graph-RAG for Requirements Traceability](https://arxiv.org/html/2412.08593v1) -- knowledge graph approach to spec cross-referencing
- [Application Dependency Mapping Overview](https://www.sciencedirect.com/topics/computer-science/application-discovery-and-dependency-mapping) -- general dependency discovery techniques

---

*Research completed: 2026-03-26*
*Valid until: 2026-06-26 (methodology-focused, stable domain -- project workflow internals)*
