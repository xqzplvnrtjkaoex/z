# GSD Patch/Reapply System Research

**Researched:** 2026-03-26
**Domain:** GSD workflow tooling, update/patch lifecycle
**Confidence:** MEDIUM (based on web sources; local filesystem verification was not possible)

## Summary

GSD (Get Shit Done) is installed at `~/.claude/get-shit-done/` and copies files into `~/.claude/` directories (`agents/`, `commands/gsd/`, `hooks/`). Since v1.17.0 (2026-02-08), the installer detects locally modified GSD files, backs them up to `gsd-local-patches/`, and creates a `backup-meta.json` manifest. The `/gsd:reapply-patches` command performs an intelligent merge of backed-up user modifications into the freshly installed files.

**Key finding:** Project-local files (`.claude/skills/`, `.claude/agents/`) are NOT managed by GSD's global installer and are safe from GSD updates. Claude Code's native precedence rules mean project-local agents/skills override global ones of the same name. This makes project-local files the preferred location for custom extensions, with the patch system only needed when modifying GSD-owned global files.

**Primary recommendation:** Keep all /case skill and custom agent modifications in project-local directories (`.claude/skills/case/`, `.claude/agents/`). Only use the patch system for modifications to GSD-owned files that cannot be achieved through project-local overrides.

## 1. Patch System Mechanics

### Storage Location

The patch backup directory is searched in priority order:
1. `$HOME/.config/opencode/gsd-local-patches`
2. `$HOME/.opencode/gsd-local-patches`
3. `$HOME/.gemini/gsd-local-patches`
4. `$HOME/.claude/gsd-local-patches`

For our Claude Code setup: **`~/.claude/gsd-local-patches/`**

### Metadata Format

- `backup-meta.json` — manifest file containing file hashes and metadata about which files were modified
- The backed-up files are stored alongside the manifest (full file copies, not diffs)

### Detection Mechanism

During `gsd:update` / reinstall:
1. Installer reads an existing `gsd-file-manifest.json` (records the hash of each originally-installed file)
2. Compares current file content against manifest hashes
3. Files whose content differs from the manifest hash = "locally modified"
4. Modified files are copied to `gsd-local-patches/`
5. `backup-meta.json` records source version, file paths, and modification status

### Merge Process (`/gsd:reapply-patches`)

For each backed-up file:
1. Loads user's archived version and the newly installed version
2. Compares them to identify user-added or user-modified sections
3. Applies user's additions/modifications to the new version
4. Flags conflicts when upstream also modified the same sections (prompts user for resolution)
5. Records outcomes: "Merged", "Skipped" (already incorporated), or "Conflict resolved"
6. Updates manifest to note which files were modified
7. Optionally cleans up the patch backup directory

**Confidence:** MEDIUM — based on the reapply-patches command definition from GitHub and search results; actual implementation details in install.js were only partially visible.

## 2. GSD Update Flow

### What Happens During `/gsd:update`

1. **Version detection** — identifies local vs global installation
2. **npm version check** — queries latest available version
3. **Version comparison** — determines if update is needed
4. **Changelog display** — shows relevant changes since current version
5. **Warning display** — shows what will be replaced
6. **User confirmation** — asks approval before proceeding
7. **Local modification backup** — detects modified files via manifest hash comparison, backs up to `gsd-local-patches/`
8. **Clean install** — re-runs `npx get-shit-done-cc@latest` which overwrites GSD-managed directories
9. **Cache clearing** — removes cached data
10. **Notification** — tells user about backed-up patches, suggests `/gsd:reapply-patches`

### What Gets Overwritten

The installer copies/overwrites these GSD-managed directories at the target location (`~/.claude/` for global install):
- `agents/` — 15+ specialized agent definitions (gsd-planner, gsd-executor, etc.)
- `commands/gsd/` — 57 slash command prompt files
- `get-shit-done/` — core framework (references/, templates/, workflows/, bin/)
- `hooks/` — git/security hooks (gsd-prompt-guard.js, gsd-workflow-guard.js)

**Confidence:** HIGH — confirmed by multiple sources including GitHub repo structure, DeepWiki analysis, and the uninstall behavior description.

## 3. File Ownership Map

### GSD-Owned (WILL be overwritten on update)

| Directory | Location | Contents |
|-----------|----------|----------|
| `~/.claude/get-shit-done/` | Global | Core framework: bin/, workflows/, templates/, references/ |
| `~/.claude/commands/gsd/` | Global | 57 slash command definitions |
| `~/.claude/agents/gsd-*.md` | Global | GSD agent definitions (planner, executor, verifier, etc.) |
| `~/.claude/hooks/gsd-*.js` | Global | Security hooks (prompt-guard, workflow-guard) |

### User-Owned (SAFE from GSD updates)

| Directory | Location | Contents |
|-----------|----------|----------|
| `.planning/` | Project-local | All planning artifacts (PROJECT.md, REQUIREMENTS.md, phases/) |
| `.planning/config.json` | Project-local | GSD project configuration |
| `.claude/skills/` | Project-local | Project-specific skills (e.g., case/) |
| `.claude/agents/` | Project-local | Project-specific agent overrides |
| `~/.claude/skills/` (non-gsd) | Global | User's personal skills (not GSD-prefixed) |
| `~/.claude/agents/` (non-gsd) | Global | User's personal agents (not GSD-prefixed) |
| `CLAUDE.md` | Project-local | Project instructions |

### Shared / Ambiguous

| File | Owner | Notes |
|------|-------|-------|
| `~/.claude/settings.json` | Shared | GSD merges its section using markers; preserves user sections |
| `~/.claude/gsd-local-patches/` | User/GSD | Created by installer, consumed by reapply-patches |
| `~/.claude/gsd-file-manifest.json` | GSD | Tracks installed file hashes for modification detection |

**Confidence:** MEDIUM — the exact set of files GSD overwrites could not be confirmed by reading `install.js` directly. The above is synthesized from repository structure, installer behavior descriptions, and the uninstall documentation ("deletes command and agent files from the runtime's config directory").

## 4. Applicability to /case Skill Modifications

### Current /case Skill Architecture

This project's custom /case skill lives in:
- `.claude/skills/case/` — project-local skill definitions
- `.claude/agents/` — project-local agent definitions (e.g., case-briefer.md)

### Safety Assessment

| Component | Location | Safe from GSD updates? | Reason |
|-----------|----------|----------------------|--------|
| `.claude/skills/case/` | Project-local | YES | GSD installer targets `~/.claude/` (global), not `./.claude/` (project) |
| `.claude/agents/*.md` | Project-local | YES | Same as above — project-local directory not touched |
| `.planning/` artifacts | Project-local | YES | GSD never overwrites planning artifacts |
| CLAUDE.md | Project-local | YES | GSD does not manage CLAUDE.md |

### When Patches WOULD Be Needed

Patches are needed ONLY if we modify GSD-owned global files. Scenarios:

1. **Modifying a GSD agent behavior** (e.g., making `gsd-planner` consume CASES.md)
   - Location: `~/.claude/agents/gsd-planner.md` (GSD-owned)
   - This WOULD be overwritten on update
   - Patch system WOULD be needed
   - **Better alternative:** Use CLAUDE.md instructions to inject behavior into the planner's context (as we currently do in the "CASES.md Integration with Plan-Phase" section)

2. **Modifying a GSD workflow** (e.g., changing execute-phase behavior)
   - Location: `~/.claude/get-shit-done/workflows/execute-phase.md` (GSD-owned)
   - This WOULD be overwritten on update
   - Patch system WOULD be needed

3. **Adding a new GSD command** (e.g., `/gsd:case`)
   - If placed in `~/.claude/commands/gsd/case.md` — GSD-owned territory, risky
   - **Better alternative:** Place in `.claude/commands/case.md` (project-local, non-gsd namespace)

### Current Project Strategy (Already Correct)

The project's current approach is already optimal:
- CLAUDE.md contains integration instructions that GSD agents read at runtime
- Custom skills/agents live in project-local `.claude/` directories
- No GSD-owned files are modified

**Confidence:** HIGH — based on confirmed Claude Code precedence rules and GSD installer scope.

## 5. Claude Code Precedence Rules

Understanding precedence is critical for knowing when project-local overrides work:

### Agent Precedence
**Project `.claude/agents/` > Global `~/.claude/agents/`**

When multiple subagents share the same name, the higher-priority (project-local) location wins. You can verify with `claude agents` which shows all agents grouped by source and indicates overrides.

### Skill Precedence
**Project `.claude/skills/` > Global `~/.claude/skills/`**

Project-level skills take precedence over global ones if they share the same name.

### Known Limitation (Bug)
There is a documented Claude Code bug ([#10061](https://github.com/anthropics/claude-code/issues/10061)): when a sub-agent is invoked via the Task tool, it loads skills from the global `~/.claude/skills/` directory instead of respecting project-local skills in `.claude/skills/`. This means project-specific skill overrides may not work within sub-agent contexts.

**Impact on us:** If a GSD agent (like the executor) spawns a sub-agent that needs our `/case` skill, it might not find the project-local skill. However, our current architecture uses CLAUDE.md directives rather than relying on skill resolution within sub-agents, which sidesteps this issue.

**Confidence:** HIGH — confirmed by Claude Code official docs and filed bug report.

## 6. Limitations and Risks

### Patch System Limitations

| Limitation | Impact | Mitigation |
|------------|--------|------------|
| Section-level merge, not line-level | May miss fine-grained changes within a section | Review merged output carefully |
| Conflicts when upstream changes same section | Requires manual resolution | Keep patches minimal; prefer project-local overrides |
| Manifest corruption | Known issue; corrupted manifest can break detection | Re-run installer if manifest issues occur |
| No version compatibility checking | Patches from old version may not apply cleanly to new version with structural changes | Test after reapply; be prepared to manually re-apply |
| Semantic conflicts not detected | Merge succeeds but behavior breaks because upstream changed assumptions | Review changelog before updating |

### Risks of Modifying GSD-Owned Files

1. **Update friction** — Every GSD update requires manual reapply + conflict resolution
2. **Silent breakage** — GSD may restructure files in ways that make patches semantically invalid even if they apply cleanly
3. **Maintenance burden** — Must track which GSD files are modified and why
4. **Version lock-in** — Complex patches may discourage updating GSD

### Risks of NOT Using Patches (Project-Local Only)

1. **Cannot change GSD agent internals** — If a GSD agent has a bug or limitation, project-local overrides cannot fix it (unless the agent name matches exactly and Claude Code precedence kicks in)
2. **CLAUDE.md injection is best-effort** — GSD agents read CLAUDE.md but may not perfectly follow every directive (depends on the agent's system prompt taking precedence)

## 7. Recommendations for This Project

### Strategy: Project-Local First, Patches as Last Resort

1. **Keep all /case modifications in `.claude/skills/case/` and `.claude/agents/`** (current approach) — SAFE, no patches needed

2. **Use CLAUDE.md for behavior injection into GSD agents** (current approach) — the "CASES.md Integration" sections in CLAUDE.md effectively tell the planner/executor/verifier how to consume CASES.md without modifying their source files

3. **If you MUST modify a GSD-owned file:**
   - Document which file and why in a tracking location (e.g., `.planning/research/GSD-PATCHES.md`)
   - Keep the modification minimal and surgical
   - After each `gsd:update`, run `gsd:reapply-patches`
   - Verify the merged result manually
   - Consider filing a GSD issue/PR if the modification represents a generally useful feature

4. **For the cross-phase constraint forwarding concern** (from `project_cross_phase_forwarding.md`):
   - If this requires modifying `gsd-planner` or `step-discuss` agents, prefer submitting a GSD issue/PR upstream
   - Alternatively, use CLAUDE.md directives to inject the behavior
   - Only patch as a temporary measure while waiting for upstream adoption

### Decision Tree

```
Need to customize GSD behavior?
|
+-- Can it be done via .planning/config.json settings?
|   YES -> Use config (safest)
|
+-- Can it be done via CLAUDE.md directives?
|   YES -> Use CLAUDE.md (safe, project-local)
|
+-- Can it be done via project-local agent/skill override?
|   YES -> Use .claude/agents/ or .claude/skills/ (safe)
|
+-- Must modify GSD-owned file?
    |
    +-- Is the change generalizable?
    |   YES -> File GSD issue/PR upstream, use patch temporarily
    |
    +-- Is the change project-specific?
        YES -> Use patch system, document in tracking file
```

## Sources

### Primary (HIGH confidence)
- [GSD GitHub Repository](https://github.com/gsd-build/get-shit-done) — repo structure, command listing
- [Claude Code Sub-agents Docs](https://code.claude.com/docs/en/sub-agents) — agent precedence rules
- [Claude Code Skills Docs](https://code.claude.com/docs/en/skills) — skill precedence rules

### Secondary (MEDIUM confidence)
- [GSD User Guide](https://github.com/gsd-build/get-shit-done/blob/main/docs/USER-GUIDE.md) — update flow, config
- [GSD CHANGELOG](https://github.com/gsd-build/get-shit-done/blob/main/CHANGELOG.md) — v1.17.0 local patch preservation
- [GSD Commands Directory](https://github.com/gsd-build/get-shit-done/tree/main/commands/gsd) — reapply-patches.md, update.md
- [DeepWiki GSD Installation](https://deepwiki.com/gsd-build/get-shit-done/2.1-installation) — installer behavior
- [Issue #721: Global install corruption](https://github.com/gsd-build/get-shit-done/issues/721) — update mechanism internals
- [Issue #10061: Sub-agent skill resolution bug](https://github.com/anthropics/claude-code/issues/10061) — project skill precedence bug
- [Skool Discussion: Custom scripts wiped](https://www.skool.com/gsd/built-a-skill-to-stop-gsd-updates-from-wiping-my-custom-scripts-is-there-a-better-way?p=e26d3077) — community experience

### Tertiary (LOW confidence)
- [codecentric Deep Dive](https://www.codecentric.de/en/knowledge-hub/blog/the-anatomy-of-claude-code-workflows-turning-slash-commands-into-an-ai-development-system) — GSD architecture overview
- [skillsmp.com reapply-patches skill listing](https://skillsmp.com/skills/amishhyadav-global-twin-agent-skills-gsd-reapply-patches-skill-md) — third-party skill listing

## Metadata

**Confidence breakdown:**
- Patch system mechanics: MEDIUM — could not read source files directly; synthesized from web sources
- File ownership map: MEDIUM — inferred from repo structure and installer behavior descriptions
- Claude Code precedence: HIGH — confirmed by official Claude Code documentation
- Recommendations: HIGH — based on well-understood precedence rules and confirmed project-local safety

**Research limitations:**
- Bash, Glob, Grep, and file Read tools were denied during this research session
- Could not inspect actual installed files at `~/.claude/get-shit-done/`, `~/.claude/gsd-local-patches/`, or `~/.claude/gsd-file-manifest.json`
- Could not verify current GSD version installed or whether any patches currently exist
- All findings are from web sources; local verification recommended

**Research date:** 2026-03-26
**Valid until:** 2026-04-26 (GSD releases ~weekly; patch system mechanics are stable since v1.17)
