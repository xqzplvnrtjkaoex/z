# Step: Phase Rules + Per-Operation Discussion

## 2.5: Phase Rules Confirmation

Before starting per-operation discussion, present Phase Rules and System Rules from the briefing's Cross-Cutting Constraints section. This happens once, before the first operation.

```
From the briefing, I identified these phase-wide constraints:

Phase Rules (apply to all operations below):
  - PR1: [constraint description] (D-XX)
  - PR2: [constraint description] (D-XX)

System Rules (from PROJECT.md, apply to this phase):
  - SR-01: [brief description] — applies to: [Op1, Op2, ...]
  - SR-02: [brief description] — applies to: [Op1, Op2]

Confirm these phase rules? Any to add, remove, or adjust?
```

The developer can:
- Confirm all
- Demote a PR to R ("that only applies to one operation")
- Promote an R to PR ("that applies to all ops")
- Add new PRs the briefer missed

After confirmation, save Phase Rules to CASE-SCRATCH.md's `## Phase Rules` section (replacing the placeholder from step-init).

If the briefing has no Cross-Cutting Constraints section (older briefing format), scan CONTEXT.md directly for cross-cutting architectural constraints and propose them.

**Mid-discussion PR promotion:** During per-operation discussion, if a constraint discovered for one operation applies to multiple operations, propose promoting it to PR:
```
That sounds like a phase-wide rule. I'll add it as PR[N]: [description].
It will apply to the remaining operations too. Confirm?
```

**SR-candidate discovery:** When the developer uses "project policy" or "same everywhere" language, flag as SR-candidate:
```
That sounds like a system-wide rule (all services, all phases).
I'll flag it as SR-candidate for PROJECT.md promotion.
For now, I'll treat it as Phase Rule PR[N].
```
Record SR-candidates in the `## SR Candidates` section of CASES.md (written during finalize). Do not modify PROJECT.md.

---

For each selected operation, run this sequence. Complete one operation fully before moving to the next.

## 3a: Anchor via flow diagram

Present the operation as an integrated flow diagram following the structure in SKILL.md `<formatting>`: Interface section + flow diagram + Rules + Cases. The AI reads context and forms understanding internally, then presents the compact Interface section as context verification -- not a separate verbose text block.

The Interface section replaces verbose anchor text. It provides the same context verification (inputs, output, auth, preconditions) in compact form. Add any additional fields relevant to the operation (e.g., `Precondition:`, `Caller:`).

In the Rules section of the flow diagram, reference confirmed Phase Rules by ID instead of repeating their text:
```
Rules:
  - R1: [operation-specific rule]
  - Inherits: PR1, PR2, SR-01
```

Only list operation-specific rules (R) in full. Phase Rules and System Rules are referenced by ID — their content is already established in Step 2.5.

After presenting, ask the developer to confirm or correct via AskUserQuestion.

## 3b: Success Cases (brief)

Apply ZOMBIES Zero -> One:

**Zero:** "Before this operation has ever been called -- what does the system look like? What if there is nothing?"

**One:** "What does the simplest successful case look like? Minimal valid input?"

**One (variants):** "Are there variant success cases? Full input with all optional fields?"

**Many:** "What about bulk or multiple-item scenarios? What does a list with many results look like? Any batch operations? Partial failure in bulk?"

Propose success cases as a batch:
```
I expect these success cases:
- S1: [case] -> [expected outcome]
- S2: [case] -> [expected outcome]

Any to add or change?
```

## 3c: Systematic Probing (the bulk)

This is where you add the most value. Apply techniques in order:

**3c-i: Input validation failures (EP per field)**

For each input field: identify valid/invalid partitions, then probe boundaries. Batch by field:

```
For the [field] field, I expect these failure cases:
- Missing entirely -> [error]
- Empty/zero value -> [error]
- Over max length/limit -> [error]
- Invalid format -> [error]

What is the max [length/size]? And at exactly max, it should succeed?
```

For multiple fields, group related probes. Ask about cross-field interactions: "Are there combinations of individually valid fields that are invalid together?"

**3c-ii: Authentication/authorization failures**

```
For auth failures:
- F_: No credentials -> rejected (unauthenticated)
- F_: Expired credentials -> rejected (unauthenticated)
- F_: Valid credentials but insufficient role -> rejected (forbidden)
- F_: Valid credentials, correct role, but not the resource owner -> [not found or forbidden?]

For that last one: should the error reveal whether the resource exists?
```

**3c-iii: Resource state failures (if applicable)**

Apply State Transition Testing when the operation depends on entity state:

```
What state must [entity] be in for this operation?
What happens if it is in [other state] instead?
What if it was soft-deleted?
```

For entities with state machines, enumerate valid/invalid transitions.

**3c-iv: Boundary values (BVA on bounded fields)**

After EP identified partitions, probe boundaries with three-value BVA:

```
You said [field] allows [min]-[max].
- [min-1] -> reject?
- [min] -> accept (boundary)?
- [max] -> accept (boundary)?
- [max+1] -> reject?
```

Apply to: collection sizes, pagination parameters, numeric ranges, string lengths.

**3c-v: Concurrency and idempotency (ZOMBIES Many)**

```
What if this operation is called twice in rapid succession?
- Same user, same input -- idempotent?
- Two users, conflicting input -- which wins?
- Client retries after timeout -- safe or dangerous?
```

**3c-vi: Side effects and data integrity**

Probe side effects systematically by category. Batch all applicable categories into one proposal:

```
When this operation succeeds, I expect these side effects:

Domain events:
- SE_: "[entity].[action]" event emitted with [key fields]

Related entity updates:
- SE_: [related entity] [created/updated/deleted] as consequence

Cache mutations:
- SE_: [cache key/region] invalidated

Audit / logging:
- SE_: Audit log entry with [action, actor, target]

Notifications:
- SE_: [notification type] sent to [recipient]

External system calls:
- SE_: [system] called with [payload summary]

Which of these apply? Any I'm missing?
What if the operation partially succeeds then fails -- are side effects rolled back or left in place?
```

Omit categories that clearly do not apply (e.g., skip "Notifications" for an internal data migration operation). Use SE_ as a working label during discussion; in the final case tables, side effects are recorded in the Expected Outcome column of the relevant S/F/E case, not as a separate category.

**3c-vii: Infrastructure failures (brief, standardized)**

```
Standard infrastructure probes:
- Database unavailable -> [specific status] (ErrorName)
- Downstream service timeout -> [specific status] (ErrorName)
- What specific error does the caller see?

These are usually the same across operations. Confirm or adjust.
```

**Classification criterion:** Does this case need its own code path?
- **Failure (F):** a distinct error condition requiring its own handling code (detection, error response, compensation). External dependency unavailability always falls here.
- **Edge (E):** a boundary variant of an existing S or F case — verifies that existing code handles surprising inputs correctly. May succeed or fail; the point is no new code path, just robustness verification at the edges.

Always specify a concrete error outcome in Expected Outcome, not generic "error". Append the domain error name in parentheses `(ErrorName)` after the error description that emerged from discussion. This names the error identity that will map to a code-level error variant, bridging specification to implementation. Omit the parenthetical only when the error is intentionally opaque by design.

Examples (the error description uses terms natural to the operation's context; `(ErrorName)` is always domain-level):
- `file too large (FileSizeExceeded); upload not started` -- domain-level (unit test, CLI, library)
- `400 Bad Request (DuplicateEmail)` -- HTTP
- `PERMISSION_DENIED (InsufficientQuota)` -- gRPC
- `credentials invalid` -- intentionally opaque, no parenthetical

Each failure case must make the observable outcome unambiguous -- this is what tests will assert against.

## 3d: Review and Close

Present an ASCII flow diagram summarizing all discovered cases for this operation, following the format defined in SKILL.md `<formatting>`. The diagram shows the operation's decision flow with S/F/E cases at their logical positions.

After the diagram, ask:
```
Anything else that could go wrong that we haven't covered?
Any domain-specific risk my systematic probes wouldn't catch?
```

Before closing, verify: every side effect identified in 3c-vi is represented in at least one case's Expected Outcome. Success cases should assert side effects OCCURRED; relevant failure cases should assert side effects DID NOT occur.

**Termination signals:**
- Per rule: 4-5 examples typical; beyond 6, consider splitting the rule
- Per operation: 10-15 cases for simple CRUD, 20-30 for complex operations
- Per operation NOT-READY: 3+ open questions on fundamental behavior -> flag as needs-answers
- Developer answers becoming terse and confident -> wrap up

## 3e: Save to scratch file

After closing each operation, append its case summary to `${phase_dir}/CASE-SCRATCH.md`. This ensures case data survives context compression during long sessions.

Append format per operation:
```markdown
## Operation: [OperationName]

### Rules
- R1: [rule]
- Inherits: PR1, PR2, SR-01

### Side Effects
- Domain event: "[entity].[action]" with [key fields]
- [other side effects by category]
> If the operation has no side effects, write: `None (read-only)` or `None (query operation)`.

### Cases
| ID | Case | Preconditions | Action | Expected Outcome | Priority |
|----|------|---------------|--------|------------------|----------|
| S1 | ... | ... | ... | ... | must |
| F1 | ... | ... | ... | ... | must |

### Open Questions
| ID | Question | Impact | Default Recommendation |
|----|----------|--------|------------------------|
| Q1 | ... | ... | ... |
```

The Side Effects sub-section serves as a quick-reference inventory of what the Expected Outcome column must include. It is not a case category -- cases remain S/F/E only.

**Then move to the next selected operation.**

---

## Transition

After saving the last operation to CASE-SCRATCH.md:
-> Read [step-finalize.md](step-finalize.md) and begin cross-operation analysis.

If mid-session and more operations remain, stay in this file and loop back to 3a for the next operation.
