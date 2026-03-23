# Example Mapping for AI-Developer Case Discovery

Research conducted for the `/case` skill — a conversational behavioral specification tool where an AI agent guides a developer through structured case discovery BEFORE writing tests.

**Research date:** 2026-03-24
**Domain:** Behavioral specification, test case discovery, Example Mapping (BDD)
**Confidence:** HIGH (primary sources: Matt Wynne's original technique, Cucumber documentation, BDD community)

---

## 1. Fundamentals: The Four Card Colors

Example Mapping was created by **Matt Wynne** (co-founder of Cucumber Ltd) around 2015. He discovered the technique while helping a client's team that was struggling to analyze the details of a user story. He used a deck of colored index cards to teach them a structured way of reaching shared understanding. The technique has since become the standard pre-development discovery practice in the BDD community.

### The Four Concepts

| Color  | Card Type    | Purpose | Placement |
|--------|-------------|---------|-----------|
| Yellow | **Story**    | The feature or user story under discussion. One per session. | Top of table |
| Blue   | **Rule**     | An acceptance criterion or business constraint. | Row beneath the story, left to right |
| Green  | **Example**  | A concrete scenario illustrating one rule. | Column beneath its parent rule, top to bottom |
| Red    | **Question** | An unknown nobody present can answer. Captured to prevent rabbit-holing. | Set aside (often to the right or bottom) |

### The Spatial Layout and What It Reveals

The physical card arrangement on a table is the technique's key insight. The layout is inherently **two-dimensional**: rules spread horizontally, examples grow vertically beneath each rule, and questions accumulate separately. After a session, the shape of the table tells you something about the story's readiness:

```
                    [YELLOW: Story]
                         |
    +-----------+-----------+-----------+-----------+
    |           |           |           |           |
[BLUE: R1]  [BLUE: R2]  [BLUE: R3]  [BLUE: R4]
    |           |           |           |
[GREEN: E1] [GREEN: E4] [GREEN: E7]  [GREEN: E9]
[GREEN: E2] [GREEN: E5] [GREEN: E8]
[GREEN: E3] [GREEN: E6]

                        [RED: Q1] [RED: Q2] [RED: Q3]
```

### Card Distribution Patterns — Diagnostic Signals

| Pattern | Signal | Action |
|---------|--------|--------|
| **Many red cards** (5+) | Too much uncertainty. The story is not ready for development. | Stop. Go get answers first. Return to map later. |
| **Many blue cards** (7+) | The story is too big and complex. It conflates multiple behaviors. | Split the story into smaller stories. Each gets its own mapping session. |
| **Many green cards under one blue card** (6+) | That rule is complex or ambiguous. It may actually be multiple rules. | Consider splitting the rule into sub-rules, each with fewer examples. |
| **Few cards total, well-balanced** | The story is well-understood and right-sized. | Ready for development. |
| **One blue card with zero green cards** | Nobody generated examples — the rule was not explored. | Go back and probe: "Can you give me an example of this rule?" |
| **Green cards that do not belong to any blue card** | Hidden rule discovered through an example. | Create a new blue card for the rule this example illustrates. |

A well-understood, well-sized story should be mappable in about **25 minutes** by a small group (3-5 people). If it takes longer, the story is too big or too uncertain. After practice, sessions can drop to 20 minutes.

### The Critical Relationship Between Rules and Examples

This is the core insight that matters most for the `/case` skill:

- **Without its examples, a rule may be ambiguous.** Rules like "users must be authenticated" seem clear until you ask: what about expired tokens? What about service-to-service calls? What about partially completed registration?
- **Without a rule, an example lacks context.** A random test case floating without a governing rule is hard to understand and maintain.
- **Each example illustrates one and only one rule.** This constraint forces precision. If an example seems to illustrate two rules, either the rules overlap (merge them) or the example is doing too much (split it).
- **Rules summarize a bunch of examples.** Conversely, a rule with no examples is unproven. The examples ARE the evidence that the rule is well-understood.
- **Together, rules and examples fully specify expected behavior** and guide development.

The one-example-per-rule-aspect principle means: for each distinct aspect of a rule's behavior, there should be one illustrative example. A rule about "authenticated users" needs at minimum: one example of a valid authenticated request, one of a missing token, one of an expired token — each illustrating a different aspect of the authentication rule.

---

## 2. Rules-to-Examples Flow

### Extracting Rules from a Feature

Rules come from the acceptance criteria of a story. They summarize a bunch of examples or express agreed constraints about the scope. In practice:

1. The product owner (or developer with domain context) describes the feature and its need.
2. Participants identify business rules — constraints, policies, validations, authorization checks.
3. Each rule is written on a blue card.

**Rule extraction heuristics** (what to look for):
- Validation constraints ("must be", "cannot exceed", "at least one")
- Authorization boundaries ("only the owner", "admins can", "anonymous users cannot")
- Uniqueness/integrity rules ("must be unique", "cannot reference deleted")
- State preconditions ("only when", "only if", "must be in draft state")
- Default behaviors ("defaults to 20 if not specified")
- Side effects ("triggers notification", "updates audit log", "emits event")

### Generating Examples from Rules

For each rule, the group asks: "Can you give me an example of that?" and "What would happen if...?"

The questioning flow:

```
Rule: "Only the book owner can delete a book"

Example 1: Owner deletes their own book -> success, book removed
Example 2: Non-owner tries to delete -> 403 Forbidden
Example 3: Admin deletes any book -> ??? (QUESTION: do admins bypass ownership?)
Example 4: Owner deletes a book that has active borrows -> ??? (QUESTION: cascading behavior?)
```

Each "what would happen if..." question either:
- Produces a new **green card** (example) — when the expected outcome is clear
- Produces a new **red card** (question) — when nobody knows the answer
- Produces a new **blue card** (rule) — when a hidden constraint is discovered

This three-way branching is what makes Example Mapping a **discovery** technique, not just a documentation format. The conversation itself creates new knowledge.

### The "One Example Per Rule Aspect" Principle

For each rule, aim to cover:
1. **The happy path** — the rule satisfied normally
2. **The violation** — the rule broken in the most common way
3. **The boundary** — the edge between satisfying and violating (the interesting values)
4. **The interaction** — what happens when this rule meets another rule

Stop generating examples for a rule when:
- New examples feel repetitive or trivially similar to existing ones
- You have covered all meaningful aspects (pass, fail, boundary)
- An example produces a question rather than a clear outcome (capture question, move on)

### How Examples Expose Ambiguity

A rule like "validated input" seems straightforward. But generating examples forces concrete thinking:

```
Rule: "Email must be valid"

Example 1: "user@example.com" -> accepted (happy path)
Example 2: "" (empty) -> rejected, "email required"
Example 3: "not-an-email" -> rejected, "invalid format"
Example 4: "user@example.com " (trailing space) -> ??? (trim or reject?)
Example 5: "USER@EXAMPLE.COM" -> ??? (case-sensitive or not?)
Example 6: "user+tag@example.com" -> ??? (allow plus-addressing?)
Example 7: Already registered email -> ??? (is this a validation rule or a business rule?)
```

Examples 4-7 expose real ambiguity in the rule. This is the power of concrete examples: they force decisions that abstract rules hide. Each question mark (???) is a red card — a discovered unknown that must be resolved before implementation.

---

## 3. Questions as First-Class Outputs

Red cards are arguably the most valuable output of an Example Mapping session — they represent **discovered unknowns**. Matt Wynne's original insight: "Instead of letting everyone share their opinion about what they think the outcome should be, simply capture the question and move on."

### Why Unanswered Questions Are MORE Valuable Than Resolved Examples

1. **They prevent premature implementation decisions.** A developer who guesses wrong will build the wrong thing. A captured question forces a deliberate decision before code is written.

2. **They prevent rabbit-holing.** In a group (or 1-on-1 with an AI), it is tempting to debate an uncertain outcome for 10 minutes. Capturing it as a question and moving on keeps the session productive.

3. **They are scope signals.** Many red cards = the feature is not ready for development. This is a clear, actionable signal to the team that more design discussion or stakeholder input is needed.

4. **They decouple discovery from resolution.** The question can be resolved later by the right person (a product owner, a security expert, a DBA) rather than guessed at by whoever happens to be in the room.

5. **They create an audit trail of decisions.** When a question is later answered and converted to a rule + examples, there is a clear record of what was decided and why.

### Categorizing Questions

Questions that emerge during Example Mapping fall into distinct categories. Recognizing the category helps determine **who** should answer and **when**:

| Category | Description | Who Resolves | When to Resolve |
|----------|-------------|-------------|-----------------|
| **Domain uncertainty** | "Does 'delete' mean soft-delete or hard-delete?" | Product owner / domain expert | Before development |
| **Technical uncertainty** | "Can we guarantee this operation is idempotent?" | Architect / senior developer | During design |
| **Scope uncertainty** | "Do we need to support bulk operations in this phase?" | Product owner | During planning |
| **Security/compliance** | "Should the error distinguish 'not found' from 'not authorized'?" | Security reviewer | Before implementation |
| **Data uncertainty** | "What's the maximum number of tags per book?" | Product owner + DBA | Before schema design |
| **Integration uncertainty** | "What happens when the external service is unavailable?" | Developer + ops | During design |

### When to Stop and Escalate vs When to Make Assumptions

**Stop and escalate (capture red card) when:**
- The outcome affects data integrity (wrong answer = data corruption)
- The outcome has security implications (wrong answer = vulnerability)
- Multiple valid answers exist and the choice is a product decision
- The answer depends on information not available in the room

**Make an assumption (capture green card with assumption noted) when:**
- The outcome is low-risk and easily changed later
- Common convention clearly applies (e.g., "empty list returns 200, not 404")
- The team has an established pattern from a previous operation
- The assumption can be validated by an automated test

Even when making an assumption, annotate it: "Assumption: trailing whitespace is trimmed (not rejected). Revisit if product disagrees."

---

## 4. Session Flow and Readiness

### How a Typical Session Progresses

1. **Setup**: Write the story on a yellow card. The product owner (or developer, in solo context) explains the need.
2. **Seed rules**: Write known acceptance criteria as blue cards. These are the rules you already know about before discussion begins.
3. **Example generation**: For each rule, generate examples. Use "Can you give me an example?" and "What would happen if..." to probe.
4. **Question capture**: When the expected outcome is unclear, write a red card and move on. Do not debate — capture and continue.
5. **Rule discovery**: Examples sometimes reveal hidden rules. "Wait, we need to also check for X before allowing this" — new blue card.
6. **Review**: Look at the table. Count the colors. Assess readiness.

### Time-Boxing

| Aspect | Guideline |
|--------|-----------|
| Session length | 25-30 minutes maximum |
| Scope | One story/operation per session |
| Frequency | Matt Wynne recommends every other day (e.g., 2-3 per sprint) |
| Exhaustion signal | Participants tire quickly due to intense focus; stop before quality drops |
| Story too big signal | Session exceeds timebox before covering all rules |

If a session consistently runs over 25 minutes, the story needs to be split.

### When to Stop Per Rule

- You have enough examples when you've covered: the happy path, key failure paths, and boundary conditions for that rule.
- If examples start feeling repetitive or trivially similar, you've likely covered the rule.
- If an example generates a question (red card), capture the question and move to the next aspect or next rule.

### When to Stop Per Story

- The group feels the scope is clear, OR
- You've run out of time (25-minute timebox is standard).

### Readiness Signals

After the session, do a quick "thumb vote" based on what the table looks like:

| Indicator | Meaning | Recommended Action |
|-----------|---------|-------------------|
| Too many red cards (>5) | Story not ready | Resolve questions first, then re-map |
| Too many blue cards (>7) | Story too big | Split into smaller stories |
| Balanced, few reds (0-2) | Ready for development | Proceed to formalization/implementation |
| Session exceeded timebox | Story too complex or too uncertain | Split or escalate questions |

### The "Not Ready" Signal

This is a key output of Example Mapping: it tells you when a story is NOT yet understandable. A table covered in red cards is a clear signal to stop and go get answers before writing any code or tests. This is far cheaper than discovering the unknowns during implementation.

---

## 5. Variations and Related Techniques

### Feature Mapping (John Ferguson Smart)

Feature Mapping extends Example Mapping by adding:
- **Actors**: Who is involved in the feature?
- **Steps**: Breaking examples into sequential steps (actions + outcomes)
- **Consequences**: Explicit business outcomes at the end of each example

The process: define feature, identify actors, break into tasks/steps, identify examples via "But what if..." questions, define rules from examples, create executable specifications.

| Criterion | Example Mapping | Feature Mapping |
|-----------|----------------|-----------------|
| Best for | Focused stories, known general shape | Larger features, first discovery |
| Duration | 25 minutes | 45-60 minutes |
| Output | Rules + examples + questions | Actors + steps + examples + rules |
| Discovery style | Rule-first, then examples | Example-first, then rules |

The `/case` skill is closer to Example Mapping for individual operations, but borrows Feature Mapping's explicit outcomes (what should happen) structure.

### Example Mapping in the BDD Pipeline

Example Mapping is a **pre-BDD** activity:

```
Example Mapping Session -> Discovers rules, examples, questions
                         |
BDD Scenario Writing     -> Formalizes examples as Given/When/Then
                         |
Test Implementation      -> Automates the scenarios
```

Example Mapping produces the raw material. BDD/Gherkin formalizes it. Tests implement it. The `/case` skill sits at the Example Mapping layer — it should produce the raw cases that later inform test code, NOT full Gherkin scenarios during the discovery conversation.

### Three Amigos and the Roles

| Role | Traditional | Perspective | Key Question |
|------|------------|-------------|--------------|
| **Business** (Product Owner) | "The one who describes" | What problem are we solving? | "Why does this feature exist?" |
| **Developer** | "The one who builds" | How will we implement this? | "What are the technical constraints?" |
| **Tester** | "The one who protests" | What could go wrong? | "What if this happens? How will this break?" |

Each role catches blind spots the others miss. The business role catches wrong workflow assumptions. The developer catches technical impossibilities. The tester catches missing error cases, edge cases, and security gaps.

### Three AI Amigos (Antony Sallas, January 2026)

Recent adaptation: the classic Three Amigos model can be mapped onto AI agents with complementary strengths, coordinated through explicit state machines and structured artifacts. In a `/case` session, the AI agent must play all three roles sequentially:

1. **Play Business role first:** "What is the user trying to accomplish? What does success look like?"
2. **Acknowledge the Developer's natural perspective:** The human developer naturally contributes technical cases. Don't duplicate — ask clarifying questions.
3. **Play Tester role aggressively:** This is where the AI adds the most value. Systematically probe: missing inputs, unauthorized access, concurrent requests, dependency failures.

---

## 6. Adapting Example Mapping for AI-Developer Conversations

### The Adaptation Challenge

Standard Example Mapping assumes 3-5 people around a physical table with index cards. Adapting for a 1-on-1 AI-developer conversation requires solving four problems:

### Problem 1: Linearizing the Spatial Layout

The physical card table is 2D (rules spread horizontally, examples grow vertically). In a text conversation, this becomes sequential: **one rule at a time, all its examples, then the next rule**.

This linearization actually has an advantage: it forces deeper exploration of each rule before moving on. In a group setting, participants often jump between rules. In 1-on-1, the AI can hold focus on one rule until it is thoroughly explored.

### Problem 2: AI as Facilitator and Adversary

The AI drives the conversation by:
- **Presenting the operation** (story/yellow card) based on plan context and code
- **Proposing initial rules** extracted from domain knowledge, proto definitions, existing code
- **Asking probing questions** to generate examples ("What would happen if...?")
- **Playing devil's advocate** with edge cases the developer might not consider
- **Capturing questions** when the developer says "I'm not sure" or "good question"
- **Recognizing hidden rules** when an example does not fit under any existing rule

### Problem 3: Structured Output from Unstructured Conversation

The conversation must produce a structured document (equivalent to the card table), not just a chat transcript. The AI must maintain an internal model of:
- The current rule being explored
- Examples confirmed so far (green cards)
- Questions captured so far (red cards)
- Rules enumerated so far (blue cards)
- Any new rules discovered during example generation

### Problem 4: Decision Forcing

In a group, social dynamics push toward resolution. In 1-on-1, the AI must explicitly ask "Is this decided or still open?" to avoid ambiguity. When the developer gives a vague answer, the AI should push for specificity:
- Developer says: "It returns an error" -> AI asks: "What kind of error? What status? What message?"
- Developer says: "Yeah, probably" -> AI asks: "Shall I mark this as decided, or keep it as an open question?"

### Sequential vs Parallel Rule Exploration

**Recommended: sequential** (one rule at a time, all examples, then next rule).

Reasons:
- Maintains focus and depth
- Prevents shallow treatment of multiple rules
- Natural for text-based conversation
- Makes it easier to notice when examples reveal hidden rules

The AI should present all proposed rules at the start for confirmation, then explore them one by one.

### Presenting Rules for Confirmation

Before diving into examples, the AI should:
1. State the operation clearly
2. List the proposed rules (extracted from plan/context/code)
3. Ask the developer to confirm, correct, remove, or add rules
4. THEN proceed to per-rule example exploration

This prevents the frustrating experience of exploring the wrong rules deeply.

---

## 7. Concrete Conversation Flow for `/case` Sessions

### Phase 1: Frame (the Operation)

The AI presents the operation and proposes initial rules.

```
AI: "Let's discuss the CreateBook operation (POST /v1/books).

Based on the plan and proto definition, I see these rules:
  R1: Request must include title (non-empty string)
  R2: Request must include at least one tag
  R3: Caller must be authenticated
  R4: Title must be unique per user

Do these look right? Anything to add, change, or remove?"
```

**Developer confirms, corrects, or adds rules.** If the developer adds a rule the AI missed, that is valuable discovery.

### Phase 2: Explore (per rule, sequentially)

For each rule, the AI walks through examples:

```
AI: "Let's explore R1: 'Request must include title (non-empty string).'

The happy path: a request with title 'My First Book' -> book is created.
Confirmed?

Now let me probe the edges:
  - What if title is missing entirely from the request body?
  - What if title is an empty string ""?
  - What if title is whitespace only "   "?
  - What if title is 1 character "A"?
  - What's the maximum title length?
  - What if title contains special characters or emoji?"
```

For each probe, the developer either:
- **States the expected outcome** -> AI captures as an example (green card)
- **Says "I'm not sure"** -> AI captures as a question (red card)
- **Reveals a new constraint** -> AI captures as a new rule (blue card)

### Phase 3: Cross-Cut (rule interactions)

After individual rules are explored, the AI probes interactions:

```
AI: "Now let's look at how these rules interact:
  - R3 says caller must be authenticated, and R4 says title must be unique per user.
    If the caller is NOT authenticated, do we even check title uniqueness?
    (i.e., do we short-circuit on auth failure before validation?)

  - For R4 (uniqueness), is the check case-sensitive?
    'My Book' vs 'my book' — are these the same or different?

  - What if two authenticated users submit CreateBook with the same title
    at the same moment? Both pass the uniqueness check, both try to insert.
    What happens?"
```

### Phase 4: Review (readiness assessment)

```
AI: "Here's what we have for CreateBook:

Rules: 4 confirmed
Examples: 14 (8 success, 6 failure)
Questions: 3 open
  Q1: Maximum title length? (need product decision)
  Q2: Case-sensitivity of title uniqueness? (need product decision)
  Q3: Concurrent creation race condition handling? (need technical decision)

Assessment: 3 open questions, but all are resolvable. The operation is
mostly well-understood. I'd suggest resolving Q1 and Q2 before implementation.
Q3 can be handled with a standard database unique constraint.

Ready to move to the next operation, or want to dig deeper here?"
```

---

## 8. Application to Backend API Development

### Mapping Example Mapping Concepts to API Concepts

| Example Mapping | Backend API | Notes |
|----------------|-------------|-------|
| Story (yellow) | API operation (endpoint + method, or gRPC RPC) | One per session |
| Rule (blue) | Business constraint, validation rule, authorization requirement | Each rule governs a category of behavior |
| Example (green) | Concrete request/response pair with preconditions | Specifies exact inputs, context, and expected outcome |
| Question (red) | Open design decision | Must be resolved before implementation |

### Full Worked Example: CreateBook gRPC Operation

**Story (Yellow):** `CreateBook` — Register a new book in the catalog

**Rules (Blue):**
- R1: Request must include title (non-empty string)
- R2: Request must include at least one tag
- R3: Caller must be authenticated with valid token
- R4: Title must be unique per user (not globally)
- R5: Tags must be from the allowed tag set

**Examples (Green) for R1 (title required):**

| # | Input | Expected |
|---|-------|----------|
| E1 | `{ title: "My Book", tags: ["fantasy"] }` | Book created, returns generated UUID |
| E2 | `{ tags: ["fantasy"] }` (no title field) | INVALID_ARGUMENT, "title is required" |
| E3 | `{ title: "", tags: ["fantasy"] }` (empty title) | INVALID_ARGUMENT, "title must not be empty" |
| E4 | `{ title: "   ", tags: ["fantasy"] }` (whitespace only) | INVALID_ARGUMENT, "title must not be blank" |

**Examples (Green) for R4 (unique per user):**

| # | Context | Input | Expected |
|---|---------|-------|----------|
| E5 | User A already has "My Book" | User A creates "My Book" | ALREADY_EXISTS |
| E6 | User B has "My Book" | User A creates "My Book" | Created (unique per user, not global) |

**Examples (Green) for R3 (authentication):**

| # | Context | Expected |
|---|---------|----------|
| E7 | No auth token in request | UNAUTHENTICATED |
| E8 | Expired token | UNAUTHENTICATED |
| E9 | Valid token, user exists | Proceeds to validation |

**Questions (Red):**
- Q1: What's the maximum title length?
- Q2: Is title uniqueness case-sensitive? ("My Book" vs "my book")
- Q3: What happens if a tag is valid but deprecated?
- Q4: Should whitespace in title be normalized (collapsed, trimmed)?

### Authorization Rules — A Particularly Rich Domain

Authorization is where Example Mapping produces the most questions, because the rules interact:

```
Story: DeleteBook

Rules:
R1: Only authenticated users can delete
R2: Users can only delete their own books
R3: Admins can delete any book

Examples for R2:
E1: User A deletes User A's book -> 200 OK, book soft-deleted
E2: User A deletes User B's book -> NOT_FOUND (not PERMISSION_DENIED, to avoid info leakage)
E3: User A's token is for User A but book ownership just transferred to User B -> NOT_FOUND

Examples for R3:
E4: Admin deletes User A's book -> 200 OK
E5: Admin deletes another admin's book -> 200 OK

Questions:
Q1: If a user is both owner AND admin, which path applies? (does it matter?)
Q2: Should the error message distinguish "not found" from "not authorized"?
Q3: Does soft-delete vs hard-delete depend on the role?
Q4: What happens to book metadata (tags, history) on delete?
```

### Data Validation Rules — Boundary Analysis

Validation is where Example Mapping produces the most test cases through boundary analysis:

```
Rule: Username must be 3-20 characters, alphanumeric plus underscore

E1: "alice" (5 chars, alpha) -> valid
E2: "ab" (2 chars) -> invalid, "minimum 3 characters"
E3: "abc" (3 chars, boundary) -> valid
E4: "a" * 20 (20 chars, boundary) -> valid
E5: "a" * 21 (21 chars, boundary) -> invalid, "maximum 20 characters"
E6: "alice_bob" (underscore) -> valid
E7: "alice-bob" (hyphen) -> invalid, "only alphanumeric and underscore"
E8: "alice bob" (space) -> invalid
E9: "___" (only underscores) -> ??? (technically matches regex, but is it allowed?)
E10: "" (empty) -> invalid, "username required"
E11: "Alice" (uppercase) -> ??? (allowed? normalized to lowercase?)
```

### gRPC-Specific Considerations

When applying Example Mapping to gRPC services:

| API Concept | gRPC Mapping | Example Mapping Consideration |
|-------------|-------------|-------------------------------|
| Success response | Specific proto message | What fields are populated? What's the ID generation strategy? |
| Validation error | `INVALID_ARGUMENT` with details | What error message? Field-level or request-level? |
| Auth error | `UNAUTHENTICATED` / `PERMISSION_DENIED` | Which status for which auth failure? |
| Not found | `NOT_FOUND` | When to use NOT_FOUND vs PERMISSION_DENIED (info leakage)? |
| Conflict | `ALREADY_EXISTS` | What uniqueness constraints exist? |
| Proto3 open enums | Unknown variant arrives | How to handle unknown enum values from future clients? |

---

## 9. Question Prompt Templates for the AI Agent

### By Probing Category

**1. Happy Path Probing:**
- "What's the simplest successful case for this operation?"
- "What does the response look like on success? Which fields are populated?"
- "Are there variant success cases? (e.g., with and without optional fields)"

**2. Boundary Probing:**
- "What's the minimum/maximum valid value for [field]?"
- "What happens right at the boundary? At [min]? At [max]? One above max?"
- "What happens if [collection] is empty? Has one item? Has maximum items?"

**3. Failure Probing:**
- "What if [required field] is missing entirely?"
- "What if [field] is present but empty?"
- "What if the user isn't authenticated?"
- "What if the resource doesn't exist?"
- "Can multiple validations fail at once? Which error takes precedence?"

**4. Adversarial Probing:**
- "What would happen if this request is sent twice in rapid succession?"
- "What if the data changes between validation and execution?"
- "Can this be abused? What would a malicious caller try?"
- "What if someone tries to access another user's [resource]?"

**5. Interaction Probing:**
- "How does this rule interact with [other rule]?"
- "If rule A and rule B conflict, which wins?"
- "Does the validation order matter? Do we short-circuit on auth failure?"

**6. State Probing:**
- "What state must the [entity] be in for this operation to succeed?"
- "What happens if the entity is in [unexpected state]?"
- "What happens to related entities when this succeeds?"
- "What if the entity was soft-deleted?"

**7. Concurrency Probing:**
- "Can two users do this simultaneously on the same entity?"
- "If the client retries after a timeout, what happens?"
- "Is this operation idempotent? What ensures that?"

**8. Infrastructure Probing:**
- "What if the database is temporarily unavailable?"
- "What if a downstream gRPC service returns an error?"
- "Should the caller retry? What should the error look like?"

### The "What If..." Meta-Template

The single most productive question pattern for case discovery:

```
"What would happen if [condition that violates an assumption]?"
```

Examples:
- "What would happen if the token expires between request receipt and database write?"
- "What would happen if two users register with the same email at the same instant?"
- "What would happen if the title contains only Unicode characters?"

Each "what if" produces either a green card (clear answer), a red card (uncertain), or a blue card (new rule discovered).

---

## 10. Design Principles for `/case` Implementation

Based on all research, the `/case` skill should follow these principles:

### 1. Examples before tests
The output is structured cases (rules + examples + questions), not test code. Test code comes later. The `/case` skill lives in the Discovery phase, not the Formulation or Automation phases.

### 2. Questions are first-class outputs
An open question is more valuable than a wrong assumption. The skill must make it easy for the developer to say "I don't know" and have that captured prominently, not buried.

### 3. One rule at a time
Don't try to discuss all rules simultaneously. Linear conversation demands sequential rule exploration. Present all rules first for confirmation, then explore each deeply.

### 4. Concrete over abstract
"User with email 'alice@example.com' and role 'member' sends CreateBook with title 'My First Book'" beats "an authenticated user creates a book." Concrete examples remove ambiguity that abstract descriptions hide.

### 5. The AI should propose, the developer should dispose
The AI suggests rules and probes with edge cases; the developer confirms, modifies, or rejects. This mirrors the tester role in Three Amigos. The AI adds value by asking questions the developer would not ask themselves.

### 6. Stop signals matter
If too many questions pile up for one operation (5+ red cards), the AI should flag it as "not ready" — the developer needs to resolve unknowns first. This is an explicit output, not a failure.

### 7. Rules beget examples beget rules
The process is not strictly linear. An example may reveal a hidden rule. A new rule may need its own examples. The AI should recognize when an example does not fit under any existing rule and suggest creating a new one.

### 8. Don't accept the first answer
When the developer says "it returns an error," push: "What kind of error? What status code? What message? Is the error distinguishable from other errors?" Vague answers become vague implementations.

---

## Sources

### Primary Sources (HIGH confidence)

- [Introducing Example Mapping (Matt Wynne, Cucumber Blog)](https://cucumber.io/blog/bdd/example-mapping-introduction/) — Original technique description
- [Example Mapping (Cucumber Documentation)](https://cucumber.io/docs/bdd/example-mapping/) — Official reference
- [Example Mapping (Matt Wynne, Medium)](https://medium.com/@mattwynne/introducing-example-mapping-42ccd15f8adf) — Author's extended description
- [Your First Example Mapping Session (Cucumber Blog)](https://cucumber.io/blog/bdd/your-first-example-mapping-session/) — Practical session guidance
- [Discovery Workshop (Cucumber Documentation)](https://cucumber.io/docs/bdd/discovery-workshop/) — BDD discovery context
- [Feature Mapping - Lightweight Requirements Discovery (John Ferguson Smart)](https://johnfergusonsmart.com/feature-mapping-a-lightweight-requirements-discovery-practice-for-agile-teams/) — Feature Mapping extension
- [Feature Mapping - From Stories to Executable Acceptance Criteria (John Ferguson Smart)](https://johnfergusonsmart.com/feature-mapping-a-simpler-path-from-stories-to-executable-acceptance-criteria/) — Feature Mapping details

### Secondary Sources (MEDIUM confidence)

- [Example Mapping - Steering the Conversation (Xebia)](https://xebia.com/blog/example-mapping-steering-the-conversation/) — Facilitation patterns
- [Heuristics on Approaching Example Mapping (Xebia)](https://xebia.com/blog/heuristics-on-approaching-example-mapping/) — Practical heuristics
- [Example Mapping in BDD (Ibuildings)](https://ibuildings.com/blog/2018/07/example-mapping/) — Readiness signals and time-boxing
- [BDD Example Mapping (Automation Panda)](https://automationpanda.com/2018/02/27/bdd-example-mapping/) — Technique summary
- [Example Mapping (InsideProduct)](https://insideproduct.co/example-mapping/) — Visual layout explanation
- [Example Mapping (fspec.dev)](https://fspec.dev/concepts/example-mapping/) — Modern adaptation
- [Shaping User Stories with Example Mapping (Smartesting)](https://www.smartesting.com/en/shaping-user-stories-with-example-mapping-2/) — Readiness heuristics
- [How to Do Example Mapping (Toby Sinclair)](https://www.tobysinclair.com/post/how-to-do-example-mapping) — Session timebox guidance
- [Gherkin Rules (Cucumber Blog)](https://cucumber.io/blog/bdd/gherkin-rules/) — Rules concept in Gherkin
- [Example Mapping: The Three Amigos (QA Masterclass)](https://www.qatouch.com/qa-masterclass/example-mapping-the-three-amigos/) — Three Amigos integration
- [The Behavior-Driven Three Amigos (Automation Panda)](https://automationpanda.com/2017/02/20/the-behavior-driven-three-amigos/) — Role definitions
- [From 3 Amigos to 4: AI-Powered BDD (Ai4Testers)](https://ai4testers.com/blog/from-3-amigos-to-4-ai-powered-bdd-is-here/) — AI in BDD context
- [Three AI Amigos (Antony Sallas, Medium, Jan 2026)](https://medium.com/@asallas/three-ai-amigos-a-multi-model-approach-to-ai-driven-development-2ef7ec2d1ef4) — Multi-model AI BDD approach
- [Three Amigos and a Generative AI Assistant (AWS Builder)](https://builder.aws.com/content/2mG3roWdRCGNe33RzpIkzdZYgqD/three-amigos-and-a-generative-ai-assistant) — AI assistant in Three Amigos

### Supporting Sources (MEDIUM confidence)

- [Example Mapping (Draft.io)](https://draft.io/example/example-mapping) — Visual template
- [How to Do Example Mapping (Toby The Tester)](https://tobythetesterblog.wordpress.com/2016/05/25/how-to-do-example-mapping/) — Practical guide
- [Example Mapping (Platform Development Playbook)](https://playbook.platformdev.amdigital.co.uk/Ways-of-Working/Toolkit/Example-Mapping/) — Team playbook
- [Example Mapping in Practice (Mechanical Rock)](https://blog.mechanicalrock.io/2023/03/21/example-mapping-in-practice.html) — Real-world application
- [BDD in Action, Second Edition (Manning)](https://livebook.manning.com/book/bdd-in-action-second-edition/chapter-5/v-3/) — Book reference
- [Experiment with Example Mapping (Lisa Crispin)](https://lisacrispin.com/2016/06/02/experiment-example-mapping/) — Practitioner experience
- [Example Mapping: Getting the Most from Cucumber (Avenue Code)](https://blog.avenuecode.com/example-mapping-a-way-back-to-cucumber-purpose) — Practical tips
- [Example Mapping (Learn, TeamUp Labs)](https://teamuplabs.com/articles/example-mapping/) — Rules-examples relationship
