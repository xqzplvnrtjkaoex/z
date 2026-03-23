# /case Skill Agent Architecture -- Synthesis

**Synthesized:** 2026-03-24
**Sources:** CASE-AGENT-NEEDS.md, CLAUDE-AGENT-MECHANISMS.md, GSD-AGENT-PATTERNS.md
**Confidence:** HIGH (all three sources are HIGH confidence, based on direct reading of official docs and GSD source files)

---

## 1. Executive Summary

The /case skill needs exactly two custom subagents. Research across all three dimensions (needs analysis, Claude Code mechanisms, GSD design patterns) converges on the same conclusions:

- **Two subagents, not more.** Steps 1d (codebase mapping) and 5 (case validation) are the only steps that benefit from isolation. Both involve heavy code reading (10-30+ source files) with well-defined structured output. All other steps must remain inline for conversational continuity and cross-operation awareness.
- **Define project-level agents, not reuse GSD's.** Create `.claude/agents/case-codebase-mapper.md` and `.claude/agents/case-assumptions-analyzer.md`. This removes the dependency on GSD being installed, allows case-specific system prompts, and enables precise tool control.
- **Use "Thin Template, Fat Agent" pattern.** All analysis methodology, quality gates, and output format specs go in the agent definition files. The dispatch prompts from case.md pass only volatile context: phase number, file paths, and mission statement.
- **Read-only tool sets.** Both agents get `Read, Grep, Glob` only. The mapper additionally gets `Write` (to produce CASE-BRIEFING.md). Neither needs Bash, Edit, or WebSearch.
- **Sonnet model for both agents.** Quality analysis requires more depth than Haiku. Opus would be overkill for structured code scanning.
- **Structured XML dispatch protocol.** Use `<objective>`, `<phase_context>`, `<files_to_read>`, `<output>` tags. Consistent with GSD conventions.
- **Defined return protocols.** Mapper returns `## BRIEFING COMPLETE`; analyzer returns `## ANALYSIS COMPLETE`. Both include counts for the main conversation to verify.
- **The two agents must remain separate** despite doing similar work (both read code). They run at different times, with different inputs, and different analytical lenses. Combining them is architecturally impossible (the case data that the analyzer checks does not exist when the mapper runs).
- **No per-operation subagents.** Step 3 (discussion) MUST stay inline. Cross-operation awareness and conversational continuity are the skill's core value. Context management for large phases uses `--resume`, not agent isolation.
- **Keep dispatch prompts under 20 lines.** With methodology in the agent file, each dispatch becomes a small XML document with paths and metadata.

---

## 2. Agent Architecture

### Overview

```
case.md (main conversation -- the Protester)
  |
  |-- Step 1d --> [case-codebase-mapper] --> CASE-BRIEFING.md
  |                                              |
  |<-- reads briefing, runs Steps 2-4 inline ----|
  |
  |-- Step 5  --> [case-assumptions-analyzer] --> structured findings (returned, not filed)
  |                                                    |
  |<-- presents findings to developer, Step 6 ---------|
```

### Agent 1: case-codebase-mapper

| Property | Value |
|----------|-------|
| **Purpose** | Scan codebase to extract operations, validation rules, test coverage, domain constraints for a phase |
| **When dispatched** | Step 1d, before any developer conversation |
| **Input** | Phase number, name, description, locked decisions, file paths to scan |
| **Output** | `{phase_dir}/CASE-BRIEFING.md` (written by agent) |
| **Tools** | `Read, Grep, Glob, Write` |
| **Model** | `sonnet` |
| **Context savings** | ~2000+ lines of raw code reading compressed to ~150 lines of structured briefing |
| **Return protocol** | `## BRIEFING COMPLETE` with operation count |

### Agent 2: case-assumptions-analyzer

| Property | Value |
|----------|-------|
| **Purpose** | Cross-check discovered behavioral cases against actual codebase to find gaps and conflicts |
| **When dispatched** | Step 5, after all discussion is complete |
| **Input** | Summary of all discovered cases, file paths to scan |
| **Output** | Structured findings returned to main conversation (no file written) |
| **Tools** | `Read, Grep, Glob` |
| **Model** | `sonnet` |
| **Context savings** | ~2000+ lines of code reading compressed to ~50-100 lines of findings |
| **Return protocol** | `## ANALYSIS COMPLETE` with findings/suggestions/conflicts counts |

### Why These Two and Only These Two

Steps that qualify for subagent isolation share ALL of these properties:

1. Heavy code reading (10-30+ source files)
2. Well-defined structured output (briefing doc or findings list)
3. No conversational interaction needed (pure analysis)
4. High compression ratio (thousands of lines in, hundreds out)
5. Main agent does not need the raw data, only the summary

Steps 2-4, 6 fail these criteria: they are conversational, context-dependent, cumulative, and require cross-operation awareness.

---

## 3. Agent Definition Format

Each agent file lives in `.claude/agents/` and follows this structure:

```
.claude/
  agents/
    case-codebase-mapper.md
    case-assumptions-analyzer.md
```

### Recommended File Structure

```markdown
---
name: {agent-name}
description: {1-2 sentence description of when Claude should delegate to this agent}
tools: {comma-separated tool list}
model: sonnet
---

# {Agent Title}

[1-2 sentences: what this agent does and why it exists]

## Methodology

[Step-by-step analysis procedure -- the domain expertise. This is the "fat" part.]

## Input Contract

| Tag | Required | Contents |
|-----|----------|----------|
| `<objective>` | Yes | Mission statement |
| `<phase_context>` | Yes | Phase metadata and locked decisions |
| `<files_to_read>` | Yes | Paths to analyze with annotations |
| `<output>` | Yes | Where to write results / what to return |

## Output Contract

### File Output (mapper) / Return Output (analyzer)
[Format specification]

### Downstream Consumer
[Who reads this output and what they need from it]

### Return Protocol
[Success and failure header formats]

## Quality Gate

[Checklist the agent verifies before returning]

## Guidelines

[Behavioral rules, edge case handling, what to include/skip]
```

### Key YAML Frontmatter Fields

| Field | case-codebase-mapper | case-assumptions-analyzer |
|-------|---------------------|--------------------------|
| `name` | `case-codebase-mapper` | `case-assumptions-analyzer` |
| `description` | Analyzes codebase to extract operations, validation, tests, and domain constraints for case discovery | Cross-checks discovered behavioral cases against codebase to find gaps and conflicts |
| `tools` | `Read, Grep, Glob, Write` | `Read, Grep, Glob` |
| `model` | `sonnet` | `sonnet` |

Fields NOT needed for these agents: `permissionMode` (default is fine), `maxTurns` (let it complete naturally), `skills` (no domain skills needed), `memory` (no cross-session learning needed), `hooks` (no lifecycle events), `background` (must run synchronously), `isolation` (no worktree needed -- read-only agents).

---

## 4. Dispatch Patterns

### How case.md Should Dispatch

The dispatch happens via pseudo-code in the command file that Claude interprets as an Agent tool call. Three parameters are available: `subagent_type` (required), `prompt` (required), `run_in_background` (optional, default false).

All agent configuration (tools, model, methodology) lives in the agent definition file, NOT in the dispatch call.

### Mapper Dispatch (Step 1d)

```
Agent(
  subagent_type: "case-codebase-mapper",
  prompt: "<objective>
Analyze the codebase for Phase {phase_number}: {phase_name}.
Extract all operations, validation patterns, test coverage, and domain constraints.
</objective>

<phase_context>
Phase: {phase_number} - {phase_name}
Description: {phase_description from ROADMAP.md}
Locked decisions: {key decisions from CONTEXT.md}
</phase_context>

<files_to_read>
- .planning/ROADMAP.md (phase description and scope)
- {phase_dir}/*-CONTEXT.md (locked decisions and discretion areas)
- proto/*.proto (service definitions and message types)
- services/{relevant_service}/src/ (handlers, domain, adapters)
</files_to_read>

<output>
Write to: {phase_dir}/CASE-BRIEFING.md
Consumer: /case main conversation (the Protester). Needs operation list with interfaces, inputs, outputs, auth requirements, grouped by category. Needs existing validation patterns and test coverage so already-handled cases are not re-discussed.
</output>",
  run_in_background: false
)
```

### Analyzer Dispatch (Step 5)

```
Agent(
  subagent_type: "case-assumptions-analyzer",
  prompt: "<objective>
Cross-check discovered behavioral cases for Phase {phase_number}: {phase_name} against the codebase.
</objective>

<cases>
{Summary of all discovered cases per operation -- the case tables from Step 3-4 discussion}
</cases>

<files_to_read>
- proto/*.proto (service definitions)
- services/{relevant_service}/src/ (handlers, validators, domain logic)
- {phase_dir}/CASE-BRIEFING.md (original operation extraction for reference)
</files_to_read>

<output>
Return structured findings to the main conversation. Each finding needs: what was found, where in code (file:line), suggested case to add or modify.
Consumer: /case main conversation, which will present findings to the developer for confirmation before incorporating them.
</output>",
  run_in_background: false
)
```

### Dispatch Prompt Constraints

- **Under 20 lines of actual content** (XML tags add structure, not bulk)
- **Pass paths, not file contents** -- the agent reads files itself
- **Annotate each path** with a parenthetical description of what the file contains
- **Include downstream consumer** -- who reads the output and what format they need
- **No methodology in the dispatch** -- all analysis instructions live in the agent definition file

---

## 5. Key Patterns to Apply

These patterns, extracted from GSD's architecture, are the most important for the /case agents:

### P1: Thin Template, Fat Agent

All methodology lives in the agent `.md` file body. The dispatch prompt passes only context (phase metadata, file paths, output location). This keeps dispatch prompts small, makes agents reusable, and avoids burning context tokens on repeated methodology text.

**Current case.md violation:** The mapper dispatch currently includes the full analysis methodology inline ("Extract all operations... Look for: service definitions, route handlers..."). This should move to the agent definition file.

### P2: Files-to-Read Delegation

Pass file paths in `<files_to_read>` tags with annotated descriptions. Let the agent read files itself. Never embed file contents in the dispatch prompt. This avoids doubling context consumption (orchestrator reads + agent receives as prompt text).

### P3: Structured XML Dispatch Protocol

Use a small, consistent tag vocabulary: `<objective>`, `<phase_context>`, `<files_to_read>`, `<output>`. For the analyzer, add `<cases>` to pass the discovered case data. Tags create a clear contract between case.md and its agents.

### P4: Downstream Consumer Documentation

Tell each agent who consumes its output and what format they need. The mapper's consumer is the Protester (main conversation) who needs operation lists with interfaces and types. The analyzer's consumer is also the Protester, who needs findings with file:line references for developer confirmation.

### P5: Quality Gates as Exit Criteria

Each agent definition includes a checklist verified before returning. This prevents premature completion. The mapper must verify all operations are identified, grouped, and annotated. The analyzer must verify all operations' source code was scanned and findings have file:line references.

### P6: Return Protocol Headers

Agents signal completion with `## ACTION STATUS` headers: `## BRIEFING COMPLETE` or `## ANALYSIS COMPLETE`. Include counts (operations found, findings count) so the main conversation can do a quick sanity check without parsing the full output.

### P7: Sequential Chaining from Orchestrator

All dispatch happens from the main conversation (case.md). Agents never spawn other agents (Claude Code does not support nested spawning). The main conversation dispatches mapper, reads its output, conducts discussion, then dispatches analyzer. Results flow through files (CASE-BRIEFING.md) or structured returns (analyzer findings).

---

## 6. Anti-Patterns to Avoid

### A1: Fat Template, Thin Agent (Inline Expertise)

Putting methodology in the dispatch prompt instead of the agent definition. Bloats every dispatch, cannot be reused, burns context tokens. The current case.md does this -- methodology must move to agent files.

### A2: Passing File Contents Inline

Reading a file in case.md and embedding its contents in the dispatch prompt string. Doubles context consumption. Pass paths instead, let the agent read selectively.

### A3: No Return Protocol

Letting agents return results in whatever format they choose. Makes it impossible for case.md to reliably determine success/failure. Define explicit `## ACTION STATUS` headers.

### A4: Per-Operation Subagents for Discussion

Isolating each operation discussion into a separate subagent. Destroys cross-operation awareness, breaks conversational continuity ("same as before" stops working), adds 30-90 seconds latency per operation, and makes Step 4 (cross-operation consistency) impossible without lossy context reconstruction.

### A5: Combining Mapper and Analyzer Into One Agent

They run at different times, with different inputs, for different purposes. The case data that the analyzer checks does not exist when the mapper runs. Even if sequenced in one agent, the mapper's code context is stale after the discussion phase (30-60+ minutes later).

### A6: Nested Agent Hierarchies

Designing agents that spawn other agents. Claude Code prevents this at the platform level. All orchestration must happen at the case.md (main conversation) level.

### A7: Giving Agents Unnecessary Tools

The mapper does not need Bash (no commands to run) or Edit (no files to modify). The analyzer does not need Write (returns findings, does not create files). Follow minimum privilege.

---

## 7. Open Questions

These decisions need developer input before implementation:

### Q1: Should the mapper write CASE-BRIEFING.md or return structured text?

**Options:**
- **(A) Write to file** (recommended): Mapper gets Write tool, produces `{phase_dir}/CASE-BRIEFING.md`. Main conversation reads the file. File is inspectable and debuggable. Follows GSD's file-mediated handoff pattern.
- **(B) Return to main conversation**: Mapper returns structured text. Main conversation receives it in context. No file artifact. Less durable but avoids a transient file.

**Recommendation:** Option A. The briefing is useful as a persistent artifact for debugging and resume scenarios. The `--resume` mechanism benefits from having the briefing on disk.

### Q2: Should the analyzer write findings to a file?

**Options:**
- **(A) Return structured text** (recommended): Analyzer returns findings to main conversation. Main conversation presents them to developer. No file written. Findings are transient -- they get incorporated into CASES.md or discarded.
- **(B) Write to file**: Analyzer writes a findings file. Main conversation reads it. Extra file to clean up.

**Recommendation:** Option A. Findings are consumed immediately and incorporated into CASES.md. A separate findings file adds maintenance overhead with no benefit.

### Q3: Model selection -- sonnet or inherit?

**Options:**
- **(A) Sonnet** (recommended): Explicit model selection ensures consistent quality regardless of what model the main conversation uses.
- **(B) Inherit**: Uses whatever model the main conversation uses. If the user is on Opus, the agents get Opus (expensive but thorough). If on Haiku, the agents get Haiku (fast but potentially shallow for code analysis).

**Recommendation:** Option A. Code analysis quality should be predictable, not vary with the user's session model.

### Q4: Should agents preload any skills?

The `skills` field in YAML frontmatter can inject skill content at agent startup. Potentially useful for giving agents domain knowledge (e.g., project-specific validation conventions). However, the agents already receive project context via `<files_to_read>`, which includes CONTEXT.md and ROADMAP.md.

**Recommendation:** No skills for initial implementation. Add them only if agents consistently miss project-specific patterns.

### Q5: How to handle the `<cases>` tag for the analyzer dispatch?

The analyzer needs the full case summary from Step 3-4 discussion. This is ~200-400 lines of case tables. Options:
- **(A) Inline in dispatch prompt**: Pass the case summary directly in a `<cases>` XML tag. Simple but consumes dispatch prompt space.
- **(B) Write to temp file, pass path**: Main conversation writes case summary to a temp file, passes path in `<files_to_read>`. Cleaner dispatch but adds a file write step.

**Recommendation:** Option A for simplicity. 200-400 lines is well within prompt limits and avoids managing temp files. If case summaries grow beyond 500 lines (very large phases), switch to Option B.

---

## 8. Next Steps

Concrete implementation actions, in order:

### Step 1: Create agent definition files

Create two files:
- `.claude/agents/case-codebase-mapper.md` -- full methodology from current case.md Step 1d dispatch + quality gate + return protocol + downstream consumer docs
- `.claude/agents/case-assumptions-analyzer.md` -- full methodology from current case.md Step 5 dispatch + quality gate + return protocol

Use the file structure from Section 3 above. Move ALL analysis methodology from case.md dispatch prompts into these files.

### Step 2: Update case.md dispatch calls

Replace the current dispatch pseudo-code in Steps 1d and 5:
- Change `subagent_type` from `gsd-codebase-mapper` to `case-codebase-mapper`
- Change `subagent_type` from `gsd-assumptions-analyzer` to `case-assumptions-analyzer`
- Slim dispatch prompts to context-only (XML tags with paths and metadata, no methodology)

### Step 3: Test with a real phase

Run `/case` on an existing phase to verify:
- Agent files are discovered (may need session restart)
- Mapper produces a well-structured CASE-BRIEFING.md
- Analyzer returns useful findings
- Return protocols are parseable
- Quality gates catch incomplete output

### Step 4: Iterate on agent methodology

After first test run, refine the agent system prompts:
- Adjust what the mapper looks for based on actual codebase patterns
- Tune the analyzer's cross-checking heuristics
- Add/remove quality gate items based on real output quality

### Step 5 (optional): Add intermediate save points

Consider auto-saving after each Step 3 operation discussion completes, appending to a scratch file. Provides crash resilience without requiring subagent isolation. The `--resume` mechanism partially covers this, but per-operation auto-save would be more robust.

---

## Sources

- `.planning/research/CASE-AGENT-NEEDS.md` -- per-step subagent analysis, context budget, combining vs separating agents
- `.planning/research/CLAUDE-AGENT-MECHANISMS.md` -- agent definition format, dispatch parameters, tool control, priority resolution, built-in types, discovery mechanism
- `.planning/research/GSD-AGENT-PATTERNS.md` -- 10 design patterns extracted from GSD source, anti-patterns, template structure recommendations, concrete agent definitions
- `.claude/commands/case.md` -- current skill definition (baseline for changes)
