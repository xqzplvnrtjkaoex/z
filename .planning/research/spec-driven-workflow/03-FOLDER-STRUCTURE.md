# Service-Organized Spec Folder Structure - Research

**Researched:** 2026-03-26
**Domain:** Planning infrastructure, specification organization, living documentation
**Confidence:** HIGH (project-internal analysis, domain patterns well-established)

## Executive Summary

The madome project currently stores all phase artifacts (CONTEXT.md, CASES.md, PLAN.md, etc.) in phase-numbered directories under `.planning/phases/`. This organization serves the build pipeline well -- each step knows where to find the artifacts it needs. But after a phase is shipped, these specs become difficult to discover across service boundaries. The case-briefer must trace ROADMAP dependencies to find relevant specs, missing implicit cross-service connections. A later phase working on the user service cannot easily answer "what behavioral contracts exist for user operations?" without scanning multiple phase directories.

The core tension is **temporal context vs. domain discovery**. Phase directories preserve when and why decisions were made. Service directories enable what questions -- what does this service do, what are its error cases, what rules govern it. The research below evaluates three candidate structures that resolve this tension differently, and recommends a **dual-view approach**: phase directories remain the authoritative source (preserving GSD workflow compatibility), while a service-organized `specs/` tree provides curated, maintained views with cross-references back to source phases.

**Primary recommendation:** Implement Candidate B (Service-organized views with phase backlinks) with automated extraction at `gsd:ship` time, triggered by a project-local post-ship hook. Keep phase directories intact as historical record. The `specs/` tree becomes the primary lookup surface for case-briefer and cross-phase tools.

## Findings Per Question

### Q1: Phase-Organized vs. Service-Organized

**Phase-organized (current) strengths:**
- Perfect alignment with GSD workflow stages (discuss -> case -> plan -> execute -> verify -> ship)
- Natural grouping of all artifacts for a single unit of work
- Temporal ordering clear: Phase 1 came before Phase 2
- No maintenance burden -- artifacts created by the pipeline, never moved
- Case-briefer, planner, validator all know exactly where to look during active phases

**Phase-organized (current) weaknesses:**
- Discovery by service/operation requires ROADMAP dependency traversal
- Implicit dependencies (e.g., auth -> user service) not captured in `Depends on`
- As phases accumulate, finding "all specs for auth service" requires scanning 3A + 3B + possibly future phases
- A single service's behavioral contract is fragmented across multiple phase files
- After completion, phase artifacts are "cold storage" -- rarely read, never updated

**Service-organized strengths:**
- Direct answer to "what does service X do?" and "what operations does service X expose?"
- Case-briefer could grep a single directory tree instead of ROADMAP traversal
- Natural home for cross-cutting service-level concerns (error taxonomy, shared rules)
- Aligns with code structure (services/auth, services/catalog, services/user)
- Easier to detect conflicts when a later phase modifies an existing operation

**Service-organized weaknesses:**
- Breaks temporal context (which phase decided this? when?)
- Requires extraction/transformation from phase artifacts (maintenance burden)
- Potential for staleness if not kept in sync with phase completions
- GSD workflow tools expect phase-directory locations during active work
- Two sources of truth if both exist without clear authority rules

**Key insight from ADR patterns:** The MADR project recommends that categorization use the same organizing principles as other artifacts. For madome, code is organized by service (`services/auth/`, `services/catalog/`, `services/user/`, `services/gateway/`). Aligning spec organization with this structure is natural.

**Key insight from monorepo patterns:** Large monorepos (Nx, Turborepo) co-locate documentation with the code it describes. However, madome's specs are planning artifacts that inform but do not live alongside code -- they describe behavioral contracts at a higher abstraction level than code comments or READMEs.

### Q2: Flexible Folder Structure for Multiple Spec Types

The project produces several distinct spec types that differ in structure and audience:

| Spec Type | Current Source | Audience | Update Frequency |
|-----------|---------------|----------|------------------|
| Operation specs (S/F/E cases) | CASES.md | Case-briefer, validator, verifier, test authors | At phase completion; may evolve if later phase modifies operation |
| Design decisions | CONTEXT.md | Discuss-phase, case-briefer, researcher | Frozen after phase ships; later phases add new decisions |
| Error taxonomy | CASES.md (implicit in failure cases) | Cross-service error handling | Grows as new operations are added |
| Domain rules / invariants | CASES.md Phase Rules + PROJECT.md System Rules | All pipeline stages | Phase Rules fixed per phase; System Rules grow |
| Proto contract summaries | CASE-BRIEFING.md | Case-briefer, planner | Updated when proto changes |
| Cross-service interaction patterns | CONTEXT.md, CASES.md (gateway compose patterns) | Gateway planner, cross-phase briefer | Updated when new compose patterns emerge |

**Naming convention analysis:**

Flat-with-prefix approach (e.g., `specs/auth/operations.md`, `specs/auth/decisions.md`) is simplest but does not scale well when a service has 20+ operations. Nested-by-operation approach (e.g., `specs/auth/operations/signup-begin.md`) provides better granularity but creates many small files that are hard to browse.

**Recommended approach:** Service-level files for each spec type, not per-operation files. Operations within a service are tightly coupled (shared Phase Rules, shared error taxonomy, cross-referenced in side effects). Splitting them into individual files loses this cohesion.

### Q3: INDEX.md Design

**Option A: Auto-generated from file headers**
- Pros: Zero maintenance, always accurate
- Cons: Requires standardized frontmatter, needs a script to run

**Option B: Manually maintained**
- Pros: Can include editorial notes, flexible structure
- Cons: Staleness risk, extra developer burden

**Option C: Hybrid -- auto-generated body with manual annotations**
- Pros: Accurate base, enriched with human context
- Cons: Merge conflicts if both human and script modify same file

**Recommended: Option A with metadata headers.** Each spec file starts with a YAML-like metadata block that a simple script (or grep) can parse. The INDEX.md is auto-generated at extraction time. Manual annotations are unnecessary because the spec files themselves contain the context.

**Metadata format for spec files:**

```markdown
---
service: auth
type: operations | decisions | rules | errors | interactions
phase_origin: 3A
last_updated: 2026-03-26
operations: [SignupBegin, SignupFinish, LoginBegin, LoginFinish, ...]
---
```

**INDEX.md structure:**

```markdown
# Spec Index

## By Service

### auth
- [Operations](auth/operations.md) -- 12 operations (Phase 3A, 3B)
- [Decisions](auth/decisions.md) -- 142 decisions (Phase 3A, 3B)
- [Rules](auth/rules.md) -- 3 PR, 15 operation rules (Phase 3A, 3B)
- [Error Taxonomy](auth/errors.md) -- (Phase 3A, 3B)

### gateway
- [Operations](gateway/operations.md) -- VerifyJwt middleware (Phase 3A)
- [Interactions](gateway/interactions.md) -- Compose patterns (Phase 3A, 6)

### user
- [Operations](user/operations.md) -- 8 operations (Phase 2)
- [Decisions](user/decisions.md) -- 44 decisions (Phase 2)

## By Operation (cross-reference)
| Operation | Service | Phase | File |
|-----------|---------|-------|------|
| SignupBegin | auth | 3A | auth/operations.md#signupbegin |
| VerifyJwt | gateway | 3A | gateway/operations.md#verifyjwt |
| CreateUser | user | 2 | user/operations.md#createuser |
```

**Case-briefer query pattern:** Instead of ROADMAP dependency traversal, briefer greps INDEX.md for operation names found in CONTEXT.md cross-references (e.g., "User.CreateUser" -> find in INDEX -> read `user/operations.md#createuser`).

### Q4: Cross-Service Operations

Gateway compose patterns (e.g., user deactivation = User.DeactivateUser + Auth.InvalidateAllSessions) call multiple services. Where does the composed operation's spec live?

**Option A: Gateway folder only**
- The composition logic lives in Gateway. The spec should too.
- Cons: Auth and User service specs don't reference the composed behavior.

**Option B: Linked from both services**
- Each service's operations.md contains the individual operations. Gateway's interactions.md describes the composition.
- Cons: Duplication risk if the composition description repeats individual operation details.

**Option C: Dedicated cross-service section**
- A top-level `specs/interactions/` directory for multi-service flows.
- Cons: Adds another location to check. Orphaned from service context.

**Recommended: Option B.** Gateway is the orchestrator, so `specs/gateway/interactions.md` is the canonical location for compose pattern specs. Individual service operations stay in their service folders. Cross-references link them:

```markdown
# Gateway Interactions

## UserDeactivation (compose pattern)

**Services:** user, auth
**Steps:**
1. User.DeactivateUser -> [user/operations.md#deactivateuser]
2. Auth.InvalidateAllSessions -> [auth/operations.md#invalidateallsessions]
**Compensation:** User.ActivateUser on Auth failure
**Source phase:** 6 (or wherever this is defined)
```

The individual operation specs in `specs/user/operations.md` and `specs/auth/operations.md` include a "Referenced by" note pointing to the gateway interaction.

### Q5: Reorganization Timing and Automation

**Timing options:**

| Timing | Pros | Cons |
|--------|------|------|
| At `gsd:ship` | Spec is finalized, all verifications passed | Adds latency to ship step |
| At `gsd:complete-milestone` | Batch processing, natural checkpoint | Delayed availability for intra-milestone phases |
| Manual trigger | Full developer control | Forgotten, inconsistent |
| At `gsd:verify` completion | Spec stable enough after UAT | May change if verify finds issues |

**Recommended: At `gsd:ship` via post-ship hook.** Rationale:
1. The spec is finalized (verified, simplified, shipped)
2. Immediate availability for the next phase's case-briefer
3. Automated, no developer discipline required
4. Intra-milestone phases (e.g., 3A -> 3B) benefit immediately

**Automation approach:**

The extraction is a structured transformation, not free-form. A script can:
1. Read CASES.md -- extract operations, rules, S/F/E cases, side effects, open questions
2. Read CONTEXT.md -- extract decisions (numbered D-XX items)
3. Read CASE-BRIEFING.md -- extract cross-cutting constraints classification
4. Write to `specs/{service}/operations.md`, `specs/{service}/decisions.md`, etc.
5. Regenerate INDEX.md
6. Commit as part of the ship step

**Implementation:** A justfile recipe (`just extract-specs <phase>`) called by gsd:ship's post-merge step. Not a shell script (CLAUDE.md convention). The recipe reads the phase directory and writes to specs/.

**Incremental updates:** When Phase 3B ships, its auth operations are appended to/merged with the existing `specs/auth/operations.md` from Phase 3A. The script must handle merge, not just overwrite. Each operation section is self-contained (identified by `## Operation: Name` heading), so the merge is heading-level: replace if exists, append if new.

### Q6: Relationship to Phase Artifacts

**Option A: Move specs, leave the rest**
- Phase dir retains PLAN.md, SUMMARY.md, VALIDATION.md, VERIFICATION.md (execution artifacts)
- CASES.md and CONTEXT.md "promoted" to specs/
- Cons: Duplication if someone reads the original phase dir

**Option B: Keep both (dual view)**
- Phase dir is untouched (complete historical record)
- specs/ contains curated, service-organized views
- Cons: Two copies of the same information

**Option C: Replace originals with symlinks**
- Phase dir CASES.md becomes symlink to specs service file
- Cons: Fragile, confusing, breaks if spec structure changes

**Recommended: Option B (keep both).** Rationale:
1. Phase directories are the authoritative historical record. GSD tools reference them during active phases.
2. specs/ is a curated, maintained view optimized for discovery.
3. "Duplication" is not a real risk because phase artifacts are frozen after ship. The specs/ view is the maintained copy that evolves (when a later phase modifies an operation). The phase copy is the immutable historical snapshot.
4. If specs/ ever becomes stale or corrupted, the phase directories are the recovery source.

**Authority rule:** During active phase work, phase directory is authoritative. After ship, specs/ is the primary reference. Phase directory is the historical archive.

### Q7: Decommissioned Operations/Services

Later phases may remove, replace, or deprecate operations. Examples:
- Phase 3B might modify SignupFinish to add step-up auth
- A future phase might deprecate an operation entirely
- Service removal (unlikely but possible)

**Recommended approach:**

1. **Modified operations:** The extracting phase's ship step updates the operation in specs/. The new version includes a `Phase history` line: `Originally: Phase 3A. Modified: Phase 3B (added step-up auth check)`.

2. **Deprecated operations:** Mark with `**Status: DEPRECATED (Phase N)**` at the top of the operation section. Include the replacement operation reference. Do not delete -- downstream phases or case-briefers may need to understand what was replaced.

3. **Removed services:** Move the service's spec folder to `specs/_archived/{service}/` with a note in INDEX.md. This preserves grep-ability while clearly marking it as non-current.

**Lifecycle markers:**

```markdown
## Operation: OldOperation

**Status:** DEPRECATED (Phase 7)
**Replaced by:** [NewOperation](../auth/operations.md#newoperation)
**Reason:** [brief explanation]

[Original spec preserved below for historical reference]
```

## Candidate Folder Structures

### Candidate A: Flat Service Directories

```
.planning/
  specs/
    INDEX.md
    auth.md              # Everything about auth in one file
    gateway.md           # Everything about gateway in one file
    user.md              # Everything about user in one file
    catalog.md           # Everything about catalog in one file
  phases/                # Unchanged
    01-foundation/...
    02-user-profile/...
    03a-authentication-core/...
```

Each service file contains all spec types (operations, decisions, rules, errors) in sections within a single document.

**Pros:**
- Simplest structure. One file per service.
- Easy to grep: `grep -n "SignupBegin" specs/auth.md`
- No directory nesting. Flat is fast.
- INDEX.md just lists service files.

**Cons:**
- Files grow large. Auth will have 100+ decisions and 20+ operations across 3A/3B. Could hit 2000+ lines.
- Mixing spec types (decisions, operations, rules) in one file makes them harder to consume separately.
- Case-briefer wants operations. Planner wants decisions. Validator wants rules. All must parse the same large file.
- Difficult to automate incremental updates (must parse internal structure to merge).

**Verdict:** Too coarse. Works for small projects but does not scale with madome's 6 services and deep spec detail.

### Candidate B: Service Directories with Typed Files (Recommended)

```
.planning/
  specs/
    INDEX.md                    # Auto-generated service + operation index
    auth/
      operations.md             # All auth operations (S/F/E cases, rules, side effects)
      decisions.md              # All auth design decisions (D-XX items from CONTEXT.md)
      errors.md                 # Auth error taxonomy (cross-operation error categorization)
    gateway/
      operations.md             # Gateway-specific operations (VerifyJwt middleware)
      decisions.md              # Gateway design decisions
      interactions.md           # Cross-service compose patterns
    user/
      operations.md             # User service operations
      decisions.md              # User service design decisions
    catalog/
      operations.md             # Catalog operations
      decisions.md              # Catalog design decisions
    _rules/
      system-rules.md           # -> Symlink or reference to PROJECT.md System-Wide Rules
      phase-rules-index.md      # Phase Rules cross-reference (which PR applies where)
    _archived/                  # Decommissioned service specs
  phases/                       # Unchanged (historical record)
    01-foundation/...
    02-user-profile/...
```

Each service directory has typed files. Common spec types:
- `operations.md` -- the primary file. Contains operation specs extracted from CASES.md.
- `decisions.md` -- design decisions extracted from CONTEXT.md, organized by topic.
- `errors.md` -- error taxonomy extracted from failure cases across operations.
- `interactions.md` -- (gateway only) cross-service compose patterns.

The `_rules/` directory (underscored = meta, not a service) provides a cross-service view of rules without duplicating them.

**operations.md internal structure:**

```markdown
---
service: auth
type: operations
phases: [3A, 3B]
last_updated: 2026-03-26
---

# Auth Service Operations

## Phase Rules

- PR1: JWT claim set contract: {sub=user_id, role, sid=session_id, ...} (D-39, D-40, D-43)
- PR2: Ceremony state deletion is best-effort (D-147)
- PR3: COOKIE_SECURE=false dev override (D-44)

---

## SignupBegin

**Phase origin:** 3A
**Interface:** POST /v1/auth/signup/begin
**Auth:** Public (no auth required)
**Requirements:** AUTH-01

### Rules
- R1: Anyone with a valid invite token can sign up (public route)
- R2: Token is single-use, 30-minute expiry (D-03)
[... full rules from CASES.md ...]

### Side Effects
- DB: invite token marked as consumed
- Cross-service: user created via User.CreateUser
[...]

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
[... full S/F/E table from CASES.md ...]

### Open Questions
(none)

---

## SignupFinish
[... same structure ...]
```

**Pros:**
- Clear separation of concerns. Each file type has one consumer profile.
- Manageable file sizes. Auth operations.md (~12 operations) stays under 1000 lines.
- Typed files allow targeted reads: case-briefer reads operations.md only.
- Directory structure mirrors code structure (services/auth -> specs/auth).
- Incremental merge by heading: `## OperationName` sections are self-contained.
- Extensible: add new file types (e.g., `migration-notes.md`) without restructuring.
- INDEX.md auto-generation is straightforward: list files per service, list operations per file.

**Cons:**
- More files to manage than Candidate A.
- decisions.md content overlaps with frozen CONTEXT.md in phase directories.
- Requires extraction script that understands multiple source file formats.

**Verdict:** Best balance of granularity, discoverability, and maintainability. Recommended.

### Candidate C: Operation-Centric with Service Grouping

```
.planning/
  specs/
    INDEX.md
    auth/
      signup-begin.md           # One file per operation
      signup-finish.md
      login-begin.md
      login-finish.md
      register-begin.md
      register-finish.md
      create-invite.md
      validate-session.md
      refresh-token.md
      logout.md
      _service.md               # Service-level overview, error taxonomy, shared rules
    gateway/
      verify-jwt.md
      _service.md
      _interactions/
        user-deactivation.md
    user/
      create-user.md
      get-user.md
      get-user-by-handle.md
      list-users.md
      update-user.md
      deactivate-user.md
      activate-user.md
      change-role.md
      _service.md
    catalog/
      [...]
  phases/
    [unchanged]
```

Each operation gets its own file containing all spec types for that operation (rules, cases, side effects, relevant decisions).

**Pros:**
- Maximum granularity. Each operation is independently addressable.
- Clean file links: `specs/auth/signup-begin.md` is unambiguous.
- Parallel editing: two people working on different operations do not conflict.

**Cons:**
- File explosion. Auth alone has 10+ operations in 3A, more in 3B. Each is a small file.
- Phase Rules and shared context must be duplicated in every file or extracted to `_service.md`.
- Cross-operation analysis (e.g., "which operations use ceremony state?") requires reading many files.
- The case-briefer reads operations together, not individually. Per-file splits its work.
- CASES.md naturally groups operations -- per-file splits break this cohesion.
- Error taxonomy spans operations -- awkward to place in one file or duplicate.

**Verdict:** Over-granular for madome's needs. The project's CASES.md format groups operations for a reason -- shared Phase Rules, cross-operation error comparisons, and side effect chains all benefit from cohabitation. Per-file splitting loses this.

## Recommended Structure: Candidate B

### Rationale

1. **Mirrors code structure.** `services/auth/` -> `specs/auth/`. Developers intuit where to look.

2. **Right granularity.** Operations within a service share rules and error patterns. Grouping them in one file preserves these connections. Separating operations from decisions prevents 2000-line monoliths.

3. **Optimized for the primary consumer: case-briefer.** The briefer needs operations and their behavioral contracts. `specs/{service}/operations.md` is exactly that. One file read per service, not ROADMAP traversal.

4. **Extensible without restructuring.** Need to add proto contract summaries later? Add `specs/{service}/proto.md`. Need migration notes? `specs/{service}/migration.md`. The pattern is `specs/{service}/{type}.md`.

5. **Compatible with living spec evolution.** If the project later moves to spec-first workflow (spec change precedes code change), the operations.md files become the living contracts. The structure supports this without reorganization.

6. **Automation-friendly.** Heading-based sections (`## OperationName`) enable incremental merge. A later phase appends new operations or replaces modified ones. Metadata headers enable INDEX.md generation.

### File Templates

**specs/{service}/operations.md:**

```markdown
---
service: {service_name}
type: operations
phases: [{list of contributing phases}]
last_updated: {date}
---

# {Service} Service Operations

## Phase Rules

- PR{N}: {rule text} ({decision refs})
[...]

---

## {OperationName}

**Phase origin:** {phase}
**Modified:** {phase} (if applicable)
**Interface:** {how called}
**Auth:** {access control}
**Requirements:** {REQ-IDs}

### Rules
{copied from CASES.md}

### Side Effects
{copied from CASES.md}

### Cases
{S/F/E table copied from CASES.md}

### Open Questions
{if any remain}

### Referenced By
- {links to gateway interactions or other operations that call this}

---
```

**specs/{service}/decisions.md:**

```markdown
---
service: {service_name}
type: decisions
phases: [{list of contributing phases}]
last_updated: {date}
---

# {Service} Design Decisions

## {Topic Group} (from Phase {X})

- **D-{XX}:** {decision text}
- **D-{XX}:** {decision text}
[...]

## {Topic Group} (from Phase {Y})
[...]
```

**specs/{service}/errors.md:**

```markdown
---
service: {service_name}
type: errors
phases: [{list of contributing phases}]
last_updated: {date}
---

# {Service} Error Taxonomy

## Client-Facing Errors

| Error Code | HTTP Status | Conditions | Operations |
|------------|-------------|------------|------------|
| unauthorized | 401 | Invalid JWT, expired session, deactivated user, bad credential | LoginFinish, VerifyJwt, Logout |
| invite_invalid | 400 | Token missing, expired, used, not found | SignupBegin |
| ceremony_invalid | 400 | State missing, expired, credential invalid | SignupFinish, RegisterFinish, LoginFinish |
| validation_error | 400 | Handle/name validation failure | SignupBegin |
| conflict | 409 | Handle already taken | SignupBegin |

## Internal Errors (logged, not exposed)

| Error | Context | Logged Fields |
|-------|---------|--------------|
| Orphan user | SignupFinish compensation failure | user_id, handle, request_id |
[...]
```

## Migration Strategy

### Phase 0: Establish Structure (One-Time)

1. Create `specs/` directory structure:
   ```
   mkdir -p .planning/specs/{auth,gateway,user,catalog,_rules,_archived}
   ```

2. Create initial INDEX.md (manually for completed phases 1 and 2).

3. Extract Phase 2 user service specs as the pilot:
   - Read `02-CONTEXT.md` -> write `specs/user/decisions.md`
   - No CASES.md exists for Phase 2, so operations.md is derived from CONTEXT.md decisions and the implemented code
   - This is a one-time backfill, not the automated path

4. Extract Phase 1 gateway specs:
   - Read `01-CONTEXT.md` -> write `specs/gateway/decisions.md`
   - Minimal operations (health routing only, likely not worth extracting)

### Phase 1: Automate for Active Phases

1. Write a justfile recipe: `just extract-specs <phase-dir>`
   - Reads CASES.md, CONTEXT.md, CASE-BRIEFING.md from phase dir
   - Parses operation sections (## Operation: Name), decision sections
   - Writes/merges into `specs/{service}/` files
   - Regenerates INDEX.md

2. Integrate into gsd:ship post-merge step:
   - After successful merge to dev, run `just extract-specs`
   - Commit specs/ changes as part of the ship commit

3. First real test: Phase 3A ships -> auth and gateway specs extracted automatically.

### Phase 2: Update Case-Briefer

1. Modify `.claude/agents/case-briefer.md` Step 4.7:
   - Instead of resolving ROADMAP dependencies and scanning phase CASES.md,
   - Read INDEX.md to find relevant service operations
   - Read `specs/{service}/operations.md` for behavioral contracts
   - Cross-reference operation names from CONTEXT.md (e.g., "User.CreateUser")

2. Benefits: No ROADMAP dependency traversal. Implicit cross-service dependencies discovered through operation name matching. Faster, more reliable, catches what `Depends on` misses.

### Phase 3: Backfill Historical Phases

1. Manually extract Phase 1 and Phase 2 specs (limited scope).
2. For phases without CASES.md (Phase 1, Phase 2 if /case was not run), derive minimal operation specs from CONTEXT.md decisions + implemented code.
3. This is low priority -- Phase 2's user operations are referenced by auth service, so backfilling them has real value. Phase 1's health RPCs are trivial.

### Migration Timeline

| Step | When | Effort | Blocking? |
|------|------|--------|-----------|
| Create directory structure | Before Phase 3A ships | 10 min | No |
| Backfill Phase 2 user specs | Before Phase 3B starts | 30 min | No (3A does not need it yet) |
| Write extract-specs recipe | Before Phase 3A ships | 2-4 hours | No (manual extraction works as fallback) |
| Integrate into gsd:ship | After extract-specs works | 30 min | No |
| Update case-briefer | After Phase 3A specs exist | 1-2 hours | No (current briefer still works) |

## Open Questions

### OQ1: CASES.md format stability
The extraction script assumes a stable CASES.md format (## Operation: Name, ### Rules, ### Cases, etc.). The current format has evolved (e.g., signup/register separation in 3A). If the format changes again, the extraction script needs updating.

**Recommendation:** Document the CASES.md format contract (heading structure, table columns) as part of the /case skill. The format has been stable for the last 3 phases of development -- changes are infrequent.

### OQ2: Spec update workflow for cross-phase modifications
When Phase 3B modifies a Phase 3A operation (e.g., adds step-up auth to passkey deletion), the spec must be updated at 3B ship time. The extraction script must detect this is a modification (not new operation) and merge appropriately.

**Recommendation:** Heading-level merge with replace semantics. If `## OperationName` already exists in operations.md, the 3B version replaces it entirely. The `Phase origin` and `Modified` metadata fields track history.

### OQ3: Gateway operations span multiple services
Gateway's VerifyJwt middleware and compose patterns reference auth, user, and catalog services. Its operations.md will contain cross-references to other service specs. The extraction must resolve these references correctly.

**Recommendation:** Use relative links (`../auth/operations.md#signupbegin`). The extractor does not resolve links -- it writes them as-is. Link validity checked by INDEX.md generation (which reads all files).

### OQ4: Spec-first workflow transition
The memory note `project_spec_driven_workflow` mentions eventual spec-first workflow where spec change precedes code change. This research covers the folder structure prerequisite but not the workflow change itself.

**Recommendation:** Defer. The folder structure in Candidate B supports spec-first workflow -- operations.md becomes the editable contract, and the extraction direction reverses (spec -> phase artifacts, instead of phase artifacts -> spec). The structure does not need to change; only the workflow direction changes.

### OQ5: Multi-service phases
A single phase may contribute operations to multiple services (e.g., Phase 3A contributes to both auth and gateway). The extraction script must route operations to the correct service directory.

**Recommendation:** Use the operation's Interface field to determine service. REST endpoints under `/v1/auth/*` -> auth. Gateway middleware -> gateway. gRPC RPCs on auth service -> auth. The CASE-BRIEFING.md already categorizes operations by service.

### OQ6: Handling discussion-phase CONTEXT.md concerns
Discuss-phase currently scans dependency CASES.md for design-level concerns (per CLAUDE.md cross-phase forwarding). Should it also scan specs/?

**Recommendation:** Yes, once specs/ is populated. Specs/ is the curated, maintained view and should replace phase-dir scanning for completed phases. Active phases still use phase directories.

## Sources

### Primary (HIGH confidence)
- Internal project analysis: `.planning/phases/*/`, `.planning/WORKFLOW.md`, `.planning/PROJECT.md`, `.claude/agents/case-briefer.md`
- MADR ADR categorization patterns (https://adr.github.io/madr/)
- Cross-phase forwarding research: `.planning/research/CROSS-PHASE-FORWARDING-SUMMARY.md`

### Secondary (MEDIUM confidence)
- Monorepo documentation patterns from Nx, Turborepo, Backstage TechDocs
- arc42/C4 model documentation structure (https://bitsmuggler.github.io/arc42-c4-software-architecture-documentation-example/)
- API specification organization patterns (OpenAPI, spec-driven development)
- ADR domain-based folder structures (https://github.com/joelparkerhenderson/architecture-decision-record)
- Living specification change management patterns (https://dev.to/juranki/change-management-when-specs-are-living-documents-1mjb)
