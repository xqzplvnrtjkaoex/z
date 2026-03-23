# Claude Code Agent Mechanisms - Research

**Researched:** 2026-03-24
**Domain:** Claude Code subagent system, custom agent creation, dispatch patterns
**Confidence:** HIGH (based on official Claude Code documentation at code.claude.com)

## Summary

Claude Code provides a comprehensive subagent system that allows custom agents to be defined as Markdown files with YAML frontmatter. Custom agents defined in `.claude/agents/` (project scope) or `~/.claude/agents/` (user scope) are automatically discovered at session start and become available as `subagent_type` values in the Agent tool. The `name` field in the agent's YAML frontmatter is what the Agent tool uses as the `subagent_type` value.

GSD registers its custom agents (like `gsd-codebase-mapper`, `gsd-phase-researcher`) through the **plugin system** -- it ships an `agents/` directory containing `.md` files with YAML frontmatter. When the GSD plugin is installed and enabled, its agents are automatically discovered and become available as subagent types. The plugin agent names appear namespaced as `get-shit-done:gsd-codebase-mapper` in the `/agents` UI but can be dispatched by their `name` field directly.

**Primary recommendation:** Define custom agents as `.claude/agents/*.md` files (project-scoped) or `~/.claude/agents/*.md` (user-scoped). Each agent's `name` field becomes the `subagent_type` value used in Agent tool dispatch. For the `/case` skill, define `case-codebase-mapper` and `case-assumptions-analyzer` as project-level agents in `.claude/agents/`.

## Agent System Architecture

### How Custom Agents Work

1. **Definition**: Custom agents are Markdown files with YAML frontmatter
2. **Location**: `.claude/agents/` (project), `~/.claude/agents/` (user), or plugin `agents/` directory
3. **Discovery**: Agents are loaded at session start. New files require session restart or `/agents` reload
4. **Dispatch**: The `name` field in YAML frontmatter maps directly to the `subagent_type` parameter in the Agent tool
5. **Invocation**: Claude can invoke agents automatically (via description matching), via @-mention, or via explicit `subagent_type` reference

### Priority Resolution

When multiple agents share the same `name`, higher-priority location wins:

| Priority | Location | Scope |
|----------|----------|-------|
| 1 (highest) | `--agents` CLI flag | Current session only |
| 2 | `.claude/agents/` | Current project |
| 3 | `~/.claude/agents/` | All projects |
| 4 (lowest) | Plugin `agents/` directory | Where plugin is enabled |

### Critical Limitation: No Nested Spawning

**Subagents cannot spawn other subagents.** This is a fundamental architectural constraint. If a command/skill dispatches a subagent, that subagent cannot in turn dispatch another subagent. Workarounds:
- Chain subagents from the main conversation (the command orchestrates sequentially)
- Use Skills with `context: fork` to run in isolation without spawning
- Use Agent Teams for sustained parallelism (experimental)

## Agent Tool (formerly Task Tool) Parameters

The Agent tool (renamed from Task in v2.1.63; `Task(...)` still works as alias) accepts these parameters:

| Parameter | Required | Type | Description |
|-----------|----------|------|-------------|
| `subagent_type` | Yes | string | Agent name -- matches `name` field in agent definition (built-in or custom) |
| `description` | Yes | string | Short 3-5 word description of the task |
| `prompt` | Yes | string | Detailed instructions for the subagent |
| `run_in_background` | No | boolean | Run concurrently (default: false, Claude decides) |

**Note:** `model`, `isolation`, `permissionMode`, `tools`, `maxTurns`, `skills`, `memory`, `effort`, and `background` are configured in the agent **definition** file, not passed as Agent tool parameters at dispatch time.

### Built-in Agent Types

| Type | Model | Tools | Purpose |
|------|-------|-------|---------|
| `Explore` | Haiku (fast) | Read-only (no Write/Edit) | Codebase search, file discovery, analysis |
| `Plan` | Inherited | Read-only (no Write/Edit) | Codebase research for plan mode |
| `general-purpose` | Inherited | All tools | Complex multi-step tasks requiring exploration and action |
| `Bash` | Inherited | Terminal commands | Running commands in separate context |
| `claude-code-guide` | Haiku | n/a | Answering questions about Claude Code features |

## Agent Definition File Format

### YAML Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Unique identifier (lowercase letters and hyphens). This becomes the `subagent_type` value |
| `description` | Yes | When Claude should delegate to this subagent (used for automatic delegation) |
| `tools` | No | Allowlist of tools. Inherits all tools if omitted |
| `disallowedTools` | No | Denylist of tools, removed from inherited or specified list |
| `model` | No | `sonnet`, `opus`, `haiku`, full model ID (e.g. `claude-opus-4-6`), or `inherit` (default) |
| `permissionMode` | No | `default`, `acceptEdits`, `dontAsk`, `bypassPermissions`, or `plan` |
| `maxTurns` | No | Maximum agentic turns before agent stops |
| `skills` | No | Skills to preload into agent context at startup |
| `mcpServers` | No | MCP servers available to this subagent |
| `hooks` | No | Lifecycle hooks scoped to this subagent |
| `memory` | No | Persistent memory scope: `user`, `project`, or `local` |
| `background` | No | Always run as background task (default: false) |
| `effort` | No | Effort level: `low`, `medium`, `high`, `max` |
| `isolation` | No | Set to `worktree` for isolated git worktree |

### Minimal Example

```markdown
---
name: case-codebase-mapper
description: Analyzes codebase to extract operations, validation rules, and test coverage for case discovery
tools: Read, Grep, Glob
model: sonnet
---

You are a codebase analysis specialist. Your job is to...
```

### Security Restrictions for Plugin Agents

Plugin-shipped agents do NOT support: `hooks`, `mcpServers`, `permissionMode`. These fields are silently ignored. To use them, copy the agent file into `.claude/agents/` or `~/.claude/agents/`.

## How GSD Registers Custom Agent Types

GSD is installed as a **Claude Code plugin**. The registration mechanism:

1. GSD has a `.claude-plugin/plugin.json` manifest at its root
2. GSD has an `agents/` directory containing 18 agent definition files
3. Each agent file (e.g., `gsd-codebase-mapper.md`) has YAML frontmatter with `name: gsd-codebase-mapper`
4. When the plugin is installed and enabled, Claude Code auto-discovers all agent files in the `agents/` directory
5. Each agent's `name` field becomes available as a `subagent_type` value

**GSD agent files found in `agents/` directory:**
- `gsd-advisor-researcher.md`
- `gsd-assumptions-analyzer.md`
- `gsd-codebase-mapper.md`
- `gsd-debugger.md`
- `gsd-executor.md`
- `gsd-integration-checker.md`
- `gsd-nyquist-auditor.md`
- `gsd-phase-researcher.md`
- `gsd-plan-checker.md`
- `gsd-planner.md`
- `gsd-project-researcher.md`
- `gsd-research-synthesizer.md`
- `gsd-roadmapper.md`
- `gsd-ui-auditor.md`
- `gsd-ui-checker.md`
- `gsd-ui-researcher.md`
- `gsd-user-profiler.md`
- `gsd-verifier.md`

**There is no special registration mechanism.** Simply placing `.md` files with YAML frontmatter in the plugin's `agents/` directory (or `.claude/agents/` for project-level) is sufficient. The `name` field is the registration.

## Dispatch Patterns for the /case Skill

### Pattern 1: Reuse GSD Plugin Agents (Current Approach)

The current `case.md` dispatches `gsd-codebase-mapper` and `gsd-assumptions-analyzer` by name. This works because GSD is installed as a plugin and its agents are discovered automatically.

```
Agent(
  subagent_type: "gsd-codebase-mapper",
  prompt: "...",
  run_in_background: false
)
```

**Pros:**
- No additional agent files needed
- Leverages existing GSD agent system prompts
- Agents are maintained by the GSD project

**Cons:**
- Tight coupling to GSD plugin being installed
- Cannot customize the agent's system prompt for case-specific needs
- If GSD changes its agents, case.md may break

### Pattern 2: Define Project-Level Custom Agents

Create dedicated agent files in `.claude/agents/`:

```markdown
---
name: case-codebase-mapper
description: Analyzes codebase operations, validation, tests, and domain constraints for behavioral case discovery
tools: Read, Grep, Glob
model: sonnet
---

You are a codebase analysis specialist for behavioral case discovery...
[Full system prompt here]
```

Then dispatch from `case.md`:
```
Agent(
  subagent_type: "case-codebase-mapper",
  prompt: "Analyze the codebase for Phase {phase_number}...",
  run_in_background: false
)
```

**Pros:**
- Full control over agent system prompt
- No dependency on GSD plugin
- Version-controlled with the project
- Can customize tools, model, and permissions per agent

**Cons:**
- Must maintain separate agent definitions
- More files to manage

### Pattern 3: Use general-purpose Agent with Detailed Prompts (Inline Template)

Instead of custom agents, use the built-in `general-purpose` type with all instructions in the prompt:

```
Agent(
  subagent_type: "general-purpose",
  prompt: "You are a codebase analysis specialist.

  Your role: analyze the codebase for Phase {phase_number}: {phase_name}.

  [All instructions that would normally go in the agent system prompt]

  Produce a structured analysis document at {phase_dir}/CASE-BRIEFING.md...",
  run_in_background: false
)
```

**Pros:**
- No additional files needed
- All logic self-contained in `case.md`
- Works without any plugin dependency
- Can dynamically adjust instructions per invocation

**Cons:**
- Longer prompts consume more context
- No tool restrictions (general-purpose has all tools)
- Cannot set model, permission mode, or other agent-level config
- Prompt template embedded in command file is harder to maintain

### Pattern 4: Use Explore Agent for Read-Only Analysis

For the codebase mapper (which only reads), use the built-in `Explore` type:

```
Agent(
  subagent_type: "Explore",
  prompt: "Analyze the codebase for Phase {phase_number}: {phase_name}.
  [instructions...]
  Return your analysis as structured text.",
  run_in_background: false
)
```

**Pros:**
- Built-in, always available, no setup
- Fast (uses Haiku model)
- Enforced read-only (no accidental writes)

**Cons:**
- Cannot write output files (no Write tool)
- Uses Haiku which may lack depth for complex analysis
- Less controllable than custom agents
- The main conversation must capture and write output

## Recommended Approach for /case Skill

### Decision Matrix

| Criterion | GSD Agents | Custom .claude/agents | general-purpose | Explore |
|-----------|-----------|----------------------|-----------------|---------|
| Setup effort | None | Moderate | None | None |
| Customizability | None | Full | Partial (prompt only) | Partial |
| Tool control | Via GSD | Full | None (all tools) | Read-only |
| Model control | Via GSD | Full | Inherited | Haiku |
| Plugin dependency | Yes (GSD) | No | No | No |
| Maintenance | External | Internal | Inline | Inline |

### Recommendation

**Use Pattern 2 (Project-Level Custom Agents)** for the following reasons:

1. **Independence**: The `/case` skill should not depend on GSD being installed. It is a project-level skill.
2. **Customization**: Case discovery needs specialized system prompts different from GSD's general-purpose agents.
3. **Tool control**: The codebase mapper should be read-only. The assumptions analyzer may need read-only too.
4. **Version control**: `.claude/agents/` files are checked into the repo, making the skill self-contained.
5. **Model selection**: Can choose `sonnet` for quality analysis rather than accepting inherited or Haiku.

**Recommended agent files:**

1. `.claude/agents/case-codebase-mapper.md` -- Read-only analysis agent (tools: Read, Grep, Glob)
2. `.claude/agents/case-assumptions-analyzer.md` -- Read-only cross-check agent (tools: Read, Grep, Glob)

**Alternative fallback**: If minimal setup is preferred, Pattern 3 (general-purpose with detailed prompts) works well and keeps everything in a single `case.md` file. The trade-off is less control over tools and model.

## Invocation Methods

### Method 1: Automatic Delegation (via description)
Claude reads agent descriptions and automatically delegates when it recognizes a matching task. Include "use proactively" in the description field to encourage this.

### Method 2: @-mention (explicit, guaranteed)
User types `@case-codebase-mapper` in the prompt. Guarantees that specific agent runs.

### Method 3: Natural Language (suggestive)
User says "Use the case-codebase-mapper agent to analyze...". Claude typically delegates but it is not guaranteed.

### Method 4: Session-Wide Agent
```bash
claude --agent case-codebase-mapper
```
The entire session runs as that agent. Not relevant for command dispatch.

### Method 5: From a Command/Skill (programmatic)
The command file includes pseudo-code that Claude interprets:
```
Agent(
  subagent_type: "case-codebase-mapper",
  prompt: "...",
  run_in_background: false
)
```
This is the pattern used in `case.md` and GSD commands. Claude interprets the pseudo-code block and calls the Agent tool with the specified parameters.

## Tool Access Control

### How It Works

- **`tools` field (allowlist)**: Only the listed tools are available. If omitted, all tools are inherited.
- **`disallowedTools` field (denylist)**: Listed tools are removed from the inherited set.
- If both are set, `disallowedTools` is applied first, then `tools` is resolved.
- **`Agent(agent_type)` syntax**: In the `tools` field, restricts which subagent types can be spawned. Only applies to agents running as main thread via `--agent`.

### Practical Tool Sets for Case Agents

**Read-only analysis agent** (codebase mapper):
```yaml
tools: Read, Grep, Glob
```
Cannot write, edit, or run bash. Safe for exploration.

**Read-write analysis agent** (if output file writing needed):
```yaml
tools: Read, Grep, Glob, Write
```
Can write the CASE-BRIEFING.md output file.

**Full agent with bash** (if code execution needed):
```yaml
tools: Read, Grep, Glob, Bash, Write
```

### Important: Subagents Cannot Spawn Subagents

This is critical for the `/case` skill design. Since `case.md` is a command (not an agent itself), the main Claude conversation executes `case.md` and CAN dispatch subagents. However, those subagents (e.g., `case-codebase-mapper`) CANNOT dispatch further subagents. All orchestration must happen from the main conversation level.

The current `case.md` design already follows this pattern correctly:
1. Main conversation (running `case.md`) dispatches `gsd-codebase-mapper`
2. Mapper completes and returns results to main conversation
3. Main conversation later dispatches `gsd-assumptions-analyzer`
4. Analyzer completes and returns results to main conversation

This sequential chaining from the main conversation is the correct pattern.

## Additional Capabilities

### Persistent Memory

Agents can maintain memory across sessions:
```yaml
memory: project  # .claude/agent-memory/<name>/
```
The agent gets a `MEMORY.md` file it can read/write to accumulate learnings. Useful for a case discovery agent that learns project patterns over time.

### Preloading Skills

Agents can have skills injected at startup:
```yaml
skills:
  - api-conventions
  - error-handling-patterns
```
The full skill content is injected into the agent's context. Useful for giving the case agents domain knowledge.

### Hooks

Agents can define lifecycle hooks:
```yaml
hooks:
  PreToolUse:
    - matcher: "Bash"
      hooks:
        - type: command
          command: "./scripts/validate.sh"
```
Not supported for plugin-shipped agents.

### Background Execution

```yaml
background: true
```
Or at dispatch time, the main conversation can ask Claude to "run this in the background". Background agents auto-deny any permission prompts not pre-approved.

### Worktree Isolation

```yaml
isolation: worktree
```
Creates a temporary git worktree for the agent session. Useful when the agent needs to make experimental changes without affecting the working directory. Automatically cleaned up if the agent makes no changes.

## Common Pitfalls

### Pitfall 1: Expecting Subagents to Spawn Subagents
**What goes wrong:** Designing a multi-level agent hierarchy where agent A spawns agent B which spawns agent C.
**Why it happens:** Intuitive mental model of delegation, but architecturally prevented.
**How to avoid:** All subagent dispatching must be done from the main conversation. Chain sequentially.

### Pitfall 2: Agents Not Discovered After File Creation
**What goes wrong:** Creating a new `.claude/agents/*.md` file but the agent is not available.
**Why it happens:** Agents are loaded at session start. New files are not hot-reloaded.
**How to avoid:** Restart the session or use `/agents` to reload.

### Pitfall 3: Plugin Agent Security Restrictions
**What goes wrong:** Setting `hooks`, `mcpServers`, or `permissionMode` in a plugin agent definition and they are silently ignored.
**Why it happens:** Security restriction on plugin-shipped agents.
**How to avoid:** If you need these fields, copy the agent to `.claude/agents/` instead.

### Pitfall 4: Confusing Skills and Agents
**What goes wrong:** Using a skill with `context: fork` when an agent is more appropriate, or vice versa.
**Key distinction:** Skills run in the main conversation context (or forked context with `context: fork`). Agents run in their own context with their own system prompt. Skills share the main conversation's system prompt; agents get only their own markdown body as system prompt.
**When to use what:** Use agents when you need isolated context, specific tool restrictions, or a specialized system prompt. Use skills when you need reusable prompt templates that augment the main conversation.

### Pitfall 5: Verbose Subagent Results Consuming Context
**What goes wrong:** Multiple subagents each returning detailed results, filling up the main conversation's context window.
**How to avoid:** Ask subagents to return concise summaries. Have them write detailed output to files instead.

## Sources

### Primary (HIGH confidence)
- [Claude Code Official Docs - Create custom subagents](https://code.claude.com/docs/en/sub-agents) -- Complete documentation on agent definition, dispatch, and configuration
- [Claude Code Official Docs - Plugins reference](https://code.claude.com/docs/en/plugins-reference) -- Plugin structure, agent registration, manifest schema

### Secondary (MEDIUM confidence)
- [GSD Plugin GitHub - agents directory](https://github.com/gsd-build/get-shit-done/tree/main/agents) -- GSD agent file listing (18 agents)
- [Claude Code System Prompts repository](https://github.com/Piebald-AI/claude-code-system-prompts) -- Agent prompt and tool description extraction

### Tertiary (LOW confidence)
- [Claude Code GitHub Issue #11205](https://github.com/anthropics/claude-code/issues/11205) -- Bug report about agent discovery issues in v2.0.35 (likely resolved in current version)
