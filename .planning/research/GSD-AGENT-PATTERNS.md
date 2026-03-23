# GSD Subagent Design Patterns - Research

**Researched:** 2026-03-24
**Domain:** GSD orchestrator-subagent architecture, prompt template design, dispatch patterns
**Confidence:** HIGH (based on direct reading of GSD source files + Claude Code agent mechanism docs)

---

## Summary

GSD's subagent system uses a consistent set of design patterns across all its workflows (research, planning, execution, debugging, verification). These patterns form a reusable catalog for designing custom agents like the `/case` skill's codebase-mapper and assumptions-analyzer.

The core insight: GSD separates **what the agent knows** (system prompt, baked into the agent `.md` file) from **what the agent works on** (context, passed via the dispatch prompt). Templates are thin context-passing vehicles; all expertise lives in the agent definition. This separation enables agent reuse across workflows while the orchestrating command provides per-invocation specificity.

**Primary recommendation:** For the `/case` skill, follow GSD's Pattern 2 (Project-Level Custom Agents) with thin dispatch prompts. Define agent expertise in `.claude/agents/*.md` files. Pass only phase-specific context at dispatch time. Use `files_to_read` for file paths and `objective` for the mission statement.

---

## 1. Pattern Catalog

### Pattern 1: Thin Template, Fat Agent

**Description:** The dispatch prompt (template) provides only context variables and output expectations. All domain expertise, methodology, quality gates, and behavioral guidelines live in the agent's system prompt (the `.md` file body).

**Evidence from GSD source files:**

The planner template (`planner-subagent-prompt.md`) explicitly states:

> **Note:** Planning methodology, task breakdown, dependency analysis, wave assignment, TDD detection, and goal-backward derivation are baked into the gsd-planner agent. This template only passes context.

Similarly, the debug template (`debug-subagent-prompt.md`) states:

> Template for spawning gsd-debugger agent. The agent contains all debugging expertise -- this template provides problem context only.

**How it works:**

```
Agent definition (.md file body):
  - Full methodology (how to research / plan / debug)
  - Quality gates and checklists
  - Output format specifications
  - Anti-patterns and guardrails
  - Tool usage strategy

Dispatch prompt (from orchestrator):
  - Phase number, name, description
  - File paths to read
  - Mode flags (standard vs gap_closure)
  - Output file path
```

**Implication for /case agents:** The `case-codebase-mapper.md` agent file should contain ALL analysis methodology (what to look for, how to structure findings, what counts as an operation, how to group categories). The dispatch prompt from `case.md` should pass only: phase number, phase name, locked decisions, and output file path.

**Confidence:** HIGH -- this pattern is universal across all 5 GSD workflows examined.

---

### Pattern 2: Files-to-Read Delegation

**Description:** Orchestrators pass file paths, not file contents. The agent reads files itself using its own Read tool. This avoids bloating the dispatch prompt and lets the agent selectively consume what it needs.

**Evidence from GSD source files:**

Every dispatch uses `<files_to_read>` with path lists:

```xml
<files_to_read>
- {context_path} (USER DECISIONS from /gsd:discuss-phase)
- {requirements_path} (Project requirements)
- {state_path} (Project decisions and history)
</files_to_read>
```

The research-phase workflow passes paths with parenthetical descriptions explaining what each file contains:

```xml
<files_to_read>
- {context_path} (USER DECISIONS from /gsd:discuss-phase)
- {requirements_path} (Project requirements)
- {state_path} (Project decisions and history)
</files_to_read>
```

The executor workflow passes more files but follows the same pattern:

```xml
<files_to_read>
- {phase_dir}/{plan_file} (Plan)
- .planning/PROJECT.md
- .planning/STATE.md
- .planning/config.json
- ./CLAUDE.md
- .claude/skills/ or .agents/skills/
</files_to_read>
```

**Key detail:** Each path has a parenthetical annotation explaining its purpose. This helps the agent prioritize which files to read first and how to use each one.

**Implication for /case agents:** The mapper dispatch should pass paths like:

```xml
<files_to_read>
- .planning/ROADMAP.md (phase description and requirements)
- {phase_dir}/*-CONTEXT.md (locked decisions)
- proto/*.proto (service definitions)
</files_to_read>
```

NOT the contents of those files inline in the prompt.

**Confidence:** HIGH -- universal pattern, no exceptions found.

---

### Pattern 3: Structured XML Sections in Dispatch Prompts

**Description:** GSD dispatch prompts use XML tags to create clear semantic sections. Each section has a specific purpose that the agent's system prompt is trained to consume.

**Evidence -- the recurring tag vocabulary:**

| XML Tag | Purpose | Used In |
|---------|---------|---------|
| `<objective>` | Mission statement -- what the agent must accomplish | All dispatches |
| `<files_to_read>` | File paths to load as context | All dispatches |
| `<additional_context>` | Extra context not in files (descriptions, IDs, instructions) | Researcher |
| `<output>` | Where to write results | Researcher |
| `<planning_context>` | Phase metadata + file paths combined | Planner |
| `<downstream_consumer>` | Who consumes the output and what format they need | Planner |
| `<quality_gate>` | Checklist of conditions that must be true before returning | Planner, Checker |
| `<verification_context>` | What to verify and against what | Checker |
| `<expected_output>` | What the return should look like | Checker |
| `<symptoms>` | Problem description with structured fields | Debugger |
| `<mode>` | Behavioral flags (goal, mode switches) | Debugger |
| `<success_criteria>` | Definition of done | Executor |
| `<execution_context>` | Additional methodology files to load | Executor |
| `<deep_work_rules>` | Anti-shallow-execution guardrails | Planner |
| `<parallel_execution>` | Parallelism-specific instructions | Executor |

**The pattern:** Tags are not random. They form a protocol between orchestrator and agent. The agent's system prompt knows which tags to expect and how to interpret each one. The tags act as structured fields in a "contract" between caller and callee.

**Implication for /case agents:** Define a small, consistent tag vocabulary for case agent dispatches. Recommended tags:

- `<objective>` -- what the agent should do
- `<phase_context>` -- phase number, name, description, locked decisions
- `<files_to_read>` -- paths to analyze
- `<output>` -- where to write and what format

Do NOT invent unnecessary tags. Fewer, well-defined tags are better than many overlapping ones.

**Confidence:** HIGH -- consistent across all examined templates.

---

### Pattern 4: Downstream Consumer Documentation

**Description:** Dispatch prompts explicitly tell the agent who will consume its output and what format they need. This prevents agents from producing well-structured but useless output.

**Evidence:**

The planner dispatch includes a `<downstream_consumer>` section:

```xml
<downstream_consumer>
Output consumed by /gsd:execute-phase. Plans need:
- Frontmatter (wave, depends_on, files_modified, autonomous)
- Tasks in XML format with read_first and acceptance_criteria fields (MANDATORY)
- Verification criteria
- must_haves for goal-backward verification
</downstream_consumer>
```

The researcher system prompt (this agent's own prompt) includes a full table:

```markdown
| Section | How Planner Uses It |
|---------|---------------------|
| **`## Standard Stack`** | Plans use these libraries, not alternatives |
| `## Architecture Patterns` | Task structure follows these patterns |
```

**Why this matters:** Without downstream consumer documentation, agents optimize for completeness or readability. With it, they optimize for the specific fields and format their consumer needs. The planner doesn't need beautiful prose; it needs tables with library names, versions, and purposes.

**Implication for /case agents:** The codebase-mapper dispatch prompt should include:

```
Your output is consumed by the /case main conversation (the Protester).
It needs:
- A list of operations with names, endpoints, input/output types
- Grouped by natural category (CRUD cluster, auth flow, etc.)
- Existing validation patterns (so the Protester doesn't re-ask about handled cases)
- Existing test coverage (so the Protester knows what's already tested)
```

The assumptions-analyzer dispatch should include:

```
Your output is consumed by the /case main conversation (the Protester).
It needs:
- A findings list, each with: what was found, where (file:line), suggested case
- Presented to the developer for confirmation before incorporation
```

**Confidence:** HIGH -- explicit in planner template and researcher system prompt.

---

### Pattern 5: Quality Gates as Checklists

**Description:** Quality gates are expressed as markdown checklists that the agent must verify before returning. They serve as exit criteria that prevent premature completion.

**Evidence:**

Planner quality gate:

```xml
<quality_gate>
- [ ] PLAN.md files created
- [ ] Valid frontmatter
- [ ] Tasks specific and actionable
- [ ] Every task has read_first with file being modified
- [ ] Every task has grep-verifiable acceptance_criteria
- [ ] Every action contains concrete values
- [ ] Dependencies identified
- [ ] Waves assigned
- [ ] must_haves derived from phase goal
</quality_gate>
```

Executor success criteria:

```xml
<success_criteria>
- [ ] All tasks executed
- [ ] Each task committed individually
- [ ] SUMMARY.md created
- [ ] STATE.md updated
- [ ] ROADMAP.md updated
</success_criteria>
```

**Placement:** Quality gates appear in both the dispatch prompt (per-invocation expectations) AND the agent system prompt (methodology-level expectations). The system prompt gates are more general; the dispatch prompt gates are more specific.

**Implication for /case agents:** The mapper agent definition should include a quality gate:

```markdown
## Quality Gate

Before returning your analysis:
- [ ] All operations for the phase scope have been identified
- [ ] Each operation has: name, interface (endpoint/RPC), input fields with types, output fields, auth requirement
- [ ] Operations are grouped by natural category
- [ ] Existing validation patterns documented with file:line references
- [ ] Existing test files listed with coverage summary
- [ ] Domain constraints extracted from code
```

**Confidence:** HIGH -- consistent across all agent types.

---

### Pattern 6: Structured Return Protocols

**Description:** Agents signal completion using a standardized header format. The orchestrator parses these headers to determine next actions (advance, retry, present to user).

**Evidence from GSD workflows:**

The researcher returns:

```markdown
## RESEARCH COMPLETE
**Phase:** {phase_number} - {phase_name}
**Confidence:** [HIGH/MEDIUM/LOW]

### Key Findings
[3-5 bullet points]

### File Created
`path/to/RESEARCH.md`
```

Or if blocked:

```markdown
## RESEARCH BLOCKED
**Phase:** {phase_number} - {phase_name}
**Blocked by:** [what]
```

The planner returns `## PLANNING COMPLETE` or `## ISSUES FOUND`.

The verifier returns `## VERIFICATION PASSED` or `## VERIFICATION FAILED`.

**The pattern:** Return headers follow `## {ACTION} {STATUS}` format. The orchestrator (workflow script or main conversation) checks for these headers to determine the next step.

**Implication for /case agents:** Define return headers for case agents:

Mapper:
```markdown
## BRIEFING COMPLETE
**Operations found:** [count]
**File:** {path to CASE-BRIEFING.md}
```

Analyzer:
```markdown
## ANALYSIS COMPLETE
**Findings:** [count]
**New cases suggested:** [count]
**Conflicts found:** [count]
```

**Confidence:** HIGH -- every GSD agent type uses this pattern.

---

### Pattern 7: Mode Flags for Behavioral Variants

**Description:** Dispatch prompts include mode flags that change agent behavior without changing the agent itself. This allows one agent definition to handle multiple scenarios.

**Evidence:**

The planner dispatch includes a `mode` field:

```xml
<planning_context>
**Phase:** {phase_number}
**Mode:** {standard | gap_closure | reviews}
```

The debugger dispatch includes an explicit `<mode>` section:

```xml
<mode>
symptoms_prefilled: {true_or_false}
goal: {find_root_cause_only | find_and_fix}
</mode>
```

**How it works:** The agent's system prompt defines behavior for each mode. The dispatch prompt selects the mode. This avoids creating separate agent definitions for minor behavioral variations.

**Implication for /case agents:** The mapper could support modes like:

```
mode: full_scan (read protos, handlers, tests, validators)
mode: delta_scan (only read files changed since last CASES.md)
```

The analyzer could support:

```
mode: validate (cross-check cases against code)
mode: discover (find behaviors not in the case list)
```

However, for the initial implementation, a single mode is sufficient. Add modes only when a concrete second use case emerges.

**Confidence:** HIGH -- used in planner and debugger dispatches.

---

### Pattern 8: Continuation via Fresh Agent + State File

**Description:** When an agent needs to continue work across checkpoints (e.g., after user input), GSD spawns a fresh agent instance that reads the prior state from a file, rather than resuming the same agent.

**Evidence from debug template:**

```xml
<objective>
Continue debugging {slug}. Evidence is in the debug file.
</objective>

<prior_state>
Debug file: @.planning/debug/{slug}.md
</prior_state>

<checkpoint_response>
**Type:** {checkpoint_type}
**Response:** {user_response}
</checkpoint_response>
```

**Why not resume the same agent?** Claude Code subagents cannot be paused and resumed. When a subagent needs user input, it must return to the orchestrator (main conversation), which collects the input and dispatches a new agent instance with the accumulated state.

**Implication for /case skill:** The `--resume` mechanism in `case.md` already follows this pattern: the main conversation reads the existing CASES.md to restore state. If case agents needed continuation (they don't -- they run to completion), the same file-based state restoration would apply.

**Confidence:** HIGH -- explicit in debug template.

---

### Pattern 9: Revision Loop (Checker-Planner Cycle)

**Description:** GSD implements iterative refinement by dispatching a checker agent after the primary agent, then re-dispatching the primary agent with checker feedback appended. Maximum 3 iterations.

**Evidence from plan-phase workflow:**

1. Planner produces PLAN.md files
2. Checker verifies plans against requirements, returns `## VERIFICATION PASSED` or `## ISSUES FOUND`
3. If issues found, planner re-runs with: `**Checker issues:** {structured_issues_from_checker}`
4. Maximum 3 iterations before escalating to user

**The pattern:**

```
primary_agent -> output
checker_agent -> {PASSED | ISSUES}
if ISSUES:
  primary_agent(original_prompt + checker_issues) -> revised_output
  checker_agent -> {PASSED | ISSUES}
  repeat max 3x
```

**Implication for /case skill:** The current `case.md` does not use a revision loop -- the assumptions-analyzer is a one-shot validation, not an iterative refinement. This is correct because the developer (human) serves as the checker during the conversation itself. Adding a formal checker agent would over-engineer the feedback loop.

**Confidence:** HIGH -- explicitly documented in plan-phase workflow.

---

### Pattern 10: Agent Tool Set Aligned to Role

**Description:** Each GSD agent has a tool set specifically matched to its responsibilities. Read-only agents get read-only tools. Write agents get write tools. No agent gets tools it doesn't need.

**Evidence -- GSD agent tool assignments:**

| Agent | Tools | Rationale |
|-------|-------|-----------|
| gsd-phase-researcher | Read, Write, Bash, Grep, Glob, WebSearch, WebFetch | Needs web for research, write for RESEARCH.md |
| gsd-planner | Read, Write, Bash, Glob, Grep, WebFetch | Needs write for PLAN.md, no web search (uses research) |
| gsd-codebase-mapper | Read, Bash, Grep, Glob, Write | Code analysis + output writing |
| gsd-assumptions-analyzer | Read, Bash, Grep, Glob | Pure analysis, no write needed |
| gsd-executor | Read, Write, Edit, Bash, Grep, Glob | Full code modification |
| gsd-verifier | Read, Write, Bash, Grep, Glob | Read code + write VERIFICATION.md |
| gsd-plan-checker | Read, Bash, Glob, Grep | Read-only verification |
| gsd-debugger | Read, Write, Edit, Bash, Grep, Glob, WebSearch | Full modification + research |

**The principle:** Minimum privilege. An agent that only reads code should not have Write or Edit tools. This prevents accidental side effects and makes the agent's capabilities explicit.

**Implication for /case agents:**

| Case Agent | Recommended Tools | Rationale |
|------------|------------------|-----------|
| case-codebase-mapper | Read, Grep, Glob, Write | Read codebase + write CASE-BRIEFING.md |
| case-assumptions-analyzer | Read, Grep, Glob | Pure analysis, returns structured text to main conversation |

Notably: neither agent needs Bash (no command execution), Edit (no modifying existing files), or WebSearch (no external research).

**Confidence:** HIGH -- consistent across all 8 GSD agent types.

---

## 2. Anti-Patterns to Avoid

### Anti-Pattern 1: Inline Expertise (Fat Template, Thin Agent)

**What it is:** Putting methodology, quality gates, and behavioral guidelines in the dispatch prompt instead of the agent definition.

**Why it's bad:**
- Bloats every dispatch prompt with the same instructions
- Makes the dispatch prompt harder to read and maintain
- Cannot be reused across different orchestrators
- Burns context tokens on every invocation

**What GSD does instead:** All methodology lives in the agent `.md` file. Dispatch prompts are thin context-passing vehicles (Pattern 1).

**Exception:** When the instructions genuinely vary per invocation (like `<deep_work_rules>` in the planner dispatch, which is added for quality enforcement). Even then, these are supplementary guardrails, not core methodology.

---

### Anti-Pattern 2: Passing File Contents Inline

**What it is:** Reading a file in the orchestrator and embedding its contents in the dispatch prompt string.

**Why it's bad:**
- Doubles context consumption (orchestrator reads + agent receives as prompt text)
- Makes prompts enormous and hard to debug
- The agent cannot selectively read portions of large files

**What GSD does instead:** Passes file paths in `<files_to_read>` and lets the agent read them itself (Pattern 2).

**Exception:** Very small, critical snippets that the agent MUST see (e.g., a one-line decision from CONTEXT.md). Even then, GSD passes the path and lets the agent read it.

---

### Anti-Pattern 3: Unstructured Dispatch Prompts

**What it is:** Writing dispatch prompts as freeform prose without XML sections.

**Why it's bad:**
- The agent may miss critical context buried in a paragraph
- No clear contract between orchestrator and agent
- Harder to validate that all required information was provided

**What GSD does instead:** Uses XML tags with a consistent vocabulary (Pattern 3). Each tag maps to a specific kind of information the agent needs.

---

### Anti-Pattern 4: No Return Protocol

**What it is:** Letting the agent return results in whatever format it chooses, then parsing the response heuristically.

**Why it's bad:**
- The orchestrator cannot reliably determine if the agent succeeded or failed
- Next-step logic becomes fragile ("does the response contain the word 'complete'?")

**What GSD does instead:** Defines explicit return headers (`## RESEARCH COMPLETE`, `## PLANNING COMPLETE`, etc.) that the orchestrator checks (Pattern 6).

---

### Anti-Pattern 5: Nested Subagent Hierarchies

**What it is:** Designing agent A to spawn agent B, which spawns agent C.

**Why it's impossible:** Claude Code does not support nested subagent spawning. Only the main conversation can dispatch subagents.

**What GSD does instead:** All subagent orchestration happens at the workflow (main conversation) level. Agents are dispatched sequentially by the orchestrator, passing results between them via files.

---

## 3. Template Structure Recommendation for /case Agents

### Agent Definition File Structure

Based on GSD patterns, each `.claude/agents/*.md` file should follow this structure:

```markdown
---
name: {agent-name}
description: {when Claude should delegate to this agent}
tools: {comma-separated tool list}
model: {sonnet | inherit}
---

# Role and Purpose

[1-2 sentences: what this agent does and why]

## Methodology

[How the agent accomplishes its task -- the domain expertise]

## Input Contract

[What XML tags the agent expects in its dispatch prompt]

| Tag | Required | Contents |
|-----|----------|----------|
| `<objective>` | Yes | Mission statement |
| `<phase_context>` | Yes | Phase metadata |
| `<files_to_read>` | Yes | Paths to analyze |
| `<output>` | Yes | Where to write results |

## Output Contract

[What the agent produces and in what format]

### File Output
[File path and format specification]

### Return Protocol
[Structured return header format]

## Quality Gate

[Checklist the agent verifies before returning]

## Guidelines

[Behavioral rules: what to include, what to skip, how to handle edge cases]
```

### Dispatch Prompt Structure

From `case.md`, each dispatch should follow:

```
Agent(
  subagent_type: "{agent-name}",
  prompt: "<objective>
{One sentence mission}
</objective>

<phase_context>
Phase: {phase_number} - {phase_name}
Description: {phase_description}
Locked decisions: {from CONTEXT.md}
</phase_context>

<files_to_read>
- {path1} (description)
- {path2} (description)
</files_to_read>

<output>
Write to: {output_path}
Format: {reference to format in agent system prompt}
</output>",
  run_in_background: false
)
```

### Concrete Agent Definitions for /case

#### case-codebase-mapper

```markdown
---
name: case-codebase-mapper
description: >
  Analyzes codebase to extract operations, validation rules, test coverage,
  and domain constraints for behavioral case discovery.
  Use when preparing a /case session.
tools: Read, Grep, Glob, Write
model: sonnet
---

# Codebase Mapper for Case Discovery

You analyze a project's codebase to produce a structured briefing of all
operations within a phase's scope. Your output feeds into a conversational
case discovery session where a developer and AI discuss behavioral cases.

## Methodology

1. Read phase context to understand scope (what operations belong to this phase)
2. Scan proto definitions for RPC methods and message types
3. Scan route handlers for REST endpoints
4. For each operation found:
   - Extract name, interface (endpoint/RPC), input fields with types, output fields
   - Identify auth requirements from middleware/interceptors
5. Scan existing validators for input validation patterns already implemented
6. Scan test files for existing coverage
7. Extract domain constraints visible in code (unique constraints, enums, state machines, role checks)
8. Group operations by natural category (CRUD cluster, auth flow, query group)

## Input Contract

| Tag | Required | Contents |
|-----|----------|----------|
| `<objective>` | Yes | "Analyze codebase for Phase X: Name" |
| `<phase_context>` | Yes | Phase number, name, description, locked decisions |
| `<files_to_read>` | Yes | ROADMAP.md, CONTEXT.md, proto files, service directories |
| `<output>` | Yes | Path to write CASE-BRIEFING.md |

## Output Contract

### File: CASE-BRIEFING.md

[format spec with operations list, validation, tests, constraints sections]

### Return Protocol

## BRIEFING COMPLETE
**Operations found:** [count]
**Categories:** [list]
**File:** {output_path}

## Quality Gate

- [ ] All operations for the phase scope identified
- [ ] Each operation has name, interface, input/output types, auth requirement
- [ ] Operations grouped by category
- [ ] Existing validation patterns documented
- [ ] Existing test files listed
- [ ] Domain constraints extracted
- [ ] CASE-BRIEFING.md written to specified path
```

#### case-assumptions-analyzer

```markdown
---
name: case-assumptions-analyzer
description: >
  Cross-checks discovered behavioral cases against actual codebase to find
  gaps, conflicts, and missed edge cases.
  Use after /case discussion is complete.
tools: Read, Grep, Glob
model: sonnet
---

# Assumptions Analyzer for Case Discovery

You cross-check a list of behavioral cases (discovered through developer
conversation) against the actual codebase. You find gaps and conflicts
that the conversation missed.

## Methodology

1. Parse the provided case summary to understand what was discovered
2. For each operation's cases, scan the relevant source code:
   - Validators: are there validation rules the cases don't cover?
   - Error handlers: are there error paths the cases missed?
   - State checks: are there state transitions the cases don't account for?
   - Business logic: are there conditional branches that map to uncovered scenarios?
3. Produce a findings list with file:line references

## Input Contract

| Tag | Required | Contents |
|-----|----------|----------|
| `<objective>` | Yes | "Cross-check cases for Phase X: Name" |
| `<cases>` | Yes | Summary of all discovered cases per operation |
| `<files_to_read>` | Yes | Proto files, service source directories |

## Output Contract

### Return: Structured Findings

Returns findings directly (no file written). Each finding:
- What was found
- Where in code (file:line)
- Suggested case to add or modify

### Return Protocol

## ANALYSIS COMPLETE
**Findings:** [count]
**New cases suggested:** [count]
**Conflicts found:** [count]

## Quality Gate

- [ ] All operations' source code scanned
- [ ] Validation rules cross-checked against failure cases
- [ ] Error handlers cross-checked against failure cases
- [ ] State transitions cross-checked against edge cases
- [ ] Each finding has a file:line reference
- [ ] Each finding has a suggested case action
```

---

## 4. Key Design Principles (Extracted)

### Principle 1: Expertise In, Context Out

Agent definitions hold stable expertise (methodology, quality gates, output formats). Dispatch prompts carry volatile context (phase numbers, file paths, mode flags). This separation enables agent reuse and keeps dispatch prompts small.

### Principle 2: File-Mediated Handoff

Agents communicate through files, not return values. The mapper writes CASE-BRIEFING.md; the main conversation reads it. The executor writes SUMMARY.md; the verifier reads it. Files are durable, inspectable, and debuggable. Return values are transient and invisible after the conversation ends.

### Principle 3: Sequential Chaining from Orchestrator

All multi-agent coordination happens at the orchestrator level (main conversation or workflow command). Agents never dispatch other agents. The orchestrator reads each agent's output and decides what to dispatch next.

### Principle 4: Annotated Path References

File paths in `<files_to_read>` always include parenthetical descriptions: `{path} (what this file contains)`. This helps the agent prioritize reading order and understand each file's role before opening it.

### Principle 5: Explicit Failure Modes

Every return protocol has both success and failure headers. Orchestrators handle both paths. There is no implicit "assume success if no error."

### Principle 6: Progressive Context Loading

Agents receive pointers to many files but do not need to read all of them. The `<files_to_read>` list is ordered by priority, and agents read selectively based on what they need. This avoids loading 10 files when 3 would suffice.

---

## 5. Comparison: GSD Patterns vs Current case.md

| Pattern | GSD Practice | Current case.md | Assessment |
|---------|-------------|-----------------|------------|
| Thin template / fat agent | Agent `.md` has methodology; dispatch is context-only | Dispatch prompt includes methodology inline | **Needs change**: move methodology to agent definition files |
| Files-to-read delegation | Passes paths, agent reads | Partially -- passes some context inline, some paths | **Mostly correct**: standardize to paths-only |
| Structured XML sections | `<objective>`, `<files_to_read>`, `<output>` | Uses `<objective>` but not `<files_to_read>` in dispatch | **Minor gap**: add `<files_to_read>` to mapper dispatch |
| Downstream consumer docs | Explicit consumer description in dispatch | Not present in mapper/analyzer dispatch | **Add**: tell agents who consumes their output |
| Quality gates | Checklist in both agent and dispatch | Not in dispatch prompts | **Add**: quality gate to agent definitions |
| Return protocol | `## ACTION STATUS` headers | Not defined for mapper/analyzer | **Add**: define return headers |
| Tool alignment | Minimum privilege per role | Uses GSD agents (tool sets match) | **Correct**: maintain when creating custom agents |

---

## 6. Decision: Pattern 1 (Reuse GSD) vs Pattern 2 (Custom Agents) vs Pattern 3 (Inline)

Based on the CLAUDE-AGENT-MECHANISMS.md research and this pattern analysis:

### Recommendation: Pattern 2 (Project-Level Custom Agents)

**Rationale:**

1. **Independence from GSD plugin**: The `/case` skill is project-level functionality. It should work even if GSD is uninstalled. Custom agents in `.claude/agents/` are project-scoped and version-controlled.

2. **Methodology specialization**: The case-codebase-mapper needs different analysis methodology than GSD's general-purpose `gsd-codebase-mapper`. GSD's mapper is designed for phase planning context; the case mapper needs operation extraction with validation/test coverage analysis.

3. **Tool control**: Custom agents allow specifying exact tool sets. The case-codebase-mapper should have `Read, Grep, Glob, Write` -- not the Bash tool that GSD's mapper includes.

4. **Follows GSD's own patterns**: GSD itself uses project-level agents (the plugin's `agents/` directory). Creating project-level agents for `/case` follows the same architecture.

5. **Thin dispatch prompts**: With methodology in the agent definition, the dispatch prompts in `case.md` become much smaller and more maintainable.

### Migration path

Current `case.md` dispatches `gsd-codebase-mapper` and `gsd-assumptions-analyzer`. To migrate:

1. Create `.claude/agents/case-codebase-mapper.md` with case-specific methodology
2. Create `.claude/agents/case-assumptions-analyzer.md` with case-specific methodology
3. Update `case.md` dispatch calls to use `case-codebase-mapper` and `case-assumptions-analyzer`
4. Simplify dispatch prompts to context-only (paths, phase metadata, mode)
5. Agent methodology, quality gates, and output format specs move to the agent `.md` files

---

## Sources

### Primary (HIGH confidence -- direct source reading)
- GSD `templates/debug-subagent-prompt.md` -- provided in research prompt
- GSD `templates/planner-subagent-prompt.md` -- provided in research prompt
- GSD `workflows/research-phase.md` -- provided in research prompt (dispatch section)
- GSD `workflows/plan-phase.md` -- provided in research prompt (dispatch sections)
- GSD `workflows/execute-phase.md` -- provided in research prompt (dispatch sections)
- GSD agent types table -- provided in research prompt
- `.planning/research/CLAUDE-AGENT-MECHANISMS.md` -- project research on Claude Code agent system
- `.planning/research/CASE-AGENT-NEEDS.md` -- project research on case skill subagent decisions
- `.planning/research/CASE-SKILL-SYNTHESIS.md` -- project research on case discovery methodology
- `.claude/commands/case.md` -- current case skill definition

### Secondary (MEDIUM confidence -- synthesized from multiple sources)
- GSD agent naming conventions -- inferred from 18 agent filenames listed in CLAUDE-AGENT-MECHANISMS.md
- Claude Code Agent tool parameters -- from official docs via CLAUDE-AGENT-MECHANISMS.md research

---

*Research completed: 2026-03-24*
