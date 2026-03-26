# Spec Management Tools & Frameworks Survey

**Researched:** 2026-03-26
**Domain:** Specification management, behavioral spec tooling, living documentation
**Confidence:** MEDIUM-HIGH (broad landscape well-covered; individual tool depth varies)

## Executive Summary

The spec management tool landscape in 2025-2026 has exploded around AI-assisted "spec-driven development" (SDD). GitHub's Spec Kit, AWS's Kiro, and Tessl represent the three major commercial/open-source entries, while BMAD, OpenSpec, cc-sdd, and the ATDD plugin represent community alternatives. Older BDD tools (Cucumber, Concordion, Serenity BDD) solve a different problem -- executable behavioral tests -- and living documentation generators (Swimm) focus on keeping docs in sync with code rather than managing specs as development artifacts.

**The critical finding:** None of these tools solve the exact problem we have. Our `/case` workflow already produces structured behavioral specs (CASES.md) that are richer than what most SDD tools generate. Our problem is spec *lifecycle management* -- keeping specs discoverable, cross-referenceable, and synchronized as the codebase evolves across phases. The best approach is to borrow patterns from these tools rather than adopt any of them wholesale.

**Key pattern to borrow:** The "change-level spec" approach (OpenSpec) combined with a service-organized index (our own idea) and automated drift detection via tests (Tessl/Concordion philosophy) is the most promising direction.

---

## Tool-by-Tool Analysis

### 1. GitHub Spec Kit

**What it is:** Open-source CLI (bash-based) for spec-driven development, released September 2025 by GitHub. 23k+ stars.

**How it works:**
- Creates `.specify/` folder with `spec.md`, `plan.md`, `tasks/` directory
- Creates `.github/` folder with agent-specific prompts
- Linear workflow: Constitution -> Specify -> Plan -> Tasks -> Implement
- Uses slash commands (`/specify`, `/plan`, `/tasks`) within AI coding assistants
- `constitution.md` establishes non-negotiable project principles
- Works with Copilot, Claude Code, Gemini CLI, Cursor

**Spec format:** Markdown files with structured sections. One spec.md per feature/project. Plans and tasks are separate documents.

**Strengths:**
- Simple, well-understood linear workflow
- Agent-agnostic (works across coding assistants)
- Constitution concept is useful (similar to our CLAUDE.md)
- Good for greenfield projects

**Weaknesses:**
- Linear workflow -- no iterative loops (our workflow has discuss<->case<->plan loops)
- Designed for greenfield; brownfield/evolution is an open issue (#916)
- Spec evolution is unsolved -- `specify init` overwrites existing files
- One spec.md per project doesn't scale to multi-service architectures
- Agents don't always follow specs (reported: agent skipped unit tests, marked task "done")
- "Sea of markdown documents, long agent run-times, unexpected friction" (Scott Logic review)
- No spec-code synchronization mechanism
- Experimental status explicitly stated

**Relevance to us:** LOW. Our workflow is already more sophisticated. Spec Kit's workflow is a subset of GSD's discuss->case->plan->execute pipeline. The constitution.md concept maps to our CLAUDE.md + PROJECT.md.

**References:**
- [GitHub Repo](https://github.com/github/spec-kit)
- [Official Site](https://speckit.org/)
- [GitHub Blog Announcement](https://github.blog/ai-and-ml/generative-ai/spec-driven-development-with-ai-get-started-with-a-new-open-source-toolkit/)
- [Scott Logic Critical Review](https://blog.scottlogic.com/2025/11/26/putting-spec-kit-through-its-paces-radical-idea-or-reinvented-waterfall.html)

---

### 2. AWS Kiro

**What it is:** Agentic IDE (Code OSS fork) from AWS with built-in spec-driven development. Launched July 2025 at AWS Summit NYC.

**How it works:**
- Natural language prompt -> EARS-format requirements -> design -> tasks
- Three markdown files: `requirements.md`, `design.md`, `tasks.md`
- EARS = Easy Approach to Requirements Syntax (from Rolls Royce engineering)
- Bidirectional sync: update code -> ask Kiro to update specs, or update specs -> trigger task generation
- Agent Hooks: event-driven automations on file save/create/delete
- Integrated into the IDE (not a standalone CLI)

**Spec format:** Structured markdown with EARS notation for requirements. Design.md describes tech stack/architecture. Tasks are implementation checklists.

**Strengths:**
- Bidirectional spec-code synchronization (unique among these tools)
- EARS notation provides structured requirement syntax beyond free-form prose
- Agent Hooks for automated reactions (closest thing to living documentation)
- IDE integration reduces workflow friction

**Weaknesses:**
- IDE lock-in (must use Kiro IDE, not your existing editor)
- AWS ecosystem coupling (pricing model: $19-39/month after preview)
- Primarily spec-first, not spec-anchored -- unclear how specs evolve long-term
- Kiro and spec-kit "are not suitable for the majority of real life coding problems" (Martin Fowler)
- No clear multi-project/multi-service story

**Relevance to us:** LOW-MEDIUM. The bidirectional sync concept is interesting but the tool is an IDE, not a pattern we can adopt. The EARS notation is worth studying as an alternative to our current freeform rule descriptions.

**References:**
- [Kiro Spec Docs](https://kiro.dev/docs/specs/)
- [Kiro Best Practices](https://kiro.dev/docs/specs/best-practices/)
- [Martin Fowler Analysis](https://martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html)

---

### 3. Tessl

**What it is:** Agent Enablement Platform from Guy Podjarny (Snyk founder). Most radical SDD vision: specs should be the only human-maintained artifact, code is transient generated output.

**How it works:**
- Tessl Framework (closed beta) + Spec Registry (open beta, 10,000+ pre-built specs)
- Each spec defines: natural language description, capabilities with linked tests, API surface
- `@generate` annotation: produce code from spec; `@describe`: document existing code
- "Tiles" -- installable methodology/library context packages for composable workflows
- Exploring spec-as-source: `// GENERATED FROM SPEC - DO NOT EDIT` in generated code

**Spec format:** Custom spec format with annotations (`@generate`, `@describe`). Capabilities linked to tests. API documentation integrated.

**Strengths:**
- Only tool explicitly pursuing spec-anchored and spec-as-source approaches
- Spec Registry for library knowledge (prevents API hallucinations)
- Capabilities-linked-to-tests model is powerful
- Tiles concept for composable context

**Weaknesses:**
- Closed beta (Framework), commercial product
- Radical vision may not match incremental development reality
- Spec-as-source assumes code is disposable -- not true for performance-critical Rust
- Vendor lock-in risk is high

**Relevance to us:** LOW for tool adoption, MEDIUM for pattern borrowing. The capabilities-with-linked-tests model is close to what our CASES.md already does (operations with S/F/E cases). The Tiles concept could inspire how we package service-level specs for cross-phase discovery.

**References:**
- [Tessl Site](https://tessl.io/)
- [Tessl Launch Blog](https://tessl.io/blog/tessl-launches-spec-driven-framework-and-registry/)
- [Martin Fowler Analysis](https://martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html)

---

### 4. OpenSpec

**What it is:** Lightweight, open-source spec-driven framework by Fission AI. Designed for iterative changes to existing codebases.

**How it works:**
- `openspec/` folder with `project.md` (worldview) and `AGENTS.md` (agent config)
- Strict three-phase state machine: Proposal -> Apply -> Archive
- `proposal.md` with reasoning, `specs/` folder with requirements, `design.md`, `tasks.md`
- Delta markers: `ADDED`/`MODIFIED`/`REMOVED` track changes relative to existing functionality
- Change archiving after completion
- Slash commands for various AI assistants

**Spec format:** Markdown with delta markers for brownfield evolution. Proposal-centric (each change is a proposal).

**Strengths:**
- Explicitly designed for brownfield/existing codebases (our situation)
- Delta markers for tracking changes -- useful for evolving specs
- Lightweight -- minimal setup, no API keys
- Archive mechanism preserves history
- Three-phase state machine prevents premature implementation

**Weaknesses:**
- Not suited for "large multi-service initiatives where specification drift during implementation creates coordination problems" (per their own docs)
- Relatively new, smaller community
- No spec-code synchronization beyond manual reconciliation

**Relevance to us:** MEDIUM. The change-level spec + delta markers + archive pattern is the closest match to our needs. Our CASES.md per phase is already a form of change-level spec. The archive/promotion pattern could inform how we consolidate completed phase specs into service-level living specs.

**References:**
- [GitHub Repo](https://github.com/Fission-AI/OpenSpec)
- [Official Site](https://openspec.dev/)
- [Comparison Review](https://hashrocket.com/blog/posts/openspec-vs-spec-kit-choosing-the-right-ai-driven-development-workflow-for-your-team)

---

### 5. BMAD-METHOD

**What it is:** "Breakthrough Method for Agile AI-Driven Development." Open-source framework with 12+ specialized AI agent personas.

**How it works:**
- Team of AI agents: Analyst, PM, Architect, Developer, UX, Scrum Master, etc.
- Linear handoff: Analyst -> PM (PRD) -> Architect (design) -> Developer
- Each agent has distinct roles, commands, and artifact outputs
- "Party Mode" for multi-agent collaboration in one session
- Scale-Domain-Adaptive planning
- Security by design in planning phase

**Spec format:** Multiple documents: project brief, PRD, architecture doc, stories. Each produced by a different agent persona.

**Strengths:**
- Comprehensive coverage from ideation to implementation
- Multi-agent collaboration model
- Security considerations baked into planning
- Good for large enterprise teams

**Weaknesses:**
- Heavy -- "sprint ceremonies, story points, and stakeholder syncs"
- Overkill for a solo developer or small team
- Persona-based approach adds cognitive overhead
- Artifact proliferation risk

**Relevance to us:** LOW. We're a solo developer project. The multi-persona approach is unnecessary complexity. Our discuss->case->plan pipeline already handles the Analyst->PM->Architect flow implicitly.

**References:**
- [GitHub Repo](https://github.com/bmad-code-org/BMAD-METHOD)
- [BMAD Docs](https://docs.bmad-method.org/)

---

### 6. Augment Code Intent

**What it is:** Desktop IDE workspace from Augment Code with multi-agent orchestration through living, bidirectional specifications.

**How it works:**
- Coordinator agent analyzes codebase, breaks spec into tasks
- Identifies dependencies, sequences work into parallel waves
- Specialist agents assigned by task type
- Specs update continuously as agents implement changes
- Real-time synchronization between documentation and code
- Verifier agent validates consistency

**Spec format:** Living specs -- not static documents but continuously updating artifacts.

**Strengths:**
- True bidirectional living specifications
- Multi-agent orchestration with Coordinator/Implementor/Verifier pattern
- Parallel execution across isolated worktrees
- Addresses documentation drift by design

**Weaknesses:**
- Commercial product, IDE lock-in
- Complex multi-agent system may be unpredictable
- Living specs could become noisy if every small change updates them
- Not clear how human review integrates

**Relevance to us:** LOW for tool adoption, MEDIUM for the Coordinator/Implementor/Verifier pattern (which maps loosely to our plan/execute/verify pipeline).

**References:**
- [Augment Code Intent](https://www.augmentcode.com/product/intent)
- [Living Specs Guide](https://www.augmentcode.com/guides/living-specs-for-ai-agent-development)

---

### 7. cc-sdd (gotalab)

**What it is:** Open-source Claude Code plugin bringing Kiro-style spec-driven workflow. v2.0.0 stable.

**How it works:**
- Kiro-style slash commands: `/kiro:spec-init`, `/kiro:spec-requirements`, `/kiro:spec-design`, `/kiro:spec-tasks`, `/kiro:spec-impl`
- Validation commands: `/kiro:validate-gap`, `/kiro:validate-design`, `/kiro:validate-impl`
- Supports 8 AI agents across 13 languages
- Brownfield validation commands
- Research.md support
- Claude Subagents support

**Spec format:** Structured markdown (requirements -> design -> tasks), Kiro-style.

**Strengths:**
- Works within Claude Code (our agent)
- Validation commands for gap analysis and implementation verification
- Brownfield support
- Open-source, customizable

**Weaknesses:**
- Mirrors Kiro's workflow, which is simpler than ours
- Adding another workflow layer on top of GSD would create conflicts
- Spec lifecycle/evolution story is unclear

**Relevance to us:** LOW. We already have GSD with custom extensions. Adding cc-sdd would create workflow conflicts. However, the validation command pattern (gap analysis between spec and implementation) is worth borrowing.

**References:**
- [GitHub Repo](https://github.com/gotalab/cc-sdd)

---

### 8. ATDD Plugin (swingerman)

**What it is:** Claude Code plugin enforcing Acceptance Test Driven Development based on Uncle Bob's approach from empire-2025.

**How it works:**
- Given/When/Then specs written before code
- Project-specific mutation testing (AST-based, language-agnostic)
- Team orchestration with specialist agents
- Two-stream testing (acceptance + unit)

**Spec format:** Given/When/Then (Gherkin-style) acceptance tests as executable specs.

**Strengths:**
- Tests ARE the spec -- no drift possible
- Mutation testing for spec completeness verification
- Language-agnostic approach
- Directly executable

**Weaknesses:**
- Gherkin overhead for Rust (our tests are already well-structured with TDD protocol)
- Given/When/Then adds a translation layer we don't need
- Mutation testing is useful but orthogonal to spec management

**Relevance to us:** LOW for tool adoption, but the "tests as specs" philosophy reinforces our existing TDD protocol direction. Our CASES.md -> `<behavior>` items -> test functions pipeline already achieves this.

**References:**
- [GitHub Repo](https://github.com/swingerman/atdd)

---

### 9. Traditional BDD Tools (Cucumber, Concordion, Serenity BDD)

**What they are:** Established frameworks for executable behavioral specifications.

| Tool | Language | Approach | Status |
|------|----------|----------|--------|
| Cucumber-rs | Rust | Gherkin `.feature` files -> step definitions -> cargo test | Active (v0.21.1) |
| Concordion | Java | HTML/Markdown specs -> executable assertions | Active but niche |
| Serenity BDD | Java | Living documentation from Cucumber/JBehave tests | Active |

**Cucumber-rs specifics:**
- Native Rust, async support, runs via `cargo test`
- `.feature` files in Gherkin syntax
- Step matchers in Rust code
- Tests live in `tests/` directory

**Strengths:**
- Specifications are literally executable -- drift is impossible
- Well-established patterns and community knowledge
- Cucumber-rs integrates with cargo test
- Serenity BDD produces excellent living documentation reports

**Weaknesses:**
- Gherkin adds translation overhead between natural language and code
- Step definition maintenance is non-trivial
- Feature files are limited in expressiveness compared to our CASES.md tables
- BDD tools focus on test execution, not spec management/discovery

**Relevance to us:** LOW for adoption. Our existing TDD protocol with `should_{behavior}_when_{condition}` naming already provides BDD-like readability without Gherkin overhead. The pattern of "specs that execute" is what we already have.

**References:**
- [Cucumber-rs Book](https://cucumber-rs.github.io/cucumber/current/)
- [Concordion](https://concordion.org/)

---

### 10. Living Documentation Generators (Swimm)

**Swimm:** Commercial tool ($29/mo) for code-coupled documentation. Auto-syncs docs when code changes (detects token/function/file renames). IDE integration. Focuses on explaining HOW code works, not behavioral specs.

**Relevance to us:** LOW. Swimm solves a different problem (code explanation docs, not behavioral specs). It doesn't handle the spec lifecycle problem we're trying to solve.

**References:**
- [Swimm](https://swimm.io/)

---

### 11. ADR Tools (adr-tools, log4brains)

**What they are:** Tools for managing Architecture Decision Records.

| Tool | Format | Features |
|------|--------|----------|
| adr-tools | Markdown (Nygard format) | Bash scripts, sequential numbering, supersede/amend |
| log4brains | Markdown (MADR format) | Static site generator, search, cross-references |

**Strengths:**
- Lightweight, well-understood format
- Decisions are append-only (never delete, only supersede)
- log4brains auto-publishes as searchable static site
- Good pattern for tracking decision evolution

**Relevance to us:** LOW for tool adoption (we track decisions in CONTEXT.md and PROJECT.md Key Decisions table). MEDIUM for pattern borrowing -- the supersede/amend pattern could inform how we evolve specs across phases.

**References:**
- [ADR GitHub](https://adr.github.io/)
- [log4brains](https://github.com/thomvaill/log4brains)

---

## Comparison Matrix

### Feature Comparison

| Feature | Spec Kit | Kiro | Tessl | OpenSpec | BMAD | cc-sdd | GSD+/case (ours) |
|---------|----------|------|-------|---------|------|--------|-------------------|
| **Spec format** | Markdown | EARS+MD | Custom+annotations | Markdown+deltas | Multi-doc | Markdown | Structured MD tables |
| **Workflow phases** | 4 (linear) | 3 (bidir) | Composable | 3 (state machine) | 6+ agents | 5 (linear) | 12+ (iterative) |
| **Iterative loops** | No | Partial | Yes | Yes (archive) | No | No | Yes (core feature) |
| **Brownfield support** | Weak | Moderate | Yes (@describe) | Strong (deltas) | Weak | Moderate | Strong (per-phase) |
| **Multi-service** | No | No | Via tiles | No | Yes | No | Yes (per-service) |
| **Spec-code sync** | Manual | Bidirectional | Via tests+@gen | Manual | Manual | Validate cmds | Via TDD protocol |
| **Cross-spec discovery** | No | No | Registry | No | No | No | Briefer scan |
| **AI-native** | Yes | Yes (IDE) | Yes | Yes | Yes | Yes | Yes |
| **Spec evolution** | Unsolved | Partial | Yes | Archive pattern | No | No | Per-phase |
| **Behavioral cases** | No | Acceptance criteria | Capabilities | No | Stories | No | S/F/E tables |
| **Test linkage** | None | Task-level | Capability tests | None | Story tests | Validate cmds | Case ID -> behavior -> test |
| **Open-source** | Yes | No | Partial | Yes | Yes | Yes | Yes |

### Philosophy Comparison

| Philosophy | Tools | Implication |
|-----------|-------|-------------|
| **Spec-first** | Spec Kit, Kiro, cc-sdd | Write spec, then implement. Spec is a starting artifact. |
| **Spec-anchored** | Tessl, OpenSpec | Spec lives on after implementation. Updated as system evolves. |
| **Spec-as-source** | Tessl (exploring) | Code is generated from spec. Human only edits spec. |
| **Tests-as-spec** | ATDD, Cucumber, Concordion | Executable tests ARE the specification. No drift by definition. |
| **Spec-per-change** | OpenSpec, our CASES.md | Each change gets its own scoped spec. Accumulates over time. |

### SDD Maturity Levels (Martin Fowler's taxonomy)

| Level | Description | Tools | Our current state |
|-------|-------------|-------|-------------------|
| Spec-first | Spec written before code, may be discarded after | Spec Kit, Kiro, cc-sdd | We do this (CASES.md -> PLAN.md -> code) |
| Spec-anchored | Spec maintained after implementation for evolution | Tessl, OpenSpec | We don't do this yet (CASES.md becomes archive) |
| Spec-as-source | Spec is the only maintained artifact; code is generated | Tessl (exploring) | Not appropriate for Rust performance work |

---

## Patterns Worth Borrowing

These patterns are valuable regardless of whether we adopt any tool.

### 1. Service-Organized Spec Index (from our own idea + OpenSpec's project.md)

**Pattern:** Reorganize completed phase specs by service with a searchable index.

**How it applies:** After a phase completes, its behavioral specs (from CASES.md) get consolidated into a service-level spec file. A top-level INDEX.md maps operation names to spec locations. The case-briefer reads INDEX.md instead of scanning all phase directories.

**Confidence:** HIGH -- this directly addresses the briefer discovery problem.

### 2. Change-Level Specs with Delta Markers (from OpenSpec)

**Pattern:** Each spec covers one change, not the entire system. Delta markers (`ADDED`/`MODIFIED`/`REMOVED`) track what changed relative to existing behavior.

**How it applies:** Our CASES.md is already change-level (per-phase). Adding delta markers when a phase modifies an existing operation's behavior would make spec evolution explicit. When Phase 5 changes behavior established in Phase 3, the Phase 5 spec shows exactly what changed.

**Confidence:** MEDIUM -- useful but adds overhead; may be simpler to just reference the prior spec.

### 3. Constitution / Non-Negotiable Principles (from Spec Kit)

**Pattern:** A single document listing non-negotiable project principles that all specs and code must respect.

**How it applies:** We already have this split across CLAUDE.md (development conventions) and PROJECT.md (architectural decisions + System-Wide Rules). No change needed -- but this validates our existing approach.

**Confidence:** HIGH -- confirms our existing pattern is aligned with industry direction.

### 4. Capabilities-Linked-to-Tests (from Tessl)

**Pattern:** Each spec capability explicitly links to its test(s). A capability without a linked test is flagged as unverified.

**How it applies:** Our CASES.md -> PLAN.md `<behavior>` -> test function chain already does this. The explicit linkage could be strengthened with a machine-readable mapping (operation -> test file -> test function) that validate-phase can check automatically.

**Confidence:** HIGH -- reinforces our existing pattern; automation opportunity.

### 5. Validation Commands (from cc-sdd)

**Pattern:** Explicit validation steps: gap analysis (spec vs implementation), design validation, implementation validation.

**How it applies:** Maps directly to our validate-phase and verify-work stages. Could formalize the gap analysis as a structured check: for each operation in CASES.md, verify a corresponding handler/usecase/test exists.

**Confidence:** HIGH -- we already do this; could be more automated.

### 6. Archive Pattern (from OpenSpec)

**Pattern:** Completed changes are archived with their specs. The archive is searchable but separate from active specs.

**How it applies:** Our phase directories already serve as archives. The missing piece is promotion of active behavioral contracts from archived phases into living service-level specs.

**Confidence:** MEDIUM -- the archive is natural; the promotion mechanism needs design.

### 7. Spec Density Growth (from brownfield SDD patterns)

**Pattern:** In brownfield development, write specs only for what you're changing. Over time, frequently-touched areas accumulate spec coverage. Don't try to spec the whole system retroactively.

**How it applies:** This matches our per-phase approach perfectly. As we build more phases, the service specs naturally grow. Phase 3 adds auth operations, Phase 4 might add catalog operations, etc. No need for a big-bang spec-writing effort.

**Confidence:** HIGH -- this is exactly how our workflow already operates.

---

## Anti-Patterns to Avoid

### 1. Spec Rot / Documentation Drift
**What goes wrong:** Specs are written upfront, code diverges during implementation, specs become stale and misleading.
**Root cause:** No enforcement mechanism. Specs and code are disconnected artifacts.
**Our mitigation:** TDD protocol links CASES.md -> `<behavior>` -> test functions. If behavior changes, tests change. But the CASES.md document itself doesn't auto-update. **Risk is real for us at the service-level spec layer if we build one.**
**Prevention:** Tests are the authoritative spec. CASES.md/service specs are reference docs. If they drift, the test suite is truth.

### 2. Over-Specification (Spec Heavier Than Code)
**What goes wrong:** "Spending more time reading AI-generated prose than actually thinking." Spec becomes bureaucratic overhead.
**Root cause:** Tool-mandated completeness over useful completeness. Everything must be spec'd regardless of complexity.
**Our mitigation:** CASES.md is optional per phase. Simple phases skip `/case`. Priority labels (must/should/could) scope the work.
**Prevention:** Never mandate spec coverage for trivial changes. Spec cost must be proportional to implementation risk.

### 3. Spec Abandonment
**What goes wrong:** Team creates specs initially, then stops maintaining them as velocity pressure increases.
**Root cause:** Maintenance cost exceeds perceived value. No one reads the old specs.
**Our mitigation:** Keep specs lightweight (structured tables, not prose). Make them machine-readable so the briefer and validate-phase consume them automatically. If specs have automated consumers, they have ongoing value.
**Prevention:** Specs must have at least one automated consumer or they will be abandoned.

### 4. Tool Lock-In
**What goes wrong:** Adopting Kiro/Tessl/etc. ties the project to a vendor. Migration cost grows over time.
**Root cause:** Proprietary spec formats, IDE-integrated workflows, cloud-dependent features.
**Our mitigation:** Stay with plain markdown + our own conventions. Tools should be patterns, not products.
**Prevention:** Spec format must be plain text (markdown). No binary formats, no proprietary annotations, no cloud dependencies.

### 5. Big Design Up Front Disguised as SDD
**What goes wrong:** Writing exhaustive specs before any code, then discovering the spec was wrong. "Reinvented waterfall."
**Root cause:** Linear workflow without feedback loops. Spec -> Plan -> Build with no iteration.
**Our mitigation:** Our workflow is explicitly iterative (case<->discuss loops, plan->discuss returns). Specs evolve through the pipeline. But service-level living specs could reintroduce this risk if we try to spec all operations before building them.
**Prevention:** Specs grow incrementally per phase. Never attempt to write a complete service spec before building the service.

### 6. Format Rigidity
**What goes wrong:** Spec format mandated by tool doesn't fit project's needs. Team fights the format.
**Root cause:** One-size-fits-all templates. Tool assumptions don't match project domain.
**Our mitigation:** Our CASES.md format evolved organically from actual use (S/F/E tables, rules, side effects, open questions). It fits because we designed it for our domain.
**Prevention:** Spec format should be project-defined, not tool-imposed.

---

## Recommendation

### Adopt a tool: NO

None of the surveyed tools solve our specific problem. Our workflow is already more sophisticated than Spec Kit, Kiro, or cc-sdd. The tools that come closest to our needs (OpenSpec for brownfield, Tessl for capabilities-with-tests) are either too lightweight or too commercial/opinionated.

### Borrow patterns: YES

The following patterns should inform our spec-driven workflow design:

| Priority | Pattern | Source | Applies To |
|----------|---------|--------|------------|
| 1 | Service-organized spec index with INDEX.md | Our idea + OpenSpec | Briefer discovery problem |
| 2 | Capabilities-linked-to-tests mapping | Tessl | Automated validation |
| 3 | Change-level specs accumulate into service specs | OpenSpec brownfield + Fowler | Spec lifecycle |
| 4 | Tests are authoritative spec, docs are reference | Concordion/ATDD philosophy | Drift prevention |
| 5 | Validation commands (gap analysis) | cc-sdd | Validate-phase automation |
| 6 | Spec density grows naturally per phase | Brownfield SDD patterns | Incremental approach |

### Build custom: YES (lightweight)

Design a custom spec management layer on top of our existing workflow:
- **Service-level spec files** promoted from completed phase CASES.md
- **INDEX.md** for operation->spec->test discovery
- **Briefer integration** reading INDEX.md instead of scanning phase directories
- **Validate-phase automation** checking operation coverage

This should be a convention + minimal tooling (index generation, not a framework).

---

## Open Questions

1. **Spec promotion mechanism:** When a phase completes, what is the exact process for extracting behavioral contracts from CASES.md into service-level specs? Manual curation? Automated extraction? Hybrid?

2. **Spec conflict resolution:** When Phase 5 modifies behavior established in Phase 3's spec, who updates the service-level spec? The Phase 5 `/case` process? A post-ship promotion step?

3. **Index granularity:** Should INDEX.md map at the operation level (SignupBegin -> auth-spec.md#signup-begin) or the service level (auth -> auth-spec.md)?

4. **Spec format for service-level documents:** Should service specs use the same S/F/E table format as CASES.md, or a more compact format focused on current behavior (without the "how we got here" history)?

5. **Automation cost vs benefit:** How much tooling is justified? A shell script that generates INDEX.md from phase CASES.md might be sufficient vs. building a full spec management system.

---

## Sources

### Primary (HIGH confidence)
- [Martin Fowler: Understanding Spec-Driven-Development: Kiro, spec-kit, and Tessl](https://martinfowler.com/articles/exploring-gen-ai/sdd-3-tools.html) -- authoritative multi-tool analysis with SDD maturity levels
- [Thoughtworks: Spec-driven development unpacking](https://www.thoughtworks.com/en-us/insights/blog/agile-engineering-practices/spec-driven-development-unpacking-2025-new-engineering-practices) -- industry analysis with anti-pattern identification
- [GitHub Spec Kit repo](https://github.com/github/spec-kit) -- primary source for Spec Kit architecture
- [OpenSpec repo](https://github.com/Fission-AI/OpenSpec) -- primary source for OpenSpec design
- [Kiro Spec Docs](https://kiro.dev/docs/specs/) -- primary source for Kiro workflow

### Secondary (MEDIUM confidence)
- [Scott Logic: Putting Spec Kit Through Its Paces](https://blog.scottlogic.com/2025/11/26/putting-spec-kit-through-its-paces-radical-idea-or-reinvented-waterfall.html) -- practical critical review
- [Marmelab: Spec-Driven Development: The Waterfall Strikes Back](https://marmelab.com/blog/2025/11/12/spec-driven-development-waterfall-strikes-back.html) -- critical analysis of SDD limitations
- [BMAD-METHOD repo](https://github.com/bmad-code-org/BMAD-METHOD) -- primary source for BMAD
- [Tessl launch blog](https://tessl.io/blog/tessl-launches-spec-driven-framework-and-registry/) -- Tessl feature details
- [Augment Code SDD guides](https://www.augmentcode.com/guides/spec-driven-development-ai-agents-explained) -- Intent/living specs analysis
- [Spec-Kit Issue #916](https://github.com/github/spec-kit/issues/916) -- evolving specs problem statement
- [ATDD repo](https://github.com/swingerman/atdd) -- ATDD approach details
- [Cucumber-rs Book](https://cucumber-rs.github.io/cucumber/current/) -- Rust BDD tooling
- [Medium: GSD vs Spec Kit vs OpenSpec vs Taskmaster](https://medium.com/@richardhightower/agentic-coding-gsd-vs-spec-kit-vs-openspec-vs-taskmaster-ai-where-sdd-tools-diverge-0414dcb97e46) -- tool comparison
- [EPAM: Spec Kit for brownfield](https://www.epam.com/insights/ai/blogs/using-spec-kit-for-brownfield-codebase) -- brownfield patterns
- [Augment Code: Best SDD tools 2026](https://www.augmentcode.com/tools/best-spec-driven-development-tools) -- landscape overview

### Tertiary (LOW confidence)
- Various Medium articles on SDD comparisons (may contain inaccuracies or marketing bias)
- SDD comparison on redreamality.com (community analysis, not verified independently)
