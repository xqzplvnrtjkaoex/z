# /case Skill Research Synthesis

**Synthesized:** 2026-03-24
**Domain:** AI-guided behavioral specification, conversational case discovery
**Sources:** 5 research reports (BDD-CASE-DISCOVERY.md, example-mapping.md, EP-BVA-TECHNIQUES.md, TDD-CASE-DISCOVERY.md, ACCEPTANCE-CRITERIA-TECHNIQUES.md)
**Confidence:** HIGH

---

## 1. Unified Framework

The five research areas form a layered system. Each layer serves a distinct purpose, and together they produce a complete conversational case discovery tool.

### Layer Architecture

```
Layer 1: SESSION STRUCTURE (Example Mapping)
  Provides the scaffolding: Story -> Rules -> Examples -> Questions.
  The four-color card model (yellow/blue/green/red) gives the AI an
  internal tracking model for what has been discovered so far.
  Readiness signals (too many red cards, too many blue cards) tell
  the AI when to stop or escalate.

Layer 2: BEHAVIORAL THINKING (BDD / GWT)
  Provides the thinking framework per case: Given (preconditions),
  When (action), Then (outcome). Forces three specific questions
  about every operation. The Three Amigos model assigns the AI the
  Protester/Tester role -- the highest-value contribution in a
  1-on-1 developer conversation.

Layer 3: SYSTEMATIC COVERAGE (EP/BVA + ZOMBIES)
  Provides the probing engine. Two complementary frameworks:
  - ZOMBIES (Zero -> One -> Many with Boundary/Interface/Exception
    cross-cuts) provides COMPLEXITY PROGRESSION per operation.
  - EP/BVA provides DOMAIN COVERAGE per input field.
  ZOMBIES tells you which complexity tier to discuss. EP/BVA tells
  you which input partitions to probe within that tier. Together
  they eliminate the "what should I ask next?" problem.

Layer 4: CONVERSATION EXECUTION (AI Protester persona)
  Provides behavioral guidelines for the AI: when to propose vs ask,
  how to batch questions, when to stop probing, how to handle
  uncertainty, how to avoid interrogation fatigue. The Protester
  persona is the unifying behavioral model.

Layer 5: OUTPUT (XX-CASES.md format)
  Provides the structured deliverable: per-operation sections with
  Rules, Success/Failure/Edge case tables, Open Questions, and a
  readiness summary. Serves both human readers and AI test writers.
```

### How They Interact

The interaction is strictly layered -- upper layers invoke lower layers:

1. **Example Mapping** (L1) determines the session structure: which operation, which rules to explore, when to move on. It sets up the "blue cards" (rules) that become the targets for systematic probing.

2. **GWT** (L2) structures the thinking for each individual case: what preconditions exist (Given), what is the action (When), what should happen (Then). This is the format for expressing discovered cases, not a discussion format shown to the developer.

3. **ZOMBIES** (L3) determines the probing order within each rule exploration: start with Zero (empty/nothing), progress through One (first meaningful case), advance to Many (complex scenarios). At each ZOM level, BIE cross-cuts apply: check Boundaries, note Interface implications, probe Exceptions.

4. **EP/BVA** (L3) applies within the Exception dimension of ZOMBIES and within each rule's example generation: identify valid and invalid partitions per field, then probe boundaries between partitions. EP always before BVA -- partitions first, then boundary values.

5. **Protester persona** (L4) governs how the AI delivers all of the above: batching related probes, proposing cases for confirmation rather than asking open-ended questions, capturing uncertainty as open questions, knowing when to stop.

6. **CASES.md** (L5) captures everything discovered into a structured, parseable document.

---

## 2. Recommended Discussion Flow

This synthesizes the flows proposed across all five reports into a single concrete flow. Where reports disagreed on ordering, this section resolves the disagreement.

### Phase 0: Init (Pre-conversation)

The AI reads phase context before the conversation starts.

**Inputs to read:**
- `$PHASE_DIR/*-CONTEXT.md` -- locked decisions, discretion areas, deferred items
- `$PHASE_DIR/*-RESEARCH.md` -- technical patterns if available
- `.planning/ROADMAP.md` -- phase description and requirements
- Proto definitions, existing code -- to extract operation signatures

**Outputs of init:**
- List of operations with names, endpoints/RPCs, input/output types
- Known constraints from locked decisions
- Grouped by natural category (CRUD cluster, auth flow, etc.)

### Phase 1: Select (Brief)

Present discovered operations, grouped by category. Developer picks which to discuss.

```
AI: "I found these operations for Phase [X]:

[GROUP NAME]:
  1. OperationA (POST /v1/...)
  2. OperationB (GET /v1/...)

[GROUP NAME]:
  3. OperationC (DELETE /v1/...)

Which operations would you like to discuss?
Enter numbers, 'all', or a group name."
```

**Ordering after selection:** Dependencies first, create before read/update/delete, simple before complex. The AI reorders silently and explains if asked.

### Phase 2: Per-Operation Discussion (The Core)

For each selected operation, run this four-step sequence. This is depth-first: exhaust one operation before moving to the next.

#### Step 2a: Anchor (1-2 minutes)

Establish shared understanding. AI proposes, developer confirms or corrects.

```
AI: "Let's discuss [OperationName] ([endpoint]).

From the context, I understand this operation:
- [purpose]
- [key inputs]
- [key outputs]
- [auth requirement]

Is this accurate? Anything to add or correct?

I see these rules governing this operation:
- R1: [constraint from context/proto/code]
- R2: [constraint]
- R3: [constraint]

Any rules I'm missing?"
```

**Source:** Example Mapping (present Story + propose Rules) combined with GWT Given-forcing ("What must be true before this can happen?").

**Resolution note:** The BDD report suggested starting with degenerate cases before happy path (following Uncle Bob's ordering: exceptional -> degenerate -> ancillary -> core). The ACCEPTANCE-CRITERIA report suggested anchoring with happy path first. **Resolution:** Start with rules (L1 Example Mapping), then progress through ZOMBIES ZOM order which naturally handles degenerate cases (Zero) before happy paths (One). This satisfies both: degenerate probes occur within the Zero step, but the developer first sees the operation framed positively through rules.

#### Step 2b: Success Cases (Brief -- 1-2 minutes)

Define what "working correctly" looks like. Keep this brief -- the developer already knows the happy path. The AI's value is elsewhere.

Apply ZOMBIES Zero -> One:

**Zero:** "What does the system look like before this operation has ever been called? What if there is nothing in the system?"
- Catches initialization behavior, empty-state responses, default values.

**One:** "What does the simplest successful case look like? What is the minimal valid input?"
- Establishes the baseline success case.
- Note interface decisions: what fields in request, what fields in response.

**One (variants):** "Are there variant success cases? Different valid input shapes? Full input with all optional fields?"
- Catches the difference between minimal and full payloads.

**Source:** ZOMBIES Z/O dimensions, Outside-In discovery (start from caller's perspective).

#### Step 2c: Systematic Probing (The bulk -- 5-10 minutes)

This is where the AI adds the most value. Apply techniques in this order, per rule:

**2c-i: Input validation failures (EP per field)**

For each input field in the operation:
1. Identify valid and invalid partitions (EP).
2. Probe boundaries between partitions (BVA -- three-value: at boundary, one inside, one outside).
3. Check cross-field interactions ("Are there combinations of individually valid fields that are invalid together?").

Conversational approach: batch related probes. Do NOT ask about each field individually.

```
AI: "Let me probe the input validation for this operation.

For the title field, I expect these failure cases:
- Missing entirely -> INVALID_ARGUMENT
- Empty string -> INVALID_ARGUMENT
- Whitespace only -> INVALID_ARGUMENT (or do we trim?)
- Over max length -> INVALID_ARGUMENT

What is the max length? And for the boundary: a title at exactly
max length should succeed, correct?

For the tags field:
- Empty array (no tags) -> should this be rejected or accepted?
- Contains duplicates -> reject or deduplicate?
- Contains empty string element -> reject?

Do any of these not apply, or am I missing any?"
```

**2c-ii: Authentication/authorization failures**

Apply EP to auth states: no credentials, malformed, expired, wrong type, valid but wrong role, valid but wrong ownership.

```
AI: "For auth failures:
- F1: No auth token -> UNAUTHENTICATED
- F2: Expired token -> UNAUTHENTICATED
- F3: Valid token but insufficient role -> PERMISSION_DENIED
- F4: Valid token, correct role, but not the resource owner -> [NOT_FOUND or PERMISSION_DENIED? This is a security design decision.]

For F4: should the error distinguish 'not found' from 'not authorized'?"
```

**2c-iii: Resource state failures**

Apply State Transition Testing: what state must the entity be in? What states are invalid?

```
AI: "What state must the [entity] be in for this operation?
What happens if it is in [other state] instead?
What if it was soft-deleted?"
```

For entities with state machines, enumerate valid and invalid transitions. Use the state-event matrix approach: for each state, which operations are valid vs invalid?

**2c-iv: Boundary values (BVA on bounded fields)**

Applied AFTER EP (partitions first, boundaries second). Three-value BVA: boundary value, one inside, one outside.

```
AI: "You said the title allows 3-200 characters.
- 2 chars -> reject?
- 3 chars -> accept (boundary)?
- 200 chars -> accept (boundary)?
- 201 chars -> reject?"
```

Also apply to collection sizes, pagination parameters, numeric ranges.

**2c-v: Concurrency and idempotency**

Apply ZOMBIES Many dimension + Exception cross-cut:

```
AI: "What if this operation is called twice in rapid succession?
- Same user, same input -- is it idempotent?
- Two users, conflicting input -- which wins?
- Client retries after timeout -- safe or dangerous?"
```

**2c-vi: Side effects and data integrity**

```
AI: "What other state changes when this operation succeeds?
- Events emitted?
- Related entities updated?
- Cache invalidated?
- What if the operation partially succeeds then fails?"
```

**2c-vii: Infrastructure failures**

Brief -- these are usually standard patterns:

```
AI: "What if the database is unavailable during this operation?
What if a downstream gRPC service times out?
What error does the caller see?"
```

**Source:** EP/BVA provide 2c-i and 2c-iv. BDD domain-specific categories provide 2c-ii, 2c-iii. ZOMBIES Exception dimension provides 2c-v, 2c-vi, 2c-vii.

**Ordering rationale:** The order follows the EP-BVA report's "most common to most subtle" progression. Input validation catches ~40% of cases, auth ~15%, state ~15%, boundaries ~15%, concurrency/side effects/infra ~15%. The developer stays engaged because the probes move from familiar territory (input validation) toward less obvious territory (concurrency, side effects).

#### Step 2d: Review and Close (1-2 minutes)

```
AI: "Here is what we have for [OperationName]:

Rules: N confirmed
Success cases: M (S1: ..., S2: ...)
Failure cases: K (F1: ..., F2: ...)
Edge cases: J (E1: ..., E2: ...)
Open questions: P
  Q1: [question]
  Q2: [question]

Anything else that could go wrong that we haven't covered?
Any domain-specific risk my systematic probes wouldn't catch?
Any cases to add, remove, or change priority on?"
```

Apply termination heuristics (see Section 4).

### Phase 3: Cross-Operation Concerns (Brief)

After all individual operations are discussed:

```
AI: "Let me check for cross-operation consistency:
- Are error response formats consistent across all operations?
- In operation A, we said [constraint]. Does operation B also enforce this?
- If operation C deletes a resource, how does operation D handle that?"
```

### Phase 4: Write XX-CASES.md

AI produces the structured output document. Present a summary for developer review before finalizing.

---

## 3. Technique Integration Map

### Always Apply (Every Operation)

| Technique | When in Flow | Purpose | Typical Cases Found |
|-----------|-------------|---------|---------------------|
| Example Mapping (Rules extraction) | Step 2a Anchor | Identify constraints governing the operation | 3-7 rules per operation |
| ZOMBIES ZOM progression | Steps 2b-2c | Complexity scaffolding: Zero -> One -> Many | Determines discussion order |
| EP (Equivalence Partitioning) | Step 2c-i, per input field | Identify valid/invalid input categories | 3-8 per parameter |
| BVA (Boundary Value Analysis) | Step 2c-iv, per bounded field | Probe partition boundaries (three-value) | 2-6 per bounded field |
| GWT thinking | Internal, per case | Structure each case as precondition/action/outcome | Not visible to developer |
| Protester persona | Throughout | Challenge assumptions, probe failures | Continuous |

### Conditionally Apply (When Triggered)

| Technique | Trigger Condition | When in Flow | Purpose |
|-----------|-------------------|-------------|---------|
| State Transition Testing | Operation changes entity state | Step 2c-iii | Enumerate valid/invalid state transitions |
| Decision Table | Multiple conditions interact to determine outcome | Step 2c-ii (auth) or 2c-iii | Map condition combinations to outcomes |
| Pairwise Thinking | Operation has 4+ independent parameters | Step 2c-i | Identify suspicious parameter pair interactions |
| Triangulation | Rule is ambiguous from single example | Step 2c general | Determine if 2-3 examples needed to clarify rule |
| TPP (Transformation Priority) | Developer is stuck on ordering | Any step | Suggest next case by complexity progression |
| Inside-Out probing | Domain has complex invariants | After 2c-iii | Discover domain-level edge cases not visible at API boundary |

### Technique Hand-Off Sequence

```
Example Mapping -> extracts Rules -> feeds into...
  ZOMBIES Z -> probes Zero (degenerate/empty) -> feeds into...
    EP -> identifies partitions for each field at Zero level
    BVA -> probes boundaries at Zero level
  ZOMBIES O -> probes One (first meaningful case) -> feeds into...
    EP -> identifies partitions at One level
    BVA -> probes boundaries at One level
    State Transition -> identifies required entity state
  ZOMBIES M -> probes Many (complex scenarios) -> feeds into...
    Concurrency probing
    Pairwise (if many parameters)
    Decision Table (if compound conditions)
  Review -> Triangulation check (enough examples per rule?)
           -> Readiness signals (Example Mapping card-count heuristics)
```

---

## 4. AI Behavior Guidelines

### Batching vs Individual Questioning

**Strong recommendation: batch related probes.** This is the single most important conversational design decision.

Bad (interrogation):
```
"What if title is missing?"
[wait for answer]
"What if title is empty?"
[wait for answer]
"What if title is whitespace?"
[wait for answer]
```

Good (batched proposal):
```
"For the title field, I expect these failure cases:
- Missing -> INVALID_ARGUMENT, 'title is required'
- Empty string -> INVALID_ARGUMENT, 'title must not be empty'
- Whitespace only -> INVALID_ARGUMENT, 'title must not be blank'
- Over max length -> INVALID_ARGUMENT, 'title exceeds maximum'

Do any of these not apply? Am I missing any?"
```

**When to batch:** Within a single probing category (all input validation for one field, all auth failures, all state failures).

**When to ask individually:** Cross-category questions, design decisions with multiple valid answers, cases where the developer's response determines the next question.

### When to Propose vs When to Ask

| Situation | AI Behavior |
|-----------|-------------|
| Behavior follows standard patterns (auth, input validation) | Propose cases, ask for confirmation/correction |
| Behavior is domain-specific or novel | Ask open questions: "What should happen when...?" |
| Developer gave a vague answer | Push for specificity: "What status code? What message?" |
| Outcome has security implications | Ask explicitly: "Should the error distinguish X from Y?" |
| Multiple valid approaches exist | Present options: "Common patterns are A and B. Which fits?" |

### Termination Signals

**Per-rule completion (when to stop probing a rule):**
- Happy path, primary failure, and boundary cases are covered.
- New examples feel like trivial variations of existing ones.
- 4-5 examples per rule is typical; beyond 6, the rule may need splitting.

**Per-operation completion (when to move to next operation):**
- All probing categories visited (input, auth, state, boundary, concurrency, side effects, infrastructure).
- Developer's answers become increasingly confident and terse.
- Case count in expected range: 10-15 for simple CRUD, 20-30 for complex operations.
- If case count exceeds 30, the operation likely needs decomposition.

**Per-operation NOT-READY signal (stop, do not push harder):**
- More than 3-4 open questions for one operation.
- Questions shift from "what should happen" to "how should we implement it."
- Developer repeatedly says "I don't know" or "good question" to fundamental behavior questions.
- Action: flag as "needs-answers" in readiness assessment. Do not force decisions.

**Session-level completion:**
- All selected operations discussed.
- Cross-operation concerns probed.
- CASES.md presented for final review and confirmed.

### Handling "I Don't Know" Responses

1. Capture as explicit open question: "I'll mark this as Q-N: [description]."
2. Offer a recommendation if common practice exists: "Common pattern here is X. Want to go with that, or leave it open?"
3. Never force a decision. Never guess silently and embed the guess as a case.
4. Questions are first-class output. An open question is more valuable than a wrong assumption.
5. Track question count per operation. If it exceeds 3-4, flag readiness concern.

### Protester Persona Calibration

**Intensity should adapt to the operation:**
- Public-facing, security-sensitive operations (auth, registration): high intensity. Probe adversarial scenarios, information leakage, token manipulation.
- Internal CRUD operations: moderate intensity. Focus on validation completeness, state consistency, concurrency.
- Read-only/query operations: lower intensity. Focus on pagination boundaries, empty results, authorization.

**Developer fatigue signals to watch for:**
- Short, impatient answers ("yeah", "same as before", "standard").
- Responses stop adding new information.
- Action: compress remaining probes into a checklist for bulk confirmation.

---

## 5. Output Format Specification

### Document Structure

```markdown
# Phase [XX]: [Name] - Behavioral Cases

**Discovered:** [date]
**Operations covered:** [count]
**Total cases:** S:[count] F:[count] E:[count] Q:[count]

---

## Operation: [OperationName]

**Description:** [what it does, from the caller's perspective]
**Endpoint:** [REST path or gRPC RPC name]
**Auth required:** [none / authenticated / role:X]

### Rules

- R1: [business rule or constraint]
- R2: [validation rule]
- R3: [authorization rule]
- R4: [state precondition]

### Success Cases

| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | [concise name] | [state before] | [what happens] | [result] | must |
| S2 | [concise name] | [state before] | [what happens] | [result] | should |

### Failure Cases

| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| F1 | [concise name] | [state before] | [what happens] | [error + status] | must |
| F2 | [concise name] | [state before] | [what happens] | [error + status] | must |

### Edge Cases

| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| E1 | [concise name] | [state before] | [what happens] | [result] | should |
| E2 | [concise name] | [state before] | [what happens] | [result] | could |

### Open Questions

| ID | Question | Impact | Default Recommendation |
|----|----------|--------|------------------------|
| Q1 | [what is uncertain] | [what it affects] | [suggested default if any] |

---
[Repeat for each operation]
---

## Cross-Operation Concerns

[Cases spanning multiple operations: consistency of error formats,
shared validation rules, cascading effects between operations]

## Summary

| Operation | S | F | E | Q | Readiness |
|-----------|---|---|---|---|-----------|
| [name]    | N | N | N | N | ready / needs-answers |
```

### Case Metadata Fields

| Field | Purpose | Format |
|-------|---------|--------|
| **ID** | Unique within operation, prefixed by type | S1, F3, E2, Q1 |
| **Case** | Concise name describing the scenario | "Empty title submitted", "Expired token used" |
| **Preconditions** | State that must exist BEFORE the action | "User has valid JWT, book exists in draft state" |
| **Action** | The specific trigger | "POST /v1/books with empty title field" |
| **Expected Outcome** | Observable result | "400 INVALID_ARGUMENT with field error for 'title'" |
| **Priority** | Risk-adjusted importance | must / should / could |

### Priority Levels

| Priority | Meaning | Typical Cases |
|----------|---------|---------------|
| **must** | Blocks release. Core behavior that MUST be tested. | Happy path, auth enforcement, required-field validation, data integrity, security boundaries |
| **should** | Should be tested for production quality. | All error codes, boundary values, concurrent writes, idempotency |
| **could** | If time permits. Nice-to-have coverage. | Unicode edge cases, performance bounds, obscure input combinations |

Risk factors that increase priority: data loss potential, security impact, user-facing frequency, blast radius, irreversibility.

### How XX-CASES.md Feeds Into Downstream Workflow

```
discuss-phase -> CONTEXT.md (locked decisions)
     |
     v
/case skill -> XX-CASES.md (behavioral specification)
     |
     v
plan-phase -> XX-PLAN.md (maps cases to test tasks:
     |          each must-priority case = required test,
     |          case IDs referenced in task acceptance criteria)
     v
execute -> implementation + tests
```

The planner uses CASES.md to:
- Map must-priority cases to required test tasks in PLAN.md.
- Identify test infrastructure needs from open questions.
- Structure test files by case category (success, failure, edge).
- Set acceptance criteria referencing specific case IDs.

---

## 6. Key Decisions and Open Questions

### Resolved Disagreements Between Reports

**1. Starting point: degenerate cases vs happy path first?**

The TDD report (citing Uncle Bob) argues for degenerate/exceptional cases first. The BDD report and ACCEPTANCE-CRITERIA report argue for happy path anchoring first. The Example Mapping report argues for rules extraction first.

**Resolution:** Start with rules extraction (Example Mapping), then progress through ZOMBIES ZOM order. Zero naturally covers degenerate cases. One covers the happy path. This satisfies all three perspectives: rules give structure, degenerate probes come early via Zero, and the happy path is established in One before diving into systematic failure probing.

**2. How deeply to probe infrastructure failures?**

The BDD report categorizes infrastructure failures as "low priority for case discussion, high for resilience." The TDD report's ZOMBIES Exception dimension includes them at every ZOM level.

**Resolution:** Infrastructure failures get a brief, standardized probe at the end of each operation (Step 2c-vii). The cases are usually the same across operations ("database down -> UNAVAILABLE, downstream service down -> UNAVAILABLE"), so they do not need deep per-operation discussion. Capture once, note as cross-cutting.

**3. Pairwise testing: when to apply?**

The EP-BVA report recommends pairwise for 4+ independent parameters. The TDD report does not mention it. The ACCEPTANCE-CRITERIA report mentions it only for search/filter endpoints.

**Resolution:** Pairwise is a conditional technique, applied only for search/filter/list endpoints with many independent parameters. For typical CRUD operations, EP per field plus cross-field interaction probing is sufficient. The AI should ask about pairwise only when it notices 4+ independent filter parameters.

**4. Case ID naming convention**

The TDD report uses S1/F1/E1/Q1 (type prefix + number). The BDD report does not specify a convention. The ACCEPTANCE-CRITERIA report uses the same S/F/E/Q convention.

**Resolution:** Use S/F/E/Q prefix + sequential number per operation. This is clear, parseable, and already agreed upon by two reports.

### Open Questions for User Input Before Skill Implementation

**Q1: Scope of a single /case session -- one phase or arbitrary?**

Research assumes per-phase operation. Should the skill also support ad-hoc case discovery for a single operation outside a phase context?

**Q2: How should the skill handle operations that span multiple services?**

Example: "Register user" involves auth service (passkey ceremony) and user service (profile creation). Should these be treated as one operation or two?

**Q3: Should the skill produce GWT-formatted cases or the flat table format?**

Research recommends flat tables (ID, Case, Preconditions, Action, Expected Outcome, Priority) as the primary format because they are more scannable and parseable. GWT is used only internally as a thinking framework. Confirm this preference.

**Q4: Should the AI auto-assign priority, or defer to the developer?**

Research suggests the AI should propose priorities (using risk heuristics: security, data loss, user frequency) and let the developer override. Confirm this approach.

**Q5: Maximum operations per session?**

Example Mapping research suggests 25-30 minutes per operation. For a full phase with 8-10 operations, that is 3-4 hours. Should the skill support splitting across multiple sessions?

### Risks and Limitations

**Risk 1: Happy path fixation.** The developer may rush through failure cases if the AI does not actively steer toward them. Mitigation: the Protester persona and ZOMBIES Exception dimension are designed to counter this.

**Risk 2: Interrogation fatigue.** Too many individual questions exhaust the developer. Mitigation: batch probes, propose-then-confirm, compress when fatigue signals appear.

**Risk 3: False completeness.** A well-structured case list can look comprehensive while missing domain-specific risks that no systematic technique would surface. Mitigation: always ask the closing question "What domain-specific thing could go wrong that my probes wouldn't catch?"

**Risk 4: Scope creep into implementation.** Discussions can drift from "what should happen" to "how to implement it." Mitigation: redirect explicitly: "That is an implementation detail. For now, what should the caller observe?"

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Session structure (Example Mapping) | HIGH | Primary sources from Matt Wynne and Cucumber official docs. Well-established technique. |
| Behavioral thinking (BDD/GWT) | HIGH | Dan North, Martin Fowler, Cucumber docs. Foundational and well-documented. |
| Systematic coverage (EP/BVA) | HIGH | ISTQB syllabus v4.0, NIST research on pairwise effectiveness. Authoritative sources. |
| Complexity progression (ZOMBIES) | HIGH | James Grenning primary source, O'Reilly article, multiple community walkthroughs. |
| AI conversation design | MEDIUM | Synthesized from AI-assisted RE research (arXiv), Socratic method research, and practical adaptation. Less battle-tested than the testing techniques themselves. |
| Output format (CASES.md) | MEDIUM | Designed from research principles, not validated in practice. Format will likely evolve after first few sessions. |

**Overall confidence:** HIGH for the technique layer, MEDIUM for the conversational execution layer.

**Primary gap:** The AI conversation design has not been validated against real developer sessions. The batching heuristics, termination signals, and fatigue detection are synthesized from testing community principles but have not been tested in the specific context of an AI-developer case discovery conversation. Expect iteration after the first few uses.

---

## Sources

Aggregated from all five research reports. See individual reports for complete source lists.

### Primary (HIGH confidence -- technique creators and authoritative standards)

- Matt Wynne -- Example Mapping (Cucumber Blog, original technique)
- Dan North -- BDD, Deliberate Discovery (dannorth.net)
- Martin Fowler -- Given/When/Then (martinfowler.com/bliki)
- James Grenning -- ZOMBIES (blog.wingman-sw.com, O'Reilly)
- Robert C. Martin -- Transformation Priority Premise (Clean Coder Blog)
- Kent Beck -- TDD by Example, Triangulation, Canon TDD (Substack)
- ISTQB CTFL Syllabus v4.0.1 -- EP, BVA, Decision Tables, State Transitions
- NIST SP 800-142 -- Combinatorial/pairwise testing effectiveness data
- John Ferguson Smart -- Three Amigos, Feature Mapping

### Secondary (MEDIUM confidence -- community synthesis and practitioner reports)

- Cucumber official documentation (bdd, gherkin, discovery workshops)
- Gaspar Nagy / Seb Rose -- BRIEF principle (Formulation book)
- Liz Keogh -- Negative scenarios in BDD
- Michael Bolton / James Bach -- Stopping heuristics (DevelopSense, Satisfice)
- Steve Freeman / Nat Pryce -- GOOS, Outside-In TDD
- Antony Sallas -- Three AI Amigos (Medium, Jan 2026)
- arXiv papers on AI-assisted requirements elicitation, Socratic method for LLMs

---

*Synthesis completed: 2026-03-24*
*Ready for skill implementation: yes, pending resolution of open questions Q1-Q5*
