# Case Validator Redesign: Discuss-Output Gap Validation

**Researched:** 2026-03-24
**Domain:** Behavioral case validation against planning artifacts (no code dependency)
**Confidence:** HIGH

---

## 1. Executive Summary

The current `case-validator` agent (`.claude/agents/case-validator.md`) is entirely code-dependent. Its methodology -- scan implementation source for validation rules, error branches, state checks, and default values not covered by cases -- requires existing code. But the /case skill runs AFTER `gsd:discuss` and BEFORE `gsd:plan`. At this point in the pipeline, no implementation code, proto files, or tests exist (WORKFLOW.md artifact availability table confirms this). The current validator is useless for new phases.

The redesign replaces code-based gap detection with **artifact cross-referencing**. Instead of "code has behavior X but no case covers it," the validator asks "artifact A says X but no case addresses X." The available artifacts at validation time are: CONTEXT.md (locked decisions), ROADMAP.md (phase requirements and success criteria), CASE-BRIEFING.md (extracted operations with interfaces), and CASE-SCRATCH.md (discovered cases from discussion).

**Primary recommendation:** Redesign the validator as a document cross-referencing agent. The methodology is structured enough to justify a subagent (not inline), but the scope should be constrained to five specific gap types with concrete extraction rules. The agent name should change from `case-validator` to reflect its new purpose.

---

## 2. Current Validator Analysis

### What It Does Today

The current validator follows five steps:

1. **Load inputs** -- read CASE-SCRATCH.md and CASE-BRIEFING.md
2. **Scan for uncovered behaviors** -- read implementation source, look for validation rules, error branches, state checks, default values, and type constraints without matching cases
3. **Check for conflicts** -- compare cases against actual code behavior (error codes, preconditions, fields/states/roles)
4. **Identify cross-operation gaps** -- look for inter-operation state dependencies, shared validation logic, ordering dependencies
5. **Compile findings** -- organize into Gaps, Conflicts, Suggestions

### What to Keep

| Element | Reason to Keep |
|---------|---------------|
| Three-category output (Gaps, Conflicts, Suggestions) | Clean, actionable structure. Categories map well to developer decisions: "add this," "fix this," "consider this" |
| Code reference per finding | Needs adaptation: change from `file:line` to `artifact:decision-ID` or `artifact:section` |
| Return protocol (`## VALIDATION COMPLETE` with counts) | Consistent with agent architecture patterns |
| Quality gate checklist | Good discipline, needs new criteria |
| "Be precise, not exhaustive" guideline | Critical for avoiding noise |
| Cross-operation gap detection (Step 4) | Still valuable, just sourced from artifacts instead of code |
| Downstream consumer note (findings presented one-by-one to developer) | Correct interaction model, keep as-is |

### What to Remove

| Element | Reason to Remove |
|---------|-----------------|
| Step 2 methodology (scan implementation source for validation rules, error branches, state checks, default values, type constraints) | No code exists. This is the core of what needs replacing |
| Step 3 conflict detection (cases vs code behavior) | No code to compare against. Replace with cases vs decisions |
| `<files_to_read>` for source code paths | No source files exist. Replace with planning document paths |
| Quality gate: "Every operation's source code was read" | No source code. Replace with artifact coverage checks |
| Quality gate: "Each gap finding references a specific code location (file:line)" | No code locations. Replace with artifact references (decision ID, requirement ID, section heading) |
| Guideline: "Distinguish code gaps from case gaps" | Distinction does not apply without code |

### What to Adapt

| Element | Current Form | New Form |
|---------|-------------|----------|
| Input tags | `<cases_file>`, `<briefing_file>`, `<files_to_read>` (code paths) | `<cases_file>`, `<briefing_file>`, `<context_file>`, `<requirements>` (artifact paths + requirement IDs) |
| Gap definition | "Code has behavior X, no case covers it" | "Artifact A states X, no case addresses it" |
| Conflict definition | "Case assumes behavior code does not implement" | "Case contradicts locked decision in CONTEXT.md" |
| Reference format | `Source: [file:line]` | `Source: [CONTEXT.md D-XX]` or `Source: [ROADMAP.md Phase N, Criterion M]` |

---

## 3. Gap Validation Methodology

### Available Artifacts at Validation Time

| Artifact | Contains | Extractable Claims |
|----------|----------|-------------------|
| **CONTEXT.md** | Locked decisions (D-XX), discretion areas, deferred items, canonical references, established patterns | Behavioral constraints, error policies, auth tiers, pagination rules, data formats, API conventions |
| **ROADMAP.md** | Phase description, requirements (REQ-IDs), success criteria | What must be true, what capabilities must exist |
| **CASE-BRIEFING.md** | Operations with interfaces, inputs, outputs, auth requirements, existing validation, domain constraints | Operation inventory, interface contracts, auth levels |
| **CASE-SCRATCH.md** | Discovered cases per operation (S/F/E tables), rules, open questions | What was actually discussed and documented |

### Step-by-Step Methodology

#### Step 1: Load All Artifacts

Read all four artifacts. Parse them into structured internal representations:

- **From CONTEXT.md:** Extract all decision IDs (D-XX) with their content. Categorize each decision as: behavioral constraint (affects what an operation does), structural constraint (affects how code is organized, irrelevant to cases), or informational (background context). Only behavioral constraints feed into gap detection.
- **From ROADMAP.md:** Extract phase requirements (REQ-IDs) and success criteria. Each requirement and success criterion is a checkable claim.
- **From CASE-BRIEFING.md:** Extract the operation inventory -- every operation name with its interface, auth level, and inputs/outputs. This is the universe of operations that should have cases.
- **From CASE-SCRATCH.md:** Extract all operations discussed, all case IDs per operation, all rules per operation, all open questions. Build an index: which topics/decisions are covered by which cases.

#### Step 2: Run Five Gap Checks

Each gap check is a specific cross-referencing operation between two artifacts. The checks are ordered from highest-value (most likely to find real gaps) to lowest-value (most likely to produce noise).

**Check A: Requirement Coverage**

Cross-reference: ROADMAP.md requirements + success criteria vs CASE-SCRATCH.md operations and cases.

For each phase requirement (REQ-ID) and success criterion:
1. Is there at least one operation in CASE-SCRATCH.md that addresses this requirement?
2. Does the operation have at least one success case (S-type) demonstrating the requirement is met?
3. Does the operation have at least one failure case (F-type) demonstrating the requirement is enforced?

Gap = requirement with no matching operation OR operation with no success case covering the requirement.

Example signal: ROADMAP.md says "AUTH-05: Auth service caches recently issued JWT per session (duplicate prevention)." If no operation in CASE-SCRATCH.md addresses JWT caching behavior, that is a requirement gap.

**Check B: Decision Coverage**

Cross-reference: CONTEXT.md behavioral decisions vs CASE-SCRATCH.md cases.

For each behavioral decision (D-XX) that implies observable behavior:
1. Filter to decisions with testable implications (error responses, boundary values, format requirements, auth rules, lifecycle rules).
2. Is there a case in CASE-SCRATCH.md whose Expected Outcome or Preconditions references this decision's constraint?

Gap = behavioral decision with no case exercising it.

Not all decisions need case coverage. Structural decisions (D-68: "Auth service follows 4-layer architecture") have no behavioral implication. The validator must filter to only decisions that affect what the caller observes.

Heuristic for identifying behavioral decisions:
- Contains error codes, status codes, or response formats -> behavioral
- Contains boundary values (TTL, max length, limits) -> behavioral
- Contains auth tier or role requirements -> behavioral
- Contains "must" or "cannot" about observable behavior -> behavioral
- Contains architecture pattern, file organization, naming convention -> structural (skip)

Example signal: CONTEXT.md D-15 says "Last passkey cannot be deleted (minimum 1 always)." If DeletePasskey in CASE-SCRATCH.md has no failure case covering "attempt to delete sole remaining passkey," that is a decision gap.

**Check C: Completeness Check**

Cross-reference: CASE-SCRATCH.md internal consistency.

For each operation in CASE-SCRATCH.md:
1. Does it have at least one success case (S-type)?
2. Does it have at least one failure case (F-type)?
3. If auth is required (from CASE-BRIEFING.md), does it have auth failure cases?
4. If it has input parameters (from CASE-BRIEFING.md), does it have input validation failure cases?
5. Are all referenced rules (R1, R2...) exercised by at least one case?
6. Open question count: more than 3 triggers a readiness warning.

Gap = operation missing an expected case category entirely, or rule with no exercising case.

**Check D: Consistency Check**

Cross-reference: CASE-SCRATCH.md operations against each other.

For patterns that should be consistent across operations:
1. Error format: do all operations use the same error response pattern?
2. Auth enforcement: do operations at the same auth tier handle auth failures the same way?
3. Pagination: do all list operations use the same pagination pattern?
4. Not-found behavior: do operations handle missing resources consistently (404 vs empty result)?

Gap = inconsistency between operations on a cross-cutting concern.

Also check: do CONTEXT.md project-wide decisions (pagination format D-121/122/123, error policy D-50/51, API conventions D-125/126) appear consistently across all relevant operations?

**Check E: Briefing Coverage**

Cross-reference: CASE-BRIEFING.md operations vs CASE-SCRATCH.md operations.

For each operation identified by the briefer:
1. Was it discussed in CASE-SCRATCH.md?
2. If not, was it explicitly skipped by the developer (mentioned in discussion) or silently dropped?

Gap = operation identified by briefer but absent from CASE-SCRATCH.md with no explicit skip.

This is the simplest check but catches operations that were accidentally forgotten during a long discussion session.

#### Step 3: Classify and Prioritize Findings

Each finding receives:
- **Category:** Requirement Gap, Decision Gap, Completeness Gap, Consistency Issue, or Briefing Gap
- **Severity:** high (requirement or security-related decision uncovered), medium (behavioral decision uncovered), low (consistency or completeness concern)
- **Source reference:** exact artifact location (e.g., "CONTEXT.md D-15", "ROADMAP.md AUTH-05", "CASE-BRIEFING.md ListPasskeys")
- **Suggested action:** what case to add, what to check, or what to reconcile

#### Step 4: Compile Report

Organize findings by category. Each finding must be self-contained (the developer should be able to evaluate it without reading the source artifact).

---

## 4. Revised Output Contract

### New Categories

| Old Category | New Category | What It Catches |
|-------------|-------------|----------------|
| Gaps (code without cases) | **Requirement Gaps** | ROADMAP.md requirement or success criterion with no covering operation/case |
| Gaps (code without cases) | **Decision Gaps** | CONTEXT.md behavioral decision with no exercising case |
| Conflicts (cases vs code) | **Consistency Issues** | Operations handling cross-cutting concerns differently, or cases contradicting locked decisions |
| Suggestions | **Completeness Gaps** | Operation missing expected case categories (no S, no F, no auth cases, unexercised rules) |
| (new) | **Briefing Gaps** | Operation from briefer not discussed in scratch |

### Return Format

```markdown
## Requirement Gaps

1. **[REQ-ID]: [brief description]**
   Source: ROADMAP.md [phase section, requirement or criterion number]
   The requirement states: "[quote]"
   No operation in CASE-SCRATCH.md addresses [specific aspect].
   Suggested action: [add operation X with case Y, or add case to existing operation Z]

## Decision Gaps

1. **[D-XX] in [OperationName]: [brief description]**
   Source: CONTEXT.md [D-XX]
   The decision states: "[quote]"
   [OperationName] has no case covering this constraint.
   Suggested case: [F/E][N] [case description] -> [expected outcome]

## Consistency Issues

1. **[brief description]**
   Affects: [OperationA, OperationB]
   [OperationA] says [X] (case [ID]), but [OperationB] says [Y] (case [ID]).
   Source: CONTEXT.md [D-XX] specifies [expected consistent behavior].
   Suggested action: [reconcile to X or Y, or confirm intentional difference]

## Completeness Gaps

1. **[OperationName]: [what is missing]**
   Source: CASE-BRIEFING.md [operation section]
   The operation has [N] success cases but no [failure/auth/validation] cases.
   [Auth level is X, suggesting auth failure cases are expected.]
   Suggested case: [F/E][N] [case description] -> [expected outcome]

## Briefing Gaps

1. **[OperationName]: not discussed**
   Source: CASE-BRIEFING.md [operation section]
   The briefer identified this operation but it does not appear in CASE-SCRATCH.md.
   [Was it intentionally skipped or overlooked?]
```

If a category has no findings, include the heading with "None found."

### Return Protocol

```
## VALIDATION COMPLETE
Requirement Gaps: [count] | Decision Gaps: [count] | Consistency: [count] | Completeness: [count] | Briefing: [count]
```

On failure:
```
## VALIDATION FAILED
Reason: [what went wrong]
```

---

## 5. Example: Phase 3 Authentication

To illustrate, here is what the validator would find given the actual Phase 3 CONTEXT.md decisions and a hypothetical CASE-SCRATCH.md.

### Hypothetical CASE-SCRATCH.md (partial)

Assume the discussion covered RegisterBegin, RegisterFinish, LoginBegin, LoginFinish, and Logout, but did NOT cover: Recovery, VerifyBegin/Finish, or API key operations.

Assume RegisterFinish has success cases but no failure case for "invite already used" (even though D-03 says single-use).

Assume LoginFinish and Logout both return 401 for expired token, but LoginFinish says "unauthorized" while Logout says "token_expired" (inconsistency given D-50).

### What the Validator Would Find

**Requirement Gaps:**

1. **AUTH-05: JWT caching per session (duplicate prevention)**
   Source: ROADMAP.md Phase 3, AUTH-05
   The requirement states: "Auth service caches recently issued JWT per session (duplicate prevention)"
   No operation in CASE-SCRATCH.md addresses JWT caching behavior or duplicate prevention.
   Suggested action: Add edge case to LoginFinish or RefreshToken covering "concurrent requests receive same JWT within cache window"

**Decision Gaps:**

1. **D-03 in RegisterFinish: single-use invite enforcement**
   Source: CONTEXT.md D-03
   The decision states: "Invite token policy: single-use, 30-minute expiry"
   RegisterFinish has no failure case for attempting registration with an already-used invite token.
   Suggested case: F_ Reused invite token -> INVALID_ARGUMENT, "invite_invalid"

2. **D-15 in DeletePasskey: last passkey protection**
   Source: CONTEXT.md D-15
   The decision states: "Last passkey cannot be deleted (minimum 1 always)"
   DeletePasskey was not discussed (briefing gap), so no case covers this.
   Suggested case: F_ Delete sole remaining passkey -> rejected, passkey count must remain >= 1

3. **D-34 in (no operation): JWT grace period**
   Source: CONTEXT.md D-34
   The decision states: "JWT grace period: 1 minute (within 1 min of JWT expiry, Gateway issues new JWT without session verification)"
   No operation has an edge case covering the 1-minute grace period window.
   Suggested case: E_ Request with JWT expiring within 1 minute -> new JWT issued in cookie, request proceeds

**Consistency Issues:**

1. **Auth error message inconsistency**
   Affects: LoginFinish, Logout
   LoginFinish says expired token -> "unauthorized" (F3), but Logout says expired token -> "token_expired" (F2).
   Source: CONTEXT.md D-50 specifies "generic `unauthorized` for all auth failures. No specific failure reason exposed."
   Suggested action: Reconcile Logout F2 to use "unauthorized" per D-50

**Completeness Gaps:**

1. **RegisterBegin: no auth failure cases**
   Source: CASE-BRIEFING.md RegisterBegin
   The operation is public tier (no auth required), so auth failure cases are not expected.
   (This would NOT be flagged -- the validator should recognize public endpoints do not need auth failure cases.)

**Briefing Gaps:**

1. **Recover: not discussed**
   Source: CASE-BRIEFING.md Recover
   The briefer identified this operation (POST /v1/auth/recover) but it does not appear in CASE-SCRATCH.md.
   Was it intentionally skipped or overlooked? Recovery is security-critical (D-23).

2. **VerifyBegin/VerifyFinish: not discussed**
   Source: CASE-BRIEFING.md VerifyBegin, VerifyFinish
   Step-up authentication operations not discussed. These gate sensitive actions (D-106, D-109).

3. **API key operations: not discussed**
   Source: CASE-BRIEFING.md CreateApiKey, ListApiKeys, RevokeApiKey
   Admin API key management operations not discussed.

---

## 6. Revised Input Contract

| Tag | Required | Contents |
|-----|----------|----------|
| `<objective>` | Yes | Mission statement with phase number and name |
| `<cases_file>` | Yes | Path to CASE-SCRATCH.md containing discovered cases |
| `<briefing_file>` | Yes | Path to CASE-BRIEFING.md for operation inventory reference |
| `<context_file>` | Yes | Path to XX-CONTEXT.md for locked decisions |
| `<requirements>` | Yes | Phase requirement IDs (e.g., "AUTH-01, AUTH-02, ...") and path to ROADMAP.md |

No `<files_to_read>` for source code. The agent reads only planning documents.

### Tool Set Change

| Current | New |
|---------|-----|
| Read, Grep, Glob | Read, Grep |

Glob is no longer needed -- the agent reads specific named files, not searching the filesystem. Grep is retained for searching within large CONTEXT.md files for specific decision IDs or patterns.

---

## 7. Revised Quality Gate

Before returning, verify:

- [ ] All five gap checks were executed (requirement, decision, completeness, consistency, briefing)
- [ ] Each finding references a specific artifact location (CONTEXT.md D-XX, ROADMAP.md REQ-ID, CASE-BRIEFING.md operation name)
- [ ] Each finding quotes the relevant text from the source artifact
- [ ] Each finding includes a suggested action (case to add, inconsistency to resolve, or question to raise)
- [ ] No finding duplicates an existing case in CASE-SCRATCH.md (cross-checked before reporting)
- [ ] Suggested cases follow the S/F/E ID convention used in CASE-SCRATCH.md
- [ ] Structural/non-behavioral decisions from CONTEXT.md were filtered out (not flagged as gaps)
- [ ] Public-tier operations were not flagged for missing auth failure cases
- [ ] Findings are prioritized (requirement and security gaps before completeness concerns)

---

## 8. Value Assessment

### Is a Subagent Justified?

**Yes**, for these reasons:

1. **Context isolation:** The validator needs to read and cross-reference 3-4 large documents (CONTEXT.md alone is often 300+ lines). Loading all of these into the main /case conversation context after a long discussion session (which already consumed significant context on case discovery) would strain context limits. A subagent gets a fresh context window.

2. **Distinct analytical lens:** The discussion step (Step 3) thinks as the Protester -- probing, proposing, debating with the developer. The validation step thinks as an auditor -- systematically cross-referencing documents. These are different cognitive modes. Isolating the audit in a subagent prevents the Protester's conversational momentum from biasing the validation.

3. **Structured output contract:** The five gap checks produce a structured report that the orchestrator presents to the developer. This is a well-defined input/output boundary suitable for agent isolation.

4. **Compression ratio:** The validator reads 1000+ lines of artifacts and produces 50-150 lines of findings. The main conversation benefits from receiving only the compressed findings.

### When to Skip Validation

The validator should be skipped when:

- **Very small phase (1-2 operations):** The discussion itself covers everything. Cross-referencing adds no value when the developer can hold the entire phase in their head.
- **No CONTEXT.md exists:** Without locked decisions, checks B and D have no source material. Only briefing coverage (check E) and completeness (check C) remain, which are simple enough to inline.
- **Resume mode adding a single operation:** Validation already ran on the bulk of operations. Re-running on the full set would re-report existing findings.

The orchestrator should check: if `operation_count <= 2 AND no CONTEXT.md`, skip validation and note "Validation skipped (small phase, no locked decisions)."

### Minimum Viable Validation

If you wanted to inline validation instead of using a subagent, the minimum viable checks are:

1. **Briefing coverage** (Check E): diff operation names between briefing and scratch. One line of logic.
2. **Completeness** (Check C): for each operation, verify at least one S and one F case exist. A few lines of logic.

These two checks could be inlined in the orchestrator's Step 4 (cross-operation concerns) without a subagent. They catch the most obvious gaps (forgotten operations, one-sided case coverage).

However, the decision and requirement gap checks (B and A) are where the real value lies, and those require reading and parsing CONTEXT.md decisions, which is too heavy for inline logic. The subagent earns its keep primarily through these checks.

**Recommendation:** Keep as a subagent. The full five-check methodology adds substantial value for phases with rich CONTEXT.md (like Phase 3 with 120+ decisions). Inline only the briefing coverage check as a quick sanity check in Step 4, with the full validation in Step 5 via subagent.

---

## 9. Agent Rename

The current name `case-validator` implies code-level validation. The redesigned agent validates case coverage against planning artifacts.

**Options considered:**

| Name | Pros | Cons |
|------|------|------|
| `case-validator` (keep) | No rename churn | Misleading: "validator" implies code validation |
| `case-gap-checker` | Describes what it does | Slightly generic |
| `case-auditor` | Matches the analytical role | Could imply compliance/formal audit |
| `case-validator` (redefine) | No rename, update description | Least disruptive |

**Recommendation:** Keep `case-validator` but update the description and frontmatter. The name is generic enough to cover both code-based and artifact-based validation. The YAML description should change from "Cross-checks discovered behavioral cases against the codebase" to "Cross-checks discovered behavioral cases against planning artifacts (CONTEXT.md, ROADMAP.md, CASE-BRIEFING.md) to find requirement gaps, decision gaps, and consistency issues."

---

## 10. Implementation Considerations

### Behavioral Decision Filtering

The hardest part of the methodology is distinguishing behavioral decisions (testable) from structural decisions (not testable via cases). This filtering must be reliable to avoid flooding the developer with irrelevant findings.

**Proposed heuristic (for the agent's methodology section):**

A decision is **behavioral** if it answers "what should the caller observe?" Examples:
- D-03: "single-use, 30-minute expiry" -> YES, caller observes rejection on reuse
- D-15: "Last passkey cannot be deleted" -> YES, caller observes rejection
- D-34: "JWT grace period: 1 minute" -> YES, caller observes automatic refresh
- D-44: "Cookie: HttpOnly=true, Secure=true..." -> YES, caller observes cookie attributes
- D-50: "generic `unauthorized` for all auth failures" -> YES, caller observes error message

A decision is **structural** if it answers "how should the code be organized?" Examples:
- D-68: "Auth service follows 4-layer architecture" -> NO, invisible to caller
- D-75: "All modules use `mod.rs` convention" -> NO, invisible to caller
- D-76: "adapter/ uses directory structure" -> NO, invisible to caller
- D-82: "Prefer `trait_variant` over `async_trait`" -> NO, invisible to caller

A decision is **informational** if it provides context without constraint:
- D-117: "Service-to-service direct gRPC calls allowed" -> NO, architectural permission, not a behavioral constraint
- D-61: "docker-compose for PostgreSQL + Redis" -> NO, infrastructure setup

The agent should apply this heuristic and only flag decisions that are behavioral and not already covered by a case.

### Handling Large Decision Sets

Phase 3 has 120+ decisions. Not all are unique behavioral claims. Many are related (D-21 through D-23 and D-103 through D-105 all cover recovery codes). The validator should group related decisions and check for coverage at the group level, not per-decision.

**Grouping heuristic:** Decisions that share a subject (e.g., "recovery code" appears in D-21, D-22, D-23, D-103, D-104, D-105) should be checked as a cluster. If the Recovery operation in CASE-SCRATCH.md has cases covering recovery code usage, validation, and regeneration, the cluster is covered even if not every individual D-XX is explicitly referenced in a case.

### Avoiding False Positives

The biggest risk is noise -- flagging things as gaps when they are intentionally omitted. Key mitigations:

1. **Respect explicit skips.** If CASE-SCRATCH.md or discussion notes say "X was intentionally skipped," do not flag it.
2. **Filter structural decisions.** As described above.
3. **Accept coverage at the operation level for related decisions.** Do not require a 1:1 mapping of decisions to cases.
4. **Require concrete suggested cases.** If the validator cannot articulate what case should be added, the finding is likely noise. Every finding must include a suggested case with S/F/E prefix, description, and expected outcome.
5. **Cap finding count.** If more than 20 findings are generated, the validator should rank by severity and present only the top 15 with a note about remaining items. Too many findings overwhelm the developer and reduce the chance any are acted upon.

---

## 11. Dispatch Pattern Update

### Current Dispatch (from case.md Step 5)

```
Agent(
  subagent_type: "case-validator",
  prompt: "<objective>...</objective>
    <cases_file>{phase_dir}/CASE-SCRATCH.md</cases_file>
    <briefing_file>{phase_dir}/CASE-BRIEFING.md</briefing_file>
    <files_to_read>{source code paths}</files_to_read>",
  run_in_background: false
)
```

### New Dispatch

```
Agent(
  subagent_type: "case-validator",
  prompt: "<objective>
Cross-check discovered behavioral cases for Phase {phase_number}: {phase_name}
against planning artifacts. Find requirement gaps, decision gaps, consistency issues,
and completeness gaps.
</objective>

<cases_file>
{phase_dir}/CASE-SCRATCH.md
</cases_file>

<briefing_file>
{phase_dir}/CASE-BRIEFING.md
</briefing_file>

<context_file>
{phase_dir}/{padded_phase}-CONTEXT.md
</context_file>

<requirements>
Phase requirements: {comma-separated REQ-IDs from ROADMAP.md}
Roadmap path: .planning/ROADMAP.md
Requirements path: .planning/REQUIREMENTS.md
</requirements>",
  run_in_background: false
)
```

Key changes:
- `<files_to_read>` (code paths) replaced with `<context_file>` and `<requirements>`
- Objective explicitly names the gap types
- No source code paths passed

---

## 12. Confidence Assessment

| Area | Confidence | Reasoning |
|------|-----------|-----------|
| Current validator is broken for new phases | HIGH | WORKFLOW.md explicitly states no code exists at /case time. Current validator methodology requires code. Direct observation, not inference |
| Five gap checks are the right decomposition | HIGH | Each check cross-references exactly two artifacts with a clear gap definition. The checks are orthogonal (no overlap) and exhaustive (cover all artifact pairs) |
| Behavioral vs structural decision filtering | MEDIUM | The heuristic is sound in principle but has not been tested against all 120+ Phase 3 decisions. Some decisions may be ambiguous. The agent will need iteration after first use |
| Subagent vs inline justification | HIGH | Context isolation and compression ratio arguments are well-supported by the agent architecture synthesis (CASE-AGENT-SYNTHESIS.md). The validator reads 1000+ lines and returns 50-150 |
| Skip conditions | MEDIUM | The "1-2 operations" threshold is a judgment call. May need adjustment after observing real usage |
| Finding cap at 15 | MEDIUM | Arbitrary but reasonable. UX research on decision overload suggests 5-7 is optimal for action, but validators benefit from completeness. 15 is a compromise |

**Overall confidence: HIGH.** The redesign is well-grounded in the existing artifact structure and follows patterns already established in the /case skill architecture.

---

## Sources

### Primary (HIGH confidence -- direct reading of project artifacts)

- `.claude/agents/case-validator.md` -- current validator agent definition (full text analyzed)
- `.claude/agents/case-briefer.md` -- briefer agent definition (output contract analyzed)
- `.claude/commands/case.md` -- /case skill orchestrator (full pipeline analyzed)
- `.planning/WORKFLOW.md` -- artifact availability table confirming no code at /case time
- `.planning/phases/03-authentication/03-CONTEXT.md` -- example CONTEXT.md with 120+ decisions (used for methodology validation and example construction)
- `.planning/ROADMAP.md` -- phase structure and requirement IDs
- `.planning/REQUIREMENTS.md` -- requirement definitions and traceability

### Secondary (HIGH confidence -- project research synthesis)

- `.planning/research/CASE-SKILL-SYNTHESIS.md` -- discussion flow, output format, technique layers
- `.planning/research/CASE-AGENT-SYNTHESIS.md` -- agent architecture patterns, dispatch conventions, anti-patterns
- `.planning/research/CASE-CATEGORY-COMPLETENESS.md` -- S/F/E category system analysis

---

*Research completed: 2026-03-24*
*Valid until: 2026-06-24 (stable domain, methodology-focused)*
