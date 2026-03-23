# TDD-Originated Case Discovery Techniques

Research for the `/case` skill: extracting case discovery insights from TDD traditions
to guide pre-coding behavioral specification discussions. We want TDD's **case discovery
intelligence** -- not its Red/Green/Refactor execution cycle.

**Research date:** 2026-03-24

---

## 1. ZOMBIES Heuristic (James Grenning)

### Source

James Grenning's blog post "TDD Guided by ZOMBIES," subsequent Agile Alliance conference
talks, O'Reilly article "Use Zombies in TDD," and podcast appearances on Agile Uprising.

### The Full Mnemonic

| Letter | Meaning | Description |
|--------|---------|-------------|
| **Z** | Zero | Simplest post-conditions of a newly created or empty state. What does the system look like before anything happens? |
| **O** | One | First meaningful element or interaction. The transition from nothing to something. |
| **M** | Many (More complex) | Generalization beyond the first element. Multiple items, richer scenarios, behavior that forces you away from hardcoded values. |
| **B** | Boundary behaviors | Edges where behavior changes: empty/non-empty, min/max, off-by-one, state transitions, wrap-around limits. |
| **I** | Interface definition | The shape of the API the caller experiences: method names, parameters, return values, error signaling, what is mutable, what is immutable. |
| **E** | Exercise exceptional behavior | Abuse cases and failure modes: invalid inputs, resource failures, timeouts, permission violations. |
| **S** | Simple scenarios, simple solutions | The overarching discipline: keep both tests and implementations minimal at every step. Start with the easy stuff, procrastinate skillfully on the hard stuff. |

### The Two-Dimensional Structure

This is the key insight that separates ZOMBIES from a simple sequential checklist.
ZOMBIES has **two axes**, not one:

```
                    B                I                E
                 Boundary        Interface        Exception
            +--------------+--------------+--------------+
    Z       |  What        |  What API    |  What error  |
   Zero     |  boundary    |  shape does  |  should I    |
            |  did I just  |  this case   |  note for    |
            |  cross?      |  force?      |  later?      |
            +--------------+--------------+--------------+
    O       |  What        |  What new    |  What could  |
   One      |  boundary    |  parameters  |  go wrong    |
            |  between     |  or returns  |  with the    |
            |  0 and 1?    |  emerged?    |  first item? |
            +--------------+--------------+--------------+
    M       |  What        |  Is the API  |  What fails  |
   Many     |  boundaries  |  still clean |  at scale    |
            |  at N, N+1,  |  with many   |  or under    |
            |  max?        |  items?      |  load?       |
            +--------------+--------------+--------------+

            S = Keep it simple at every cell in this grid
```

**Vertical axis (ZOM):** A complexity progression. You move from Zero to One to Many,
each level building on the previous. This mirrors the natural way requirements unfold --
from trivial to real.

**Horizontal axis (BIE):** Cross-cutting concerns that you ask about **at every ZOM
level**, not just at the end. You do not "finish" boundaries and then "finish" interfaces.
You notice boundaries, interface decisions, and exceptions from the very first Zero test,
and you keep noticing them as the behavior expands.

**S (Simple):** Not an axis but an overarching principle that connects both axes. It says:
at every cell in this grid, choose the simplest scenario and the simplest solution. Do not
leap to a complex Many test before you have nailed down Zero and One.

### Why Zero -> One -> Many Is a Natural Progression

**Zero** forces you to define the initial state and the most basic postconditions. For a
collection, Zero answers: "What does an empty collection look like? How does it report its
emptiness?" For an API operation, Zero answers: "What happens when there are no matching
resources? What does the response look like with no data?"

Zero is valuable because it:
- Defines the "blank slate" behavior that every other test builds on
- Forces the first interface decisions (what does the constructor/endpoint signature look like?)
- Catches initialization bugs and missing default values
- Is the degenerate case that many developers skip

**One** is the first meaningful transition. It crosses the boundary from empty to
non-empty, from nothing to something. For a collection, One answers: "What does a
collection with exactly one element look like?" For an API, One answers: "What does a
single successful operation look like?"

One is where the real interface takes shape:
- The first meaningful input → output relationship is established
- "Fake it" implementations (returning constants) start feeling inadequate
- The transition from empty to non-empty is itself a boundary worth testing

**Many** is where you stop faking it and start generalizing. Constants become variables,
single paths become loops or lookups, hardcoded responses become real computations.

Many reveals:
- Whether the generalization works for N items, not just 0 and 1
- Pagination, sorting, and ordering concerns
- Performance characteristics that are invisible with one item
- Race conditions that cannot occur with a single entity

### "Boundary" as the Pivot Between One and Many

The boundary between One and Many is the most productive place to look for defects.
The classic boundaries are:

| Boundary | What It Tests |
|----------|---------------|
| 0 -> 1 | Empty to non-empty transition |
| 1 -> 2 | Single item to multiple items (forces generalization) |
| N-1 -> N | Just under a limit to exactly at the limit |
| N -> N+1 | At the limit to just over (overflow, pagination break) |
| Full | What happens when capacity is reached? |
| Wrap-around | For circular buffers, token rotation, cycling IDs |

In ZOMBIES, you do not wait until the "B" step to think about boundaries. You notice
boundaries **at every ZOM level**:
- At Zero: the boundary between "nothing exists" and "something exists"
- At One: the boundary between "one item" and "no items" (deletion returns to Zero)
- At Many: off-by-one in pagination, exactly-at-max, one-over-max

### "Interface" -- What the Operation's Contract Promises

In ZOMBIES, "Interface" is not about writing an `interface` keyword in your language. It
is about **the shape of the API your caller experiences**: method names, parameters, return
types, how errors are signaled, what is mutable, what is immutable.

Every test you write is forcing an interface decision. The question at each ZOM level is:
"What interface did this test case force me to define or reveal?"

- At Zero: "There is a `ListBooks` method, and it returns a `ListBooksResponse` with an
  `items` field and a `total_count` field."
- At One: "There is a `CreateBook` method, it takes a `CreateBookRequest`, and it returns
  a `CreateBookResponse` with a `book` field containing the created entity."
- At Many: "There is a `next_page_token` in the response, and the request accepts a
  `page_token` parameter."

For the `/case` skill, the Interface dimension translates to: "For this case, what does
the caller send and what do they get back? What new fields or parameters does this case
force into the contract?"

### "Exception" -- Systematic Failure Discovery

Exception is the systematic failure discovery step. At each ZOM level, ask: "What can go
wrong?" This is not just about error handling -- it is about discovering all the ways the
operation can fail, misbehave, or produce surprising results.

- At Zero: "What if the caller requests a resource that has never existed? What gRPC
  status code? What error detail structure?"
- At One: "What if the single item was soft-deleted? What if the caller lacks permission
  to see it? What if the data is corrupt?"
- At Many: "What if a page token is invalid or expired? What if items are added or deleted
  between page fetches? What if the query times out due to too many results?"

Grenning's advice: exercise exceptional behavior **after** you have stable happy paths, so
you can interpret failures clearly. But **note** exceptions from the start -- write them
down as you go, even if you test them last.

### ZOMBIES Walkthrough: `CreateBook` gRPC Service Method

This is a concrete walkthrough applying the full two-dimensional ZOMBIES grid to a
`CreateBook` RPC on the catalog service.

**Story:** `CreateBook` -- registers a new book in the catalog.

```
rpc CreateBook(CreateBookRequest) returns (CreateBookResponse);

message CreateBookRequest {
  string title = 1;
  repeated string tags = 2;
  optional string source_url = 3;
}

message CreateBookResponse {
  Book book = 1;
}
```

#### Z -- Zero

Start with nothing. What does "zero" mean for a create operation?

| Dimension | Case | Expected |
|-----------|------|----------|
| **Zero state** | No books exist in the catalog yet. Create the first one. | Should succeed -- there is no dependency on pre-existing state. |
| **Boundary** | The transition from 0 books to 1 book. | `total_count` goes from 0 to 1 in subsequent list queries. The created book is immediately findable by ID. |
| **Interface** | The Zero case forces the core contract: what fields does `CreateBookRequest` require? What does `CreateBookResponse` contain? | `title` is required. Response contains the full `Book` with a server-generated `id`, `created_at`, and the input fields echoed back. |
| **Exception** | What if the request has NO fields at all? Empty `CreateBookRequest`. | `INVALID_ARGUMENT` -- "title is required." This is the Zero-level exception: the most degenerate invalid input. |

#### O -- One

One meaningful creation.

| Dimension | Case | Expected |
|-----------|------|----------|
| **One item** | Create a book with title "Naruto" and tags ["manga", "action"]. | Returns `Book` with generated UUID, title "Naruto", tags preserved in order, `created_at` set to server time. |
| **Boundary** | Minimal valid input: title only, no tags, no source_url. | Should succeed with empty tags and null source_url. |
| **Interface** | One case forces: ID generation is server-side (UUID); `created_at` is server-set; tags are a repeated field (order preserved or not?). | Caller never provides `id` or `created_at`. These are output-only fields. |
| **Exception** | Create with empty title `""`. | `INVALID_ARGUMENT` -- "title must not be empty." (Different from missing title -- proto3 default for string is `""`.) |
| **Exception** | Create with whitespace-only title `"   "`. | `INVALID_ARGUMENT` -- "title must not be blank." (Should the service trim? Or reject?) **QUESTION.** |

#### M -- Many

Multiple creations and their interactions.

| Dimension | Case | Expected |
|-----------|------|----------|
| **Many items** | Create 50 books with distinct titles. | All 50 created successfully. List returns them with pagination. |
| **Boundary** | Create two books with the same title. | Should both succeed? Or is title unique? If unique: second gets `ALREADY_EXISTS`. If not unique: both succeed with different IDs. **QUESTION -- uniqueness policy.** |
| **Boundary** | Title at maximum length (e.g., 500 chars). | Succeeds. Title at 501 chars: `INVALID_ARGUMENT`. |
| **Boundary** | Maximum number of tags (e.g., 20). 20 tags succeeds, 21 tags fails. | Need to define the limit. **QUESTION.** |
| **Interface** | Many creations force: is there a bulk create? Or only single-item? | For now, single-item only. Batch is a separate operation if needed. |
| **Exception** | Two concurrent `CreateBook` calls with the same unique constraint (e.g., same `source_url`). | Exactly one succeeds, the other gets `ALREADY_EXISTS`. Database uniqueness constraint handles the race. |
| **Exception** | Database is unavailable during create. | `UNAVAILABLE` -- no partial state written. |
| **Exception** | Caller is not authenticated. | `UNAUTHENTICATED`. |
| **Exception** | Caller is authenticated but lacks `catalog:write` permission. | `PERMISSION_DENIED`. |

#### Summary of Discovered Cases

```
SUCCESS CASES:
  S1: Create first book (Zero → One transition) → succeeds with generated ID
  S2: Create with all fields populated → all fields persisted
  S3: Create with minimal fields (title only) → succeeds with empty tags, null source_url
  S4: Create 50 books → all succeed, all queryable

FAILURE CASES:
  F1: Empty request (no fields) → INVALID_ARGUMENT
  F2: Empty title "" → INVALID_ARGUMENT
  F3: Whitespace-only title → INVALID_ARGUMENT (design decision: trim or reject?)
  F4: Title exceeds max length → INVALID_ARGUMENT
  F5: Too many tags → INVALID_ARGUMENT
  F6: Duplicate unique constraint violation → ALREADY_EXISTS
  F7: No auth token → UNAUTHENTICATED
  F8: Insufficient permissions → PERMISSION_DENIED
  F9: Database unavailable → UNAVAILABLE

EDGE CASES:
  E1: Two concurrent creates with same unique key → exactly one succeeds
  E2: Title at exactly max length → succeeds (boundary)
  E3: Tags at exactly max count → succeeds (boundary)
  E4: Unicode/emoji in title → stored and returned correctly
  E5: source_url with unusual but valid format → stored as-is

OPEN QUESTIONS:
  Q1: Is title unique per user or globally?
  Q2: Should whitespace-only titles be trimmed or rejected?
  Q3: What is the maximum title length?
  Q4: What is the maximum number of tags?
  Q5: Are tags normalized (lowercase, trimmed)?
```

### How ZOMBIES Maps to API Operations

| ZOMBIES | REST/gRPC Meaning | Questions to Ask |
|---------|-------------------|------------------|
| **Zero** | Empty request / no auth / no data in system | "What if the system is empty? What if the request is empty?" |
| **One** | Single valid request, single resource | "What does the simplest successful call look like?" |
| **Many** | Bulk operations, pagination, multiple users | "What happens with many items? Many concurrent callers?" |
| **Boundary** | Limits: page sizes, field lengths, rate limits, state transitions | "What are the limits? What happens at exactly the limit?" |
| **Interface** | Request/response contract: fields, types, status codes, error format | "What does the caller send? What do they get back?" |
| **Exception** | All error codes: validation, auth, not-found, conflict, unavailable | "What can go wrong? What error does the caller see?" |
| **Simple** | One behavior per test, minimal setup, clear assertions | "What is the simplest test that proves this behavior?" |

---

## 2. Transformation Priority Premise (Robert C. Martin)

### Source

Robert C. Martin's 2013 blog post "The Transformation Priority Premise" on the Clean
Coder Blog, his CleanCoders video episodes 24 (Parts 1 and 2), and subsequent refinements
on Wikipedia and community discussions.

### Core Insight for Case Discovery

Transformations are the operations that change code behavior. They have a **priority
ordering from simple to complex**. The premise states:

> If you choose tests that force transformations in priority order (simple before complex),
> you will avoid impasses where a single test forces a complete rewrite.

For case discovery (not implementation), TPP provides a **test ordering heuristic**: it
tells you which cases to discuss first and which to discuss later. Cases that correspond
to simpler transformations should be discovered and specified before cases that correspond
to more complex transformations.

### The Transformation List (Ordered Simple -> Complex)

| # | Transformation | Description | Case Discovery Question |
|---|---------------|-------------|------------------------|
| 1 | {} -> nil | No code at all -> return nil/null/empty | "What should the operation return when there is nothing?" |
| 2 | nil -> constant | nil -> return a hard-coded value | "What is the single simplest correct return value?" |
| 3 | constant -> constant+ | Simple constant -> more complex constant | "What if the return value needs to be richer (more fields, more structure)?" |
| 4 | constant -> scalar | Replace constant with a variable/argument | "What if the result depends on the input?" |
| 5 | statement -> statements | Add more unconditional statements | "What other unconditional work must always happen?" |
| 6 | unconditional -> if | Split execution path with a conditional | "What is the first condition that changes the outcome?" |
| 7 | scalar -> array | Replace a single value with a collection | "What if there are multiple items instead of one?" |
| 8 | array -> container | Replace array with a richer container (map, set) | "Do we need keyed lookup, deduplication, or ordering?" |
| 9 | statement -> recursion | Replace statements with recursive call | "Does this operation need to process nested structures?" |
| 10 | if -> while | Replace conditional with a loop | "Does this need to repeat for an unknown number of items?" |
| 11 | expression -> function | Replace expression with a function/algorithm | "Is this computation complex enough to extract?" |
| 12 | variable -> assignment | Replace value of a variable | "Does state change during the operation?" |

### How TPP Guides the ORDER of Case Discovery

TPP's key contribution to case discovery is **ordering**. Start with cases that correspond
to the simplest transformations, then progress to cases that force more complex ones.
This has two benefits:

1. **Natural complexity ramp-up.** You discuss the trivial, degenerate, and base cases
   first, building shared understanding before tackling the complex scenarios.

2. **Gap detection.** When you skip a transformation level, you often skip the cases that
   would have caught bugs at that level. TPP acts as a checklist: for each transformation,
   ask "Is there a case that exercises this transition?"

### Applied to `GetBook` (gRPC Service Method)

| TPP Level | Case | What It Forces |
|-----------|------|----------------|
| {} -> nil | No implementation yet; method exists but returns default response | "GetBook exists as an RPC. What is the return type?" |
| nil -> constant | No books exist, get any ID -> return NOT_FOUND | Implementation returns a constant error. |
| constant -> scalar | One book exists, get its ID -> returns the book | Implementation uses the ID parameter to look up a value. |
| unconditional -> if | One book exists, get a DIFFERENT ID -> NOT_FOUND | Implementation adds a conditional: if ID matches, return book; else error. |
| scalar -> array | Multiple books exist, get a specific one -> returns the correct book | Implementation generalizes from "the one book" to "look up in a collection." |
| if -> while | (Not applicable for simple lookup, but would apply to search/filter operations) | "Can multiple books match? Do we need to iterate?" |

**The progression reveals cases in a natural order:**
1. What if there is nothing? (degenerate)
2. What if there is exactly one matching thing? (base case)
3. What if the thing I am looking for does not exist among many things? (conditional)
4. What if there are many things and I need the right one? (generalization)

### Missing Case Detection via TPP

For each transformation in the list, ask: "Is there a test case that forces this
transition?" If not, you may have a gap:

| Skipped Transformation | Missing Case |
|------------------------|-------------|
| nil -> constant | No test for empty/null return value |
| constant -> scalar | No test where the input actually matters |
| unconditional -> if | No test where the result changes based on a condition |
| scalar -> array | No test with multiple items |

### Connection to ZOMBIES

TPP and ZOMBIES are complementary, not competing:
- **ZOMBIES** tells you **what categories** of cases to explore (Zero, One, Many, Boundary, etc.)
- **TPP** tells you **what order** to explore them in (simple transformations first)
- Together: use ZOMBIES to brainstorm cases, then use TPP to order them from simplest to
  most complex

---

## 3. Triangulation and Parameterization

### Source

Kent Beck's "Test-Driven Development: By Example" (2002), Emmanuel Valverde Ramos's
article "Triangulation in TDD and the Rule of Three," and Jason Gorman's "Triangulation
Patterns."

### The Three Green-Phase Strategies

Kent Beck describes three strategies for making a failing test pass:

| Strategy | What It Means | When to Use |
|----------|---------------|-------------|
| **Obvious Implementation** | Type the real implementation directly | When you know exactly what the code should be |
| **Fake It** | Return a constant, then gradually replace constants with variables | When you are not sure, or want to go slower |
| **Triangulation** | Add another test case (a second example) to force generalization | When you cannot see the right abstraction yet |

### How Triangulation Guides Case Selection

Triangulation is the practice of **generalizing one test at a time**, doing the simplest
and least general thing at each step, and making the code more general only when a new
example demands it.

The core rule, from Beck: **abstract only when you have two or more examples.**

This has direct implications for case discovery:

**When one example is enough:**
- The behavior is simple and unambiguous (e.g., "empty input returns empty output")
- You can see the obvious implementation immediately
- There is only one valid partition for this scenario

**When you need multiple examples:**
- The behavior has a pattern that is not obvious from a single case
- A single example could be satisfied by a constant (faked) implementation
- The rule governing the behavior is ambiguous until you see several instances

**When you need three examples (Rule of Three):**
- The second example tells you "a constant is not enough" but does not always tell you
  "what the stable rule really is"
- The third example often reveals the actual pattern
- Practical maxim: "First time, do it. Second time, notice the duplication. Third time,
  refactor."

### Concrete Example: Tag Normalization

Suppose the rule is: "Tags should be normalized to lowercase with no leading/trailing
whitespace."

| # | Example | Why Needed |
|---|---------|-----------|
| 1 | `"Action"` -> `"action"` | First example: could be satisfied by hardcoding `"action"` |
| 2 | `"COMEDY"` -> `"comedy"` | Second example: now a constant is not enough; forces `to_lowercase()` |
| 3 | `"  Drama  "` -> `"drama"` | Third example: reveals the trim requirement that the first two did not |

After three examples, the rule is clear: `input.trim().to_lowercase()`. A single example
would have been ambiguous.

### Parameterized Case Groups

Triangulation naturally reveals **parameterized case groups** -- multiple cases that are
actually the same behavior pattern with different data.

Recognition signal: when you are listing cases and multiple cases follow the pattern
"input X -> expected Y" with the same transformation applied, they are a parameterized
group.

```
Validation cases for title field:
  - "" -> INVALID_ARGUMENT "title must not be empty"
  - "   " -> INVALID_ARGUMENT "title must not be blank"
  - "a" * 501 -> INVALID_ARGUMENT "title exceeds max length"
```

These are three different **invalid input partitions** for the same field. In the `/case`
discussion, recognizing parameterized groups helps:
1. **Reduce redundancy:** "These are all title validation failures" instead of discussing
   each as a separate behavior.
2. **Spot gaps:** "We have empty, whitespace, and too-long. What about special
   characters? Unicode? Null bytes?"
3. **Map to parameterized tests:** The group naturally becomes a `#[test_case]` or
   `Scenario Outline` with an examples table.

### Relevance to /case Skill

The AI agent should use triangulation thinking to determine case density:

- **One example is enough** when the behavior is binary (authorized/unauthorized,
  exists/not-exists) and there is no ambiguity about what happens.
- **Two or more examples** when the behavior involves a pattern (normalization, formatting,
  calculation) and a single example could be misleading about the rule.
- **Parameterized group** when three or more examples follow the same pattern with
  different data -- group them and ask "are there more values in this partition?"

The agent should NOT generate exhaustive lists of parameterized examples during
discussion. Instead: identify the group, give 2-3 representative examples, and note
"this is a parameterized group -- test implementation should cover the full partition."

---

## 4. Outside-In vs Inside-Out Case Discovery

### Source

Martin Fowler's "Mocks Aren't Stubs," Steve Freeman and Nat Pryce's "Growing Object-
Oriented Software, Guided by Tests" (GOOS), testdouble's "Discovery Testing," and
community comparisons on DevLead.io, Khalil Stemmler's blog, and others.

### Two Schools, Two Directions

| Aspect | London School (Outside-In) | Chicago School (Inside-Out) |
|--------|---------------------------|----------------------------|
| Also called | Mockist, GOOS, Discovery Testing | Classicist, Detroit School |
| Starting point | External API / controller / acceptance test | Domain model / core logic / pure functions |
| Direction | Outside -> Inside (API -> Use Case -> Domain -> Adapter) | Inside -> Outside (Domain -> Use Case -> API) |
| Test doubles | Heavy use of mocks for undiscovered collaborators | Minimal mocks; real objects when possible |
| Case discovery focus | **Behavior and collaboration**: what messages flow between layers? | **State and computation**: given this input, what state/output results? |
| Strength | Discovers missing abstractions and error propagation paths | Discovers domain invariants and algorithm edge cases |
| Risk | Tests coupled to implementation; fragile mocks | Over-engineering; building code not needed by the outer layers |

### What Each Approach Discovers

**Outside-In discovers (starting from the API boundary):**
- API contract cases: valid request shapes, response structures, status codes
- Error propagation cases: how does a repository timeout surface as a gRPC status?
- Collaboration cases: which services/repositories are called, in what order?
- Missing abstractions: mocking forces you to define port traits before implementations
- Security cases: what happens at the auth middleware boundary?

**Inside-Out discovers (starting from the domain core):**
- Domain invariant cases: "a book cannot have a negative page count"
- Pure computation cases: tag normalization, slug generation, ISBN validation
- State transition cases: book status lifecycle (draft -> published -> archived)
- Value object validation: "an Email must contain @", "a BookId must be a valid UUID"

### The Double Loop

Outside-In TDD uses a "double loop":
- **Outer loop (acceptance test):** A failing test at the API boundary that describes the
  end-to-end behavior. This test stays red until the full feature is implemented.
- **Inner loop (unit tests):** Smaller tests that drive individual components. As you mock
  collaborators, you discover what they need to do, then switch to implementing them with
  their own inner-loop tests.

For case discovery, the double loop means:
1. First discuss cases at the **API boundary**: "What does the caller send? What do they
   get back in success? In failure?"
2. Then dive **inward** to cases at the **domain level**: "What invariants does the
   domain entity enforce? What computations have edge cases?"

### Recommended Hybrid for the /case Skill

Start **outside-in** for each operation to discover:
1. What are the valid request shapes? (the caller's perspective)
2. What responses/errors does the caller see? (the contract)
3. What collaborators are needed? (the ports)

Then go **inside-out** for domain logic to discover:
4. What are the domain invariants? (the rules)
5. What are the computation edge cases? (the algorithms)
6. What state transitions are possible? (the lifecycle)

**Why outside-in first:** The `/case` skill is discussing operations, which are defined
by their API contract. Starting from the caller's perspective ensures we cover all the
cases the caller can experience. Domain-level cases are discovered as we drill into "how
does this operation work internally?"

### What Each Direction Misses

| Direction | Blind Spot | Remedy |
|-----------|-----------|--------|
| Outside-In only | Domain invariants that are not visible at the API boundary (e.g., internal state consistency after partial failure) | Add inside-out probing after API cases |
| Inside-Out only | Integration failures, error propagation, middleware behavior, contract mismatches | Add outside-in probing to catch API-level surprises |
| Both miss | Cross-service interaction failures, deployment-specific issues, timing-dependent bugs | Add infrastructure/concurrency probing (ZOMBIES E dimension) |

---

## 5. The "Degenerate Case" Technique

### Source

Robert C. Martin's TDD teaching ("start with the most degenerate test case"), Kent Beck's
"Chapter 2: Degenerate Objects" in TDD by Example, and community practice codified in
the Clean Coders episodes.

### What Is a Degenerate Case?

A degenerate case is the **most trivial, minimal, or extreme input** that the operation
could receive. It is the case where there is essentially "nothing" to process. Common
degenerate inputs:

| Input | Degenerate Value | Why It Matters |
|-------|-----------------|----------------|
| String | `""` (empty) | Catches missing "required" validation |
| String | `null` / absent | Catches null-pointer / missing-field bugs |
| Number | `0` | Catches division-by-zero, off-by-one, "no items" |
| Number | `-1` | Catches unsigned-assumption bugs |
| Collection | `[]` (empty) | Catches "no items to process" path |
| Collection | `[single_item]` | Catches "assumed multiple items" bugs |
| UUID | nil UUID (all zeros) | Catches "treated as no ID" bugs |
| Request | All defaults / empty body | Catches missing validation, proto3 default values |

### Uncle Bob's Test Ordering Strategy

Robert C. Martin advises a specific ordering for test discovery:

1. **Exceptional behaviors first:** What if the operation cannot even begin? (no auth,
   malformed request, system down)
2. **Degenerate behaviors second:** What if the input is empty, null, zero, or otherwise
   trivial? (the operation can begin but there is nothing to do)
3. **Ancillary behaviors third:** What about the supporting behaviors? (logging, event
   emission, cache invalidation)
4. **Core feature last:** The actual business logic that the operation is designed for.

The rationale: "Gradually increase the complexity of your tests by staying away from the
center of the algorithm for as long as possible, dealing with the degenerate, trivial,
and simple administrative tasks first."

### Why Degenerate Cases Reveal Bugs

Degenerate cases expose:

**1. Initialization bugs.** If the zero-state is not handled, the operation crashes or
returns garbage before any real work begins.

**2. Assumption violations.** Code that assumes "there will always be at least one item"
fails on empty collections. Code that assumes "the ID field will always be populated"
fails on default proto3 messages.

**3. Missing guard clauses.** Without a degenerate test, the guard clause (early return
for empty input) is never written, and the first production caller with empty input
triggers an unhandled error.

**4. Type system gaps.** In proto3, the default value for `string` is `""` and for
`bytes` is empty. A field that "looks present" may actually be the default value. Testing
with the default value catches code that does not distinguish "explicitly set to empty"
from "not set at all."

### Conversational Prompts for the /case Skill

The AI agent should start every operation discussion with degenerate case probes:

```
"What is the simplest possible input to this operation?"
"What if the request body is completely empty?"
"What if [required field] is its default value (empty string, zero, nil UUID)?"
"What if there are no [entities] in the system at all?"
"What if the caller provides nothing — no auth, no body, no parameters?"
```

These questions should come BEFORE the happy path discussion. They establish the baseline
behavior and often reveal design decisions that the developer has not yet made.

### Degenerate Case Checklist for gRPC Operations

For any gRPC service method, systematically test:

| Category | Degenerate Input | Expected |
|----------|-----------------|----------|
| **Empty request** | `SomeRequest {}` (all default values) | Should fail validation for required fields |
| **Default string** | `title: ""` (proto3 default for string) | INVALID_ARGUMENT if title is required |
| **Default bytes** | `id: b""` (proto3 default for bytes) | INVALID_ARGUMENT if ID is required |
| **Default int** | `page_size: 0` (proto3 default for int32) | Use default page size, or INVALID_ARGUMENT? |
| **Empty repeated** | `tags: []` (no elements) | Accept as "no tags" or reject as "at least one required"? |
| **No metadata** | Request with no auth metadata | UNAUTHENTICATED |
| **Nil resource** | Get/Update/Delete with nil/zero UUID | NOT_FOUND or INVALID_ARGUMENT? |

---

## 6. Integration: How These Techniques Combine for API Case Discovery

### The Unified Case Discovery Model

Each technique contributes a different dimension to case discovery. They are not
alternatives -- they are complementary lenses:

| Technique | What It Contributes | When to Use It |
|-----------|--------------------|--------------------|
| **ZOMBIES (ZOM)** | Complexity progression: nothing -> one -> many | Primary framework for every operation |
| **ZOMBIES (BIE)** | Cross-cutting concerns at each level | Applied at every ZOM level |
| **TPP** | Ordering: simple cases before complex ones | When deciding which case to discuss next |
| **Triangulation** | Case density: when one example is enough vs. when you need more | When a rule has multiple valid/invalid inputs |
| **Outside-In** | API-boundary cases first | Starting point for every operation |
| **Inside-Out** | Domain invariant cases | After API cases are established |
| **Degenerate Cases** | Trivial/minimal inputs first | Before the happy path |

### Recommended Case Discussion Order for the /case Skill

This order combines all six techniques into a single conversational flow:

```
STEP 1: DEGENERATE PROBES (Degenerate Case Technique)
  "What if the request is empty? What if there is nothing in the system?"
  → Establishes baseline, catches missing validation, forces early interface decisions

STEP 2: ZERO (ZOMBIES Z)
  "What does the system look like with no data? What is the initial state?"
  → Zero-state behavior, empty responses, default values
  → [Interface] "What does the empty response look like?"
  → [Boundary] "Where is the boundary between nothing and something?"
  → [Exception] "What can go wrong even before any real data exists?"

STEP 3: ONE -- HAPPY PATH (ZOMBIES O + TPP simple transformations)
  "What does the simplest successful case look like?"
  → First valid input → first valid output
  → [Interface] "What fields are in the request? What fields are in the response?"
  → [Boundary] "What is the minimum valid input?"

STEP 4: ONE -- FAILURE PATHS (ZOMBIES O + E + Degenerate)
  "What can the caller get wrong with a single request?"
  → Missing required fields, wrong types, invalid values
  → Auth failures: no token, expired token, wrong role
  → Not-found: the resource does not exist
  → [TPP ordering] Simple failures first (missing field), then conditional failures
    (wrong value), then state-dependent failures (already exists)

STEP 5: MANY (ZOMBIES M + TPP complex transformations)
  "What changes when there are many items/users/requests?"
  → Pagination, sorting, filtering
  → Multiple valid inputs producing different outputs
  → [Interface] "Does the API still work cleanly with many items?"
  → [Boundary] "What are the limits? page_size, max results, max tags?"

STEP 6: BOUNDARIES (ZOMBIES B, across all levels)
  "What are the exact limits? What happens at exactly the limit?"
  → String length boundaries, collection size boundaries
  → Pagination boundaries (first page, last page, off-by-one)
  → Rate limit boundaries

STEP 7: CONCURRENCY AND STATE (ZOMBIES E + Inside-Out)
  "What if two callers do this simultaneously?"
  → Race conditions, idempotency, optimistic locking
  → State transitions: what state must the entity be in?
  → What state transitions are forbidden?

STEP 8: INFRASTRUCTURE FAILURES (ZOMBIES E)
  "What if the database is down? What if a downstream service times out?"
  → Dependency failure handling
  → Retry semantics, circuit breaking

STEP 9: TRIANGULATION CHECK
  "For each rule we discovered, do we have enough examples to be confident?"
  → Identify parameterized groups (same rule, different data)
  → Identify rules that need a second or third example to clarify
  → Identify rules where one example is sufficient

STEP 10: REVIEW AND OPEN QUESTIONS
  "What are we still unsure about?"
  → Capture unresolved design decisions
  → Flag cases that need product input
  → Assess readiness: are there too many open questions?
```

### ZOMBIES Applied to a gRPC Service Method Walkthrough

The following table shows how each ZOMBIES dimension maps to questions for any gRPC
service method. This is the practical reference the `/case` skill should internalize:

| ZOMBIES | gRPC Question | Typical Cases |
|---------|---------------|---------------|
| **Z** | "What if the system is empty? What if the request is empty?" | Empty request → validation error. Empty collection → empty response. First-ever call → initialization behavior. |
| **O** | "What does the simplest valid call look like?" | Minimal valid request → success response. One matching resource → single-item response. |
| **M** | "What happens with many items? Many callers?" | Pagination. Bulk operations. High concurrency. |
| **B** | "What are the limits? What happens at exactly the limit?" | page_size at min/max. String at max length. Enum at boundary values. |
| **I** | "What does the contract look like? What fields? What types?" | Request/response message shapes. Status codes. Error detail types. |
| **E** | "What errors are possible? What can go wrong?" | INVALID_ARGUMENT, UNAUTHENTICATED, PERMISSION_DENIED, NOT_FOUND, ALREADY_EXISTS, UNAVAILABLE, INTERNAL. |
| **S** | "Is this the simplest case that proves this behavior?" | One assertion per case. No unnecessary setup. Clear cause → effect. |

---

## 7. Key Takeaways for the /case Skill

1. **ZOMBIES is the primary discussion framework.** The ZOM progression (Zero -> One ->
   Many) with BIE cross-cuts (Boundary, Interface, Exception) provides the most
   comprehensive and memorable structure for API case discovery. The two-dimensional grid
   ensures no dimension is skipped.

2. **Start with degenerate cases, not happy paths.** Following Uncle Bob's advice: probe
   the trivial, empty, and null inputs BEFORE discussing what happens when everything goes
   right. This catches missing validation and unexamined assumptions early.

3. **TPP provides ordering when the developer is stuck.** When the conversation stalls,
   prompt: "What is a slightly more complex variation of the last case?" This walks up
   the transformation priority list naturally.

4. **Start outside-in, then drill inside-out.** Begin from the caller's perspective
   (request -> response/error) to discover API contract cases. Then drill into domain
   internals to discover invariants, computations, and state transitions.

5. **Use triangulation to calibrate case density.** One example for binary behaviors.
   Two or three examples when the rule is ambiguous. Parameterized groups when many
   examples follow the same pattern. The agent should NOT generate exhaustive lists
   during discussion -- identify the group and note "parameterized test" for
   implementation.

6. **Systematic pessimism is the agent's primary value.** The developer naturally thinks
   about the happy path. The agent's job is to relentlessly ask "what could go wrong?" at
   every ZOM level, using the Exception dimension as a forcing function.

7. **The discussion IS the deliverable.** The goal is a comprehensive, prioritized list
   of behavioral cases. The conversation surfaces ambiguities, forces design decisions,
   and reveals open questions. Test code comes later.

8. **S is not optional.** At every point in the discussion, keep cases simple. If a case
   requires elaborate setup to describe, the operation may be too complex and needs
   decomposition.

---

## Sources

### Primary (direct from creators)

- [TDD Guided by ZOMBIES -- James Grenning](http://blog.wingman-sw.com/tdd-guided-by-zombies)
- [The Transformation Priority Premise -- Robert C. Martin (Clean Coder Blog)](http://blog.cleancoder.com/uncle-bob/2013/05/27/TheTransformationPriorityPremise.html)
- [Canon TDD -- Kent Beck (Substack)](https://tidyfirst.substack.com/p/canon-tdd)
- [Notes on "TDD by Example" by Kent Beck](https://stanislaw.github.io/2016-01-25-notes-on-test-driven-development-by-example-by-kent-beck.html)
- [Mocks Aren't Stubs -- Martin Fowler](https://martinfowler.com/articles/mocksArentStubs.html)

### Secondary (explanations, walkthroughs, community synthesis)

- [Use Zombies in TDD -- O'Reilly](https://www.oreilly.com/library/view/use-zombies-in/9781098172732/ch01.html)
- [Slicing a task using ZOMBIES -- Samman Coaching](https://sammancoaching.org/learning_hours/small_steps/zombies.html)
- [TDD Guided by ZOMBIES -- Emmanuel Valverde Ramos](https://emmanuelvalverderamos.substack.com/p/tdd-guided-by-zombies)
- [TDD Zombies -- COSOSO](https://www.cososo.co.uk/2017/01/tdd-zombies/)
- [Thoughts on "TDD Guided by Zombies" -- orscsblog](https://orscsblog.wordpress.com/2017/10/16/thoughts-on-tdd-guided-by-zombies/)
- [Zero-One-Many in TDD -- XP123](https://xp123.com/zero-one-many-in-tdd/)
- [Zombie Testing: One Behavior at a Time -- HackerNoon](https://hackernoon.com/zombie-testing-one-behavior-at-a-time-9s2m3zjo)
- [Transformation Priority Premise -- Wikipedia](https://en.wikipedia.org/wiki/Transformation_Priority_Premise)
- [The Transformation Priority Premise -- reinhard.codes](https://blog.reinhard.codes/2016/03/04/the-transformation-priority-premise/)
- [Extract of Uncle Bob's TPP -- Zoltan Peto](https://medium.com/@zolipeto/extract-of-uncle-bobs-transformation-priority-premise-post-85ab20216fb1)
- [Triangulation in TDD and the Rule of Three -- Emmanuel Valverde Ramos](https://emmanuelvalverderamos.substack.com/p/triangulation-in-tdd-and-the-rule)
- [Triangulation Patterns -- Jason Gorman (codemanship)](http://www.codemanship.co.uk/parlezuml/blog/?postid=382)
- [Make it run, make it right -- Relentless Development](https://relentlessdevelopment.wordpress.com/2014/06/18/make-it-run-make-it-right-the-three-implementation-strategies-of-tdd/)
- [The Three Modes of TDD -- DZone](https://dzone.com/articles/three-modes-of-tdd)
- [Getting Stuck While Doing TDD: Triangulation to the Rescue -- TDD Fellow](https://www.tddfellow.com/blog/2016/08/31/getting-stuck-while-doing-tdd-part-3-triangulation-to-the-rescue/)

### TDD Schools (Outside-In vs Inside-Out)

- [Discovery Testing -- testdouble](https://github.com/testdouble/contributing-tests/wiki/Discovery-Testing)
- [London vs Chicago TDD -- DevLead.io](https://devlead.io/DevTips/LondonVsChicago)
- [London vs Chicago in TDD -- Medium/GeekCulture](https://medium.com/geekculture/london-vs-chicago-in-tdd-77067077d0cc)
- [Outside-In TDD Overview -- outsidein.dev](https://outsidein.dev/concepts/outside-in-tdd/)
- [Outside-In Development with Double Loop TDD -- Coding Is Like Cooking](https://coding-is-like-cooking.info/2013/04/outside-in-development-with-double-loop-tdd/)
- [Double Loop TDD -- Samman Coaching](https://sammancoaching.org/learning_hours/bdd/double_loop_tdd.html)
- [TDD Styles: Classicist vs London -- Emmanuel Valverde Ramos](https://emmanuelvalverderamos.substack.com/p/test-driven-development-styles-classicist)
- [Beginners Explanation of Chicago and London -- DEV Community](https://dev.to/hiboabd/a-beginners-explanation-of-the-chicago-london-approaches-4o5f)

### Degenerate Cases and Test Ordering

- [Dijkstra's Algorithm (degenerate case example) -- Uncle Bob](https://blog.cleancoder.com/uncle-bob/2016/10/26/DijkstrasAlg.html)
- [Tests and Types -- Uncle Bob](https://blog.cleancoder.com/uncle-bob/2019/06/08/TestsAndTypes.html)
- [Chapter 2: Degenerate Objects -- TDD by Example walkthrough](http://www.emmanuelgenard.com/tdd_by_example/chapter-2-degenerate-objects.html)
- [TDD: The Simplest Code -- Ashley Bye](https://medium.com/@ambye/tdd-the-simplest-code-to-pass-a-failing-test-isnt-always-the-simplest-code-4f571bd85185)
- [TDD Heuristics: Faking and Cheating -- QWAN](https://www.qwan.eu/2021/07/20/tdd-faking-cheating.html)

### API Testing and gRPC

- [Status Codes -- gRPC](https://grpc.io/docs/guides/status-codes/)
- [gRPC Status Codes -- grpc/grpc GitHub](https://github.com/grpc/grpc/blob/master/doc/statuscodes.md)
- [Agile Alliance: TDD Guided by ZOMBIES session](https://agilealliance.org/resources/sessions/test-driven-development-guided-by-zombies/)
- [Agile Uprising Podcast: TDD Guided by ZOMBIES with James Grenning](https://agileuprising.libsyn.com/tdd-guided-by-zombies-with-james-grenning)
