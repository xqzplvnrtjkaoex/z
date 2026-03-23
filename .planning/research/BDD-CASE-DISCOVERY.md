# BDD & Scenario Discovery Techniques for AI-Developer Case Discussion

**Researched:** 2026-03-24
**Domain:** Behavioral specification discovery, BDD, structured case elicitation
**Confidence:** HIGH

Research for the `/case` skill: techniques an AI agent can use to conduct structured behavioral specification discussions with a developer, surfacing success cases, failure cases, and edge cases BEFORE writing tests.

---

## 1. Given/When/Then (GWT) as a Discovery Tool

### 1.1 Structure as a Thinking Framework

GWT was introduced by Dan North and Chris Matts as part of BDD. Martin Fowler formalized the description: Given describes the world before you begin the behavior; When describes the behavior; Then describes the expected changes.

The critical insight for the `/case` skill is that GWT is not just a test-writing format. It is a **thinking framework** that forces three specific questions about any operation:

| Section | Thinking It Forces | Discovery Questions |
|---------|-------------------|---------------------|
| **Given** | What preconditions must hold? What state must exist? | "What must be true before this can happen? What if it isn't true? What other states are possible?" |
| **When** | What exactly is the trigger? What constitutes the action? | "What precisely does the caller do? Can they do it twice? Can they do it with missing data?" |
| **Then** | What is the observable outcome? What changes? What stays the same? | "What does the response look like? What side effects occur? What should NOT change?" |

Connectors `And` and `But` extend any section without repeating the keyword.

### 1.2 Declarative vs Imperative Scenarios

**Declarative scenarios** describe WHAT should happen in domain language. **Imperative scenarios** describe HOW the system does it with implementation details.

**Declarative (good for discovery):**
```
Given an authenticated user who owns the book "Domain-Driven Design"
When the user deletes the book
Then the book is no longer visible in their library
And the book's data is preserved for administrative recovery
```

**Imperative (bad for discovery):**
```
Given I send POST /api/auth/login with {"email": "test@example.com", "password": "pass"}
And the response contains a JWT token
When I send DELETE /api/books/550e8400-e29b-41d4-a716-446655440000
  with header Authorization: Bearer <token>
Then the response status is 204
And the database row has deleted_at set to non-null
```

**Why declarative matters for case discovery:**

1. **Implementation details obscure behavioral gaps.** When a scenario says "send DELETE to /api/books/{id}", the reader focuses on HTTP mechanics rather than asking "who can delete? what if the book is already deleted? what about cascade effects?"

2. **Declarative scenarios invite "what if" questions.** "The user deletes the book" naturally invites: "What if the user doesn't own the book? What if the book is already deleted? What if the user is an admin?"

3. **Imperative scenarios prematurely constrain discussion.** Once you write specific HTTP methods and JSON payloads, you've made implementation decisions that close off alternative approaches and edge case exploration.

4. **Domain language catches domain-level gaps.** "The book's data is preserved for administrative recovery" immediately raises: "What does 'preserved' mean? For how long? Who can recover it?" Implementation-level language ("deleted_at set to non-null") hides these questions.

**The rule for `/case` discussions:** Use domain language exclusively during case discovery. HTTP methods, status codes, and JSON schemas are formulation/implementation concerns. The only exception is when the case IS about the API contract (e.g., "what status code distinguishes 'unauthorized' from 'forbidden'?").

### 1.3 Scenario Outlines as a Variation Discovery Tool

Scenario Outlines (parameterized templates) are powerful for surfacing parametric variations during discovery. The key technique: create a table of examples for a rule, then look for **missing rows**.

```gherkin
Scenario Outline: Book registration rejected for invalid title
  Given an authenticated author
  When the author registers a book with title "<title>"
  Then the registration is rejected with reason "<reason>"

  Examples:
    | title                  | reason                         |
    | (empty string)         | title must not be empty        |
    | (whitespace only)      | title must not be blank        |
    | "a" * 501              | title exceeds maximum length   |
    | ???                    | ???                            |
```

The "???" row is the discovery prompt. Looking at the table, an agent can ask:
- "What about a title with special characters like `<script>`?"
- "What about Unicode titles? Emoji titles?"
- "What about a title that is exactly at the maximum length -- should it succeed?"
- "What about a title that is a duplicate of an existing book?"

**Scenario Outlines reveal gaps visually.** When you lay out input variations in a table, missing categories become obvious. This makes them a better discovery tool than writing standalone scenarios.

### 1.4 Common GWT Anti-Patterns That Hide Edge Cases

| Anti-Pattern | Why It Hides Cases | Fix |
|---|---|---|
| **Incidental details** | Including irrelevant setup data buries the load-bearing preconditions, making it hard to ask "what if THIS condition were different?" | Strip everything not essential for THIS scenario |
| **Implementation leakage** | HTTP methods, JSON fields, database tables shift attention from behavioral questions to technical mechanics | Use domain language: "registers," "is rejected," "sees an error" |
| **Multiple behaviors** | More than one When-Then pair conflates two cases, making it impossible to isolate which behavior is actually being specified | Split into separate scenarios |
| **Too vague** | "Given the system is set up" hides preconditions, making it impossible to ask what happens when they vary | Specify exact preconditions with concrete values |
| **Testing through the UI** | Steps tied to interface elements that change frequently distract from behavioral specification | Describe behavior at the domain/API level |
| **Obvious scenarios** | "Given balance is 0, when I check balance, then it is 0" adds noise without testing any interesting rule | Delete or rephrase to validate a genuine rule |
| **Writing scenarios after code** | Scenarios become test scripts confirming existing behavior rather than discovering expected behavior | Write scenarios BEFORE implementation |
| **Only happy paths** | The most common anti-pattern: writing Given/When/Then only for success, skipping failure and edge cases entirely | For every happy path, immediately ask "what if Given is false? When fails? Then is different?" |

---

## 2. Discovery Workshops / Three Amigos

### 2.1 The Three Roles

John Ferguson Smart describes the Three Amigos as "One to request, one to suggest, and one to protest":

| Role | Title | Perspective | Key Question | What They Catch |
|------|-------|-------------|--------------|-----------------|
| **Business** | "The one who requests" | What problem are we solving? What does the user need? | "Why does this feature exist? What value does it deliver?" | Missing user context, wrong workflow assumptions, unnecessary complexity |
| **Developer** | "The one who suggests" | How will we implement this? What are the technical constraints? | "How will this execute? What are the roadblocks behind the scenes?" | Technical impossibilities, hidden dependencies, performance implications, state management issues |
| **Tester** | "The one who protests" | What could go wrong? What hasn't been considered? | "What if this happens? What else could happen? How will this break?" | Missing error cases, edge cases at boundaries, concurrency issues, security gaps |

### 2.2 Why Three Amigos Surfaces Cases That Individuals Miss

The power is in **cognitive diversity**: each role has trained intuitions the others lack.

**Developers** rarely think:
- "What if the user double-submits?"
- "What happens to UX when the service is slow?"
- "Does the admin see the same thing as the user?"

**Testers** rarely think:
- "The database constraint will prevent this anyway"
- "This requires eventual consistency -- we can't guarantee ordering"
- "The framework already handles this case"

**Business** rarely thinks about either, but catches:
- "Actually, users always do X before Y, never in isolation"
- "We don't need that case -- it never happens in practice"
- "The user might also be an admin -- should they see different results?"

Teams using Three Amigos report **significant reduction in defects found late**, because cases are surfaced before code is written rather than in QA or production.

### 2.3 How an AI Agent Simulates the Tester/Protester Role

In a `/case` conversation, the developer naturally occupies the developer perspective. The AI agent must play the other two roles, with the **tester/protester role being the highest-value contribution**.

**The Protester's Questioning Toolkit:**

#### Assumption Challenging
- "What assumption are you making about the input format?"
- "You said 'the user must be logged in.' What kind of authentication? Is a service-to-service call also valid?"
- "You mentioned 'the book exists.' What kind of existence -- active? soft-deleted? archived?"

#### Counter-Example Generation
For every stated rule, generate the negation:
- Rule: "Only the owner can delete" -> "What happens when a non-owner tries to delete?"
- Rule: "Title must be unique" -> "Unique per user, or globally? Case-sensitive or not?"
- Rule: "Token must be valid" -> "What about expired? Malformed? Revoked? From a different service?"

#### "What If" Systematic Probing
Walk through categories. For each operation, the agent should probe:
- "What if [input] is missing? empty? null? malformed? too long? too short?"
- "What if [precondition] is not met? was true but changed? never existed?"
- "What if this operation is called twice in rapid succession?"
- "What if the caller is not authorized for this?"
- "What if a downstream dependency fails? times out? returns garbage?"
- "What happens to data consistency if this fails halfway through?"
- "What if [entity] was modified between the time we read it and now?"

#### Cross-Reference Probing
- "In the registration operation, we said email must be unique. Does the update-profile operation also enforce this?"
- "The create endpoint validates X. Does the update endpoint validate the same thing?"
- "When we delete a book, what happens to the reviews that reference it?"

### 2.4 Deliberate Discovery (Dan North)

Dan North's concept of **Deliberate Discovery** provides the philosophical foundation for the protester role. The key insight: **ignorance is the single greatest impediment to throughput**.

North identifies **three orders of ignorance**:
1. **First-order:** You know you don't know something (conscious gaps)
2. **Second-order:** You don't know that you don't know something (hidden gaps)
3. **Third-order:** You don't know how to find out what you don't know

The tester/protester role exists to convert second-order ignorance into first-order ignorance -- to surface the things nobody realized they didn't know. The technique: **ask questions that sound obvious but expose hidden assumptions.**

Eric Evans calls up-front analysis "locking in our ignorance." Example Mapping is the most effective technique for deliberate discovery because it captures questions (red cards) as first-class artifacts rather than glossing over them.

### 2.5 Deliberate Contrarian Questioning Patterns

The protester doesn't randomly challenge -- they use systematic patterns:

**Pattern 1: Negate every precondition**
For each "Given" in a scenario, ask what happens if that Given is false:
```
Given: user is authenticated -> "What if they're not? What error?"
Given: book exists -> "What if it doesn't? Was deleted? Wrong ID format?"
Given: user owns the book -> "What if they don't? What if ownership transferred?"
```

**Pattern 2: Multiply the action**
For each "When", ask what happens if it occurs multiple times or simultaneously:
```
When: user deletes book -> "What if they delete twice? What if two users delete simultaneously?"
When: user registers -> "What if they submit the form twice? With same email from two browsers?"
```

**Pattern 3: Challenge the outcome**
For each "Then", ask if the outcome is complete and whether anything else changes:
```
Then: book is deleted -> "What about reviews? Tags? Reading progress? Bookmarks? Cached data?"
Then: returns 200 -> "What's in the response body? Is it always the same shape?"
Then: user is created -> "Are any events emitted? Any welcome emails? Any default settings created?"
```

**Pattern 4: Inject adversarial conditions**
Ask what a malicious or misbehaving actor could do:
```
"What if the JWT is stolen? Replayed? Modified?"
"What if the input contains SQL injection? XSS? Null bytes?"
"What if someone sends a 10MB request body?"
```

---

## 3. Scenario Discovery Heuristics

### 3.1 BRIEF Principle for Scenario Quality

The BRIEF principle (from Gaspar Nagy and Seb Rose's "Formulation" book) provides quality criteria for scenarios. Each letter represents a principle, and "brief" itself is the sixth principle:

| Letter | Principle | Description | Discovery Impact |
|--------|-----------|-------------|-----------------|
| **B** | Business language | Words drawn from the business domain, terms business people understand | Forces domain-level thinking, prevents implementation leakage |
| **R** | Real data | Concrete, illustrative values, not abstractions like "valid email" | Exposes boundary conditions and hidden assumptions |
| **I** | Intention revealing | Reveals what actors are trying to achieve, not mechanics of how | Keeps focus on behavioral questions |
| **E** | Essential | Only parts that directly illustrate the rule; no incidental details | Makes it easy to spot which precondition varies between cases |
| **F** | Focused | Each scenario illustrates a single rule | Prevents conflation of multiple behaviors |
| **Brief** | Short | 5-6 steps maximum | Forces simplicity; long scenarios indicate over-complex operations that need decomposition |

**Applying BRIEF to case discovery:** If a proposed case violates BRIEF, it signals a problem:
- Not using business language -> the case may be testing implementation, not behavior
- Not using real data -> the case may hide boundary conditions
- Not intention revealing -> the case may miss the actual user goal
- Not essential -> the case may be testing multiple things, hiding gaps
- Not focused -> the case should be split; each split reveals new edge cases
- Not brief -> the operation may need decomposition into smaller operations

### 3.2 When You Have "Enough" Scenarios

The law of diminishing returns applies to scenario discovery. Research and practitioner consensus suggest:

**Per rule:**
- First 1-2 examples capture the core behavior (happy path + primary failure)
- Next 1-2 examples catch boundary conditions
- Beyond 4-5 examples per rule, additional scenarios provide marginal value
- If a rule needs more than 5-6 examples, the rule itself is too complex and should be split

**Per operation:**
- The first 2-3 scenarios capture 70-85% of behavioral value
- Happy path + primary validation failure + authorization failure covers the "must-test" tier
- Beyond ~12 scenarios per operation, you're typically in the "could-test" tier
- Edge cases beyond the first dozen often belong as unit test cases rather than BDD scenarios

**Completeness signals:**
- All systematic probe categories have been addressed (input, auth, state, concurrency, dependencies, boundaries)
- The developer stops saying "oh, I hadn't thought of that"
- New examples start feeling like variations of existing ones
- Questions (red cards) are about fine details rather than fundamental behavior

**Stop signals (Example Mapping):**
- Many red cards (questions) -> stop; the operation is not well-understood enough to continue
- Session exceeds 25-30 minutes per operation -> the operation is too complex; decompose it
- Many blue cards (rules) -> the operation is too big; split it into smaller operations

### 3.3 Coverage Heuristics: Systematic Category Probing

Rather than asking "have we tested everything?", ask "have we probed every category?"

A case discussion has achieved adequate coverage when each applicable category has been addressed:

| Category | Addressed? | Minimum Cases |
|----------|-----------|---------------|
| Happy path (minimal valid input) | Required | 1 |
| Happy path (full valid input) | Required | 1 |
| Input validation (per required field) | Required | 1 per field |
| Authentication failure | Required if auth exists | 1-2 |
| Authorization failure | Required if authz exists | 1-2 |
| Resource not found | Required if operating on existing resource | 1 |
| Resource wrong state | Required if entity has state machine | 1 per invalid transition |
| Duplicate/conflict | Required if uniqueness constraint exists | 1 |
| Concurrency | Recommended for write operations | 1 |
| Dependency failure | Recommended | 1 |
| Boundary values | Recommended for bounded inputs | 2 per boundary |
| Side effects verification | Recommended if side effects exist | 1 |

### 3.4 The Role of Negative Scenarios

Liz Keogh's insight on negative scenarios in BDD: **negative scenarios are hugely powerful, especially since they often have the most interesting stories attached to them.** Yet they are systematically under-discovered because:

1. **Cognitive bias toward success.** Developers think about how things should work, not how they should fail. The happy path is obvious; the failure paths require deliberate effort.

2. **BDD's collaborative context favors agreement.** The business role in Three Amigos naturally describes what the system SHOULD do. Failure cases require someone to actively ask "but what if it doesn't?"

3. **Negative scenarios are unbounded.** There are infinitely many ways something can go wrong but only a few ways it can go right. This makes exhaustive coverage impossible, which discourages even partial coverage.

4. **Negative scenarios feel "obvious."** Developers often think "of course we'll handle null input" but don't specify WHAT the handling should be. The gap between "we'll handle it" and "here's exactly what happens" is where bugs live.

**How to systematically surface negative scenarios:**

The most effective technique is **rule negation**: for every stated rule, ask "what if this rule is violated?"

```
Rule: "Email must be valid"
  -> What if email is empty?
  -> What if email has no @?
  -> What if email domain doesn't exist?

Rule: "User must be authenticated"
  -> What if no token is provided?
  -> What if token is expired?
  -> What if token is malformed?
  -> What if token is for a deleted user?

Rule: "Title must be unique per user"
  -> What if the exact same title exists?
  -> What if a case-different title exists?
  -> What if the title was used by a now-deleted book?
```

**Negative scenario priority heuristic:**
A negative scenario is high-priority when:
- **Data corruption is possible** if the case is mishandled
- **Security is at stake** (auth bypass, data leakage, privilege escalation)
- **User experience degrades silently** (error is swallowed, wrong data returned)
- **The failure mode is ambiguous** (the team disagrees on what should happen)

A negative scenario is low-priority when:
- The framework/library already guarantees the handling
- The failure is immediately visible and easily correctable
- The case is physically impossible in the current architecture

---

## 4. Practical Application to Backend API Development

### 4.1 How GWT Maps to API Operations

For backend APIs (REST or gRPC), GWT maps naturally:

| GWT Section | REST API Equivalent | gRPC Equivalent |
|-------------|--------------------|--------------------|
| **Given** | System state before request: existing entities, authentication context, service state | Same: entity state in DB, caller identity in metadata, service health |
| **When** | HTTP method + URL + headers + body | RPC method invocation with request message |
| **Then** | Response status + body + side effects | Response message + status code + side effects |

**Example -- REST:**
```
Given a registered user "alice" with a valid session token
And a book "Clean Code" owned by "alice" in Published state
When alice sends DELETE /api/v1/books/{book_id}
Then the response is 204 No Content
And the book is soft-deleted (has deleted_at timestamp)
And the book no longer appears in alice's library listing
```

**Example -- gRPC:**
```
Given a registered user "alice" with caller identity in request metadata
And a book "Clean Code" owned by "alice" in Published state
When alice calls DeleteBook(book_id)
Then the response status is OK
And the book record has deleted_at set
And subsequent ListBooks for alice excludes the book
```

### 4.2 Domain-Specific Case Categories for Backend Services

#### Authentication Cases
| Case Category | Specific Cases |
|---------------|---------------|
| No credentials | Request with no auth header/metadata -> UNAUTHENTICATED |
| Malformed credentials | Invalid JWT structure, wrong encoding -> UNAUTHENTICATED |
| Expired token | Valid structure, past expiry -> UNAUTHENTICATED |
| Revoked session | Token valid but session invalidated -> UNAUTHENTICATED |
| Wrong token type | Refresh token used as access token -> UNAUTHENTICATED |
| Valid credentials | Correctly authenticated -> proceed to authorization |

#### Authorization Cases
| Case Category | Specific Cases |
|---------------|---------------|
| Role insufficient | User role lacks permission for operation -> PERMISSION_DENIED / 403 |
| Resource ownership | User tries to access another user's resource -> PERMISSION_DENIED or NOT_FOUND (security choice) |
| Admin override | Admin accesses any user's resource -> allowed (if policy) |
| Cross-service auth | Service A calls Service B with service identity -> depends on service auth policy |
| Escalation attempt | User manipulates request to claim higher privilege -> PERMISSION_DENIED |

**Security design decision:** Should "resource not found" and "resource exists but forbidden" return the same error? Returning NOT_FOUND for both prevents information leakage but makes debugging harder. This is a question the `/case` discussion should surface.

#### CRUD Operation Cases

**Create:**
| Category | Cases |
|----------|-------|
| Success | Minimal valid payload, full payload with all optional fields |
| Validation | Missing each required field, wrong types, format violations, boundary values |
| Uniqueness | Duplicate natural key, case-sensitivity of uniqueness check |
| Referential | Referenced entity doesn't exist (FK violation) |
| Concurrency | Two concurrent creates with same unique key |
| Idempotency | Same create request sent twice (with/without idempotency key) |

**Read (single entity):**
| Category | Cases |
|----------|-------|
| Success | Entity exists and caller is authorized |
| Not found | ID doesn't exist, wrong format, soft-deleted entity |
| Authorization | Caller not authorized to view this entity |
| Representation | Response shape (which fields included, field visibility by role) |

**Read (list/query):**
| Category | Cases |
|----------|-------|
| Success | Results exist, empty result set (zero matches) |
| Pagination | First page, last page, exact-fit last page, beyond-last page |
| Filtering | Valid filters, invalid filter values, no filters (default behavior) |
| Sorting | Default sort, explicit sort, sort stability |
| Boundaries | page_size=0, page_size=max+1, huge offset |
| Concurrency | Items added/deleted during pagination (cursor stability) |

**Update:**
| Category | Cases |
|----------|-------|
| Success | Valid update, partial update, no-op update (same values) |
| Not found | Target doesn't exist |
| Validation | Same as create, plus cross-field validation against existing state |
| Concurrency | Optimistic locking (stale version), concurrent updates |
| State | Entity must be in updatable state |
| Authorization | Own vs others' resource, field-level authorization |

**Delete:**
| Category | Cases |
|----------|-------|
| Success | Soft-delete vs hard-delete, response body on success |
| Not found | Target doesn't exist, target already deleted |
| Cascade | Child entities, referencing entities, associated data |
| Authorization | Own vs others' resource, admin override |
| Idempotency | Delete same entity twice |
| Concurrency | Delete while another request reads/updates |

#### Validation Cases (Input)
| Input Type | Cases to Probe |
|-----------|---------------|
| String | null, empty, whitespace-only, at min length, at max length, over max, Unicode/emoji, injection attempts |
| Integer/Number | 0, negative, MAX, MIN, floating point precision (if applicable) |
| UUID | nil UUID, valid format but non-existent entity, malformed string |
| Email | valid, missing @, missing domain, trailing whitespace, plus-addressing, case sensitivity |
| Collection/Array | empty, single element, at max size, over max, contains duplicates, contains invalid elements |
| Enum | each valid variant, unknown string, empty, numeric |
| Optional field | absent, present with valid value, present with null, present with invalid value |
| Nested object | all fields valid, inner field invalid, outer null, outer absent, extra unknown fields |
| Timestamp | epoch, far future, far past, invalid format, timezone handling |

#### Concurrency Cases
| Pattern | What to Ask |
|---------|-------------|
| Double submit | "What if the same request arrives twice? Is the operation idempotent?" |
| Race condition | "What if two users perform conflicting operations on the same entity simultaneously?" |
| Read-modify-write | "Is there an optimistic locking mechanism? What version field do we use?" |
| TOCTOU | "What if the data changes between our validation check and our write?" |
| Ordering | "Does the operation depend on events arriving in a specific order?" |

### 4.3 State-Dependent Behaviors

Many API operations behave differently depending on entity state. For any entity with a state machine, use **state transition testing**:

1. **Enumerate states:** Draft, Published, Archived, Deleted, etc.
2. **Enumerate transitions:** Which operations cause which state changes?
3. **Test valid transitions:** The happy path for each allowed state change.
4. **Test invalid transitions:** What happens when an operation is attempted on an entity in the wrong state?
5. **Test concurrent transitions:** What if two operations try to transition the same entity simultaneously?

**Example -- Book state machine:**
```
States: Draft -> Published -> Archived
                    |
                    v
                  Deleted (soft-delete from any state)

Valid transitions to test:
  Draft -> Published (publish)
  Published -> Archived (archive)
  Any -> Deleted (delete)

Invalid transitions to test:
  Draft -> Archived (skip Published -- allowed?)
  Archived -> Published (un-archive -- allowed?)
  Deleted -> anything (operate on deleted entity)
  Published -> Draft (un-publish -- allowed?)
```

For each invalid transition, the agent should ask: "What happens if someone tries to [action] a [state] book? What error? What status code?"

### 4.4 GWT-Based Question Patterns for Each API Operation Type

**For any Create operation:**
```
Given: "What must exist before this can be created? (auth, parent entity, available quota)"
When:  "What are the required fields? What are the optional fields? What formats/constraints?"
Then:  "What does the response contain? (ID, timestamps, computed fields) What side effects occur?"

Edge: "What if a create is attempted with a duplicate key?"
Edge: "What if a referenced entity (FK) doesn't exist?"
Edge: "What if the creator's quota/limit is reached?"
```

**For any Read operation:**
```
Given: "What authorization is needed to read this? Can everyone read, or only the owner?"
When:  "What identifier is used? What if the format is wrong?"
Then:  "What fields are returned? Are any fields role-dependent?"

Edge: "What if the entity was soft-deleted?"
Edge: "What about the response when the entity doesn't exist -- 404? Empty body? Error body?"
```

**For any Update operation:**
```
Given: "What state must the entity be in? Who can update it?"
When:  "Is this a full replacement (PUT) or partial update (PATCH)? What fields are updatable?"
Then:  "What changed in the response? Are timestamps updated? Are events emitted?"

Edge: "What if the update changes nothing (same values)?"
Edge: "What if someone else modified the entity since we last read it?"
Edge: "Can a field be 'unset' by passing null, or only changed to a new value?"
```

**For any Delete operation:**
```
Given: "Who can delete this? Owner only, or admin too?"
When:  "Is this soft-delete or hard-delete?"
Then:  "What happens to child/referencing entities? What does the response look like?"

Edge: "What if it's already deleted?"
Edge: "Is delete idempotent? (Same response on re-delete?)"
Edge: "What if someone reads the entity between the delete check and the actual deletion?"
```

**For any List/Search operation:**
```
Given: "What default filters apply? (only own resources? only non-deleted?)"
When:  "What filter/sort parameters exist? What are valid values for each?"
Then:  "What's the response shape? (items, total_count, pagination tokens)"

Edge: "What if zero results match?"
Edge: "What about pagination when items are added/deleted between pages?"
Edge: "What's the maximum page size? What happens if exceeded?"
```

---

## 5. Synthesis: Question Patterns for AI-Driven Case Discovery

### 5.1 The Complete Discussion Flow

For each operation, the AI agent should follow this sequence:

```
Phase 1: CLARIFY (30 seconds)
  "What does this operation do? Who calls it? What is the input and output?"
  Purpose: Establish shared understanding before diving into cases.

Phase 2: RULES EXTRACTION (1-2 minutes)
  "What business rules govern this operation?"
  For each rule: "Can you give me an example of it being satisfied? And violated?"
  Purpose: Surface the constraints that generate cases.

Phase 3: HAPPY PATH (1 minute)
  "Walk me through the success case. What are the exact preconditions?
   What exactly happens? What is the response?"
  Purpose: Establish the baseline. Developers know this well; don't over-discuss.

Phase 4: SYSTEMATIC PROBE (3-5 minutes -- this is where the value is)
  Walk through edge case categories:
  - Input validation: "What if [field] is missing? empty? invalid?"
  - Auth/Authz: "What if the caller is unauthorized? Wrong role? Not the owner?"
  - State: "What if the target doesn't exist? Is in wrong state? Was just modified?"
  - Concurrency: "Can this be called twice? Race conditions? Idempotency?"
  - Dependencies: "What if [downstream service/database] is down?"
  - Boundaries: "What are the limits? What happens at the edges?"
  - Side effects: "What else changes? Events? Related entities?"
  Purpose: Surface the cases the developer wouldn't think of alone.

Phase 5: QUESTIONS (30 seconds)
  "What are we unsure about? What needs clarification before implementation?"
  Purpose: Capture unknowns rather than assuming.

Phase 6: SUMMARY (30 seconds)
  Enumerate all discovered cases organized by:
  Success -> Failure -> Edge cases
  Purpose: Confirm completeness and agreement.
```

### 5.2 Key AI Agent Behaviors

1. **Don't accept the first answer.** When the developer says "it returns an error," ask: "What kind of error? What status code? What message? Is the error distinguishable from other errors?"

2. **Use counter-examples for every rule.** For every rule stated, ask for both the positive and negative case: "You said the email must be unique. What happens when it's not unique? What exactly is returned?"

3. **Cross-reference operations.** "In the registration operation, we said email must be unique. Does the update-profile operation also enforce this? What about case sensitivity?"

4. **Challenge implicit assumptions.** "You said 'the user must be logged in.' What kind of authentication? Is a service-to-service call also valid? What about an admin override?"

5. **Probe for missing cases by pattern.** After discussing one operation's edge cases, ask: "Does [same category] also apply to [other operation]?"

6. **Name each case concisely.** Use the "The one where..." convention:
   - "The one where the email is already taken"
   - "The one where the token is expired mid-request"
   - "The one where two users register the same email simultaneously"

7. **Capture questions, don't resolve them in-discussion.** When the developer says "I'm not sure," write it as a question. Don't guess. Don't assume. The question IS the valuable output.

8. **Play devil's advocate with confidence.** The tester role requires asking "uncomfortable" questions. Don't apologize for asking about edge cases. Frame it as: "Let me check: what happens when..."

### 5.3 The "What If..." Master Checklist for Backend Operations

For ANY backend operation, systematically ask:

```
INPUT:
  - What if a required field is missing?
  - What if a field has the wrong type?
  - What if a string field is empty? Null? Whitespace-only?
  - What if a numeric field is 0? Negative? MAX_INT?
  - What if the input is syntactically valid but semantically wrong?

IDENTITY:
  - What if the caller has no credentials?
  - What if the credentials are expired?
  - What if the caller is authenticated but not authorized?
  - What if an admin performs this action?

STATE:
  - What if the target resource doesn't exist?
  - What if the target resource was soft-deleted?
  - What if the resource was just modified by another request?
  - What if this is the first time this operation has ever been called?

CONCURRENCY:
  - What if two identical requests arrive simultaneously?
  - What if the client retries after a timeout?
  - What if a dependent service is temporarily unavailable?

OUTCOME:
  - What does the success response look like exactly?
  - What fields are included? What fields are excluded?
  - Is the response consistent with related endpoints?
  - Does the response include enough info for the client to proceed?

SIDE EFFECTS:
  - What other state changes when this operation succeeds?
  - Are there events/notifications triggered?
  - What happens to related/child resources?
  - Is the operation reversible?
```

### 5.4 Path Categories for Organizing Discovered Cases

| Path | Description | Discovery Priority | Example |
|------|-------------|-------------------|---------|
| **Happy path** | Normal success with valid input | Establish first, discuss briefly | Register with valid email and password -> account created |
| **Sad path** | Common, expected failure from user error | High -- these are the most frequent failures | Register with taken email -> rejection with clear message |
| **Bad path** | Invalid input caught at validation | High -- systematic per field | Register with malformed email -> 400 with validation error |
| **Evil path** | Malicious or adversarial input | Medium -- important for security | SQL injection in email field -> sanitized, no damage |
| **Edge path** | Unusual but legitimate boundary conditions | Medium -- where interesting bugs live | Register with 255-char email (at RFC max) -> succeeds |
| **Infrastructure path** | External dependency failures | Low for case discussion, high for resilience | Database timeout during registration -> 503, no partial state |

---

## 6. Integration with Existing Research

This document synthesizes and builds upon the companion research files:

- **example-mapping.md** -- Detailed Example Mapping mechanics (card colors, session flow, variations, AI facilitation design)
- **ACCEPTANCE-CRITERIA-TECHNIQUES.md** -- AC formats (scenario-based, rule-based, checklist), INVEST criteria, decomposition techniques, quality attributes, blind spot taxonomy
- **EP-BVA-TECHNIQUES.md** -- Equivalence Partitioning, Boundary Value Analysis, Decision Tables, State Transition Testing, Pairwise Testing, conversational question patterns
- **TDD-CASE-DISCOVERY.md** -- Test List technique (Kent Beck), Transformation Priority Premise, ZOMBIES framework, Outside-In vs Inside-Out discovery, test case categories

**How they fit together for the `/case` skill:**

```
Technique Layer 1 (Structure):
  Example Mapping provides the session structure
  (Story -> Rules -> Examples -> Questions)

Technique Layer 2 (Behavioral Thinking):
  GWT format structures thinking about preconditions, actions, outcomes
  Three Amigos provides role-based questioning perspectives
  BRIEF ensures scenario quality

Technique Layer 3 (Systematic Coverage):
  EP/BVA provides systematic input coverage
  State Transition Testing covers entity state cases
  Decision Tables cover multi-condition interactions
  ZOMBIES provides complexity progression (Zero -> One -> Many)

Technique Layer 4 (Quality/Priority):
  INVEST criteria ensure testability
  Path categories organize by risk/priority
  Diminishing returns heuristics determine when to stop
```

---

## Sources

### Primary (HIGH confidence)
- [Given When Then -- Martin Fowler](https://martinfowler.com/bliki/GivenWhenThen.html)
- [Introducing Example Mapping -- Matt Wynne, Cucumber](https://cucumber.io/blog/bdd/example-mapping-introduction/)
- [Example Mapping -- Cucumber Docs](https://cucumber.io/docs/bdd/example-mapping/)
- [Discovery Workshop -- Cucumber Docs](https://cucumber.io/docs/bdd/discovery-workshop/)
- [Behaviour-Driven Development -- Cucumber Docs](https://cucumber.io/docs/bdd/)
- [Keep Your Scenarios BRIEF -- Cucumber Blog](https://cucumber.io/blog/bdd/keep-your-scenarios-brief/)
- [Writing Better Gherkin -- Cucumber Docs](https://cucumber.io/docs/bdd/better-gherkin/)
- [The Anatomy of a Three Amigos Requirements Discovery Workshop -- John Ferguson Smart](https://johnfergusonsmart.com/three-amigos-requirements-discovery/)
- [Negative Scenarios in BDD -- Liz Keogh](https://lizkeogh.com/2015/06/19/negative-scenarios-in-bdd/)
- [Introducing Deliberate Discovery -- Dan North](https://dannorth.net/blog/introducing-deliberate-discovery/)
- [The Cadence of BDD -- Jakub Sobolewski](https://jakubsobolewski.com/blog/bdd-cadence/)
- [Canon TDD -- Kent Beck (Substack)](https://tidyfirst.substack.com/p/canon-tdd)
- [TDD Guided by ZOMBIES -- James Grenning](http://blog.wingman-sw.com/tdd-guided-by-zombies)

### Secondary (MEDIUM confidence)
- [Anti-patterns -- Cucumber](https://cucumber.io/docs/guides/anti-patterns/)
- [Cucumber Anti-patterns Part 1](https://cucumber.io/blog/bdd/cucumber-antipatterns-part-one/)
- [Cucumber Anti-patterns Part 2](https://cucumber.io/blog/bdd/cucumber-anti-patterns-part-two/)
- [BDD 101: Writing Good Gherkin -- Automation Panda](https://automationpanda.com/2017/01/30/bdd-101-writing-good-gherkin/)
- [The Behavior-Driven Three Amigos -- Automation Panda](https://automationpanda.com/2017/02/20/the-behavior-driven-three-amigos/)
- [Feature Mapping -- John Ferguson Smart](https://johnfergusonsmart.com/feature-mapping-a-lightweight-requirements-discovery-practice-for-agile-teams/)
- [Mastering BDD Scenarios: Declarative, Imperative, and Hybrid](https://seniorautomationengineer.com/index.php/2025/01/26/mastering-bdd-scenarios-declarative-imperative-hybrid-styles/)
- [The Key Role Testing Plays in Three Amigos -- TechTarget](https://www.techtarget.com/searchsoftwarequality/tip/The-key-role-testing-plays-in-a-Three-Amigos-strategy)
- [Clean Up Bad BDD Scenarios -- Gaspar Nagy](https://gasparnagy.com/2019/05/clean-up-bad-bdd-scenarios/)
- [Deliberate Discovery of Requirements with BDD -- Gaspar Nagy, Ministry of Testing](https://www.ministryoftesting.com/dojo/lessons/deliberate-discovery-of-requirements-with-bdd-with-gaspar-nagy)

### Tertiary (supporting context)
- [BDD in Action, Second Edition -- John Ferguson Smart (Manning)](https://livebook.manning.com/book/bdd-in-action-second-edition/chapter-5/v-3/)
- [Gherkin Reference -- Cucumber](https://cucumber.io/docs/gherkin/reference/)
- [User Stories and BDD Part 2: Discovery -- Cucumber](https://cucumber.io/blog/bdd/user-stories-and-bdd-part-2-discovery/)
- [Three Amigos -- Agile Alliance](https://agilealliance.org/glossary/three-amigos/)
- [Behavior-driven development -- Wikipedia](https://en.wikipedia.org/wiki/Behavior-driven_development)
- [From 3 Amigos to 4: AI-Powered BDD -- Ai4Testers](https://ai4testers.com/blog/from-3-amigos-to-4-ai-powered-bdd-is-here/)
