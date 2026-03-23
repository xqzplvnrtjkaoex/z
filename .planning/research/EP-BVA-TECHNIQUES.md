# Equivalence Partitioning, Boundary Value Analysis, and Related Techniques

Research report for the `/case` skill -- how classical test design techniques can guide
an AI agent through structured behavioral specification discussions with developers.

**Research date:** 2026-03-24
**Focus:** Practical application to backend API development, conversational question
patterns for case discovery.

---

## Table of Contents

1. [Equivalence Partitioning (EP)](#1-equivalence-partitioning-ep)
2. [Boundary Value Analysis (BVA)](#2-boundary-value-analysis-bva)
3. [Decision Table Testing](#3-decision-table-testing)
4. [State Transition Testing](#4-state-transition-testing)
5. [Pairwise / Combinatorial Testing](#5-pairwise--combinatorial-testing)
6. [EP to BVA: The Natural Conversational Order](#6-ep-to-bva-the-natural-conversational-order)
7. [Technique Selection Guide](#7-technique-selection-guide)
8. [Application to Backend API Development](#8-application-to-backend-api-development)
9. [Conversational Question Patterns for an AI Agent](#9-conversational-question-patterns-for-an-ai-agent)
10. [Technique Comparison: ROI for Case Discovery](#10-technique-comparison-roi-for-case-discovery)
11. [Sources](#11-sources)

---

## 1. Equivalence Partitioning (EP)

### Core Concept

EP divides the input domain into classes (partitions) where every value within a class
is expected to produce the **same behavior**. Testing one representative from each class
is sufficient -- if the representative passes, the entire class is considered covered;
if it fails, all values in the class are presumed to fail the same way.

Every partition must be:
- **Mutually exclusive** -- no value belongs to two partitions.
- **Collectively exhaustive** -- every possible input belongs to some partition.

Two fundamental categories:
- **Valid partitions** -- inputs the system should accept and process correctly.
- **Invalid partitions** -- inputs the system should reject with appropriate errors.

### Beyond Numeric Ranges

EP is often taught with numeric examples (age 18-65), but the technique applies to
every input type encountered in backend development:

#### String Formats
For an email field:
| Partition | Example | Expected |
|-----------|---------|----------|
| Valid format, existing domain | `user@example.com` | Accept |
| Valid format, non-existing domain | `user@nonexistent.test` | Accept (format-valid, delivery separate) |
| Missing `@` | `userexample.com` | Reject |
| Missing local part | `@example.com` | Reject |
| Missing domain | `user@` | Reject |
| Empty string | `""` | Reject |
| Null / absent | (field missing) | Reject |
| Unicode local part | `user+tag@example.com` | Depends on spec |

#### Enums / Discriminated Unions
For a `BookStatus` enum with values `{Draft, Published, Archived}`:
| Partition | Example | Expected |
|-----------|---------|----------|
| Valid enum value | `"Published"` | Accept |
| Valid but wrong case | `"published"` | Depends on parser strictness |
| Invalid enum string | `"Deleted"` | Reject |
| Empty string | `""` | Reject |
| Numeric instead of string | `1` | Reject (type mismatch) |

#### Collections (Arrays, Vectors)
For a `tags` field accepting 1-10 tags:
| Partition | Example | Expected |
|-----------|---------|----------|
| Valid count | `["action", "comedy"]` | Accept |
| Empty array | `[]` | Reject (min 1) |
| Null / absent | (field missing) | Depends on optionality |
| Over max | 11 tags | Reject |
| Contains duplicates | `["action", "action"]` | Depends on spec |
| Contains empty string | `["action", ""]` | Reject (invalid element) |

#### Nested Objects
For a request body `{ "author": { "name": "...", "email": "..." } }`:
| Partition | Example | Expected |
|-----------|---------|----------|
| All fields valid | `{ "name": "J. Doe", "email": "j@e.com" }` | Accept |
| Outer object present, inner field invalid | `{ "name": "", "email": "j@e.com" }` | Reject |
| Outer object null | `"author": null` | Reject |
| Outer object absent | (field missing entirely) | Reject |
| Extra unknown fields | `{ "name": "J", "age": 30 }` | Depends on strict/lenient parsing |

#### Authentication / Authorization States
| Partition | Description | Expected |
|-----------|-------------|----------|
| No credentials | Request with no auth header | 401 Unauthenticated |
| Malformed credentials | `Authorization: Bearer not-a-jwt` | 401 Unauthenticated |
| Expired token | Valid JWT structure, past expiry | 401 Unauthenticated |
| Valid token, insufficient role | Authenticated but wrong role | 403 Forbidden |
| Valid token, correct role | Fully authorized | 200 OK |
| Valid token, resource owned by another user | Correct role but wrong ownership | 403 Forbidden |

### Multi-Dimensional Partitioning

When an operation has multiple input parameters, each parameter creates its own set
of partitions. **Multi-dimensional partitioning** considers these parameters together.

#### Single-Dimension (One Parameter at a Time)

For `CreateBook(title, category)`:
- **title partitions:** valid string, empty, too long, null
- **category partitions:** valid enum, invalid enum, null

Testing one parameter at a time yields 4 + 3 = 7 cases.

#### Multi-Dimensional (Parameters Together)

The question becomes: "Do interactions between parameters matter?"

| title | category | Expected | Why |
|-------|----------|----------|-----|
| valid | valid | Accept | Happy path |
| valid | invalid | Reject (category) | Single invalid |
| empty | valid | Reject (title) | Single invalid |
| empty | invalid | Reject (both) | Which error wins? Error priority. |
| null | null | Reject | All missing |

The multi-dimensional view surfaces cases single-dimension misses:
- **Error priority:** When both fields are invalid, which error is reported first?
  Does the API report all errors or just the first?
- **Cross-field interactions:** Are there valid combinations of individual values that
  are invalid together? (e.g., `start_date` and `end_date` both valid individually,
  but `start_date > end_date` is invalid as a pair)

#### When to Use Multi-Dimensional

- **Always** for cross-field validation rules (date ranges, password + confirm_password)
- **Selectively** for independent fields: test happy path (all valid) plus each field
  individually invalid. Full cross-product is usually unnecessary for independent fields.
- **Pairwise** when 4+ parameters interact (see Section 5)

#### Conversational Pattern for Multi-Dimensional EP

```
"We've identified the valid and invalid classes for each field individually.
 Now let me ask: are there combinations of fields that are individually valid
 but invalid TOGETHER?"

"When multiple fields are invalid at once, which error takes priority?
 Does the system report all errors or just the first one it finds?"
```

### Systematic Identification Process

1. **List every input** -- path params, query params, headers, request body fields.
2. **For each input, list its rules** -- type, required/optional, range, format,
   allowed values, business constraints.
3. **Split into partitions** -- at minimum: one valid partition, one invalid partition
   per rule. More partitions for distinct behaviors.
4. **Ask: "Is every value in this partition truly equivalent?"** -- if not, subdivide.
5. **Pick one representative per partition** for test case design.
6. **Check cross-field interactions** -- identify multi-dimensional cases.

### Key Insight for the /case Skill

EP is the **first technique to apply** for every operation input. It systematically
ensures no category of input is forgotten. The AI agent should walk through each
input parameter and ask: "What are the valid classes? What are the invalid classes?"
Then follow up with: "Do any of these fields interact with each other?"

---

## 2. Boundary Value Analysis (BVA)

### Core Concept

Defects cluster at boundaries. BVA tests values at and near the edges of equivalence
partitions. The classic pattern for a range [min, max]:

```
min-1  |  min  |  min+1  |  ...  |  max-1  |  max  |  max+1
 invalid  boundary  valid         valid    boundary  invalid
```

### Two-Value vs Three-Value BVA (ISTQB)

The ISTQB Foundation Level Syllabus v4.0 defines two levels of BVA rigor:

#### Two-Value BVA (Standard)

For each boundary, test **two** values: the boundary value itself and its closest
neighbor in the adjacent partition.

For a range [1, 100]:
| Boundary | Values Tested | Partitions Covered |
|----------|--------------|-------------------|
| Lower boundary | 0 (invalid), 1 (valid) | invalid-valid transition |
| Upper boundary | 100 (valid), 101 (invalid) | valid-invalid transition |

Total: **4 test values** for one bounded range.

#### Three-Value BVA (Rigorous)

For each boundary, test **three** values: the boundary value and **both** its
neighbors (one from each side).

For a range [1, 100]:
| Boundary | Values Tested | Partitions Covered |
|----------|--------------|-------------------|
| Lower boundary | 0 (invalid), 1 (boundary), 2 (valid) | both sides of lower boundary |
| Upper boundary | 99 (valid), 100 (boundary), 101 (invalid) | both sides of upper boundary |

Total: **6 test values** for one bounded range.

#### Why Three-Value Catches More Bugs

Consider an implementation bug where `if (x <= 10)` is incorrectly coded as `if (x == 10)`.

- **Two-value BVA** tests x=10 and x=11. Both produce correct results for the buggy
  code (x=10 passes, x=11 fails). The bug goes **undetected**.
- **Three-value BVA** also tests x=9. The buggy code rejects x=9 when it should
  accept it. Bug **detected**.

#### Recommendation for /case Skill

Use **three-value BVA** as the default in conversation. The extra question ("What
about the value just inside the boundary?") costs almost nothing in discussion time
and catches a meaningful class of off-by-one implementation errors.

### BVA for Non-Numeric Domains

#### String Length
If a username allows 3-20 characters:
| Boundary | Value | Expected |
|----------|-------|----------|
| Below min | 2 chars `"ab"` | Reject |
| At min | 3 chars `"abc"` | Accept |
| Above min | 4 chars `"abcd"` | Accept |
| Below max | 19 chars | Accept |
| At max | 20 chars | Accept |
| Above max | 21 chars | Reject |

#### Collection Size
If a batch API accepts 1-100 items:
| Boundary | Value | Expected |
|----------|-------|----------|
| Empty | 0 items | Reject |
| Min | 1 item | Accept |
| Min+1 | 2 items | Accept |
| Max-1 | 99 items | Accept |
| Max | 100 items | Accept |
| Max+1 | 101 items | Reject |

#### Pagination
If `page_size` is 1-50, default 20:
| Boundary | Value | Expected |
|----------|-------|----------|
| 0 | Reject or use default? |
| 1 | Accept (minimum page) |
| 50 | Accept (maximum page) |
| 51 | Reject or clamp to 50? |
| -1 | Reject |
| Absent | Use default (20) |

Additional pagination boundaries:
- **First page** -- does `page=0` or `page=1` start the sequence?
- **Last page** -- what happens when requesting a page beyond available data?
- **Exact fit** -- 100 items with page_size=10 gives exactly 10 pages (no partial page).
- **Off-by-one** -- 101 items with page_size=10 gives 11 pages (last page has 1 item).

#### Date/Time Ranges
If a report API accepts dates from `2020-01-01` to "today":
| Boundary | Value | Expected |
|----------|-------|----------|
| Day before range start | `2019-12-31` | Reject |
| Range start | `2020-01-01` | Accept |
| Today | `2026-03-24` | Accept |
| Tomorrow | `2026-03-25` | Reject |
| Leap day | `2024-02-29` | Accept (valid leap year) |
| Non-leap Feb 29 | `2023-02-29` | Reject (invalid date) |

#### Rate Limits
If an API allows 100 requests per minute:
| Boundary | Description | Expected |
|----------|-------------|----------|
| Request 99 | Below limit | 200 OK |
| Request 100 | At limit | 200 OK |
| Request 101 | Over limit | 429 Too Many Requests |
| Request 1 after window reset | New window | 200 OK |

#### Database Column Constraints
If a `VARCHAR(255)` column stores a title:
| Boundary | Value | Expected |
|----------|-------|----------|
| 254 chars | Accept |
| 255 chars | Accept (max) |
| 256 chars | Reject or truncate? |
| 0 chars (empty) | Depends on NOT NULL + CHECK |

#### Enum Boundaries
Enums have boundaries too -- at the "edge" of the valid set:
| Boundary | Value | Expected |
|----------|-------|----------|
| First valid variant | `BookStatus::Draft` | Accept |
| Last valid variant | `BookStatus::Archived` | Accept |
| One past last (unknown) | Integer 4 when only 0-2 defined | Reject |
| Proto3 default (0) | Unset enum field | Depends on whether 0 maps to a valid variant |

### Key Insight for the /case Skill

BVA is **most productive when applied after EP** -- once you know the partitions, ask
"what are the boundaries between them?" This is where the most bugs live. The question
pattern is simple and highly productive: **"What happens at exactly N? What about N-1?
What about N+1?"** (three-value BVA).

---

## 3. Decision Table Testing

### Core Concept

Decision tables map **combinations of conditions** to **actions/outcomes**. Each column
is a "rule" -- a specific combination of condition values and the expected result.

Structure:
```
            Rule 1    Rule 2    Rule 3    Rule 4
Cond 1        Y         Y         N         N
Cond 2        Y         N         Y         N
Action 1      X                   X
Action 2                X                   X
```

With N binary conditions, there are 2^N possible rules. This grows fast (the
"combinatorial explosion" problem).

### Reducing Combinatorial Explosion

#### 1. Collapsed Rules ("Don't Care" Conditions)
If the outcome is the same regardless of a condition's value, mark it as "don't care"
(represented with `-`). This merges multiple rules into one.

Example -- file upload validation:
```
                 Rule 1   Rule 2   Rule 3   Rule 4   Rule 5
Valid format?       Y        Y        N        -        -
Under size limit?   Y        N        -        -        -
Authenticated?      Y        Y        Y        N        -
Upload allowed?     Yes      No       No       No       No
```

Without collapsing, this would be 2^3 = 8 rules. Collapsed: 5 rules. In practice,
reductions of 50-70% are common.

#### 2. Identifying Impossible Combinations
Not all condition combinations are possible. For example, if "user is admin" is true,
then "user has write permission" is necessarily true. These impossible rows can be
eliminated.

#### 3. Breaking Down Complex Tables
If a table has 6+ conditions, split it into smaller tables grouped by related
conditions. For example, separate "input validation" conditions from "authorization"
conditions from "business rule" conditions.

### Application: Authorization Matrix

For a book management API with roles (Anonymous, Reader, Author, Admin) and
operations (List, Read, Create, Edit, Delete):

```
              Anon    Reader   Author          Admin
List books     Yes     Yes      Yes             Yes
Read book      Yes     Yes      Yes             Yes
Create book    No      No       Own only        Yes
Edit book      No      No       Own only        Yes
Delete book    No      No       No              Yes
```

This immediately surfaces cases like:
- Author tries to edit someone else's book (forbidden).
- Author tries to delete their own book (forbidden).
- What happens when an Author is also an Admin? (role precedence)

### Building Decision Tables Conversationally

Decision tables do not need to be presented as formal tables in conversation. The agent
builds them incrementally:

```
Step 1: Identify conditions
  "What factors determine whether this operation succeeds or fails?"
  → Conditions: authenticated, owns_resource, resource_state

Step 2: Walk through one condition at a time
  "If the user IS authenticated and DOES own the resource, what happens?"
  → Success

Step 3: Vary one condition
  "What if the user IS authenticated but does NOT own the resource?"
  → 403 Forbidden

Step 4: Look for "don't care" conditions
  "Does resource_state matter if the user is not authenticated?"
  → No, always 401 regardless of state
  → Mark resource_state as "don't care" for unauthenticated

Step 5: Check for impossible combinations
  "Can a user own a resource without being authenticated?"
  → No, impossible combination. Eliminate.
```

### Key Insight for the /case Skill

Decision tables are most useful when an operation has **multiple interacting
conditions** that together determine the outcome. The agent should ask: "Are there
multiple conditions that together determine what happens? Can we map them out?"

Particularly powerful for:
- Authorization rules (role + ownership + resource state)
- Business rules with compound conditions
- Validation rules where multiple fields interact

---

## 4. State Transition Testing

### Core Concept

Systems that behave differently based on their current state need tests for:
1. **Valid transitions** -- happy path state changes.
2. **Invalid transitions** -- attempts to make disallowed state changes.
3. **Transition sequences** -- chains of transitions (0-switch, 1-switch coverage).

Components:
- **States** -- the possible states of the entity.
- **Events** -- triggers that cause transitions.
- **Guards** -- conditions that must be true for a transition.
- **Actions** -- side effects that occur during a transition.

### State Transition Table vs State Diagram

A state diagram shows valid transitions visually, but **a state transition table**
shows both valid AND invalid transitions, making it superior for test case design.

For an Order entity:

```
Current State    Event           Guard              Next State      Action
-------------    -----           -----              ----------      ------
Created          pay()           payment valid      Paid            charge card
Created          cancel()        -                  Cancelled       -
Paid             ship()          stock available    Shipped         create shipment
Paid             refund()        -                  Refunded        reverse charge
Shipped          deliver()       -                  Delivered       notify buyer
Shipped          return()        within window      ReturnPending   -
Delivered        return()        within window      ReturnPending   -
Cancelled        pay()           -                  [INVALID]       error
Delivered        ship()          -                  [INVALID]       error
Refunded         ship()          -                  [INVALID]       error
```

### Test Coverage Levels

- **0-switch coverage**: Test every single valid transition at least once. Also test
  every documented invalid transition once. This is the minimum. N-switch coverage is
  measured as (number of consecutive transitions) minus 1.
- **1-switch coverage**: Test every pair of consecutive transitions. This catches bugs
  where state is corrupted by the first transition, affecting the second. Example:
  `Created -> pay() -> Paid -> ship() -> Shipped` tests the pay-then-ship sequence.
- **N-switch coverage**: Test sequences of N+1 consecutive transitions. Rarely needed
  beyond 1-switch in practice.

For the `/case` skill, **0-switch is the minimum**: every valid transition and every
invalid transition should surface as a case. 1-switch is valuable when the developer
reports that transition ordering matters or when state corruption is a concern.

### Invalid Transition Discovery

For each state, ask: "What events SHOULD NOT be possible in this state?"

The systematic way to discover these: create a **full state-event matrix** where
every cell is either a valid transition (with next state) or explicitly marked INVALID.

| Current State \ Event | pay() | cancel() | ship() | refund() | deliver() | return() |
|-----------------------|-------|----------|--------|----------|-----------|----------|
| Created | Paid | Cancelled | **INVALID** | **INVALID** | **INVALID** | **INVALID** |
| Paid | **INVALID** | **INVALID** | Shipped | Refunded | **INVALID** | **INVALID** |
| Shipped | **INVALID** | **INVALID** | **INVALID** | **INVALID** | Delivered | ReturnPending |
| Delivered | **INVALID** | **INVALID** | **INVALID** | **INVALID** | **INVALID** | ReturnPending |
| Cancelled | **INVALID** | **INVALID** | **INVALID** | **INVALID** | **INVALID** | **INVALID** |
| Refunded | **INVALID** | **INVALID** | **INVALID** | **INVALID** | **INVALID** | **INVALID** |

Every **INVALID** cell is a test case: "What happens when event X is attempted in
state Y?"

### Guiding State Discovery Conversationally

```
Step 1: Enumerate states
  "What states can a [book/order/user] be in?"
  → Draft, Published, Archived, Deleted

Step 2: For each state, enumerate valid transitions
  "When a book is in Draft state, what operations can change its state?"
  → publish() -> Published

Step 3: For each state, enumerate INVALID transitions
  "When a book is in Draft state, what operations SHOULD NOT be allowed?"
  → archive() should fail (can't archive an unpublished book)
  → delete()... wait, CAN you delete a draft? (surfaces a design decision)

Step 4: Check guards
  "Are there conditions that must be true for this transition to succeed?"
  → publish() requires at least one tag and a non-empty description

Step 5: Check concurrent transitions
  "Can two operations race on the same entity?"
  → Two users try to publish the same draft simultaneously
```

### Concurrent State Transitions

What happens when two requests try to transition the same entity simultaneously?
- Two `pay()` calls on the same Created order (double payment?)
- `cancel()` and `pay()` race on the same order (which wins?)
- `ship()` called while `refund()` is being processed

### Key Insight for the /case Skill

State transition testing is critical for any operation that changes entity state. The
agent should ask: **"What state must the entity be in for this operation to succeed?
What happens if it is in a different state?"** and **"Can two operations race on the
same entity?"**

---

## 5. Pairwise / Combinatorial Testing

### Core Concept

When full combinatorial testing is impractical (e.g., 5 parameters with 3 values each
= 3^5 = 243 combinations), **pairwise testing** guarantees that every pair of parameter
values appears together in at least one test case.

### The NIST Finding

Research by the National Institute of Standards and Technology (1999-2004) found:
- **66%** of software failures are triggered by a **single parameter value**.
- **70-95%** of failures are triggered by interactions of **just 2 variables**.
- **97%** of failures are triggered by interactions of **1-2 variables**.
- **100%** of studied failures were triggered by interactions of **6 or fewer variables**.

This means pairwise (2-way) coverage catches the vast majority of defects, and going
up to 4-6 way coverage catches practically all of them.

### Test Case Reduction

| Parameters | Values Each | Exhaustive | Pairwise | Reduction |
|------------|-------------|------------|----------|-----------|
| 3 | 3 | 27 | 9 | 67% |
| 4 | 3 | 81 | 9 | 89% |
| 5 | 3 | 243 | 15 | 94% |
| 10 | 3 | 59,049 | 18 | 99.97% |
| 5 | 5 | 3,125 | 25 | 99.2% |

### When to Use Pairwise

Pairwise is most valuable when:
- An operation has **4+ independent input parameters**, each with multiple valid values.
- Full combinatorial testing is too expensive.
- Parameters are **not strongly correlated** (if they are, use decision tables instead).

Examples in backend development:
- Search endpoint with filters: `status`, `sort_by`, `order`, `page_size`, `format`.
- Configuration combinations: `database_type` x `cache_enabled` x `auth_mode` x `log_level`.

### Constraints in Pairwise Testing

Not all combinations are valid. Tools like PICT and ACTS allow constraints:
```
# PICT constraint syntax
IF [auth_mode] = "none" THEN [role] = "anonymous";
IF [status] = "deleted" THEN [visibility] <> "public";
```

### How to Identify Important Pairs Conversationally

Rather than generating formal pairwise tables in conversation, the agent should ask:

```
"This operation has several independent parameters: [list them].
 Which PAIRS of these parameters might interact to cause unexpected behavior?"

"For example, does the combination of [param A] = X and [param B] = Y
 behave differently than you'd expect from testing each alone?"

"Are there any parameter combinations that are explicitly invalid together?"
```

### Key Insight for the /case Skill

Pairwise thinking is useful when the agent notices an operation with many independent
parameters. Rather than asking about every combination, ask: **"Which pairs of
parameter values might interact to cause unexpected behavior?"** Focus discussion on
the most suspicious 2-way interactions.

---

## 6. EP to BVA: The Natural Conversational Order

### Why Partitions First, Then Boundaries

The EP-then-BVA order is not arbitrary. It reflects how humans naturally think about
input domains, and it produces the most efficient conversational flow:

#### 1. EP Narrows the Space

EP identifies the **categories** of behavior. Without EP, BVA has no anchor -- you
cannot test boundaries if you do not know what the partitions are.

Example: For a `page_size` parameter, EP identifies:
- Valid range (1-50)
- Zero
- Negative numbers
- Above maximum
- Absent (use default)

Only after knowing these partitions can you meaningfully ask: "What happens at exactly
1? At exactly 50? At 51?"

#### 2. BVA Dives Deep Where EP Left Off

EP gives you one representative per partition. BVA zooms in on the **transitions
between partitions** -- exactly where implementation errors cluster.

```
EP discovers:  "There is a valid partition [1, 50] and an invalid partition [51, ...)"
BVA asks:      "Is the boundary at 50 inclusive or exclusive? What about 51?"
```

#### 3. The Conversational Flow

In a discussion, EP questions feel natural as openers:
- "What kinds of inputs are valid?"
- "What kinds of inputs should be rejected?"

BVA questions feel natural as follow-ups:
- "You said it accepts 1-50. What happens at exactly 50?"
- "What about 51?"
- "Is 0 rejected or does it use a default?"

Reversing the order (asking about boundaries before establishing partitions) leads to
confusion -- the developer does not yet have a mental model of the input space.

### The Complete EP-BVA Workflow

```
For each input parameter:

Step 1 (EP): Identify partitions
  "What are the valid values? What are the invalid values?"
  → Result: N partitions with one representative each

Step 2 (EP): Verify partition quality
  "Is every value in this partition truly equivalent?"
  → Subdivide if not

Step 3 (BVA): For each ordered partition boundary
  "What is the boundary between [partition A] and [partition B]?"
  "What happens at exactly the boundary value?"
  "What about one step inside? One step outside?" (three-value BVA)
  → Result: 3 values per boundary

Step 4 (Multi-dimensional EP): Check cross-parameter interactions
  "Do any parameters interact? Are there combinations that are
   individually valid but invalid together?"
  → Result: Cross-field validation cases

Step 5 (Decision Table, if needed): Map interacting conditions
  "For the interacting parameters, what are all the combinations
   of conditions and their outcomes?"
  → Result: Collapsed decision table
```

### Typical Case Yield Per Step

| Step | Technique | Typical Cases | Nature of Cases |
|------|-----------|---------------|-----------------|
| 1-2 | EP | 3-8 per parameter | Category coverage: valid, each type of invalid |
| 3 | BVA | 2-6 per boundary | Precision: off-by-one, inclusive/exclusive, edge handling |
| 4 | Multi-dim EP | 1-4 per parameter pair | Interaction: cross-field rules, error priority |
| 5 | Decision Table | Varies | Combination: compound business rules |

---

## 7. Technique Selection Guide

### Decision Tree: Which Technique for Which Operation?

```
START: Look at the operation you are discussing.

Q1: Does it accept INPUT parameters?
 ├─ YES → Apply EP to each parameter (always)
 │        Then apply BVA to each bounded parameter (always)
 └─ NO  → Skip EP/BVA (e.g., health check endpoint)

Q2: Do multiple conditions INTERACT to determine the outcome?
 ├─ YES → Apply Decision Table
 │        (authorization, compound business rules, cross-field validation)
 └─ NO  → Skip Decision Table

Q3: Does the operation CHANGE ENTITY STATE?
 ├─ YES → Apply State Transition Testing
 │        (valid transitions, invalid transitions, concurrent transitions)
 └─ NO  → Skip State Transition

Q4: Does the operation have 4+ INDEPENDENT parameters with multiple values each?
 ├─ YES → Consider Pairwise thinking
 │        (search filters, configuration parameters)
 └─ NO  → Skip Pairwise

Q5: After all systematic techniques, ask:
    "Anything domain-specific that could go wrong?"
    → Error Guessing (experience-based catch-all)
```

### Technique Applicability Matrix

| Operation Type | EP | BVA | Decision Table | State Transition | Pairwise |
|----------------|:--:|:---:|:--------------:|:----------------:|:--------:|
| **Create resource** | Always | String/numeric limits | Auth rules | Pre-conditions | Rarely |
| **Read single resource** | ID format | N/A | Auth + ownership | Soft-delete state | N/A |
| **List/search resources** | Filter values | Pagination limits | Filter combinations | N/A | Filter combos (4+) |
| **Update resource** | Field values | Field limits | Auth + ownership + state | Must be updatable | Rarely |
| **Delete resource** | ID format | N/A | Auth + ownership | Must be deletable | N/A |
| **State change** (publish, archive) | N/A | N/A | Auth + guards | Core technique | N/A |
| **Authentication** (login) | Credential formats | Password length | Rate limit + lockout | Account state | N/A |
| **Batch operation** | Item validity | Batch size limits | Auth per item? | Per-item state | Item combos |

### When Techniques Compose

Some operations naturally require multiple techniques in combination:

**Update a book (PATCH /books/:id):**
1. EP on request body fields (valid values, invalid values, missing, extra)
2. BVA on bounded fields (title length, tag count)
3. Decision Table on authorization (role x ownership x book_state)
4. State Transition on book_state precondition (can't update archived book)
5. Concurrency check (two simultaneous updates -- optimistic locking?)

**Search books (GET /books?status=X&sort=Y&page=Z):**
1. EP on each query parameter (valid filter values, invalid values)
2. BVA on page/page_size parameters
3. Pairwise on filter combinations (status x sort x tag x author)
4. BVA on result set (zero results, one result, exactly page_size, page_size+1)

---

## 8. Application to Backend API Development

### REST / gRPC Input Validation

For each endpoint, apply techniques in this order:

1. **EP** on each input parameter:
   - Path params: valid ID format, invalid format, non-existent ID, another user's ID
   - Query params: each filter value category, missing vs present, valid vs invalid
   - Headers: present vs absent, valid vs malformed (Content-Type, Authorization, etc.)
   - Body fields: valid type vs wrong type, present vs absent, within range vs outside

2. **BVA** on bounded inputs:
   - String lengths (min/max per field)
   - Numeric ranges
   - Collection sizes (min/max items)
   - Pagination parameters
   - File upload sizes

3. **Decision tables** on interacting conditions:
   - Authorization: role + ownership + resource state
   - Business rules: multiple fields that together determine outcome
   - Validation: cross-field validation (end_date must be after start_date)

### Database Operations

#### Create
| Technique | Application |
|-----------|-------------|
| EP | Valid data, duplicate unique key, FK reference exists/not-exists |
| BVA | Max column lengths, numeric precision limits |
| State | Entity pre-conditions (e.g., can only create child if parent exists) |

#### Read
| Technique | Application |
|-----------|-------------|
| EP | Existing ID, non-existent ID, deleted (soft-delete) ID, wrong format ID |
| BVA | Pagination boundaries, filter range boundaries |
| Pairwise | Multiple filter combinations |

#### Update
| Technique | Application |
|-----------|-------------|
| EP | Valid update, no-op update (same values), partial update, empty update |
| BVA | Changing bounded values to boundary values |
| State | Entity must be in updatable state |
| Decision | Authorization: own resource vs other's, role-based field restrictions |
| Concurrent | Two simultaneous updates to same entity (last-write-wins? conflict error?) |

#### Delete
| Technique | Application |
|-----------|-------------|
| EP | Existing ID, non-existent ID, already-deleted ID |
| State | Can only delete entities in certain states |
| Decision | Authorization: own vs other's, role-based |
| Cascade | What happens to child entities? FK constraint violations? |
| Concurrent | Delete while another request is reading/updating |

### Authentication / Authorization

Apply EP to authentication states (see Section 1 table), then for each authenticated
state, apply a **decision table** crossing:
- User role (admin, author, reader)
- Resource ownership (own, other's, shared)
- Resource state (draft, published, archived)
- Operation type (read, create, update, delete)

### Concurrent Operations and Race Conditions

State transition testing naturally surfaces concurrency concerns. For every state
transition, ask:
- "What if two requests trigger this transition simultaneously?"
- "What if one request reads while another writes?"
- "What if a dependent resource is modified between our read and write?"

Common patterns:
| Scenario | Risk | Mitigation to Test |
|----------|------|--------------------|
| Double submit | Duplicate creation | Idempotency key |
| Read-modify-write race | Lost update | Optimistic locking (version field) |
| Check-then-act race | TOCTOU vulnerability | Database-level constraints |
| Concurrent delete + update | Update on deleted entity | FK constraints, soft-delete checks |

---

## 9. Conversational Question Patterns for an AI Agent

This section defines the concrete question patterns the `/case` skill agent should
use to guide developers through case discovery. Techniques are applied implicitly --
the developer does not need to know EP/BVA terminology.

### Phase 1: Success Cases (EP -- valid partitions)

Start with the happy path and expand to all valid variations.

```
"What does a successful [operation] look like? Walk me through the typical input."

"Are there different KINDS of valid input that should all succeed but might behave
slightly differently?"
  -> (Surfaces multiple valid equivalence classes)

"Does the caller's identity matter? Could different authenticated users succeed
in different ways?"
  -> (Surfaces authorization-based valid partitions)

"What does the response look like on success? Does it vary by input type?"
```

### Phase 2: Failure Cases (EP -- invalid partitions + Decision Tables)

Systematically cover rejection scenarios.

```
"Let's go through each input field. For [field X]:
 - What happens if it's missing entirely?
 - What happens if it's the wrong type?
 - What happens if it's the right type but an invalid value?"
  -> (EP invalid partitions per field)

"Are there combinations of fields that are individually valid but invalid together?"
  -> (Cross-field validation, decision table thinking)

"What error should the caller receive? HTTP status? Error message structure?"
  -> (Ensures error contract is specified)

"Who should be forbidden from this operation? What do they see?"
  -> (Authorization failure partitions)
```

### Phase 3: Edge Cases (BVA + State Transitions + Concurrency)

This is where the most valuable discoveries happen.

```
"For [field X], what are the minimum and maximum allowed values?
 What happens at exactly the minimum? One below it? One above the maximum?"
  -> (BVA on each bounded field)

"What state must the [entity] be in for this operation to work?
 What happens if it's in [other state] instead?"
  -> (State transition -- valid + invalid)

"Can this operation be called twice in quick succession on the same entity?
 What should happen?"
  -> (Idempotency, double-submit)

"If two users call this at the same time on the same entity, what wins?"
  -> (Race condition)

"What about an empty result? Is it possible to have zero matches?
 How should that be represented?"
  -> (Empty set boundary)

"What if the referenced [related entity] no longer exists by the time
 this operation runs?"
  -> (Referential integrity, TOCTOU)

"What's the largest realistic input this could receive? How should the
 system behave?"
  -> (Performance/resource boundary)
```

### Technique Application Heuristics

The agent should choose techniques based on the operation's characteristics:

| Operation Characteristic | Primary Technique | Key Question |
|--------------------------|-------------------|--------------|
| Has input fields | EP | "What types of values are valid vs invalid?" |
| Has bounded inputs | BVA | "What are the limits? What happens at them?" |
| Changes entity state | State Transition | "What state must it be in? What states forbid this?" |
| Multiple conditions determine outcome | Decision Table | "Do these conditions interact? Let's map the combinations." |
| Many independent parameters | Pairwise | "Which pairs of parameters might interact unexpectedly?" |
| Modifies shared data | Concurrency | "What if two requests do this simultaneously?" |

---

## 10. Technique Comparison: ROI for Case Discovery

Ranked by typical case discoveries per unit of discussion effort in a conversational
setting:

| Rank | Technique | Cases Found | Effort | Best For | Notes |
|------|-----------|-------------|--------|----------|-------|
| 1 | **EP** | High | Low | Every operation | Foundation technique. Always apply first. Catches whole categories of forgotten inputs. |
| 2 | **BVA** | High | Low | Bounded inputs | Simple question pattern ("what at the boundary?") yields high-value cases. |
| 3 | **State Transition** | High | Medium | Stateful entities | Surfaces invalid transitions and concurrency issues that developers often overlook. |
| 4 | **Decision Table** | Medium | Medium | Multi-condition logic | Best for authorization and compound business rules. Skip for simple CRUD. |
| 5 | **Error Guessing** | Variable | Low | After systematic techniques | Experienced developers surface domain-specific cases. Use as a closing question: "Anything else that could go wrong?" |
| 6 | **Pairwise** | Medium | High | Many-parameter operations | Rarely needed in conversation. Useful as a mental model for search/filter endpoints. |

### Recommended Discussion Flow per Operation

```
1. EP:    "What are the valid inputs? What are the invalid inputs?"    (~40% of cases)
2. BVA:   "What are the boundaries? What happens at the edges?"       (~25% of cases)
3. State: "What state preconditions exist? What about invalid states?" (~15% of cases)
4. DT:    "Do multiple conditions interact? Let's map them."          (~10% of cases)
5. Conc:  "What about simultaneous calls? Race conditions?"           (~5% of cases)
6. Misc:  "Anything domain-specific that could go wrong?"             (~5% of cases)
```

---

## 11. Sources

### Primary (HIGH confidence)
- [ISTQB CTFL Syllabus v4.0.1](https://istqb.org/wp-content/uploads/2024/11/ISTQB_CTFL_Syllabus_v4.0.1.pdf) -- Foundation-level definitions of EP and BVA (two-value and three-value)
- [Boundary Value Analysis White Paper -- ISTQB](https://istqb.org/wp-content/uploads/2025/10/Boundary-Value-Analysis-white-paper.pdf) -- Authoritative BVA definitions and worked examples
- [Equivalence Partitioning -- Wikipedia](https://en.wikipedia.org/wiki/Equivalence_partitioning) -- Multi-dimensional partitioning
- [Boundary Value Analysis -- Wikipedia](https://en.wikipedia.org/wiki/Boundary-value_analysis) -- BVA theory and variants
- [All-Pairs Testing -- Wikipedia](https://en.wikipedia.org/wiki/All-pairs_testing) -- Pairwise testing theory
- [NIST: Interactions Involved in Software Failures](https://csrc.nist.gov/projects/automated-combinatorial-testing-for-software/combinatorial-methods-in-testing/interactions-involved-in-software-failures) -- Research on parameter interaction fault frequency
- [NIST Practical Combinatorial Testing SP 800-142](https://nvlpubs.nist.gov/nistpubs/legacy/sp/nistspecialpublication800-142.pdf) -- Detailed combinatorial testing research

### Secondary (MEDIUM confidence)
- [3-Value BVA: Misconception and Reality -- Giorgos Valamatsas](https://medium.com/@giorgos.valamats/3-value-boundary-value-analysis-misconception-and-reality-25a008739660) -- Detailed two-value vs three-value BVA comparison
- [Equivalence Class Partitioning: A Complete Guide -- Katalon](https://katalon.com/resources-center/blog/equivalence-class-partitioning-guide) -- Practical EP examples
- [Equivalence Partitioning: Complete Guide -- TestSigma](https://testsigma.com/blog/equivalence-partitioning/) -- EP applied to various input types
- [Equivalence Partitioning -- ToolsQA / ISTQB](https://toolsqa.com/software-testing/istqb/equivalence-partitioning/) -- ISTQB-aligned EP explanation
- [Using Equivalence Partitioning to Design QA Tests -- Ranorex](https://www.ranorex.com/blog/using-equivalence-partitioning/) -- Practical EP application
- [Boundary Value Analysis: Complete Guide -- MasterSoftwareTesting](https://mastersoftwaretesting.com/testing-fundamentals/boundary-value-analysis) -- BVA worked examples
- [Boundary Value Analysis: Complete Guide -- Katalon](https://katalon.com/resources-center/blog/boundary-value-analysis-guide) -- BVA for various domains
- [Decision Table Testing -- BrowserStack](https://www.browserstack.com/guide/decision-table) -- Decision table construction
- [Decision Table Testing Example -- Guru99](https://www.guru99.com/decision-table-testing.html) -- Worked decision table examples
- [Decision Table Testing -- ToolsQA / ISTQB](https://toolsqa.com/software-testing/istqb/decision-table-testing/) -- ISTQB-aligned decision table explanation
- [Collapsed Decision Table -- ISTQB Glossary](https://istqb-glossary.page/collapsed-decision-table/) -- Don't-care condition collapsing
- [State Transition Testing Technique -- SoftwareTestingHelp](https://www.softwaretestinghelp.com/state-transition-testing-technique-for-testing-complex-applications/) -- State transition worked examples
- [State Transition Testing -- TMAP](https://www.tmap.net/wiki/state-transition-testing/) -- N-switch coverage definitions
- [State Transition Testing -- GeeksforGeeks](https://www.geeksforgeeks.org/software-engineering/state-transition-testing/) -- State transition theory
- [Pairwise Testing Explained -- TestRail](https://www.testrail.com/blog/pairwise-testing/) -- Pairwise testing practical guide
- [Pairwise Testing: Complete Guide -- MasterSoftwareTesting](https://mastersoftwaretesting.com/testing-fundamentals/types-of-testing/pairwise-testing) -- Pairwise testing with constraint handling
- [Pairwise.org -- Combinatorial Test Case Generation](https://www.pairwise.org/) -- Tools and theory for pairwise testing
- [API Authorization Matrix -- Equixly](https://equixly.com/blog/2025/10/07/authorization-matrix/) -- Authorization decision tables for APIs
- [Error Guessing: Experience-Based Testing -- MasterSoftwareTesting](https://mastersoftwaretesting.com/testing-fundamentals/error-guessing) -- Error guessing as complement to formal techniques
- [Error Guessing: Complete Guide -- TestSigma](https://testsigma.com/blog/error-guessing/) -- Error guessing checklists

### Additional
- [Test Design Techniques Overview -- SysGears](https://sysgears.com/articles/test-design-techniques-overview/) -- Technique comparison and selection
- [Test Design Techniques: BVA, State Transition, and more -- testRigor](https://testrigor.com/blog/test-design-techniques-bva-state-transition-and-more/) -- Technique selection guidance
- [Test Case Design Techniques: A Practical 2025 Guide -- Quinnox](https://www.quinnox.com/blogs/test-case-design-techniques/) -- Current best practices
- [Edge Case Testing -- TestSigma](https://testsigma.com/blog/edge-case-testing/) -- Edge case categorization
- [Boundary Value Analysis for Non-Numerical Variables: Strings -- Oriental Journal of Computer Science](https://www.computerscijournal.org/vol3no2/boundary-value-analysis-for-non-numerical-variables-strings/) -- BVA for string domains
- [Test Concurrent CRUD Operations -- Microsoft Coyote](https://microsoft.github.io/coyote/tutorials/test-concurrent-operations/) -- Concurrency testing patterns
- [Reliably Testing Race Conditions -- Doppler](https://www.doppler.com/blog/reliably-testing-race-conditions) -- Race condition testing strategies
