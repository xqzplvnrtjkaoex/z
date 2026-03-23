# Case Category Completeness Audit: S/F/E + SE

**Researched:** 2026-03-24
**Domain:** Behavioral case categorization systems, test taxonomy
**Confidence:** HIGH

---

## Summary

This research audits whether the /case skill's behavioral case category system -- S (Success), F (Failure), E (Edge), plus a planned SE (Side Effect) -- is complete for its purpose: surfacing behavioral cases for each operation BEFORE writing tests.

The audit surveys established categorization systems from testing literature, analyzes each potential gap category, and concludes with a recommendation.

**Primary recommendation:** S/F/E is sufficient as-is. Do NOT add SE as a separate category. Side effects are a cross-cutting concern that manifests as assertions within existing S/F/E cases, not as a distinct behavioral category.

---

## 1. Survey of Existing Categorization Systems

### 1.1 Use Case Analysis: Happy / Alternate / Exception

The classical use case analysis tradition (Cockburn, Jacobson) uses three path types:

| Path | Definition | Maps to S/F/E |
|------|-----------|----------------|
| **Happy Path** (Main) | The preferred, most common sequence that successfully achieves the goal | S (Success) |
| **Alternate Path** | A deviation that still achieves the goal through adjusted steps | S (variant) or E (Edge) |
| **Exception Path** | A complication that prevents the goal from being achieved | F (Failure) |

Source: Modern Analyst, "Happy, Alternate, and Exception Paths" -- extends the same taxonomy to activity diagrams, process diagrams, and state change diagrams.

**Assessment:** S/F/E maps cleanly. The "Alternate Path" (valid but non-standard success) lands in S (if it is a recognized variant) or E (if it is unusual but valid). No gap.

### 1.2 BDD / Cucumber: Positive / Negative Scenarios

BDD literature (Dan North, Matt Wynne, Liz Keogh, Cucumber official docs) does not prescribe a formal case taxonomy. The community convention is:

| Category | Description | Maps to S/F/E |
|----------|------------|----------------|
| **Positive scenario** | The feature works as intended | S |
| **Negative scenario** | Invalid input, unauthorized access, error conditions | F |
| **Edge scenario** | Boundary conditions, unusual but valid inputs | E |

Some practitioners add "sad path" (user makes common mistakes) vs "bad path" (junk/adversarial input), but these are both subtypes of F. BDD emphasizes that scenarios should illustrate behavior, not classify it -- classification is secondary to discovery.

**Assessment:** S/F/E is a superset of the BDD convention. No gap.

### 1.3 Example Mapping: Rules / Examples / Questions

Matt Wynne's Example Mapping does not categorize cases at all. It categorizes *discussion artifacts*:

| Card Color | Artifact | Role |
|-----------|----------|------|
| Yellow | Story | The operation being discussed |
| Blue | Rule | A business constraint |
| Green | Example | A concrete case illustrating a rule |
| Red | Question | An unresolved uncertainty |

Examples (green cards) are not typed as success/failure/edge. The typing happens downstream when examples are organized into test suites. Example Mapping is deliberately category-agnostic at the discovery phase.

**Assessment:** S/F/E is downstream categorization that Example Mapping does not address. No conflict, no gap.

### 1.4 ISTQB CTFL v4.0: Test Design Techniques

ISTQB classifies *techniques* (EP, BVA, Decision Tables, State Transition, Use Case Testing), not *case categories*. The closest to a case taxonomy is the EP distinction:

| Partition Type | Description | Maps to S/F/E |
|---------------|------------|----------------|
| **Valid partition** | Inputs the system should accept | S |
| **Invalid partition** | Inputs the system should reject | F |
| **Boundary values** | Values at partition edges | E (boundary) |

ISTQB also distinguishes *test types* (functional, non-functional, white-box, change-related) and *test levels* (unit, integration, system, acceptance). These are orthogonal to S/F/E -- they describe where and how to test, not what behavior to specify.

**Assessment:** S/F/E aligns with ISTQB's implicit case taxonomy. No gap.

### 1.5 ZOMBIES (Grenning): Zero / One / Many + Boundary / Interface / Exception

ZOMBIES is a *progression framework*, not a categorization system. It tells you the *order* in which to discover cases, not how to label them:

| Dimension | Purpose | Cases Feed Into |
|-----------|---------|-----------------|
| Z (Zero) | Empty/initial state | S (first success) or E (degenerate) |
| O (One) | First meaningful case | S (happy path) |
| M (Many) | Multiple/complex | S, F, or E depending on scenario |
| B (Boundary) | Where behavior changes | E (boundary) |
| I (Interface) | API shape decisions | S (interface design) |
| E (Exception) | Failure modes | F (error paths) |

**Assessment:** ZOMBIES feeds into S/F/E categorization. No tension, no gap.

### 1.6 Kaner's Bug Taxonomy

Cem Kaner's approach (STAR East 2003, with Giri Vijayaraghavan) creates *bug taxonomies* -- catalogs of what can go wrong, organized by domain feature. The taxonomy is expandable and domain-specific. It does not prescribe fixed categories but uses the taxonomy as a *test idea generator*.

Key insight: Kaner's taxonomy operates at a different level. It helps *discover* cases (what could go wrong with shopping cart checkout?), while S/F/E helps *organize* discovered cases. They are complementary, not competing.

**Assessment:** No gap. Kaner's approach is a discovery aid, not a categorization challenge.

### 1.7 Osherove/Khorikov: Three Exit Points

Roy Osherove (The Art of Unit Testing, 3rd ed.) and Vladimir Khorikov (Unit Testing: Principles, Practices, and Patterns) identify three types of *observable outcomes* from a unit of work:

| Exit Point | Description | Assertion Approach |
|-----------|------------|-------------------|
| **Return value** | The function returns a useful value | Assert on the returned value |
| **State change** | The system's observable state changed | Assert on the new state via queries |
| **Third-party call** | A call to an external dependency (side effect) | Verify the interaction occurred (mock) |

This is the most directly relevant framework to the SE question. Osherove and Khorikov explicitly distinguish side effects (third-party calls) as a separate *exit point type* that requires a different *assertion technique* (mocks/spies instead of value assertions).

**Assessment:** This framework explains WHY side effects feel like they need a separate category -- they require different assertion techniques. But the categorization is about *how to assert*, not *what behavior to specify*. See Section 2.

---

## 2. Analysis: Should SE Be a Separate Category?

### 2.1 The Core Question

The question is whether side effects (session creation, cache invalidation, event emission, related entity updates) need a separate output category (SE) in CASES.md, or whether they fit within existing S/F/E categories.

### 2.2 What a "Category" Means in the /case Skill

In the current output format, a category determines:
1. **Which table** a case appears in (Success Cases, Failure Cases, Edge Cases)
2. **The ID prefix** (S1, F1, E1)
3. **The assertion pattern** implied to downstream consumers (planner, test writer)
4. **The summary counts** (S:5 F:8 E:3)

A new SE category would add: a new table section, an SE prefix, a new count column, and signal a different assertion pattern (verify interaction rather than check return value).

### 2.3 The Assertion-Pattern Argument FOR SE

The Osherove/Khorikov framework shows that side effects require fundamentally different verification:

| S/F/E Assertions | SE Assertions (proposed) |
|------------------|------------------------|
| Assert return value matches expected | Verify side effect occurred |
| Assert error code/message matches | Verify side effect did NOT occur (on failure) |
| Assert behavior is defined | Verify side effect parameters are correct |

This is a real distinction. When a test writer sees "S1: Create book succeeds," they know to assert on the response. If side effects (event emitted, related entity updated) are buried in the same row's "Expected Outcome" column, they might be missed.

### 2.4 The Case-Identity Argument AGAINST SE

A side effect is not a *case*. It is a *consequence* of a case. Consider:

```
Operation: CreateBook
S1: Minimal valid book -> 201 Created, returns book with generated ID
```

The side effects of S1 (audit log written, "book created" event emitted, cache warmed) are not separate behavioral cases. They are additional assertions *within* S1's expected outcome. The preconditions and action are identical -- only the assertion target differs.

If SE were a separate category:
```
S1: Minimal valid book -> 201 Created, returns book with generated ID
SE1: Minimal valid book -> "book.created" event emitted with book ID
SE2: Minimal valid book -> audit log entry written with user ID and book ID
```

SE1 and SE2 have *identical* preconditions and actions as S1. They differ only in what is being asserted. This creates redundancy and makes the case list harder to scan -- three rows that are really one case with three assertion points.

### 2.5 The Cross-Cutting Nature of Side Effects

Side effects cross-cut ALL categories, not just Success:

| Category | Side Effect Concern |
|----------|-------------------|
| S (Success) | Event emitted, cache updated, related entity created |
| F (Failure) | Event NOT emitted, cache NOT corrupted, partial state NOT left behind |
| E (Edge) | Idempotent retry: event emitted only once, not twice |

If SE were its own category, the failure-path side effect cases ("on auth failure, no event is emitted") would logically belong in *both* F and SE. This creates a classification ambiguity that does not exist when side effects are assertions within S/F/E cases.

### 2.6 What Real-World Behavioral Spec Systems Do

No established behavioral specification system (BDD, ATDD, Specification by Example, Example Mapping) uses a separate "side effect" category. Instead:

- **Cucumber/Gherkin:** Side effects appear as additional `Then` or `And` clauses within a scenario. "Then the book is created AND a 'book.created' event is published."
- **Specification by Example (Adzic):** Side effects are part of the expected outcome column in example tables.
- **Example Mapping:** Side effects surface as rules ("R4: Creating a book must emit a domain event") with examples that include the side effect in their outcome.

The convention is universal: side effects are part of the outcome specification, not a separate case category.

### 2.7 The Practical Limit on Categories

Research on large-scale BDD (ScienceDirect, 2021) identifies that teams with many scenarios struggle with organization, ownership, and versioning. Adding categories increases cognitive overhead per case (the classifier must decide: is this S, F, E, or SE?) without adding information (the same case would appear in S with the side effect in the outcome column).

The sweet spot for categories is 3-5. At 3 (S/F/E), classification is fast and unambiguous. At 4 (S/F/E/SE), the SE-vs-S decision creates friction. At 5+, the system becomes a taxonomy maintenance burden.

---

## 3. Gap Analysis: Categories NOT Covered by S/F/E

For each potential gap identified in the research objective, analysis of whether it needs a separate category or fits within S/F/E.

### 3.1 Performance / SLA Cases

**Examples:** Response time under 200ms, rate limiting kicks in at 100 req/s, timeout after 30s.

**Fits within S/F/E?** YES, with caveats.
- Rate limiting at threshold: E (boundary). "E5: 100th request in window -> 429 Too Many Requests"
- Timeout behavior: F (infrastructure failure). "F8: Downstream service unresponsive for 30s -> 504 Gateway Timeout"
- Response time guarantees: NOT a behavioral case. Performance SLAs are non-functional requirements tested by different tools (load testing, benchmarks). They do not belong in CASES.md at all.

**Verdict:** No new category. Rate limiting and timeout are E/F cases. Performance benchmarks are out of scope for behavioral specification.

### 3.2 Security-Specific Cases

**Examples:** SQL injection in title field, CSRF token missing, privilege escalation via ID manipulation.

**Fits within S/F/E?** YES.
- Input injection: F (input validation failure). The /case skill already probes this via EP/BVA on input fields. "F12: Title contains SQL injection attempt -> INVALID_ARGUMENT"
- Missing CSRF token: F (auth failure). Same probing category as missing/malformed auth token.
- Privilege escalation (IDOR): F (authorization failure). Already probed in Step 3c-ii. "F7: Valid token, correct role, not resource owner -> NOT_FOUND or PERMISSION_DENIED"

Security cases are Failure cases with a security motivation. The assertion pattern is identical: "given malicious input, the system rejects with appropriate error." No separate assertion technique is needed.

**Verdict:** No new category. Security cases are F cases. The Protester persona already calibrates intensity for security-sensitive operations.

### 3.3 Observability Cases

**Examples:** Expected log entries, metrics emitted, trace spans created.

**Fits within S/F/E?** Partially -- but most are implementation details.

Logging, metrics, and tracing are *implementation concerns*, not *behavioral specifications*. The caller does not observe whether a log entry was written. The /case skill's scope guardrail explicitly excludes implementation details.

Exception: structured audit logs that are part of the business requirements ("the system must log all access to sensitive data"). These are side effects that belong in the Expected Outcome column of S cases.

**Verdict:** No new category. Audit-log requirements are side effects within S/F cases. Observability infrastructure is out of scope for behavioral specification.

### 3.4 Data Consistency / Integrity Cases

**Examples:** Referential integrity maintained, eventual consistency behavior, uniqueness constraints enforced.

**Fits within S/F/E?** YES.
- Uniqueness violation: F (state conflict). "F4: Create book with duplicate title -> ALREADY_EXISTS"
- Referential integrity: F (invalid reference). "F6: Create book referencing non-existent author -> NOT_FOUND"
- Eventual consistency: E (timing edge case). "E3: Read immediately after write -> may return stale data"

**Verdict:** No new category. These are F or E cases depending on whether the behavior is an error or a valid-but-unusual outcome.

### 3.5 Lifecycle / Cleanup Cases

**Examples:** Resource cleanup on deletion, TTL expiration, garbage collection of orphaned records.

**Fits within S/F/E?** YES.
- TTL expiration: E (time-based edge case). "E4: Access resource after TTL -> NOT_FOUND or GONE"
- Cleanup on deletion: Side effect within S. "S3: Delete book -> 204 No Content, associated tags removed"
- Partial failure cleanup: E (partial success). "E5: Book creation fails after tags saved -> tags are cleaned up"

**Verdict:** No new category. Lifecycle behaviors are S, F, or E cases depending on context.

### 3.6 Compatibility / Migration Cases

**Examples:** Backward compatibility with old API version, version negotiation, deprecated field handling.

**Fits within S/F/E?** YES.
- Old API version still works: S (variant success). "S4: Request with v1 content-type -> 200 OK with v1 response format"
- Deprecated field ignored: E (edge). "E6: Request includes deprecated 'summary' field -> accepted, field ignored"
- Incompatible version rejected: F (validation failure). "F9: Request with unsupported API version -> 400 or 415"

**Verdict:** No new category. These are standard S/F/E cases with compatibility context.

### 3.7 Configuration / Feature Flag Cases

**Examples:** Feature enabled/disabled, environment-dependent behavior, feature flag rollout.

**Fits within S/F/E?** YES. Feature flags are preconditions.
- Feature enabled: S (success with feature flag precondition). "S5: Feature X enabled, create book with X field -> 201 with X in response"
- Feature disabled: F or E depending on behavior. "F10: Feature X disabled, request includes X field -> INVALID_ARGUMENT" or "E7: Feature X disabled, request includes X field -> accepted, X field silently ignored"

**Verdict:** No new category. Feature flags are preconditions within existing S/F/E cases.

---

## 4. The Side Effect Problem: Solved Without SE

The gap identified in the research objective is real: side effects are probed during discussion (Step 3c-vi) but have no explicit output representation. The question is whether the solution is a new category or a better use of existing structure.

### 4.1 The Actual Problem

Side effects get lost not because of a missing category, but because the Expected Outcome column in S/F/E tables tends to focus on the *response* and omit *consequences*:

```
Bad:
| S1 | Create book | Valid auth, valid input | POST /v1/books | 201 Created with book ID | must |

Good:
| S1 | Create book | Valid auth, valid input | POST /v1/books | 201 Created with book ID; "book.created" event emitted; audit log written | must |
```

The solution is not a new table (SE) but ensuring that the Expected Outcome column captures ALL observable outcomes, including side effects.

### 4.2 How to Surface Side Effects Without a New Category

The /case skill already probes side effects in Step 3c-vi ("What other state changes when this operation succeeds?"). The discovered side effects should be:

1. **Recorded as rules.** "R5: Every mutation must emit a domain event." This ensures they are tracked at the operation level.
2. **Included in Expected Outcome.** Each S/F/E case that triggers (or should NOT trigger) a side effect includes it in the outcome column.
3. **Flagged in cross-operation concerns.** "All mutation operations emit domain events. Verify: CreateBook (S1), UpdateBook (S1), DeleteBook (S1)."

### 4.3 Case Annotations for Side Effects

The current output format already supports per-case annotations (footnotes below the table). Side effects with complex verification can use these:

```
### Success Cases

| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | Create book | Valid auth, valid input | POST /v1/books | 201 Created; event emitted; audit logged | must |

- **S1:** Side effects: (1) "book.created" event on message bus with {book_id, user_id, timestamp}, (2) audit log entry with action="create_book". On partial failure, neither side effect should occur (atomic).
```

This captures the side effect specification without category proliferation.

---

## 5. Analysis of Other Categories Considered

### 5.1 INV (Invariant) -- Rejected

**Concept:** Cases that verify system invariants hold after operations (total count consistency, balance equations, referential integrity).

**Why rejected:** Invariants are assertions within existing cases, not separate cases. "After creating a book, total book count increases by 1" is an assertion within S1, not a separate invariant case. Property-based / invariant testing is a test *technique*, not a case *category*.

### 5.2 P (Performance) -- Rejected

**Concept:** Cases specifying performance bounds (response time, throughput).

**Why rejected:** Performance specifications are non-functional requirements. The /case skill specifies *what should happen*, not *how fast it should happen*. Performance testing requires different tools (load generators, profilers) and different assertion patterns (statistical, not deterministic). Including P cases would conflate behavioral specification with performance engineering.

### 5.3 SEC (Security) -- Rejected

**Concept:** Explicitly security-motivated cases (injection, CSRF, privilege escalation).

**Why rejected:** Security cases are F (Failure) cases. The assertion pattern is identical: "given adversarial input, the system rejects." Adding a SEC category would create classification ambiguity ("is missing auth token an F or a SEC?") without adding information. The Protester persona already increases probing intensity for security-sensitive operations.

### 5.4 X (Cross-Operation) -- Rejected

**Concept:** Cases that span multiple operations (create-then-read, delete-then-read-again).

**Why rejected:** Cross-operation concerns already have a dedicated section in the output format ("## Cross-Operation Concerns"). This is not a case category but a section for consistency analysis. Individual cross-operation scenarios are still S/F/E within their primary operation.

---

## 6. Recommendation

### Final Category List: S / F / E (unchanged)

| Category | ID Prefix | Definition | Assertion Pattern |
|----------|-----------|-----------|-------------------|
| **S (Success)** | S1, S2... | Valid inputs, expected preconditions, operation achieves its goal | Return value matches expected; side effects occurred as specified |
| **F (Failure)** | F1, F2... | Invalid inputs, auth failures, state conflicts, operation correctly rejected | Error code/message matches expected; no side effects occurred |
| **E (Edge)** | E1, E2... | Boundary values, concurrency, unusual but valid scenarios | Behavior is defined (could be success or graceful failure); side effects appropriate to outcome |

### Why NOT Add SE

1. **Side effects are not cases.** They are consequences of cases. A side effect has the same preconditions and action as the case it accompanies -- only the assertion target differs.
2. **Side effects cross-cut all categories.** They appear in S (event emitted), F (event NOT emitted), and E (event emitted only once). A separate SE category cannot cleanly capture this.
3. **No established system uses a separate side effect category.** BDD, ATDD, Specification by Example, Example Mapping, ISTQB, ZOMBIES -- none separate side effects into their own case type.
4. **The real problem is output fidelity, not categorization.** Side effects get lost when the Expected Outcome column focuses only on the response. The fix is ensuring outcomes capture ALL observable effects, including side effects.
5. **Adding a category increases classification overhead without adding information.** Every case that triggers a side effect would need to be classified as both S (or F or E) and SE, creating ambiguity.

### What to Do Instead

1. **Enhance the Expected Outcome column guidance.** Explicitly state that Expected Outcome must include ALL observable effects: return values, state changes, side effects (events, notifications, cache mutations, related entity updates), and the *absence* of side effects on failure paths.
2. **Add a "Side Effects" sub-section per operation (optional).** After the Rules section, before the case tables, list the operation's side effects as reference. This is not a case category but a checklist:
   ```
   ### Side Effects
   - Domain event: "book.created" emitted on success
   - Audit log: entry written with user_id and action
   - Cache: book list cache invalidated
   ```
3. **Use case annotations for complex side effect specifications.** When a side effect has non-obvious parameters or atomicity requirements, add a footnote below the table.
4. **Cross-operation concerns section catches side effect consistency.** "All mutation operations emit domain events" belongs here.

---

## 7. Confidence Assessment

| Area | Confidence | Reasoning |
|------|-----------|-----------|
| S/F/E is sufficient | HIGH | Surveyed 7 established categorization systems. None use a separate side effect category. The S/F/E framework maps cleanly to every surveyed system. |
| SE should NOT be added | HIGH | The Osherove/Khorikov exit-point framework is the strongest argument FOR SE, but it describes assertion techniques, not case categories. Multiple established systems confirm side effects belong in outcome specifications. |
| No other missing categories | HIGH | Analyzed 7 potential gap areas (performance, security, observability, data consistency, lifecycle, compatibility, configuration). All fit within S/F/E. |
| Side effects need better output representation | HIGH | The gap is real but is about output format, not categorization. The Expected Outcome column and per-case annotations are sufficient vehicles. |

**Overall confidence: HIGH.** The recommendation is consistent across all surveyed sources and withstands the strongest counterargument (assertion-pattern differences).

---

## Sources

### Primary (HIGH confidence -- foundational testing literature)

- Roy Osherove -- The Art of Unit Testing, 3rd ed. Three exit point types (return value, state change, third-party call)
- Vladimir Khorikov -- Unit Testing: Principles, Practices, and Patterns. Observable behavior vs implementation details; two criteria for observable behavior
- ISTQB CTFL Syllabus v4.0.1 -- EP/BVA classification (valid/invalid partitions), test type taxonomy
- Cem Kaner & Giri Vijayaraghavan -- Bug Taxonomies: Use Them to Generate Better Tests (STAR East 2003). Domain-specific, expandable, discovery-oriented
- Matt Wynne -- Example Mapping. Category-agnostic discovery (Rules/Examples/Questions)
- James Grenning -- ZOMBIES. Progression framework, not categorization
- Dan North / Martin Fowler -- BDD, Given/When/Then. Positive/negative scenarios, side effects in Then clauses

### Secondary (MEDIUM confidence -- synthesized from multiple practitioner sources)

- Modern Analyst -- Happy/Alternate/Exception path taxonomy (extends Cockburn use cases)
- Gojko Adzic -- Specification by Example. Side effects as part of outcome specification
- Martin Fowler -- Test Invariant (bliki). Invariants as complementary assertions, not separate categories
- Liz Keogh -- BDD categories; negative scenarios in BDD
- ScienceDirect (2021) -- "Adapting BDD for large-scale software systems." Organizational challenges at scale
- Mark Ploeh -- "Mocks for Commands, Stubs for Queries" (CQS perspective on side effect verification)

### Tertiary (LOW confidence -- general web sources, verified against primary)

- Wikipedia -- Happy path, Command-query separation, Behavior-driven development
- BrowserStack, TestDevLab -- TDD vs BDD vs ATDD overview
- Enterprise Craftsmanship (Khorikov blog) -- Pragmatic integration testing, database state assertions

---

*Research completed: 2026-03-24*
*Valid until: 2026-06-24 (stable domain, foundational concepts)*
