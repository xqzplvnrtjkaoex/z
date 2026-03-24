---
name: Phase 3 split plan
description: Phase 3 (authentication) is too large (~126 decisions) and will be split into 2 smaller phases. Advisor mode to be enabled first.
type: project
---

Phase 3 (authentication) has ~126 decisions — nearly 2x Phase 2's ~66. User agreed to split it.

**Why:** Single phase is too large to manage effectively. Phase 2 at ~66 decisions was comfortable; ~40-70 is the sweet spot per phase.

**How to apply:**
1. Next session: Run `gsd:profile-user` to create USER-PROFILE.md (enables advisor mode for discuss-phase)
2. Session after that: Split Phase 3 into 2 phases using 2-split strategy:
   - 3a: Auth service infra + Passkey registration/login + JWT/Session + Gateway middleware (core auth — "user can log in and access protected endpoints")
   - 3b: Recovery + Step-up auth + API key + Admin management (extended auth features)
3. Each sub-phase gets its own CONTEXT.md derived from the existing 03-CONTEXT.md decisions
4. ROADMAP.md must be updated to reflect the new phase structure
5. Downstream phases (4-6) may need renumbering
