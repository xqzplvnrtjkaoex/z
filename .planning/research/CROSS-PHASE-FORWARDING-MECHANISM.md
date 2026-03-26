# Cross-Phase Concern Forwarding Mechanism Design

**Researched:** 2026-03-26
**Domain:** GSD workflow pipeline -- inter-phase concern propagation
**Confidence:** HIGH (analysis of existing artifacts + concrete 3A->3B scenario)

## 1. Current State Analysis

### What Exists

The GSD pipeline has several mechanisms that partially touch cross-phase information flow, but none of them reliably forward specific concerns from Phase A to Phase B:

| Mechanism | What It Does | Why It Fails for Forwarding |
|-----------|-------------|---------------------------|
| ROADMAP.md `Depends on` | Declares phase dependency ordering | Not read by /case agents. Declares dependency, not WHAT concerns to inherit |
| CONTEXT.md `<deferred>` section | Lists ideas explicitly out of scope | Downstream phase may have its own CONTEXT.md that doesn't reference these. The deferred items are generic ("all 3B features"), not actionable concerns |
| CONTEXT.md `(3A SCOPE)` annotations | Marks decisions with phase boundaries | Briefer reads CONTEXT.md but only the current phase's CONTEXT.md, not dependency phases |
| CASE-SCRATCH.md Open Questions | Captures unresolved questions during /case | Stays in the phase directory. No agent reads dependency phase open questions |
| Memory entries | Non-deterministic context persistence | Claude may or may not load relevant memories. No guarantee of surfacing specific concerns |
| Seeds (`gsd:plant-seed`) | Ideas with trigger conditions | Surface at milestone boundary (too late for same-milestone phases) |
| CASES.md SR Candidates | Constraints that should be promoted to PROJECT.md | Only for system-wide rules, not phase-specific deferred concerns |

### The Gap

**No artifact or agent instruction causes Phase B's /case or discuss agents to look at Phase A's discovered concerns.**

The case-briefer reads: CONTEXT.md (current phase), ROADMAP.md, REQUIREMENTS.md, PROJECT.md. It does NOT read dependency phase CASES.md, CASE-SCRATCH.md, or CONTEXT.md deferred items.

The step-discuss orchestrator reads: CASE-BRIEFING.md (produced by briefer for current phase). It does NOT scan dependency phases.

The discuss-phase skill (gsd:discuss) reads: ROADMAP.md, PROJECT.md, prior phase CONTEXT.md. But "prior phase" means the immediately preceding phase context for understanding what was built, not for extracting deferred concerns. There is no structured extraction of "concerns Phase A flagged for Phase B."

### Concrete Evidence: 3A -> 3B

**3A artifacts contain at least 4 categories of forward-looking concerns:**

1. **Open Questions targeting 3B:**
   - CASE-SCRATCH.md RegisterBegin Q2: "Design Register ceremony logic to be reusable for 3B passkey addition"
   - RefreshToken R4 (the trigger concern): "JWT claims from session data -- freshness maintained by update operations, not RefreshToken. When 3B introduces user info modification, verify refreshed JWT carries session-cached (possibly stale) values"

2. **Deferred behavioral items in CONTEXT.md (3A->3B):**
   - D-02: Role-selectable invites (3B adds role parameter)
   - D-09: Admin invite query/cancel endpoints
   - D-22: Recovery code regeneration
   - D-36: Logout all sessions (verified tier)
   - D-55: Rate limiting (deferred entirely)
   - D-140: Admin user deletion

3. **Scope-boundary annotations (things 3A built that 3B extends):**
   - D-39: JWT claims -- 3B adds `recovery`, `verified_at` conditional claims
   - D-46: Gateway middleware -- 3A=2-tier, 3B=5-tier
   - D-56: All 5 DB tables created in 3A, api_keys populated in 3B
   - D-59: Additional RPCs added in 3B
   - D-71/74/78: Additional usecase flows, domain types, ports in 3B

4. **Design intent that constrains 3B implementation:**
   - RegisterBegin R11: Handle reservation via Redis key with ceremony TTL
   - Logout R2: Parameterized verify function (verify+refresh vs verify-only)
   - RefreshToken R2/R3: Per-session lock + multi-instance safety abstraction

**Currently, NONE of these are guaranteed to surface during 3B's /case run.** The 3B CONTEXT.md independently specifies its decisions (it was written during discuss-phase for 3B), so items 2 and 3 are partially covered. But items 1 and 4 have no forwarding path.

---

## 2. Candidate Evaluation

### Candidate A: case-briefer Extension

**Description:** The briefer reads dependency phase CASES.md (and CASE-SCRATCH.md if no final CASES.md exists), extracts Open Questions targeting downstream phases + deferred items, and outputs an "Inherited Concerns" section in CASE-BRIEFING.md.

**Mechanism:**
1. Briefer's `<files_to_read>` gains: dependency phase CASES.md (path derived from ROADMAP.md `Depends on`)
2. Briefer adds a new Step (between current Step 1 and Step 2): scan dependency CASES.md for Open Questions, deferred items, and design intent rules that reference downstream phases
3. Output gains `## Inherited Concerns` section after `## Cross-Cutting Constraints`

**Trade-off Analysis:**

| Dimension | Assessment |
|-----------|------------|
| Automation level | Fully automatic -- briefer reads dependency artifacts without developer action |
| New files/artifacts | None -- extends existing CASE-BRIEFING.md |
| Invasiveness | Low -- one new step in briefer, one new output section. No changes to orchestrator, validator, or step files |
| Agent context cost | Medium -- adds one CASES.md read per dependency phase (~200-400 lines for 3A). Briefer uses Sonnet, so context is relatively cheap |
| Reliability | Medium -- depends on briefer's extraction quality. Sonnet may miss subtle design intent buried in operation rules. But Open Questions and deferred items are structurally marked, so extraction is straightforward |
| Scalability | Good -- CASES.md per phase is bounded. Even with 6 phases, reading 3-4 dependency CASES.md files is manageable |

**What it catches:**
- Open Questions targeting downstream phases (Q2 in RegisterBegin)
- Design intent rules that mention downstream phase names (RefreshToken R4 mentioning "3B")
- Phase Rules that may need re-evaluation when scope changes

**What it misses:**
- Deferred items from CONTEXT.md (briefer already reads CONTEXT.md -- but only the CURRENT phase's CONTEXT.md, not the dependency phase's)
- Concerns that are implicit in the implementation but not documented (e.g., if an adapter was built with a known limitation not captured as an open question)

### Candidate B: step-init Pre-flight

**Description:** The init step itself scans dependency phase CASES.md before dispatching the briefer, surfaces concerns directly to the developer for awareness.

**Mechanism:**
1. step-init.md gains a new substep 1c.5: "Pre-flight dependency scan"
2. Uses Grep/Read to scan dependency phase CASES.md for Open Questions and rules containing downstream phase references
3. Presents findings to developer via AskUserQuestion before briefer dispatch
4. Developer acknowledges or adds to the list
5. Findings are passed to the briefer as additional context in the dispatch prompt

**Trade-off Analysis:**

| Dimension | Assessment |
|-----------|------------|
| Automation level | Semi-automatic -- finds concerns automatically, developer must acknowledge before proceeding |
| New files/artifacts | None |
| Invasiveness | Medium -- modifies step-init + changes briefer dispatch prompt to include inherited concerns |
| Agent context cost | Low for the scan itself (simple grep). But adds a conversational round-trip before briefer dispatch |
| Reliability | High -- developer sees concerns directly, can add context the grep missed, ensures nothing is silently dropped |
| Scalability | OK for 2-3 dependency phases. Becomes noisy with 6+ accumulated concerns. Developer fatigue risk |

**What it catches:**
- Everything Candidate A catches (Open Questions, design intent rules)
- Developer can add concerns they remember but didn't document

**What it misses:**
- Same as A for undocumented concerns
- Developer may dismiss findings without careful review (fatigue if list is long)

### Candidate C: Separate FORWARD.md Artifact

**Description:** Each phase produces a FORWARD.md file during its finalize step, listing explicit concerns for downstream phases. The briefer or init step reads this file from dependency phases.

**Mechanism:**
1. step-finalize.md gains a new section after Cross-Operation Concerns: "Forward to Downstream Phases"
2. During finalize, the /case orchestrator asks the developer: "Any concerns from this phase that downstream phases should know about?"
3. Confirmed items are written to `{phase_dir}/FORWARD.md`
4. Downstream phase briefer reads dependency phase FORWARD.md files

**Trade-off Analysis:**

| Dimension | Assessment |
|-----------|------------|
| Automation level | Manual -- requires developer to explicitly identify forward items at finalize time |
| New files/artifacts | **New artifact: FORWARD.md per phase** |
| Invasiveness | Medium -- new step in finalize, new file convention, briefer reads additional file |
| Agent context cost | Very low -- FORWARD.md would be 10-20 lines per phase |
| Reliability | High for items the developer thinks to forward. Low for implicit concerns the developer doesn't recognize as forward-worthy at the time |
| Scalability | Excellent -- small fixed-size artifact, one per phase |

**What it catches:**
- Explicitly identified concerns (the developer writes them)
- Deferred items the developer wants to emphasize

**What it misses:**
- Concerns the developer didn't think to forward (the RefreshToken R4 staleness concern is a perfect example -- it was discovered during re-validation, not during the original /case session)
- Requires discipline: if the developer forgets to write FORWARD.md, nothing is forwarded

### Candidate D: CONTEXT.md Deferred Tag Parsing

**Description:** Enhance the briefer to read the dependency phase's CONTEXT.md `<deferred>` section and extract items that match the current phase's scope.

**Mechanism:**
1. Briefer reads dependency phase CONTEXT.md (already has access to prior phase CONTEXT.md per WORKFLOW.md)
2. Parses `<deferred>` section for items mentioning the current phase or matching current phase requirements
3. Surfaces as "Deferred from Phase X" in briefing

**Trade-off Analysis:**

| Dimension | Assessment |
|-----------|------------|
| Automation level | Fully automatic |
| New files/artifacts | None |
| Invasiveness | Very low -- briefer already reads CONTEXT.md, just extends scope to dependency phase's CONTEXT.md |
| Agent context cost | Low -- deferred sections are typically 5-10 lines |
| Reliability | Low-Medium -- deferred items are often generic ("All Phase 3B features") without specific behavioral concerns. The staleness concern (RefreshToken R4) would NOT appear here because it's not a deferred feature, it's a discovered interaction |
| Scalability | Excellent -- minimal additional reading |

**What it catches:**
- Explicitly deferred features that have behavioral implications
- Scope boundaries that need re-evaluation

**What it misses:**
- The most valuable type of concern: things discovered DURING case analysis that weren't part of the original plan (Open Questions, design intent implications)
- Generic deferred items don't carry the specificity needed for case discussion

### Candidate E: Unified Concern Registry (CONCERNS.md)

**Description:** A single project-level file (`.planning/CONCERNS.md`) where any phase can register concerns for any other phase. Agents scan it at the start of any pipeline step.

**Mechanism:**
1. Any /case session, discuss session, or manual edit can append to CONCERNS.md
2. Format: `| Phase | Concern | Source | Target Phase | Status |`
3. Briefer reads CONCERNS.md, filters for current phase as target
4. Concerns marked as "resolved" after target phase addresses them

**Trade-off Analysis:**

| Dimension | Assessment |
|-----------|------------|
| Automation level | Requires manual registration, but consumption is automatic |
| New files/artifacts | **New artifact: project-level CONCERNS.md** |
| Invasiveness | Low for consumption (briefer reads one more file). Medium for registration (need to teach multiple skills to write to it) |
| Agent context cost | Low -- one file, grows linearly with concerns |
| Reliability | High for consumption (single source of truth). Medium for registration (depends on skills remembering to write to it) |
| Scalability | Good -- central registry prevents scatter. But could grow large over a long project |

**What it catches:**
- Anything explicitly registered from any source
- Can be manually populated at any time (e.g., during re-validation sessions like the one that found RefreshToken R4)

**What it misses:**
- Same as FORWARD.md: only catches what someone thinks to register

### Candidate F: Hybrid (A + C-lite)

**Description:** Combine automatic scanning (Candidate A -- briefer reads dependency CASES.md) with lightweight explicit forwarding (simplified FORWARD section in CASES.md itself, not a separate file).

**Mechanism:**
1. **Automatic:** Briefer reads dependency phase CASES.md, extracts Open Questions and rules referencing downstream phases
2. **Explicit:** CASES.md output format gains a `## Forward Concerns` section at the bottom (next to SR Candidates). During finalize, the orchestrator asks about forward items. These are written into CASES.md itself, not a separate file
3. **Consumption:** Briefer reads dependency CASES.md and finds both the Open Questions (automatic extraction) AND the Forward Concerns section (explicit forwarding)

**Trade-off Analysis:**

| Dimension | Assessment |
|-----------|------------|
| Automation level | Automatic extraction + explicit forwarding in one pass |
| New files/artifacts | **None** -- extends existing CASES.md |
| Invasiveness | Low-Medium -- briefer gains one more file to read + extraction logic; finalize gains one more section |
| Agent context cost | Medium -- same as A (reads dependency CASES.md) |
| Reliability | High -- catches both structurally-marked concerns (Open Questions) and developer-identified concerns (Forward section) |
| Scalability | Good -- everything stays in CASES.md, no new artifact proliferation |

---

## 3. Trade-off Comparison Matrix

| Dimension | A: Briefer | B: Pre-flight | C: FORWARD.md | D: Deferred parse | E: Registry | F: Hybrid |
|-----------|:---:|:---:|:---:|:---:|:---:|:---:|
| Automation | Full | Semi | Manual | Full | Manual+Auto | Full+Manual |
| New artifacts | 0 | 0 | 1 per phase | 0 | 1 project-level | 0 |
| Files changed | 1 (briefer) | 2 (init+briefer) | 3 (finalize+briefer+SKILL) | 1 (briefer) | 3+ (briefer+multiple skills) | 2 (briefer+finalize) |
| Implicit concerns | Yes | Yes+developer | No | No | No | Yes |
| Explicit concerns | No | Developer adds | Yes | No | Yes | Yes |
| Context cost | Medium | Low+roundtrip | Very low | Very low | Low | Medium |
| Reliability | Medium | High | Developer-dependent | Low | Medium | High |
| Scalability | Good | Fair | Excellent | Excellent | Good | Good |

---

## 4. Concrete 3A -> 3B Walkthrough

### The concerns that need to surface in 3B's /case run:

1. **RefreshToken R4 staleness:** "When 3B introduces user info modification, verify refreshed JWT carries session-cached (possibly stale) values"
2. **RegisterBegin Q2:** "Design Register ceremony logic to be reusable for 3B passkey addition"
3. **Design intent -- parameterized verify function:** Logout R2 established that verify middleware is parameterized (verify+refresh vs verify-only). 3B's new tiers (verified, admin, scraper, recovery) must integrate with this design
4. **D-02 deferred:** Role-selectable invites -- 3A hardcoded `user` role, 3B adds role parameter
5. **D-39 deferred:** JWT conditional claims (`recovery`, `verified_at`) not issued in 3A, added in 3B

### How each candidate handles these:

#### Candidate A (Briefer reads CASES.md):

1. **RefreshToken R4:** Briefer reads 3A CASE-SCRATCH.md, finds RefreshToken R4 mentioning "3B". Extracts into Inherited Concerns: "RefreshToken R4: JWT claims from session data -- when 3B introduces user info modification, verify freshness." **CAUGHT**
2. **RegisterBegin Q2:** Briefer reads 3A CASE-SCRATCH.md, finds Q2 explicitly referencing "3B passkey addition". Extracts into Inherited Concerns. **CAUGHT**
3. **Parameterized verify:** Briefer reads Logout R2 -- but this rule mentions "3B" only via "logout-all is 3B" in R1, not in R2's design intent about parameterized functions. The briefer might or might not connect the dots. **MAYBE -- depends on Sonnet's reasoning**
4. **D-02 deferred:** Briefer reads current phase CONTEXT.md (3B), which already has D-02 with full role-selectable specification. This concern is already surfaced by normal briefer operation. **ALREADY COVERED by normal flow**
5. **D-39 deferred:** Same as D-02 -- 3B CONTEXT.md already specifies the conditional claims. **ALREADY COVERED by normal flow**

**Score: 2 definite catches + 1 maybe + 2 already covered = strong coverage**

#### Candidate B (Pre-flight scan):

1. **RefreshToken R4:** Grep finds "3B" in CASE-SCRATCH.md R4. Surfaces to developer. **CAUGHT**
2. **RegisterBegin Q2:** Grep finds "3B" in Q2. Surfaces to developer. **CAUGHT**
3. **Parameterized verify:** Grep finds "3B" in Logout R1 ("logout-all is 3B"). Surfaces nearby context including R2. Developer recognizes the connection. **CAUGHT (developer-mediated)**
4. **D-02 deferred:** Already in 3B CONTEXT.md. Pre-flight could also scan 3A CONTEXT.md deferred section. **CAUGHT (redundant)**
5. **D-39 deferred:** Same. **CAUGHT (redundant)**

**Score: 3 direct catches + 2 redundant = very strong, but at the cost of developer round-trip before every /case run**

#### Candidate C (FORWARD.md):

Depends entirely on whether the developer created a FORWARD.md at the end of 3A's /case session.

If the developer was asked "Any concerns for downstream phases?" during 3A finalize:
1. **RefreshToken R4:** Possibly mentioned if it was discovered during 3A /case. But this specific concern was discovered during RE-VALIDATION (a later session), not during the original /case. **MISSED unless manually added later**
2. **RegisterBegin Q2:** Possibly mentioned -- Q2 explicitly says "3B". Developer might forward it. **MAYBE**
3-5: Possibly, depends on developer diligence.

**Score: Highly variable -- 0 to 5 catches depending on developer discipline**

#### Candidate D (Deferred tag parsing):

1. **RefreshToken R4:** NOT in deferred section. This is a discovered concern, not a deferred feature. **MISSED**
2. **RegisterBegin Q2:** NOT in deferred section. This is an open question. **MISSED**
3. **Parameterized verify:** NOT in deferred section. **MISSED**
4. **D-02 deferred:** 3A deferred section says "All Phase 3B features" generically. Would need to match "role-selectable invite creation" against 3B's scope. **PARTIAL**
5. **D-39 deferred:** Same generic reference. **PARTIAL**

**Score: 0 direct catches + 2 partial = weak for the most valuable concern types**

#### Candidate E (Registry):

Same as C -- depends on someone registering concerns. The RefreshToken R4 concern was discovered during re-validation. If someone had run `gsd:note` or manually added to CONCERNS.md, it would surface. But there's no automatic trigger.

**Score: 0-5 depending on discipline, same as C but with a central file**

#### Candidate F (Hybrid):

1. **RefreshToken R4:** Automatic scan catches it from CASE-SCRATCH.md R4. **CAUGHT (automatic)**
2. **RegisterBegin Q2:** Automatic scan catches it from CASE-SCRATCH.md Q2. **CAUGHT (automatic)**
3. **Parameterized verify:** If the developer added it to 3A's Forward Concerns section during finalize, it would be caught explicitly. Even without that, the automatic scan may catch "3B" references in Logout rules. **CAUGHT (explicit or automatic)**
4-5: Already covered by normal flow.

Plus: any concern the developer thinks of at 3A finalize time is captured in the Forward Concerns section of CASES.md itself, without a separate file.

**Score: 2-3 automatic + 0-2 explicit + 2 already covered = strongest coverage with minimal developer burden**

---

## 5. Living Spec Relationship Analysis

### What is the Living Spec Idea?

From memory: "Phase spec documents (CASES.md, CONTEXT.md) have high reference value after phase completion but are currently treated as read-only archives. The developer feels effort invested in spec documents is wasted if they become stale archives."

### How does each candidate relate?

**Forwarding (all candidates):** Treats phase documents as stable artifacts that are READ by downstream phases. The documents themselves don't change after the phase is complete.

**Living spec:** Would transform phase documents from per-phase archives into per-domain living documents that update across phases. E.g., a single "Auth Behavioral Spec" that spans 3A and 3B, updated as each phase adds operations.

### Would Living Spec make forwarding unnecessary?

**Partially.** If there was a single living auth spec document:
- Items 4 and 5 (scope extensions) would be visible naturally -- the spec shows "CreateInvite: role=user (3B: role=selectable)"
- Items 1 and 2 (open questions) would persist in the same document and be visible when 3B's operations are added

**But not entirely.** The forwarding problem is specifically about surfacing concerns AT THE RIGHT TIME -- when the /case agent starts a new phase's discussion. Even with a living spec, the briefer needs to know to extract "Open Questions from previous operations that are relevant to new operations being discussed." The extraction logic is the same; only the source file location changes (one living doc vs multiple phase docs).

### Are they complementary or competing?

**Complementary.** They solve different problems:
- **Forwarding:** Ensures concern X from Phase A is SEEN by Phase B's agents at the right moment
- **Living spec:** Ensures all behavioral decisions for a domain are in ONE place, preventing staleness

Forwarding solves the immediate problem (agent blindness to dependency phase concerns). Living spec solves the long-term problem (documents becoming stale archives). Forwarding is a prerequisite for living spec -- you need the extraction logic regardless of where the documents live.

### Recommendation: Implement forwarding now

**Rationale:**
1. Forwarding solves the concrete, immediate problem (3B's /case doesn't see 3A's concerns)
2. It requires minimal changes (1-2 files modified)
3. It works with the current per-phase document structure
4. The extraction logic developed for forwarding is reusable if/when living spec is implemented
5. Living spec is a larger design decision that needs its own discuss session (document structure, update triggers, merge semantics, agent reading patterns)
6. Forwarding does not conflict with later living spec adoption -- it's additive

---

## 6. Recommendation

### Primary: Candidate F (Hybrid -- briefer auto-scan + explicit Forward Concerns section)

**Why F over A alone:**
- A catches Open Questions and rules with downstream phase references automatically
- But A cannot catch concerns the developer thinks of during finalize that aren't structurally marked
- F adds a lightweight explicit channel (one section in CASES.md) without a new artifact

**Why F over B:**
- B requires a developer round-trip before every /case session, adding friction
- F's automatic component runs inside the briefer (agent-to-agent, no developer interaction needed)
- F still has the explicit channel for developer-initiated forwarding

**Why F over C/E:**
- C and E require a new artifact and depend entirely on developer discipline
- F has an automatic component that catches structurally-marked concerns without developer action

### Implementation Specification

#### Change 1: case-briefer.md -- Dependency Phase Scan

**New step** between current Step 1 (Understand phase scope) and Step 2 (Discover operations):

```
### Step 1.5: Scan dependency phase concerns

From ROADMAP.md, identify phases listed in the current phase's "Depends on" field.
For each dependency phase, check if CASES.md or CASE-SCRATCH.md exists in the
phase directory.

If found, scan for:
1. Open Questions (### Open Questions sections) -- extract all
2. Rules containing the current phase name or number (e.g., "3B", "Phase 3B")
3. Forward Concerns section (## Forward Concerns) -- extract all items targeting
   this phase

Output as a new section in CASE-BRIEFING.md.
```

**Output format addition:**

```markdown
## Inherited Concerns

> Concerns from dependency phases that may affect this phase's operations.

### From Phase 3A: Authentication Core

**Open Questions:**
| Source | Question | Impact on This Phase |
|--------|----------|---------------------|
| RegisterBegin Q2 | Design ceremony logic to be reusable for 3B passkey addition | AddPasskey operation shares ceremony infrastructure |
| [RefreshToken R4] | JWT claims from session data -- when user info changes, refreshed JWT carries session-cached values | User info modification operations must update session data |

**Forward Concerns:**
| Concern | Source Rule/Case | Recommended Action |
|---------|-----------------|-------------------|
| (from Forward Concerns section of 3A CASES.md, if any) | | |

If no dependency phases have CASES.md: "No dependency phase CASES.md found."
If no concerns found: "No inherited concerns found in dependency phases."
```

**Briefer dispatch prompt change** in step-init.md:

```
<files_to_read>
- .planning/ROADMAP.md -- phase description, success criteria, requirement IDs, dependency phases
- {phase_dir}/*-CONTEXT.md -- locked decisions, constraints
- .planning/REQUIREMENTS.md -- requirement ID descriptions
- .planning/PROJECT.md -- architecture reference (service topology, patterns)
- {dependency_phase_dir}/*-CASES.md or CASE-SCRATCH.md -- inherited concerns (if exists)
</files_to_read>
```

#### Change 2: step-finalize.md -- Forward Concerns Section

**New section** in CASES.md output format, after SR Candidates and before Cross-Operation Concerns:

```markdown
## Forward Concerns

> Concerns from this phase that downstream phases should be aware of.
> Consumed by the case-briefer when downstream phases run /case.

| Target Phase | Concern | Source |
|-------------|---------|--------|
| 3B | Ceremony logic reuse: register ceremony extracted for passkey addition | RegisterBegin Q2 |
| 3B | Session data staleness: JWT refresh uses session-cached user info | RefreshToken R4 |
```

**During finalize, after cross-operation analysis:**

```
Before writing CASES.md, I'll check for concerns that downstream phases should know about.

From open questions and design intent rules, I identified these forward concerns:
1. [concern] -> [target phase]
2. [concern] -> [target phase]

Any additional concerns to forward to downstream phases?
```

This is one AskUserQuestion, not a separate step. It adds ~30 seconds to the finalize flow.

#### Change 3: step-discuss.md -- Inherited Concerns Awareness

**In Step 2.5 (Phase Rules confirmation), after presenting Phase Rules:**

```
Inherited concerns from dependency phases:
- [From 3A] Ceremony logic reuse for passkey addition (RegisterBegin Q2)
- [From 3A] Session data staleness when user info changes (RefreshToken R4)

I'll incorporate these into the relevant operation discussions.
```

This is informational, not confirmatory -- the developer has already acknowledged these concerns during discuss-phase. The /case orchestrator uses them to ask targeted questions during the relevant operation's discussion.

### Files Modified (3 total)

| File | Change | Lines Added |
|------|--------|-------------|
| `.claude/agents/case-briefer.md` | Add Step 1.5 (dependency scan) + Inherited Concerns output section + update `<files_to_read>` guidance | ~40 |
| `.claude/skills/case/step-finalize.md` | Add Forward Concerns section to output format + finalize prompt | ~20 |
| `.claude/skills/case/step-discuss.md` | Add inherited concerns display in Step 2.5 | ~10 |

### What Does NOT Change

- step-init.md (no pre-flight scan -- briefer handles it)
- case-validator.md (does not validate inherited concerns -- they're informational)
- SKILL.md (no formatting changes needed)
- WORKFLOW.md (pipeline order unchanged)
- No new artifacts created
- No new agent types
- Existing CASES.md format gains one optional section

### Implementation Order

1. **case-briefer.md** -- Add Step 1.5 + Inherited Concerns output. Can test independently by re-dispatching briefer on 3B with 3A CASE-SCRATCH.md present.
2. **step-finalize.md** -- Add Forward Concerns section. Independent of briefer change.
3. **step-discuss.md** -- Add inherited concerns display. Depends on briefer change (consumes Inherited Concerns section from CASE-BRIEFING.md).

Steps 1 and 2 are independent and can be done in parallel. Step 3 depends on Step 1.

---

## 7. Risk Analysis

| Risk | Severity | Mitigation |
|------|----------|------------|
| Briefer (Sonnet) misses subtle concerns during dependency scan | Medium | Forward Concerns section provides explicit fallback. Developer can add concerns manually |
| Inherited Concerns section adds noise to CASE-BRIEFING.md | Low | Section is skippable if empty. Typically 2-5 items per dependency phase |
| Long dependency chains accumulate many concerns (e.g., Phase 6 depends on 3+4) | Low | Only scan direct dependencies, not transitive. Concerns that matter across 3+ phases should be promoted to PROJECT.md SRs |
| Forward Concerns section forgotten during finalize | Low | The orchestrator explicitly asks. Even without it, the automatic scan catches structurally-marked concerns |
| Dependency phase has no CASES.md (skipped /case) | None | Briefer handles gracefully: "No dependency phase CASES.md found." Normal briefer operation continues |

---

## 8. What This Does NOT Solve

1. **Concerns discovered AFTER /case completes** (like the RefreshToken R4 staleness concern discovered during re-validation). The only path for these is manual addition to CASES.md Forward Concerns section or a memory entry. This is acceptable -- post-/case discoveries are rare and inherently require human judgment about forwarding.

2. **Implementation-level concerns** (e.g., "the adapter uses a specific Redis key pattern that 3B must not collide with"). These are implementation details, not behavioral concerns. They belong in code comments or architecture docs, not in the /case pipeline.

3. **Cross-milestone forwarding.** This solution addresses same-milestone phases (3A->3B, 4->5). Cross-milestone concerns (e.g., Milestone 1 Phase 6 -> Milestone 2 Phase 7) should use seeds or the backlog, which already surface at milestone boundaries.

---

## Confidence Assessment

| Area | Confidence | Reason |
|------|------------|--------|
| Problem diagnosis | HIGH | Concrete evidence from 3A->3B scenario, confirmed by memory entry |
| Candidate evaluation | HIGH | 6 candidates analyzed against 5 concrete concerns with clear scoring |
| Recommendation (Hybrid F) | HIGH | Strongest coverage in walkthrough, minimal new artifacts, builds on existing patterns |
| Implementation spec | HIGH | 3 files, ~70 lines total, follows established briefer/finalize/discuss patterns |
| Living spec relationship | MEDIUM | Relationship is clear but living spec design is out of scope for this research |

---

## Sources

### Primary (HIGH confidence)
- `.planning/phases/03a-authentication-core/CASE-SCRATCH.md` -- real 3A case data with Open Questions and design intent rules
- `.planning/phases/03a-authentication-core/03a-CONTEXT.md` -- real 3A deferred items and scope annotations
- `.planning/phases/03b-authentication-operations/03b-CONTEXT.md` -- real 3B decisions showing what's independently specified vs what needs forwarding
- `.claude/agents/case-briefer.md` -- current briefer agent definition (files it reads, output format)
- `.claude/skills/case/step-init.md` -- current init step (briefer dispatch mechanism)
- `.claude/skills/case/step-discuss.md` -- current discuss step (Step 2.5 Phase Rules flow)
- `.claude/skills/case/step-finalize.md` -- current finalize step (output format, cross-operation analysis)
- `.planning/ROADMAP.md` -- phase dependency structure
- `.planning/WORKFLOW.md` -- artifact availability and pipeline flow

### Secondary (HIGH confidence)
- `.planning/research/CASE-CONSTRAINT-FORWARDING-SUMMARY.md` -- within-phase constraint forwarding research (complementary, not competing)
- Memory: `project_cross_phase_forwarding.md` -- original problem statement and candidate list
- Memory: `project_unified_spec_idea.md` -- living spec relationship

---

*Research completed: 2026-03-26*
*Valid until: 2026-06-26 (methodology-focused, stable domain)*
