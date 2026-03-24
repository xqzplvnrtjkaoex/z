# TDD Research Round 2: GSD Execute Workflow Internals

**Researched:** 2026-03-24
**Domain:** GSD execute-phase orchestrator, execute-plan workflow, gsd-executor agent, TDD injection points
**Confidence:** HIGH (based on existing project research files, GSD docs, DeepWiki analysis, web search verification)

---

## Summary

This document analyzes the GSD execute workflow chain to identify where and how TDD behavior can be injected into the executor. The chain has three layers: (1) the **execute-phase orchestrator** (`execute-phase.md`) which discovers plans, groups them into waves, and dispatches parallel executors; (2) the **execute-plan workflow** (`execute-plan.md`) which is referenced via `@` in each PLAN.md's `<execution_context>` and defines the per-plan task loop; and (3) the **gsd-executor agent** (`agents/gsd-executor.md`) which contains the implementation methodology, deviation rules, and commit patterns.

The key discovery: GSD already has a `tdd="true"` task attribute that the planner generates (visible in existing plans like `02-01-PLAN.md` Task 2 and `03-04-PLAN.md` Tasks 1-2). The gsd-planner has "TDD detection" baked into its methodology. However, what the executor *does* with `tdd="true"` tasks -- whether it enforces red-green-refactor ordering -- is controlled by the executor agent definition and the execute-plan workflow, both of which live in `~/.claude/get-shit-done/` (GSD plugin territory, not project-modifiable). This shapes the injection point analysis significantly.

**Primary recommendation:** Use a multi-layer injection strategy: (1) project-level CLAUDE.md TDD enforcement rules that ALL executors read, (2) planner-level TDD task structuring in PLAN.md action fields, and (3) optionally, a project-local executor override agent. The CLAUDE.md approach is the most reliable injection point because every GSD executor reads it before executing any task.

---

## 1. Execute-Phase Orchestrator

### 1.1 Location and Access

The orchestrator lives at `~/.claude/get-shit-done/workflows/execute-phase.md`. It is a GSD plugin file, not a project file. Direct reading was blocked by tool permissions in this research session.

### 1.2 How It Dispatches Work (Reconstructed from Evidence)

Based on PLAN.md frontmatter, WORKFLOW-RESEARCH.md, GSD-AGENT-PATTERNS.md, and DeepWiki/Mintlify documentation:

```
1. DISCOVER: Scan {phase_dir}/*-PLAN.md for all plan files
2. PARSE: Read frontmatter from each plan (wave, depends_on, autonomous, type)
3. GROUP: Group plans into waves based on `wave` field and `depends_on` references
4. FOR each wave (sequential):
   a. FOR each plan in wave (parallel):
      - Spawn gsd-executor subagent with:
        * PLAN.md file path (contains @-referenced execution_context and context)
        * run_in_background: true (for parallel execution within wave)
   b. WAIT for all parallel executors in wave to complete
   c. RUN post-wave verification (must_haves checks)
   d. IF verification fails: attempt node repair or escalate
5. COLLECT results from all waves
6. WRITE phase-level VERIFICATION.md
7. UPDATE STATE.md and ROADMAP.md
```

### 1.3 What Context Is Passed to Each Executor

Each executor receives context through TWO mechanisms:

**Mechanism A: Dispatch prompt** (from execute-phase orchestrator)
- Phase number, plan number
- Plan file path
- Mode flags

**Mechanism B: @-references in PLAN.md** (read by the executor itself)
The PLAN.md contains `<execution_context>` and `<context>` sections with `@` file references:

```xml
<execution_context>
@~/.claude/get-shit-done/workflows/execute-plan.md   <!-- execution methodology -->
@~/.claude/get-shit-done/templates/summary.md          <!-- output format -->
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/{phase}/{padded}-CONTEXT.md
@.planning/phases/{phase}/{padded}-RESEARCH.md
</context>
```

The executor also reads `./CLAUDE.md` (confirmed by GSD-AGENT-PATTERNS.md Pattern 2: "The executor workflow passes more files... `./CLAUDE.md`"). This is the critical injection point.

### 1.4 Wave Parallelism Details

| Property | Value |
|----------|-------|
| Parallelism unit | Plan (not task) |
| Within-wave | Plans run in parallel (background subagents) |
| Between-waves | Sequential -- wave N+1 waits for wave N |
| Context isolation | Each executor gets fresh 200K context |
| Git isolation | All executors operate on the same branch (no worktree isolation by default) |
| Post-wave step | must_haves verification against plan frontmatter |

### 1.5 Can It Inject Additional Behavior?

The execute-phase orchestrator **cannot easily be modified** because it is a GSD plugin file. However:
- It passes file paths, and the executor reads them -- so anything in those files is injectable
- CLAUDE.md is read by every executor -- project-level TDD rules here affect all execution
- The `<execution_context>` in PLAN.md can reference additional files (planner controls this)
- The orchestrator reads `config.json` for `model_overrides.gsd-executor` -- but config does not control behavior, only model selection

**Confidence:** HIGH for the overall dispatch model. The specific dispatch prompt template was not directly read (tool permissions), but is thoroughly documented in GSD-AGENT-PATTERNS.md and confirmed by DeepWiki sources.

---

## 2. Execute-Plan Workflow

### 2.1 Location and Access

Lives at `~/.claude/get-shit-done/workflows/execute-plan.md`. Referenced via `@` in every PLAN.md's `<execution_context>`. The executor reads this file as methodology instructions.

### 2.2 Per-Plan Execution Loop (Reconstructed)

Based on PLAN.md structure, SUMMARY.md output, GSD docs, and web search:

```
FOR each <task> in <tasks> (sequential within a plan):
  1. READ: Load files listed in <read_first>
  2. EXECUTE: Follow steps in <action>
     - For tdd="true" tasks: expected to write tests first, then implementation
     - For standard tasks: implementation then tests (or as action directs)
  3. VERIFY: Run commands in <verify><automated>
     - Check <acceptance_criteria> (grep-verifiable conditions)
  4. IF verify fails:
     - Apply deviation rules (up to 3 auto-fix attempts)
     - If still failing after 3 attempts: document as Deferred Issue, continue
  5. COMMIT: Atomic git commit for this task
     - Conventional commit message
     - Only files in <files> list
  6. CHECK <done>: Confirm done condition is met
```

### 2.3 Task Success vs Failure

| Outcome | Behavior |
|---------|----------|
| Task succeeds | Atomic commit, move to next task |
| Task verification fails | Deviation rules: 3 auto-fix attempts |
| 3 auto-fixes exhausted | Document as "Deferred Issue" in SUMMARY.md, continue to next task |
| Checkpoint task | Execution stops, control returns to orchestrator for user input |
| Auth error | Treated as authentication gate (not failure) |

### 2.4 Per-Task vs Per-Plan Verification

There are TWO verification layers:

1. **Per-task:** `<verify><automated>` command + `<acceptance_criteria>` check (within execute-plan loop)
2. **Per-plan:** `must_haves` verification in plan frontmatter (post-wave, by orchestrator)

The per-task verification is the executor's responsibility. The per-plan verification is the orchestrator's responsibility.

### 2.5 TDD Task Handling

The `tdd="true"` attribute on `<task>` elements appears in existing plans. Evidence from project plans:

```xml
<task type="auto" tdd="true">
  <name>Task 2: Domain layer (types, ports, errors) and sea-orm entity with migration</name>
  ...
</task>
```

Based on GSD documentation: "TDD requires RED->GREEN->REFACTOR cycles consuming 40-50% context." The planner separates TDD tasks because they are resource-intensive. The **executor is expected to follow the action steps**, which for `tdd="true"` tasks should describe test-first implementation.

**Critical finding:** Whether the executor actually enforces red-green-refactor ordering depends on:
1. The gsd-executor agent definition (its methodology section)
2. The `<action>` content in the plan (what the planner wrote)
3. Project-level instructions in CLAUDE.md (read by executor)

The planner already has "TDD detection" baked in (from GSD-AGENT-PATTERNS.md: "Planning methodology, task breakdown, dependency analysis, wave assignment, TDD detection, and goal-backward derivation are baked into the gsd-planner agent"). But whether the executor respects the `tdd` attribute beyond just following the action steps is unclear from available evidence.

**Confidence:** MEDIUM. The task loop is well-documented. The specific TDD enforcement mechanism in the executor is inferred, not directly read.

---

## 3. GSD-Executor Agent Definition

### 3.1 Location

`~/.claude/get-shit-done/agents/gsd-executor.md` (GSD plugin directory). The `.claude/get-shit-done/agents/` directory does NOT exist at the expected path -- the agents are likely at a different path within the plugin structure. The GSD plugin ships 18 agent definition files including `gsd-executor.md`.

### 3.2 Known Properties (from CLAUDE-AGENT-MECHANISMS.md + GSD docs)

| Property | Value | Source |
|----------|-------|--------|
| Name | `gsd-executor` | CLAUDE-AGENT-MECHANISMS.md |
| Tools | Read, Write, Edit, Bash, Grep, Glob | GSD-AGENT-PATTERNS.md Pattern 10 |
| Model | sonnet (via config override) | config.json |
| Role | Execute PLAN.md files with atomic commits | Multiple sources |

### 3.3 Executor Methodology (Reconstructed)

Based on all available evidence, the gsd-executor methodology includes:

1. **Read CLAUDE.md** for project-specific conventions
2. **Read execute-plan.md** workflow (from `<execution_context>`)
3. **Read plan context files** (from `<context>` @-references)
4. **For each task:**
   a. Read `<read_first>` files
   b. Follow `<action>` steps precisely
   c. Run `<verify>` commands
   d. Check `<acceptance_criteria>`
   e. Apply deviation rules if verification fails (max 3 attempts)
   f. Create atomic git commit
5. **After all tasks:** Write SUMMARY.md, update STATE.md and ROADMAP.md

### 3.4 Deviation Rules (4 Rules)

1. **Scope Boundary:** Only auto-fix issues directly caused by the current task. Pre-existing issues logged to deferred-items.md.
2. **Auto-Fix Limit:** Maximum 3 attempts per task before documenting as Deferred Issue and moving on.
3. **Checkpoint Handling:** When encountering a checkpoint task (type="checkpoint:*"), execution stops and control returns to orchestrator.
4. **Auth Gate:** Authentication errors treated as authentication gates, not failures.

### 3.5 Test-Related Behavior

The executor already has test-adjacent behavior:
- It runs `<verify><automated>` commands (often `cargo test`)
- It checks `<acceptance_criteria>` (often grep-verifiable test-related conditions)
- It follows `<action>` instructions which may include "write tests first"
- The `tdd="true"` attribute signals the task should use test-first methodology

What is NOT clear (and is the key question for this research):
- Does the executor agent definition have explicit TDD methodology instructions?
- Does it check `tdd="true"` and modify its behavior accordingly?
- Or does it simply follow whatever the `<action>` field says?

**Best available answer:** Based on GSD's "Thin Template, Fat Agent" pattern (Pattern 1), the executor agent definition likely contains TDD methodology instructions. The planner's "TDD detection" would be meaningless if the executor didn't have corresponding TDD execution methodology. However, the extent of enforcement (strict red-green-refactor vs. "follow the action field") is unknown without reading the file directly.

**Confidence:** MEDIUM. Reconstructed from indirect evidence. Direct reading of `gsd-executor.md` would confirm.

---

## 4. Injection Points Analysis

### 4.1 Overview of All Injection Points

| # | Injection Point | Location | Scope | Requires GSD Modification | Project-Specific |
|---|----------------|----------|-------|---------------------------|-----------------|
| A | CLAUDE.md TDD rules | `./CLAUDE.md` | This project only | No | Yes |
| B | PLAN.md action field | Per-plan, per-task | This project only | No | Yes |
| C | Project-local executor agent | `.claude/agents/gsd-executor.md` | This project only | No | Yes |
| D | GSD executor agent definition | `~/.claude/get-shit-done/agents/gsd-executor.md` | All GSD projects | Yes | No |
| E | Execute-plan workflow | `~/.claude/get-shit-done/workflows/execute-plan.md` | All GSD projects | Yes | No |
| F | Execute-phase orchestrator dispatch | `~/.claude/get-shit-done/workflows/execute-phase.md` | All GSD projects | Yes | No |

### 4.2 Injection Point A: CLAUDE.md TDD Rules

**How it works:** Every GSD executor reads `./CLAUDE.md` before executing tasks. Add TDD enforcement rules that the executor must follow.

**What to add:**
```markdown
## TDD Enforcement (for GSD Executors)

When executing tasks with `tdd="true"`:
1. Write failing test(s) FIRST, commit them, then run to confirm RED
2. Write MINIMAL implementation to make tests pass, confirm GREEN
3. Refactor if needed
4. Each cycle (red, green) should be verifiable via `cargo test`

When executing ANY task that involves new behavior (not just tdd="true"):
- Write tests before or alongside implementation
- The <verify> command must include test execution
- Never commit implementation code without corresponding test coverage
```

**Feasibility:** HIGH -- zero risk, no GSD modification, immediately effective.

**Scope of change:** Add ~15 lines to CLAUDE.md.

**Limitation:** This is an instruction to the executor, not a hard enforcement mechanism. The executor (an LLM) will follow these instructions with high reliability because CLAUDE.md is treated as authoritative project configuration, but it cannot be programmatically enforced. If the executor's built-in methodology contradicts CLAUDE.md, there could be tension -- but GSD's design gives CLAUDE.md high priority.

**Confidence:** HIGH -- GSD-AGENT-PATTERNS.md confirms executors read CLAUDE.md. WORKFLOW-RESEARCH.md explicitly recommends CLAUDE.md as the integration mechanism for custom behavior.

### 4.3 Injection Point B: PLAN.md Action Field

**How it works:** The planner writes `<action>` steps for each task. For TDD tasks, the action can explicitly prescribe red-green-refactor ordering with specific commands.

**Example:**
```xml
<task type="auto" tdd="true">
  <name>Task 2: Create user usecase with unit tests</name>
  <action>
  **Step 1 (RED):** Write failing unit tests for create_user usecase:
  - tests/test_create_user.rs with 5 test cases
  - Run `cargo test -p user -- create_user` and confirm all fail

  **Step 2 (GREEN):** Implement create_user usecase:
  - src/usecase/create_user.rs with CreateUser struct
  - Run `cargo test -p user -- create_user` and confirm all pass

  **Step 3 (REFACTOR):** Clean up implementation...
  </action>
</task>
```

**Feasibility:** HIGH -- the planner already generates action steps, and the executor follows them precisely.

**Scope of change:** Planner instructions (in gsd-planner agent or CLAUDE.md planner guidance) to structure TDD task actions with explicit RED/GREEN/REFACTOR steps.

**Limitation:** Requires the planner to generate correctly structured TDD actions. The planner already has "TDD detection" baked in -- this would refine what it generates for TDD tasks. The planner is also a GSD agent (not directly modifiable), but it reads CLAUDE.md too.

**Confidence:** HIGH -- this is how the system currently works. Existing plans with `tdd="true"` already have test-related action steps (e.g., "Add `#[cfg(test)] mod tests` with tests for...").

### 4.4 Injection Point C: Project-Local Executor Agent Override

**How it works:** Claude Code agent priority resolution: `.claude/agents/` (project) overrides plugin `agents/` (lower priority). Creating `.claude/agents/gsd-executor.md` with `name: gsd-executor` would override the GSD plugin's executor for this project only.

**What it would contain:** A copy of GSD's executor agent definition with added TDD methodology sections.

**Feasibility:** MEDIUM -- technically possible, but:
- Requires knowing the full content of GSD's executor agent definition (not directly accessible in this session)
- Must maintain compatibility with GSD's execute-plan workflow expectations
- If GSD updates its executor, the project override becomes stale
- The override replaces the ENTIRE agent, not just adding to it

**Scope of change:** Create one `.md` file (~200-500 lines, mirroring GSD's executor + TDD additions).

**Limitation:** Maintenance burden. GSD updates will not propagate to the project-local override. Risk of drift between project executor and GSD's evolving executor methodology.

**Confidence:** HIGH for the mechanism (confirmed by CLAUDE-AGENT-MECHANISMS.md priority resolution). MEDIUM for practical implementation (requires GSD executor source content).

### 4.5 Injection Point D: GSD Executor Agent Definition (Global)

**How it works:** Modify `~/.claude/get-shit-done/agents/gsd-executor.md` directly.

**Feasibility:** LOW -- this is a GSD plugin file. Modifications would:
- Affect ALL projects using GSD (not project-specific)
- Be overwritten on GSD plugin updates
- Require understanding GSD's internal agent structure

**Not recommended** for project-specific TDD enforcement.

### 4.6 Injection Point E: Execute-Plan Workflow (Global)

**How it works:** Modify `~/.claude/get-shit-done/workflows/execute-plan.md` to add TDD-specific task loop steps.

**Feasibility:** LOW -- same issues as Point D. Global effect, overwritten on updates.

**Not recommended.**

### 4.7 Injection Point F: Execute-Phase Orchestrator Dispatch (Global)

**How it works:** Modify the dispatch prompt template in `execute-phase.md`.

**Feasibility:** LOW -- same issues. Also, the dispatch is thin (Pattern 1: "Thin Template, Fat Agent").

**Not recommended.**

---

## 5. Wave Parallelism vs TDD

### 5.1 Parallelism Model Recap

```
Wave 1: [Plan 01] [Plan 02] [Plan 03]  <-- parallel, fresh 200K context each
         ↓          ↓          ↓
Wave 2: [Plan 04] [Plan 05]             <-- waits for Wave 1, then parallel
```

Within each plan, tasks execute **sequentially** (task 1 -> task 2 -> task 3). Parallelism is at the plan level, not the task level.

### 5.2 TDD Interaction with Parallelism

**Question:** Can parallel executors write tests that depend on each other's implementations?

**Answer:** No, and this is already handled by GSD's dependency system:
- Plans in the same wave have `depends_on: []` (no cross-dependencies)
- Plans that depend on others are in later waves
- Each executor has its own 200K context and operates independently

**TDD adds no new parallelism constraints.** The existing wave/dependency system already ensures that if Plan 04 depends on Plan 01's output, Plan 04 runs in a later wave. TDD within Plan 01 (writing tests then implementation) is purely internal to that executor's sequential task loop.

### 5.3 Git Conflicts from Parallel TDD

**Potential issue:** Two parallel executors in the same wave both writing to the same test file or the same source file.

**How GSD handles this:** The `files_modified` frontmatter field declares which files each plan touches. The planner ensures no two plans in the same wave modify the same files. This is enforced by the plan-checker's 8-dimension verification.

**TDD implication:** If TDD tasks create test files, those test file paths should be in `files_modified` to prevent conflicts. The planner's existing task generation already includes test files when `tdd="true"`.

### 5.4 Ordering Constraints TDD Introduces

Within a single plan (sequential execution):

| Without TDD | With TDD |
|-------------|----------|
| Task 1: implement feature | Task 1: write failing tests |
| Task 2: write tests | Task 2: implement to pass tests |
| Task 3: verify | Task 3: refactor |

TDD may increase the number of tasks per plan (splitting test-write from implementation-write). This is already reflected in GSD's recognition that "TDD requires RED->GREEN->REFACTOR cycles consuming 40-50% context" and "TDD gets its own dedicated plan."

Between plans in different waves: no new constraints. Dependencies already encode this.

---

## 6. Recommended Injection Strategy

### 6.1 Primary: CLAUDE.md TDD Enforcement (Injection Point A)

**Why:** Highest reliability, zero GSD modification, immediately effective, project-scoped, version-controlled with the project, read by every executor before task execution.

**What to add to CLAUDE.md:**

A `## TDD Enforcement` section (or expand the existing `## Testing` section) with:

1. **For executors:** When a task has `tdd="true"`, follow red-green-refactor ordering. Write failing tests first, run them to confirm failure, then implement.
2. **For all tasks with new behavior:** Prefer test-first. The `<verify>` step must include test execution.
3. **Commit discipline:** For TDD tasks, the atomic commit should include both tests and passing implementation (not tests separately then implementation separately, as that would create a commit with failing tests).

### 6.2 Secondary: Planner TDD Task Structuring (Injection Point B)

**Why:** Ensures the action steps are explicitly structured as RED/GREEN/REFACTOR, so even an executor that doesn't "understand" TDD will follow the right order because the action says so.

**How:** Add to CLAUDE.md a section that instructs the planner on how to write TDD task actions:

```markdown
### PLAN.md TDD Task Format (for GSD Planner)

When creating tasks with `tdd="true"`, structure the <action> as:
1. **RED phase:** Write test(s) with explicit expected behavior, run to confirm failure
2. **GREEN phase:** Write minimal implementation, run to confirm passage
3. **REFACTOR phase:** Clean up, optimize, add documentation
Include specific `cargo test` commands in each phase.
```

### 6.3 Optional: Project-Local Executor Override (Injection Point C)

**When:** Only if CLAUDE.md instructions prove insufficient (i.e., the executor ignores CLAUDE.md TDD rules because its built-in methodology overrides them).

**How:** Create `.claude/agents/gsd-executor.md` with `name: gsd-executor` that overrides the GSD plugin executor. This requires obtaining the GSD executor's current content and adding TDD methodology.

**Risk:** Maintenance burden. Should be considered a fallback, not the primary approach.

### 6.4 Decision Matrix

| Criterion | A: CLAUDE.md | B: PLAN.md actions | C: Local executor |
|-----------|-------------|-------------------|-------------------|
| Implementation effort | Low (add section) | Low (add planner guidance) | High (full agent copy) |
| GSD update resilience | Fully resilient | Fully resilient | Breaks on GSD update |
| Enforcement strength | Instructional | Structural (steps dictate order) | Methodological |
| Project-specific | Yes | Yes | Yes |
| Requires GSD source | No | No | Yes (for initial copy) |
| Composable | Yes (with B and C) | Yes (with A) | Replaces, not composes |

### 6.5 Recommended Implementation Order

1. **Add CLAUDE.md TDD rules** (Point A) -- immediate, no risk
2. **Verify executor behavior** by running a phase with `tdd="true"` tasks
3. **If executor follows TDD ordering:** done. Points A + B are sufficient.
4. **If executor ignores TDD ordering:** add project-local executor override (Point C)

---

## 7. Key Findings

### 7.1 The Execute Workflow Chain

```
execute-phase.md (orchestrator)
  |
  +-- Groups plans into waves
  +-- FOR each wave:
       +-- Dispatches gsd-executor per plan (parallel)
       |    |
       |    +-- Reads execute-plan.md (methodology)
       |    +-- Reads CLAUDE.md (project rules)  <-- PRIMARY INJECTION POINT
       |    +-- Reads PLAN.md (tasks)
       |    +-- FOR each task (sequential):
       |         +-- read_first -> action -> verify -> commit
       |
       +-- Post-wave verification (must_haves)
```

### 7.2 Existing TDD Support in GSD

GSD already has TDD support at multiple levels:
- **Planner:** Has "TDD detection" baked in, generates `tdd="true"` task attributes
- **Plan checker:** Verifies automated test coverage exists for tasks (8th dimension)
- **Nyquist validation:** Maps test commands to requirements before execution
- **Executor:** Has deviation rules with auto-fix attempts (testing feedback loop)

What is missing: **Explicit TDD enforcement in the executor** -- confirming that the executor actually writes tests before implementation when `tdd="true"` is set. The current evidence suggests TDD behavior is primarily driven by the `<action>` content (what the planner writes), not by executor-level TDD methodology.

### 7.3 CLAUDE.md as Universal Injection Point

CLAUDE.md is read by EVERY GSD agent: researcher, planner, plan-checker, executor, verifier. It is the single most effective place to inject project-specific behavior rules. The TDD enforcement rules added to CLAUDE.md will:
- Guide the **planner** to structure TDD tasks correctly
- Guide the **executor** to follow TDD ordering
- Guide the **plan-checker** to verify TDD compliance
- Be **version-controlled** with the project
- Survive **GSD updates** without any maintenance

### 7.4 The `tdd="true"` Attribute

This attribute already exists in GSD plans and is generated by the planner's TDD detection. It appears on tasks that the planner determines are TDD candidates (business logic, validation, algorithms). It signals intent but may not enforce behavior without corresponding executor instructions.

---

## 8. What Could Not Be Verified

The following items could not be directly verified due to tool permission restrictions (unable to read `~/.claude/get-shit-done/` files or use WebFetch):

1. **Exact content of `execute-phase.md`** -- the dispatch prompt template
2. **Exact content of `execute-plan.md`** -- the per-plan task loop methodology
3. **Exact content of `gsd-executor.md`** -- the executor's TDD-specific instructions
4. **Whether the executor has explicit `tdd="true"` handling** in its methodology

These are the key unknowns. Reading these files directly would confirm or refute the hypothesis that TDD enforcement is primarily action-driven (the executor follows whatever `<action>` says) vs. attribute-driven (the executor checks `tdd="true"` and switches methodology).

**Recommended next step:** Read these three files directly in a session with appropriate permissions to confirm the injection strategy.

---

## Sources

### Primary (HIGH confidence -- direct project artifacts)
- `.planning/research/WORKFLOW-RESEARCH.md` -- GSD workflow stages, execute-phase details, artifact flow
- `.planning/research/GSD-AGENT-PATTERNS.md` -- 10 GSD agent patterns including executor tool set and dispatch patterns
- `.planning/research/CLAUDE-AGENT-MECHANISMS.md` -- Agent priority resolution, custom agent override mechanism
- `.planning/WORKFLOW.md` -- Extended workflow pipeline, artifact availability table
- `.planning/config.json` -- GSD configuration including model_overrides.gsd-executor
- `.planning/phases/02-user-profile/02-01-PLAN.md` -- Real plan with `tdd="true"` task, `<execution_context>` structure
- `.planning/phases/03-authentication/03-01-PLAN.md` -- Real plan with `<execution_context>` reference pattern
- `.planning/phases/02-user-profile/02-01-SUMMARY.md` -- Executor output format
- `CLAUDE.md` -- Existing project instructions including TDD preference

### Secondary (MEDIUM confidence -- GSD documentation + DeepWiki analysis)
- [GSD GitHub Repository](https://github.com/gsd-build/get-shit-done) -- README, user guide, release notes
- [DeepWiki: Agent Reference](https://deepwiki.com/gsd-build/get-shit-done/6-agent-reference) -- gsd-executor properties, deviation rules
- [DeepWiki: Agents and Orchestration](https://deepwiki.com/gsd-build/get-shit-done/3.5-agents-and-orchestration) -- Wave parallelism, dispatch model
- [DeepWiki: Execution Commands](https://deepwiki.com/gsd-build/get-shit-done/4.4-execution-commands) -- Execute-phase command details
- [GSD Mintlify Docs: Multi-Agent Orchestration](https://gsd-build-get-shit-done.mintlify.app/concepts/multi-agent-orchestration) -- Executor role, inputs, outputs

### Tertiary (LOW confidence -- web search, needs verification)
- Web search: "gsd-executor TDD behavior" -- confirms TDD as a recognized feature but does not detail enforcement mechanism
- Web search: "execute-plan workflow" -- confirms task loop structure but not TDD-specific steps

---

*Research completed: 2026-03-24*
*Valid until: 2026-04-24 (GSD is actively maintained; executor behavior may evolve)*
