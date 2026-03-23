# AI-Assisted Behavioral Specification & Conversational Case Discovery

**Research date:** 2026-03-24
**Domain:** AI-guided specification discussions, conversational case discovery, structured output for test planning
**Confidence:** HIGH (synthesized from five dedicated research reports + supplementary web research)

## Summary

This document synthesizes research on how an AI agent should conduct conversational behavioral specification sessions with a developer. The `/case` skill's purpose is to guide a structured discussion that surfaces success cases, failure cases, and edge cases for each operation in a software feature -- BEFORE any tests are written. The output is a `XX-CASES.md` document consumable by both humans and AI test writers.

The research draws from five completed sub-reports (BDD case discovery, Example Mapping, TDD techniques, Equivalence Partitioning/BVA, and Acceptance Criteria writing) plus supplementary research on AI conversation design, the Socratic method applied to LLMs, stopping heuristics from exploratory testing, and AI-assisted requirements elicitation patterns.

**Primary recommendation:** The AI agent should adopt a "Protester" persona (from Three Amigos) combined with Example Mapping's Rule-to-Example flow, using ZOMBIES (Zero/One/Many + Boundary/Interface/Exception) as the primary progression framework, operating depth-first per operation with structured output in a prioritized case format.

---

## 1. The AI as Tester/Protester

### 1.1 The Protester Persona

The "Protester" role comes from Three Amigos practice (business/developer/tester triad). The tester is "the one who protests" -- systematically looking for hidden assumptions, edge cases, and inconsistencies. In a 1-on-1 AI-developer conversation, the AI must play this role because the developer naturally occupies the developer/builder perspective.

**Core mindset traits:**

| Trait | Description | Manifestation |
|-------|-------------|---------------|
| Systematic doubt | Question every stated behavior for unstated alternatives | "You said it returns 404 -- but what if the resource exists but is soft-deleted?" |
| Boundary awareness | Probe limits, extremes, and transitions | "What is the maximum? What happens at exactly the maximum? One above?" |
| Failure-first thinking | Assume things will go wrong and ask how | "What if the database is unreachable during this operation?" |
| Assumption surfacing | Make implicit knowledge explicit | "You mentioned 'authenticated user' -- what token type? What claims?" |
| Adversarial creativity | Imagine misuse and abuse scenarios | "Can a user manipulate the ID parameter to access another user's data?" |

**Key behavioral rule:** The AI should not accept the first answer. When the developer says "it returns an error," the AI probes further: "What kind of error? What status code? What message? Is it distinguishable from other errors? Does it leak information?"

### 1.2 Question Types

The agent uses five question categories, applied in context:

**Clarifying questions** -- establish shared understanding:
- "What does this operation do from the caller's perspective?"
- "What are the required vs optional inputs?"
- "What does the success response look like exactly?"

**Challenging questions** -- surface hidden assumptions:
- "You said 'the user must be logged in' -- what kind of authentication? Is service-to-service also valid?"
- "You assumed the resource exists -- what if it was just deleted by another request?"
- "Earlier you said X, but now you're saying Y -- how do those interact?"

**Boundary-probing questions** -- find edges where behavior changes:
- "What is the minimum valid value? What happens at one below minimum?"
- "What if the list has zero results? Exactly one? Ten thousand?"
- "What if the string is exactly at the maximum length? One character over?"

**Failure-injecting questions** -- explore error paths:
- "What if the database times out during this operation?"
- "What if this request arrives with no credentials? With expired credentials?"
- "What if a downstream service returns an unexpected error?"

**State-probing questions** -- discover state dependencies:
- "What state must the entity be in for this to succeed?"
- "What happens if the entity is in a deleted/archived/locked state?"
- "If this operation changes state, what other operations become possible or impossible?"

### 1.3 Balancing Thoroughness with Developer Experience

The AI must not become adversarial to the point of annoyance. Practical heuristics:

- **Spend proportional effort.** Happy path: brief (developer already knows this). Failure and edge cases: thorough (this is where value is added).
- **Group related probes.** Don't ask about missing fields one by one -- ask about the input validation category, then move on.
- **Read the room.** If the developer gives short, impatient answers, compress remaining probes into a checklist for confirmation rather than individual questions.
- **Propose, don't interrogate.** "I expect these failure cases: [list]. Do any of these not apply, or am I missing any?" is better than asking twenty separate questions.
- **Accept "I don't know" gracefully.** Capture it as an open question and move on. Never force a decision.

---

## 2. Conversational Flow Design

### 2.1 Depth-First per Operation

The skill operates **depth-first**: exhaust all cases for one operation before moving to the next. Rationale:
- Each operation has its own state space, inputs, and failure modes
- Context-switching between operations loses conversational momentum
- Depth-first produces a complete, reviewable case set per operation
- Breadth-first (survey all operations first) only makes sense for the initial selection phase

### 2.2 The Four-Phase Discussion Structure per Operation

```
PHASE 1: ANCHOR (Establish Shared Understanding)
  Goal: Both parties agree on what the operation does
  Duration: Brief

  "Let me describe what I understand about [operation].
   [AI states the operation, its inputs, outputs, and purpose from context].
   Is this accurate? Anything to add or correct?"

  Then: "What are the rules or constraints governing this operation?"
  → Captures the "blue cards" (rules) from Example Mapping

PHASE 2: SUCCESS CASES (Happy Path + Variants)
  Goal: Define what "working correctly" looks like
  Duration: Brief (developer knows this well)

  "What does the simplest successful case look like?"
  "Are there variant success cases? Different valid input shapes?"
  "What exactly does the response contain on success?"
  → Captures S-series cases

PHASE 3: SYSTEMATIC PROBING (Failures + Edges)
  Goal: Surface everything that can go wrong
  Duration: The majority of the discussion

  Applied in this order (from EP-BVA and TDD research):
  a. Input validation failures (EP per field)
  b. Authentication/authorization failures
  c. Resource state failures (not found, wrong state, conflict)
  d. Boundary values (BVA on bounded fields)
  e. Concurrency (double-submit, race conditions, idempotency)
  f. Side effects and data integrity
  g. Infrastructure failures (dependency down, timeout)
  → Captures F-series and E-series cases

PHASE 4: REVIEW AND CLOSE
  Goal: Confirm completeness, capture open questions
  Duration: Brief

  AI presents a summary: "We found N success cases, M failure cases,
  K edge cases, and J open questions for [operation]."
  "Is there anything else that could go wrong that we haven't covered?"
  "Any cases you want to add, remove, or change priority on?"
  → Final confirmation before moving to next operation
```

### 2.3 Progressive Deepening within Phase 3

Phase 3 uses ZOMBIES progression combined with category sweeps:

```
For each probing category (input, auth, state, boundary, concurrency, etc.):
  1. Zero case: "What if there is nothing? (no input, no entity, no credentials)"
  2. One case:  "What about the first/simplest failure in this category?"
  3. Many case: "What about combinations or complex scenarios?"
  4. Boundary:  "Where does behavior change in this category?"
```

### 2.4 Handling "I Don't Know" Responses

When the developer is uncertain:

1. **Capture as explicit question** -- "I'll mark this as an open question: [Q-XX: description]"
2. **Offer a recommendation if possible** -- "Common practice in this situation is X. Want to go with that, or leave it open?"
3. **Never force a decision** -- Questions are first-class output. An open question is more valuable than a wrong assumption.
4. **Track question count** -- Many questions for one operation signals it's not ready for implementation. Flag this to the developer.

### 2.5 Cross-Operation Awareness

After completing individual operations, the AI should briefly probe cross-operation interactions:
- "In operation A, we said email must be unique. Does operation B also enforce this?"
- "If operation C deletes a resource, how does operation D's listing handle that?"
- "Are error response formats consistent across all operations?"

---

## 3. Structured Output: The CASES.md Format

### 3.1 Design Principles

The output document must serve two audiences:
1. **Human readers** -- scannable, grouped by category, with enough context to understand each case
2. **AI test writers** -- parseable structure with operation scope, case ID, preconditions, action, and expected outcome

### 3.2 Document Structure

```markdown
# Phase [X]: [Name] - Behavioral Cases

**Discovered:** [date]
**Operations covered:** [count]
**Total cases:** S:[count] F:[count] E:[count] Q:[count]

## Operation: [OperationName]

**Description:** [what it does]
**Endpoint:** [REST path or gRPC RPC name]
**Auth required:** [yes/no, what level]

### Rules
- R1: [business rule or constraint]
- R2: [business rule or constraint]
- ...

### Success Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | [name] | [state before] | [what happens] | [result] | must |
| S2 | [name] | [state before] | [what happens] | [result] | should |

### Failure Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| F1 | [name] | [state before] | [what happens] | [error + status] | must |
| F2 | [name] | [state before] | [what happens] | [error + status] | must |

### Edge Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| E1 | [name] | [state before] | [what happens] | [result] | should |
| E2 | [name] | [state before] | [what happens] | [result] | could |

### Open Questions
| ID | Question | Impact | Default Recommendation |
|----|----------|--------|----------------------|
| Q1 | [what is uncertain] | [what it affects] | [suggested default if any] |

---
[Repeat for each operation]
---

## Cross-Operation Concerns
[Any cases that span multiple operations]

## Summary
| Operation | S | F | E | Q | Readiness |
|-----------|---|---|---|---|-----------|
| [name] | N | N | N | N | ready / needs-answers |
```

### 3.3 Case Metadata Fields

Each case captures:

| Field | Purpose | Example |
|-------|---------|---------|
| **ID** | Unique within operation, prefixed by type | S1, F3, E2, Q1 |
| **Case** | Concise name using "The one where..." convention | "Empty title submitted" |
| **Preconditions** | State that must exist BEFORE the action | "User has valid JWT, book exists, book is in 'draft' state" |
| **Action** | The specific trigger | "POST /v1/books with empty title field" |
| **Expected Outcome** | Observable result | "400 INVALID_ARGUMENT with field error for 'title'" |
| **Priority** | must / should / could | Drives test implementation order |

### 3.4 Priority Levels

Adapted MoSCoW for cases (from Acceptance Criteria research):

| Priority | Meaning | Examples |
|----------|---------|---------|
| **must** | Blocks release. Core behavior that MUST be tested. | Happy path, auth enforcement, required-field validation, data integrity |
| **should** | High confidence. Should be tested for production quality. | All error codes, boundary values, concurrent writes, idempotency |
| **could** | If time permits. Nice-to-have coverage. | Unicode edge cases, performance bounds, obscure input combos |

Risk factors that increase priority: data loss potential, security impact, user frequency, blast radius, irreversibility.

### 3.5 Rules Section

Each operation's rules section captures the "blue cards" from Example Mapping:
- Business rules (authorization, ownership, uniqueness)
- Validation rules (required fields, format constraints, ranges)
- State rules (what state the entity must be in)
- Invariant rules (things that must always or never be true)

Rules contextualize cases and make the document self-contained.

---

## 4. Multi-Select Operation Flow

### 4.1 Presenting Operations

When a phase has multiple operations, the AI should:

1. **Extract operations from context** -- parse phase CONTEXT.md, PLAN.md files, or REQUIREMENTS.md to identify all operations
2. **Group by natural category** -- CRUD groups, auth flows, admin operations, query operations
3. **Present with context** -- show operation name, brief description, and suggested grouping

Example presentation:
```
I found these operations for Phase 3 Authentication:

REGISTRATION FLOW:
  1. Register Begin (POST /v1/auth/register/begin)
  2. Register Finish (POST /v1/auth/register/finish)

LOGIN FLOW:
  3. Login Begin (POST /v1/auth/login/begin)
  4. Login Finish (POST /v1/auth/login/finish)

SESSION MANAGEMENT:
  5. Refresh Token (POST /v1/auth/refresh)
  6. Logout (POST /v1/auth/logout)
  7. Logout All Sessions (POST /v1/auth/sessions/logout-all)

PASSKEY MANAGEMENT:
  8. List Passkeys (GET /v1/auth/passkeys)
  9. Rename Passkey (PATCH /v1/auth/passkeys/{id})
  10. Delete Passkey (DELETE /v1/auth/passkeys/{id})

Which operations would you like to discuss? (Enter numbers, "all", or a
group name like "registration flow")
```

### 4.2 Grouping Heuristics

| Group Type | Pattern | Example |
|------------|---------|---------|
| **Flow sequence** | Begin/Finish pairs, multi-step workflows | Register begin + finish |
| **CRUD cluster** | Create/Read/Update/Delete on same resource | Passkey list/rename/delete |
| **Auth level** | Group by required auth tier | Public, authenticated, verified |
| **Dependency chain** | Operations that must succeed in order | Register -> Login -> Access |

### 4.3 Discussion Ordering

After selection, reorder operations for logical discussion flow:

1. **Dependencies first** -- discuss operations that other operations depend on first
2. **Create before read/update/delete** -- the creation case establishes the entity's shape
3. **Happy flow order** -- follow the natural user journey (register -> login -> use)
4. **Complex after simple** -- discuss simpler operations first to build shared vocabulary

### 4.4 Skip and Resume

- Allow skipping operations already specified elsewhere
- Allow returning to a previously discussed operation to add cases
- Allow deferring an operation: "Let's come back to this one after we discuss [other]"

---

## 5. Discussion Termination Heuristics

### 5.1 Per-Operation Completion Signals

Drawing from Michael Bolton's stopping heuristics for exploratory testing and Example Mapping's session conventions:

**Positive signals (operation is well-specified):**

| Signal | Description |
|--------|-------------|
| **Category saturation** | All systematic probing categories (input, auth, state, boundary, concurrency, infrastructure) have been visited |
| **No new partitions** | New questions produce cases that are trivial variations of existing ones |
| **Question convergence** | Developer's answers are increasingly confident and specific |
| **Balanced card table** | Roughly proportional: few rules, several examples per rule, few or no open questions |

**Negative signals (stop and flag, NOT push harder):**

| Signal | Description |
|--------|-------------|
| **Question explosion** | Too many open questions (> 3-4) for one operation. Operation is not ready for implementation. |
| **Implementation drift** | Questions shift from "what should happen" to "how should we implement it" -- discussion has left specification territory |
| **Repetition loop** | Same cases being restated in different words |
| **Diminishing returns** | Each new probe yields "same as above" or "standard behavior" answers |

### 5.2 The Termination Protocol

When completion signals appear:

```
1. AI announces: "I think we've covered [operation] well. Let me summarize."
2. AI presents the case count: "N success, M failure, K edge, J questions."
3. AI asks the closing question: "Anything else that could go wrong that
   we haven't covered? Any cases to add or change?"
4. Developer confirms or adds final cases.
5. AI transitions: "Moving on to [next operation]."
```

### 5.3 Time-Boxing Heuristic

Example Mapping research recommends 25-30 minutes per operation. In a conversational AI context, approximate this by:
- If the operation is simple (single CRUD, no complex state): aim for 10-15 cases total
- If the operation is complex (multi-step, state transitions, multiple roles): aim for 20-30 cases
- If case count exceeds 30, the operation may need decomposition

### 5.4 Session-Level Termination

For the entire `/case` session (covering multiple operations):
- All selected operations have been discussed
- Cross-operation concerns have been probed
- The CASES.md document has been presented for final review
- Developer has confirmed or requested revisions

---

## 6. Technique Application Reference

### 6.1 Primary Frameworks (Always Apply)

**Example Mapping flow (per operation):**
Story (operation) -> Rules (constraints) -> Examples (concrete cases) -> Questions (unknowns)

**ZOMBIES progression (per probing category):**
Zero (nothing/empty) -> One (simplest meaningful) -> Many (complex/multiple) + Boundaries + Interface + Exceptions at each level

**Equivalence Partitioning (per input field):**
Identify valid and invalid classes. Test one representative per class.

**Boundary Value Analysis (per bounded field):**
Test at min, min-1, max, max+1. Ask: "What happens at exactly the limit?"

### 6.2 Secondary Frameworks (Apply When Triggered)

**State Transition Testing** -- when operation changes entity state:
- "What state must the entity be in?"
- "What are all the invalid states for this operation?"
- "What happens with concurrent state transitions?"

**Decision Table Testing** -- when multiple conditions interact:
- "Do role + ownership + resource state combine to determine the outcome?"
- "Let me map out the authorization matrix for this."

**Pairwise Thinking** -- when operation has many independent parameters:
- "Which pairs of parameter values might interact unexpectedly?"
- Most useful for search/filter/list endpoints

### 6.3 Probing Category Checklist

For each operation, the agent should visit every category. The order is designed to go from most common to most subtle:

```
[ ] Input validation: missing, empty, wrong type, wrong format, too long, too short
[ ] Authentication: no creds, expired, malformed, wrong type
[ ] Authorization: wrong role, wrong ownership, escalation attempt
[ ] Resource state: not found, soft-deleted, wrong lifecycle state
[ ] Boundary values: min, max, zero, negative, at-limit, over-limit
[ ] Concurrency: double-submit, race condition, idempotency
[ ] Side effects: event emission, related entity changes, partial failure
[ ] Data integrity: FK violations, unique constraint, cascade behavior
[ ] Infrastructure: DB down, service unavailable, timeout
[ ] Response contract: correct status code, correct body shape, correct error format
```

---

## 7. Concrete Dialogue Examples

### 7.1 Opening an Operation Discussion

```
AI: "Let's discuss the Register Begin operation (POST /v1/auth/register/begin).

From the context, I understand this operation:
- Accepts an invite token in the request body
- Validates the token (exists, not expired, not used)
- Returns a WebAuthn challenge for passkey registration
- Requires no authentication (public endpoint)

Is this accurate? Any corrections or additions?"

Developer: "Yes, that's right. The user also provides their name and handle
in the begin step."

AI: "Got it. So the inputs are: invite_token, name, and handle. Let me
capture the rules governing this operation:
- R1: Invite token must exist, not be expired, and not be previously used
- R2: Name must meet validation rules (non-empty, max length)
- R3: Handle must be unique, URL-safe, within length bounds
- R4: No authentication required (public endpoint)

Any rules I'm missing?"
```

### 7.2 Probing Failure Cases

```
AI: "Let's explore what can go wrong with the invite token (R1).

For token validation, I see these failure cases:
- F1: Token is missing from request body -> 400 INVALID_ARGUMENT
- F2: Token string doesn't match any invite -> 404 NOT_FOUND (or 400?)
- F3: Token exists but has expired (past 30-minute window) -> 400/410
- F4: Token has already been used -> 400/410/409

A few questions:
1. For F2 and F3, should the error distinguish 'not found' from 'expired',
   or use a generic message to avoid token enumeration?
2. For F4, what status code fits best -- 400, 409, or 410 Gone?"

Developer: "Generic message for all -- 'invalid or expired token'. Use 400."

AI: "Good. So F2, F3, F4 all collapse to the same error response:
400 INVALID_ARGUMENT 'invalid or expired token'. That's a security
decision worth noting.

Now for the handle (R3): what happens if the handle is already taken
by another user?"
```

### 7.3 Surfacing Edge Cases

```
AI: "A few edge cases for this operation:

- E1: User submits register/begin, gets the challenge, but never calls
  register/finish. The challenge expires. Then they call register/begin
  again with the same token. Should this work?

- E2: Two users receive different invite tokens. Both try to register
  with the same handle simultaneously. One should succeed, the other
  should get a 409 Conflict for the handle.

- E3: What if the invite token was cancelled (revoked) by an admin
  between when the user received it and when they submit register/begin?"

Developer: "E1: Yes, re-calling begin should work as long as the token
hasn't expired. E2: Correct, database unique constraint handles this.
E3: Good catch -- cancelled tokens should fail with the same generic error."
```

### 7.4 Closing an Operation

```
AI: "Let me summarize Register Begin:

Rules: 4
Success cases: 2 (minimal input, full input with all optional fields)
Failure cases: 7 (3 token failures, 2 handle failures, 2 name failures)
Edge cases: 4 (re-begin after timeout, concurrent handles, cancelled token,
             unicode in handle)
Open questions: 1 (Q1: should begin step reserve the handle temporarily
                to prevent race conditions?)

Anything else that could go wrong? Any cases to add or change?"

Developer: "That looks comprehensive. Let's move on."
```

---

## 8. Integration with GSD Workflow

### 8.1 Position in the Workflow

```
discuss-phase -> CONTEXT.md (locked decisions)
     |
     v
/case skill -> XX-CASES.md (behavioral specification)
     |
     v
plan-phase -> XX-PLAN.md (implementation tasks with test references)
     |
     v
execute -> implementation + tests
```

The `/case` skill reads from CONTEXT.md and phase REQUIREMENTS to understand what has been decided. Its output (CASES.md) feeds into plan-phase, where each case becomes a test to include in implementation tasks.

### 8.2 Reading Phase Context

The skill should:
1. Read `$PHASE_DIR/*-CONTEXT.md` for locked decisions, discretion areas, and deferred items
2. Read `$PHASE_DIR/*-RESEARCH.md` if it exists for technical patterns
3. Read `.planning/ROADMAP.md` for phase description and requirements
4. Extract operations from the decisions and requirements

**Locked decisions constrain case discovery.** If CONTEXT.md says "use generic error message for token failures," the skill does not explore alternatives.

### 8.3 Output Location and Naming

```
.planning/phases/XX-name/XX-CASES.md
```

Follows the same pattern as CONTEXT.md, RESEARCH.md, and PLAN.md files.

### 8.4 How the Planner Consumes CASES.md

The planner uses CASES.md to:
- **Map cases to test tasks** -- each must-priority case becomes a required test in a PLAN.md
- **Identify test infrastructure needs** -- open questions may need resolution before planning
- **Structure test files** -- success, failure, and edge case groupings inform test module organization
- **Set acceptance criteria** -- each PLAN.md task can reference specific case IDs

---

## 9. Existing Tools and Research

### 9.1 AI-Assisted Specification Tools

| Tool/Approach | What It Does | Relevance to /case |
|---------------|--------------|-------------------|
| **GitHub Copilot test generation** | Generates test code from existing code context | Generates tests from code (post-hoc), not behavioral specs (pre-code). Produces good structure but misses edge cases and security concerns. |
| **ChatPRD + MCP elicitation** | Interactive dialogs for requirement clarification during AI processing | Similar concept: AI asks clarifying questions. But focused on product requirements, not behavioral test cases. |
| **Spec Kit (GitHub)** | Structured spec -> detailed plan -> code generation | Complementary: provides the spec format. /case provides the behavioral cases that enrich the spec. |
| **RECOVER (NLP for RE)** | Extracts requirements from stakeholder conversations | Post-hoc extraction from meetings. /case is proactive: guides the conversation to surface cases. |
| **Multi-agent RE** | Multiple AI models generating and assessing user stories | Academic approach. Useful insight: multiple "perspectives" (roles) improve requirement quality. /case applies this by playing multiple roles. |

### 9.2 Key Research Insights

**The Socratic method applied to LLMs** (Princeton NLP): Rather than fixed prompts, AI agents can use free-form Socratic dialogue to self-discover knowledge. Applied to /case: the AI should ask genuine questions (not rhetorical ones) that drive toward shared understanding.

**Counterfactual reasoning**: "What if" questions are a form of counterfactual thinking that LLMs handle well. This aligns with the Protester role's core technique of systematically asking "what if [assumption] is wrong?"

**Copilot's weakness is the /case skill's strength**: Research shows Copilot generates structurally good tests but misses edge cases, security concerns, and cross-operation interactions (29% of generated Python code had security weaknesses). The /case skill addresses this gap by discovering cases BEFORE code, not from code.

**Diminishing returns in testing**: Nielsen's usability research shows 5 evaluators find ~75% of issues, with diminishing returns after that. Similarly, in case discovery, the first 10-15 cases per operation capture the majority of important behaviors. The stopping heuristics in Section 5 apply this principle.

---

## 10. Question Prompt Templates by Discovery Phase

### Phase 1: Anchor

```
"Let me describe what I understand about [OPERATION]. [description]. Is this accurate?"
"What are the rules or constraints governing this operation?"
"Who calls this? What triggers it?"
"What are the inputs? Which are required vs optional?"
"What does the response contain on success?"
```

### Phase 2: Success Cases

```
"What is the simplest successful case?"
"Are there variant success cases with different valid inputs?"
"Does the caller's identity matter for the success response shape?"
"What side effects should occur on success? (events, state changes)"
```

### Phase 3a: Input Validation (EP)

```
"For [FIELD]: what if it's missing? Empty string? Wrong type?"
"What format does [FIELD] require? What's an example of invalid format?"
"Are there multiple validation errors at once? Which one takes precedence?"
"What about unexpected/extra fields in the request body?"
```

### Phase 3b: Auth Failures

```
"What if there are no credentials at all?"
"What if the token is expired? Malformed?"
"What if the caller has the wrong role for this operation?"
"What if the caller tries to access another user's resource?"
```

### Phase 3c: State Conflicts

```
"What if the target resource doesn't exist?"
"What if it exists but was soft-deleted/deactivated?"
"What if it's in a state that makes this operation invalid?"
"What if the resource was just modified by another request? (stale data)"
```

### Phase 3d: Boundaries (BVA)

```
"What are the minimum and maximum for [FIELD]?"
"What happens at exactly the minimum? One below?"
"What about zero items? One item? Maximum items? One over maximum?"
"What about the first page? The last page? A page beyond the data?"
```

### Phase 3e: Concurrency

```
"What if this exact request is submitted twice simultaneously?"
"Is this operation idempotent? Safe to retry?"
"Can two users perform conflicting operations on the same resource?"
"What uniqueness constraints could be violated by concurrent requests?"
```

### Phase 3f: Infrastructure

```
"What if the database is unreachable?"
"What if a downstream gRPC service is unavailable?"
"What error does the caller see for infrastructure failures?"
```

### Phase 4: Close

```
"Here is the summary: [N] success, [M] failure, [K] edge, [J] questions."
"Anything else that could go wrong that we haven't covered?"
"Any cases to add, remove, or reprioritize?"
"Ready to move to the next operation?"
```

---

## 11. Common Pitfalls for the AI Agent

### Pitfall 1: Happy Path Fixation
**What goes wrong:** Spending too long on success cases where the developer already has clear understanding.
**How to avoid:** Keep Phase 2 brief. The agent's value is in Phases 3 and 4.

### Pitfall 2: Interrogation Mode
**What goes wrong:** Asking too many individual questions feels like an interrogation. Developer disengages.
**How to avoid:** Batch related probes. Propose cases in groups and ask for confirmation/correction rather than asking open questions for each.

### Pitfall 3: Implementation Leakage
**What goes wrong:** Discussion drifts from "what should happen" to "how should we implement it."
**How to avoid:** Redirect: "That's an implementation detail we can decide later. For now, what should the caller observe?"

### Pitfall 4: Assumed Universals
**What goes wrong:** AI assumes every operation needs every category of probe (e.g., asking about pagination on a create endpoint).
**How to avoid:** Match probing categories to operation type. Not every category applies to every operation.

### Pitfall 5: False Completeness
**What goes wrong:** AI presents a comprehensive-looking list that actually misses domain-specific cases the developer would catch if asked.
**How to avoid:** Always ask the closing question: "What domain-specific thing could go wrong that my systematic probes wouldn't catch?"

### Pitfall 6: Question Avoidance
**What goes wrong:** AI guesses at answers to avoid creating open questions, leading to incorrect assumptions baked into cases.
**How to avoid:** Questions are first-class output. An honest "Q-XX: we don't know this yet" is more valuable than a wrong case.

---

## Sources

### Primary Sources (HIGH confidence)

**Project research files (completed sub-reports):**
- `.planning/research/BDD-CASE-DISCOVERY.md` -- GWT format, Gherkin, Three Amigos, BDD discovery process, conversation flow patterns
- `.planning/research/example-mapping.md` -- Example Mapping fundamentals, rules-to-examples flow, questioning techniques, AI facilitation design
- `.planning/research/TDD-CASE-DISCOVERY.md` -- Test list technique, ZOMBIES, TPP, degenerate case checklist, outside-in vs inside-out discovery
- `.planning/research/EP-BVA-TECHNIQUES.md` -- Equivalence partitioning, boundary value analysis, decision tables, state transitions, pairwise testing, conversational question patterns
- `.planning/research/ACCEPTANCE-CRITERIA-TECHNIQUES.md` (previous version) -- Acceptance criteria formats, INVEST criteria, decomposition techniques, blind spot taxonomy, quality attributes

### Secondary Sources (MEDIUM confidence)

**Three Amigos / Protester Role:**
- [Three Amigos Requirements Discovery - John Ferguson Smart](https://johnfergusonsmart.com/three-amigos-requirements-discovery/)
- [Key Role Testing Plays in Three Amigos - TechTarget](https://www.techtarget.com/searchsoftwarequality/tip/The-key-role-testing-plays-in-a-Three-Amigos-strategy)
- [Three Amigos in Agile - Agile Alliance](https://agilealliance.org/glossary/three-amigos/)
- [Behavior-Driven Three Amigos - Automation Panda](https://automationpanda.com/2017/02/20/the-behavior-driven-three-amigos/)

**AI-Assisted Requirements Elicitation:**
- [AI-Based Multiagent Approach for Requirements Elicitation (arXiv)](https://arxiv.org/html/2409.00038v1)
- [Advancing Requirements Engineering through Generative AI (arXiv)](https://arxiv.org/pdf/2310.13976)
- [Dialogue as Discovery: Navigating Human Intent Through Principled Inquiry (arXiv)](https://arxiv.org/html/2510.27410v1)
- [Socratic Method for Self-Discovery in LLMs - Princeton NLP](https://princeton-nlp.github.io/SocraticAI/)

**Stopping Heuristics:**
- [When Do We Stop a Test? - Michael Bolton / DevelopSense](https://developsense.com/blog/2009/09/when-do-we-stop-test)
- [When Do We Stop Testing? One More Sure Thing - DevelopSense](https://developsense.com/blog/2009/10/when-do-we-stop-testing-one-more-sure)
- [Heuristic Test Strategy Model - James Bach / Satisfice](https://www.satisfice.com/download/heuristic-test-strategy-model)

**AI Specification / Test Generation:**
- [Spec-Driven Development with AI - GitHub Blog](https://github.blog/ai-and-ml/generative-ai/spec-driven-development-with-ai-get-started-with-a-new-open-source-toolkit/)
- [How to Write a Good Spec for AI Agents - Addy Osmani](https://addyosmani.com/blog/good-spec/)
- [Copilot AI Code Quality 2025 - GitClear](https://gitclear-public.s3.us-west-2.amazonaws.com/GitClear-AI-Copilot-Code-Quality-2025.pdf)
- [TDD and Pair Programming as Companions for Copilot - Thoughtworks](https://www.thoughtworks.com/en-us/insights/blog/generative-ai/tdd-and-pair-programming-the-perfect-companions-for-copilot)
- [From 3 Amigos to 4: AI-Powered BDD - Ai4Testers](https://ai4testers.com/blog/from-3-amigos-to-4-ai-powered-bdd-is-here/)

### Tertiary Sources (for context, not load-bearing)

- [Edge Cases in Software Development - Testomat.io](https://testomat.io/blog/edge-cases-in-software-development/)
- [What Are Edge Test Cases - testRigor](https://testrigor.com/blog/what-are-edge-test-cases/)
- [AI for Smarter Requirements Elicitation - Raj Jonnalagedda / Medium](https://jonnalageddaraj.medium.com/ai-for-smarter-requirements-elicitation-4bf7c2db9163)
