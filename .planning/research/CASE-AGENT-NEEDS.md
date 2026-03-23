# /case Skill: Subagent Needs Analysis

**Analyzed:** 2026-03-24
**Source:** `.claude/commands/case.md`, `.planning/research/CASE-SKILL-SYNTHESIS.md`
**Confidence:** HIGH (analysis based on direct reading of the skill definition and synthesis)

---

## 1. Per-Step Analysis

### Overview Table

| Step | Work Type | Context Load | Subagent? | Reasoning |
|------|-----------|-------------|-----------|-----------|
| 1a: Phase setup | Bash + JSON parse | LOW | No | Trivial init, 5 lines of bash |
| 1b: Load phase context | File reading | LOW-MEDIUM | No | Reads 3-4 files, sets up main agent's own context |
| 1c: Resume check | File reading + user question | LOW | No | Read one file, ask one question |
| 1d: Operation extraction | Codebase analysis | HIGH | **Yes** | Heavy code reading with structured output |
| 2: Select operations | User conversation | LOW | No | Present list, get selection |
| 3: Per-op discussion | Conversation + reasoning | HIGH (cumulative) | No | Core skill value, requires continuity |
| 4: Cross-operation | Conversation | MEDIUM | No | Requires memory of all Step 3 discussions |
| 5: Validate cases | Codebase analysis | HIGH | **Yes** | Code reading against case list, structured output |
| 6: Write output | File generation | MEDIUM | No | Requires memory of all discussions |

### Detailed Per-Step Analysis

#### Step 1a-1c: Phase Setup, Context Loading, Resume Check

**Work:** Run a bash command to get phase metadata. Read CONTEXT.md, RESEARCH.md, ROADMAP.md. Check for existing CASES.md.

**Context consumed:** Low. These are small metadata/planning files (typically 50-200 lines each). The main agent NEEDS this context anyway to conduct Step 3 discussions.

**Subagent benefit:** None. The main agent must internalize this context to be the Protester. Offloading it to a subagent would mean either (a) the subagent reads files and produces a summary the main agent then reads -- adding latency for no context savings, or (b) the subagent does the work and passes nothing back -- losing essential context.

**Tools needed:** Read, Bash (for gsd-tools init).

**Verdict: Inline.** Zero benefit from isolation. The context loaded here is the foundation for all subsequent steps.

---

#### Step 1d: Operation Extraction (gsd-codebase-mapper)

**Work:** Scan the codebase to extract operations relevant to the phase. Read proto files, route handlers, RPC method definitions, existing validators, existing tests. Produce a structured CASE-BRIEFING.md with operations list, existing validation, existing tests, and domain constraints.

**Context consumed:** HIGH. Must read potentially 10-30 source files across proto definitions, service handlers, domain types, existing test files. The raw code is voluminous but the output is compressed: a structured briefing of ~100-200 lines.

**Subagent benefit analysis:**

| Gained | Lost |
|--------|------|
| Context savings: main agent receives ~150 lines instead of reading ~2000+ lines of raw code | Latency: subagent spawn + execution adds 30-90 seconds |
| Isolation: if the mapper fails, main agent context is clean | Handoff overhead: must describe phase scope clearly in prompt |
| Specialization: mapper prompt can be tuned for code scanning without Protester persona overhead | Cannot ask clarifying questions to the developer during scanning |

**Output well-defined?** Yes. CASE-BRIEFING.md has a clear structure: operations list with fields/types, existing validation patterns, existing test coverage, domain constraints. This is an ideal handoff point.

**Verdict: Subagent.** The context savings ratio is excellent (~150 lines output from ~2000+ lines of code reading). The output format is well-defined and self-contained. The main agent does not need to see the raw code -- it needs the extracted operation signatures and constraints.

**Risk:** The mapper might miss operations or misinterpret phase scope. Mitigation: the developer corrects gaps in Step 2 ("Any operations I missed?").

---

#### Step 2: Select Operations

**Work:** Present the discovered operations (from Step 1d briefing) grouped by category. Ask the developer which to discuss. Reorder selected operations for logical flow (dependencies first, simple before complex).

**Context consumed:** Low. Uses the briefing from 1d (~150 lines) plus user's selection response.

**Subagent benefit:** None. This is a brief conversational exchange (one message out, one response back). The reordering logic is trivial. No code reading involved.

**Tools needed:** None (pure conversation).

**Verdict: Inline.** Fastest possible execution as inline conversation. Subagent would add latency for zero benefit.

---

#### Step 3: Per-Operation Discussion

**Work:** For each selected operation, conduct a depth-first conversation: anchor (confirm understanding), success cases, systematic probing (input validation, auth, state, boundaries, concurrency, side effects, infrastructure), review and close. This is the core of the skill -- typically 5-15 conversational turns per operation.

**Context consumed:** HIGH and cumulative. Each operation adds ~20-50 lines of discovered cases to the running context. For a phase with 8 operations, that is ~200-400 lines of accumulated case data by the end. Additionally, the main agent must retain awareness of decisions made in earlier operations to maintain consistency (e.g., "in CreateBook we decided NOT_FOUND should not reveal existence -- does that apply here too?").

**Subagent benefit analysis (per-operation isolation):**

| Gained | Lost |
|--------|------|
| Context reset between operations: each operation starts fresh | Cross-operation awareness: subagent does not know what was decided for prior operations |
| Parallelism: could theoretically discuss multiple operations simultaneously (but developer is one person) | Conversational flow: developer loses the "session" feeling, each operation feels disconnected |
| Resumability: if one operation discussion fails, others are unaffected | Consistency enforcement: Step 4 cross-operation concerns become much harder because the main agent did not witness the discussions |

**The critical loss is cross-operation awareness.** The /case skill explicitly includes cross-operation consistency checking (Step 4). If each operation is discussed in an isolated subagent, the main agent must reconstruct the full context from subagent outputs to do Step 4. This reconstruction is lossy -- the subagent's structured output captures cases but not the reasoning, tradeoffs, or "we'll handle it the same way as X" decisions that emerged during discussion.

**For very large phases (10+ operations):** Context accumulation is a legitimate concern. However, the mitigation is not per-operation subagents but rather session splitting (the synthesis document's Q5 already identifies this). The developer can run `/case` twice, using `--resume` to continue.

**Tools needed:** AskUserQuestion (conversation with developer), Read (occasional code snippet lookup during discussion).

**Verdict: Inline.** The conversational continuity and cross-operation awareness are essential. Per-operation isolation would damage the skill's core value proposition. Context management for large phases is handled by session splitting (`--resume`), not subagent isolation.

---

#### Step 4: Cross-Operation Concerns

**Work:** After all operations are discussed, check for consistency: error format consistency, shared constraint enforcement, cascading effects between operations (e.g., delete + read-after-delete).

**Context consumed:** Medium. Requires memory of all Step 3 discussions but does not add much new context.

**Subagent benefit:** Negative. This step's entire purpose is synthesizing across operations. It MUST have witnessed (or have full memory of) all prior discussions. A subagent would need the complete case data plus the discussion nuances to be effective.

**Tools needed:** None (pure reasoning from accumulated context).

**Verdict: Inline.** Fundamentally requires the accumulated context of Step 3. Cannot be isolated.

---

#### Step 5: Validate Cases Against Codebase (gsd-assumptions-analyzer)

**Work:** Take the summary of all discovered cases and cross-check against the codebase. Find: implemented behaviors not covered by cases, assumptions in cases that conflict with code, edge cases visible in code but missed in discussion. Return structured findings with file:line references.

**Context consumed:** HIGH. Must read the same (or overlapping) set of source files as Step 1d, but with a different lens -- now checking discovered cases against actual code. The input is the case summary (~200-400 lines). The output is a findings list (~50-100 lines).

**Subagent benefit analysis:**

| Gained | Lost |
|--------|------|
| Context savings: main agent receives ~50-100 lines of findings instead of re-reading ~2000+ lines of code | Latency: subagent spawn + execution adds 30-90 seconds |
| Separation of concerns: code analysis agent vs conversational agent | Risk of redundant code reading (overlaps with Step 1d) |
| Well-defined handoff: case summary in, findings list out | Cannot dynamically ask developer for clarification during analysis |

**Output well-defined?** Yes. Each finding has: what was found, where in code (file:line), suggested case to add/modify. Clean structured output.

**Verdict: Subagent.** Same rationale as Step 1d -- heavy code reading with a well-defined structured output. The main agent only needs the findings to present to the developer. The code analysis is purely mechanical (pattern matching against the case list) and benefits from isolation.

**Risk:** The analyzer might produce false positives (finding code that looks relevant but is not). Mitigation: all findings are presented to the developer for confirmation before being incorporated into the case list.

---

#### Step 6: Write XX-CASES.md

**Work:** Generate the structured output document from all accumulated case data. Present summary for developer review. Write to disk.

**Context consumed:** Medium. Uses the accumulated case data from Step 3 plus any additions from Step 5. No new code reading.

**Subagent benefit:** None. The main agent has all the data in context already. Writing is fast. A subagent would need the entire case data transferred to it, adding latency for no savings.

**Tools needed:** Write (file creation), AskUserQuestion (final confirmation).

**Verdict: Inline.** The main agent already has everything it needs. File writing is a single tool call.

---

## 2. Recommendation Summary

### Two subagents, four inline steps

| Step | Execution | Agent Type |
|------|-----------|------------|
| 1a-1c: Init/Context/Resume | Inline (main agent) | -- |
| 1d: Operation extraction | **Subagent** | `gsd-codebase-mapper` |
| 2: Select operations | Inline (main agent) | -- |
| 3: Per-op discussion | Inline (main agent) | -- |
| 4: Cross-operation | Inline (main agent) | -- |
| 5: Validate cases | **Subagent** | `gsd-assumptions-analyzer` |
| 6: Write output | Inline (main agent) | -- |

This matches the current design in case.md. The current design is correct.

### Why only two subagents

The subagent decision follows a clear pattern:

**Subagent-worthy steps share these properties:**
1. **Heavy code reading** (10-30+ source files)
2. **Well-defined structured output** (briefing doc or findings list)
3. **No conversational interaction needed** (pure analysis)
4. **High compression ratio** (thousands of lines in, hundreds out)
5. **Main agent does not need the raw data** (only the summary)

**Inline steps share these properties:**
1. **Conversational** (require developer interaction)
2. **Context-dependent** (need memory of prior steps)
3. **Low standalone code reading** (planning docs, not source code)
4. **Cumulative** (each step builds on the previous)

---

## 3. Combining vs Separating: Mapper and Analyzer

### Could Steps 1d and 5 share a single agent?

Both steps do codebase analysis. Both read proto files, handlers, validators, and tests. On the surface, combining them seems efficient -- one agent that knows the codebase could serve both purposes.

**However, they should remain separate agents. Here is why:**

| Factor | Combine | Separate |
|--------|---------|----------|
| **Timing** | Would need to run at start, but cases do not exist yet for validation | Each runs at the right time with the right inputs |
| **Input** | Step 1d needs phase scope only; Step 5 needs phase scope + full case list | Clean, minimal inputs for each |
| **Output** | Combined output would be confusing (operations list + validation findings mixed) | Each produces a focused, well-defined document |
| **Context accumulation** | Single agent accumulates all code context, but only Step 5 needs the case summary -- which does not exist when Step 1d runs | Each agent starts fresh with exactly what it needs |
| **Failure isolation** | If the combined agent fails, both outputs are lost | If the mapper fails, the analyzer can still run later |

**The timing argument is decisive.** Step 1d runs before the discussion (it produces the input for discussion). Step 5 runs after the discussion (it validates the output of discussion). They cannot run simultaneously, and the gap between them is the entire discussion phase (potentially 30-60+ minutes of developer conversation). A single agent cannot span that gap.

**Even if they ran sequentially in one agent, the context would be wasted.** The mapper's code reading context is stale by the time the analyzer runs -- new understanding from the discussion changes what to look for. The analyzer benefits from a fresh read of the codebase with the case list as its lens.

**Verdict: Keep separate.** Different timing, different inputs, different outputs, different analytical lenses.

---

## 4. Step 3 (Discussion) Deep Analysis

### Could per-operation discussion benefit from subagent isolation?

This is the most nuanced question. For a phase with 8 operations and 5-10 turns per operation, the main agent accumulates significant context. Would per-operation subagents help?

### Arguments for per-operation subagents

1. **Context ceiling:** For very large phases (12+ operations), the main agent might hit context limits. Each operation adds ~30-50 lines of case data plus ~20-30 lines of conversational reasoning. At 12 operations, that is ~600-960 lines of accumulated context.

2. **Parallelism potential:** In theory, if the developer could handle it, multiple operations could be discussed "simultaneously" (interleaved). In practice, this is terrible UX -- the developer can only focus on one operation at a time.

3. **Crash resilience:** If the main agent crashes at operation 7 of 8, all discussion context for operations 1-6 is lost (unless intermediate results were saved). A subagent per operation would preserve completed operations.

### Arguments against per-operation subagents (stronger)

1. **Cross-operation awareness is essential.** The skill explicitly includes:
   - Step 3a Anchor: "I see these rules..." -- the rules often reference decisions made for prior operations
   - Step 3c-ii Auth: "In OperationA we decided NOT_FOUND should not reveal existence -- same here?"
   - Step 4: Cross-operation consistency checking requires memory of ALL discussions

   A per-operation subagent would lose all of this. The main agent would have to reconstruct it from structured output, which is lossy.

2. **Conversational continuity matters.** The developer builds mental momentum through the session. "Same as before" is a valid and efficient response that only works when the agent remembers "before." Subagent isolation forces the developer to re-explain shared conventions for each operation.

3. **The compression ratio is poor.** Unlike Step 1d (2000 lines of code -> 150 lines of briefing), per-operation discussion context is already fairly compressed (case tables are 30-50 lines per operation). The subagent's output would be nearly as large as its accumulated context. No savings.

4. **Latency penalty is per-operation.** Each subagent spawn adds 30-90 seconds. For 8 operations, that is 4-12 minutes of pure overhead in a skill that targets 30 minutes per operation of developer time.

5. **The `--resume` mechanism already handles the real problem.** Context accumulation for very large phases is solved by session splitting, not agent splitting. The developer runs `/case phase --resume` and picks up where they left off, with the main agent reading the partially-written CASES.md to restore context.

### What about a hybrid: subagent for SOME operations?

One could imagine: inline for the first 5 operations, then subagent for the rest to avoid context overflow. This is worse than either pure approach:
- Inconsistent behavior is confusing for the developer
- The cross-operation awareness problem still applies to the subagented operations
- The point at which to switch is arbitrary and hard to determine

### Verdict on Step 3

**Keep inline. Use `--resume` for context management.**

The discussion step is the core value of the /case skill. Its effectiveness depends on accumulated cross-operation awareness and conversational continuity. These are fundamentally incompatible with per-operation isolation.

For the rare case of phases with 12+ operations (which should themselves be questioned -- a phase that large may need decomposition), the `--resume` mechanism provides a clean session boundary without losing the benefits of conversational continuity within each session.

---

## 5. Context Budget Estimate

To make the inline-vs-subagent decision concrete, here is a rough context budget for a typical /case session (6 operations, moderate complexity):

| Source | Lines | Cumulative |
|--------|-------|------------|
| Phase context (CONTEXT.md, RESEARCH.md, ROADMAP excerpt) | ~200 | 200 |
| CASE-BRIEFING.md (from Step 1d subagent) | ~150 | 350 |
| Per-operation case tables (6 ops x ~40 lines) | ~240 | 590 |
| Conversational overhead (developer responses retained) | ~200 | 790 |
| Step 5 findings (from analyzer subagent) | ~80 | 870 |
| Final CASES.md generation | ~0 (output only) | 870 |

**~870 lines of accumulated context** for a 6-operation phase. This is well within Claude's context window. The subagent isolation for Steps 1d and 5 saves an estimated ~2000-3000 lines of raw code reading context, which IS significant.

Without the subagents (all inline), the budget would be:

| Source | Lines |
|--------|-------|
| Phase context | ~200 |
| Raw code reading for operation extraction (Step 1d inline) | ~1500-2500 |
| Per-operation discussion | ~440 |
| Raw code reading for validation (Step 5 inline) | ~1500-2500 |
| **Total** | **~3640-5640** |

The two subagents save roughly 2500-4000 lines of context. That is the real value -- not the discussion steps.

---

## 6. Final Recommendation

### Current case.md design is correct

The existing design makes exactly the right subagent decisions:

1. **gsd-codebase-mapper (Step 1d):** Correct subagent. Heavy code reading, structured output, no conversation needed, excellent compression ratio.

2. **gsd-assumptions-analyzer (Step 5):** Correct subagent. Heavy code reading, structured output, no conversation needed, good compression ratio.

3. **Steps 2-4, 6 (inline):** Correct. Conversational, context-dependent, cumulative. Subagent isolation would damage the skill's core value.

### No changes needed

The design already follows the pattern: **offload code scanning to subagents, keep conversation and synthesis inline.** This is the optimal split for a conversational skill that needs codebase awareness.

### One potential enhancement (not a subagent change)

Consider adding an intermediate save point between Step 3 operations. After each operation discussion is complete (Step 3d review), append the operation's cases to a scratch file. This provides crash resilience without requiring subagent isolation. The `--resume` mechanism partially addresses this, but an auto-save after each operation would be more robust.

---

## Sources

- `/Users/syr/Developments/madome/.claude/commands/case.md` -- full skill definition with all 6 steps
- `/Users/syr/Developments/madome/.planning/research/CASE-SKILL-SYNTHESIS.md` -- design rationale and technique layers
- Analysis based on Claude Code Agent tool semantics (subagent spawn overhead, context isolation, handoff mechanisms)
