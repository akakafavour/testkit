# TestKit Source Alignment & V1 Recommendation

**Prepared:** 2026-10-11  
**Purpose:** Reconcile all TestKit source documents and recommend a coherent, buildable V1.

## Executive conclusion

The TestKit source set has evolved beyond the earlier 04 blueprint. The current document hierarchy is explicit in the newer files:

1. **`16-testkit-product-scope-v2.md` — current product scope.** It says it supersedes the scope framing in 04 and 06, which are retained as vision/heritage.
2. **`09-testkit-plugin-architecture.md` — current V1 plugin and implementation design.** It is dated 2026-10-11 and says it replaces §5 of the v0.3 PRD.
3. **`17-testkit-capability-map-by-mode.md` — companion definition of the operating modes.** It describes one always-on TRACK substrate plus six user-facing modes.
4. **`18-testkit-tool-adapter-and-evidence-integration-plan.md` — full-product integration and evidence plan.** It makes browser/runtime evidence, UI quality, security, production observation, and external test-platform interoperability explicit product capabilities.
5. **`04` and `06` — broad product vision and lifecycle heritage.** Reuse compatible ambition under the current scope and evidence rules.
6. **`05`, `07`, the product brief, and PRDs v0.2/v0.3 — supporting proposals and historical design inputs.** Reuse compatible detail; explicitly supersede conflicting claims with 16/09/17/18.

**The full product is not merely a generic test runner.** It is a persistent evidence and verification system that can orchestrate specialist tools for browser/runtime evidence, UI quality, security, and production observation. Its core differentiator remains the trustworthy **assertion oracle**: expectations come from requirements, declared invariants, and sourced documentation—not from observing the implementation it is supposed to verify. It correlates the tools' evidence, confirms and replays findings, and gives them to the coding agent. The first development slice should prove this on one reference stack; that slice sequences implementation and does not reduce the full product scope.

**Correction to the earlier explanation:** before discovering 09/16/17, I treated 04 as the governing scope. That was based on the then-visible file inventory. The newly discovered, later-dated documents explicitly supersede that interpretation: **16 is now the current scope; 04 remains the vision document.** The earlier explanation should be read with this update in mind.

---

## 1. Document-by-document role

| Document | Role in the current design | Reuse / qualification |
|---|---|---|
| **04 — Master Blueprint v1.0** | Broad vision: Rust + TypeScript, living map, virtual actor swarm, five-layer recorder, project-aware sandboxes, SRE Sentinel, blue/green operations, architecture gate. | Valuable north star and capability inventory. Its 1,000+ actor scale, Rust-first implementation, microsecond precision, traffic switching, and complete scope are not the V1 plan after 16/09. |
| **05 — Sourcing & Codespaces** | Build environment and candidate integration tools; distinguishes building TestKit in Codespaces from users running it. | Strong “wrap proven tools, build the unifying oracle” principle. Its no-Docker/user-runtime and other portability claims need reconciliation with the container-based V1 in 09/16. |
| **06 — Full Lifecycle Co-founder** | Planning, research, Git Guardian, measurable architecture standards, verification, production operations. | Planning and measurable consequences complement 16. Git Guardian and autonomous deployment/traffic control are separate product areas, not V1 verification scope. |
| **07 — Swarm & Skills** | Live dashboard, nine proposed agents, customer feedback-to-test, telemetry adapters. | UI/backend/data watcher decomposition informs 09. Feedback synthesis and production adapters are later/optional. Do not ship every named agent as a separate service. |
| **Product brief** | Concise local-first verifier proposition and trust guarantees. | Strong explanation of sandboxing, source-read-only behavior, real test identities, and evidence. Its “never production” boundary is updated by 16’s read-only production witness mode. |
| **PRD v0.2** | Detailed local-first verification PRD. | Useful sandbox/session/identity/reporting detail. Historical scope; later PRDs and 16/09 refine it. |
| **PRD v0.3** | Adds scenario generation, permission modes, invariants/watchers, AI contracts, deploy rehearsal, and flake history. | Excellent trust and execution detail. It conflicts with 16 on read-only production observation and is superseded by 09 on plugin architecture. |
| **09 — Plugin Architecture v1** | V1 packaging, oracle thesis, sub-agent roster, MCP surface, doc-grounding, hooks, runtime choice, evidence packet, ship order. | The clearest build plan. Its proposed V1 is TypeScript/Node, wraps mature tools, prioritizes oracle/replay receipts, and defers Git/SRE/feedback features. |
| **16 — Product Scope v2** | Current scope: one evidence engine across test, debug, architecture, teach, attack, and production modes; stack-agnostic and consequence-focused. | Explicitly supersedes scope framing in 04/06. Defines gate vs advisory and read-only production witness. Must become the top-level PRD/scope reference. |
| **17 — Capability Map by Mode** | Companion operational view of continuous tracking and the six user-facing modes. | Makes 16 tangible. Its “everything/any tool/any stack” language should be implemented as a capability aspiration with explicit permissions and coverage limits, not a literal V1 guarantee. |

---

## 2. Where the documents align

### A. The evidence loop is the shared core

Across the blueprint, PRDs, plugin architecture, product scope, and mode map, the same loop recurs:

**Track/model → act → watch → record → correlate → finding → agent fixes → replay/retest → regression case.**

This is the strongest common thread and the right center of V1. The product is not a dashboard or a collection of agent personas. It is the ability to turn behavior into a reliable, replayable evidence object.

### B. The coding agent fixes; TestKit verifies

The PRDs, 09, 16, and 17 all preserve a critical separation: TestKit does not edit application source. It can explain likely causes and propose a fix location, but the host coding agent applies changes. TestKit then independently rechecks them. This separation supports trust and prevents a verifier from silently making its own test pass.

### C. Semantic, API, and state-level operation should lead

The older blueprint stresses broad actors and high throughput; the PRDs and newer scope make the efficient execution pattern clearer: most work should use semantic/API/state operations, with a thin real-browser or device layer for critical journeys. Load is a separate controlled phase, not thousands of brittle UI sessions.

### D. Specialized observation layers make a stronger causal trace

The 07 watcher roles—frontend, backend, and data—map naturally to the 09 oracle/watchers and 17 TRACK mode. They can feed one correlator. This is more credible than making every sub-agent a fully autonomous generalist.

### E. Measurable quality gates complement the architecture vision

04/06 want to prevent “AI slop” and enforce modularity. 16 gives that intent a better rule: **a measured, reproducible consequence can gate; subjective preferences remain advisory or unreported.** Examples include exposed permissions, measured latency, complexity, duplication, or missing retry behavior. This keeps quality checks useful instead of taste-policing.

### F. Local-first and reusable tooling remain strong foundations

05’s reuse strategy, PRDs’ local-first trust model, and 09’s TypeScript wrapper strategy fit together: integrate Playwright/Stagehand, Testcontainers, Toxiproxy, Artillery, and API/static tools behind a consistent TestKit protocol, rather than recreating each tool. Local evidence and explicit governance can be default; optional connectors should be separately scoped.

---

## 3. Material conflicts to resolve explicitly

| Decision | Conflicting proposals | Recommended V1 resolution |
|---|---|---|
| **What governs product scope?** | 04/06 describe a full lifecycle co-founder and production-ops system; 16 says it supersedes their scope framing. | 16 governs current scope; 04/06 are vision/heritage. 09 governs plugin implementation; 17 defines modes. |
| **How broad is V1?** | “Everything, any stack, continuous, any tool” in 16/17 versus 09’s specific, staged proof-first plan. | Build the universal protocol shape, but validate one real stack and a small toolchain first. Clearly report unsupported/blocked coverage. |
| **Production** | 04 includes SRE-triggered hotfixes and blue/green traffic flip; v0.3 prohibits production; 16/17 allow read-only witness plus branch reproduction. | Adopt 16/17: production observation is read-only and optional; synthetic actors never hit live users; reproduction occurs on a branch/sandbox. Defer traffic switching and any production mutation. Update the security policy because v0.3’s absolute no-production statement is now outdated. |
| **Architecture advice** | 06 sometimes says choose the “optimal” stack; 16 says never tell users to replace a decision they already made. | Separate **greenfield planning before a decision** from **consequence analysis after a decision**. Once a stack is chosen, report measurable consequences and alternatives only when requested; do not shame or silently override the choice. |
| **Runtime language** | 04 mandates Rust + TypeScript; 09 says TypeScript/Node for V1 and Rust only after profiling. | Use TypeScript/Node in V1; wrap existing native tools. Revisit Rust only when measured bottlenecks justify it. |
| **Flight-recorder precision** | 04 promises microsecond synchronization across five layers; 09/16 recommend starting with two correlated layers and not faking precision. | Start with a reliable DB transaction watcher plus actor/request trace, then add backend logs and browser/network. Use honest timestamp precision and mark inferred links. |
| **How many agents?** | 07 proposes nine; 09 proposes a smaller V1 roster but its table/count language is internally ambiguous; 16/17 describe modes, not a requirement for one agent per mode. | Keep named **roles/capabilities**, not a fleet of separate LLM agents. Deterministic services handle parsing, watching, replay, and gating; use a small number of reasoning jobs. Clarify the 09 roster count before implementation. |
| **Continuous tracking** | 16/17 say always on; PRDs emphasize permission, local storage, and low resource usage. | Make local incremental model updates low-cost by default, but make evidence collection, external telemetry, production watching, retention, and resource-heavy monitoring visible, configurable, and stoppable. |
| **Stack portability / sandbox** | 05 suggests zero-Docker execution with PGlite for end users; 09 uses Testcontainers and 16 provisions project services. | For the first proof, use a reproducible containerized reference app. Keep a sandbox adapter interface and optional PGlite path; do not claim universal zero-setup coverage until validated. |
| **What is a passing result?** | Older specs often use “100% green”; newer evidence design has PROVEN/FAILED/NOT_PROVEN/BLOCKED and coverage gaps. | Use four-way scenario verdicts. Never convert missing credentials, unsupported tools, weak expectations, or skipped layers into PASS. Readiness is an aggregate policy, not a bug-free guarantee. |
| **Feedback, Git, SRE** | 07/06 make them part of the broad team; 09 defers feedback synthesis, Git Guardian, and autonomous SRE/blue-green work. | Keep them as future modes/integrations. They are not needed to prove the oracle and would expand V1 risk and cost. |

### The sub-agent count needs an editorial fix

09 says “Seven ship. Four are deferred” but its table groups standard/edge/red-team actors and frontend/backend/data watchers into combined rows, while naming multiple distinct jobs inside those rows. 07 separately says nine agents. Before a build plan is frozen, convert this into a **capability roster** with clear distinctions between deterministic workers, actor identities, and LLM reasoning jobs. Avoid designing one microservice or model agent per label.

---

## 4. Recommended first implementation slices: prove the oracle before expanding the harness

This section is an implementation sequence, not a reduced product definition. The complete capability
and adapter plan remains in `16`, `17`, and `18`; each slice should preserve the interfaces and evidence
contract needed by the full product.

### V1 thesis

> **Given a requirement or invariant independent of the implementation, TestKit can detect a real violation during app operation, show the causal evidence, replay it without an LLM, and prevent a false “pass.”**

The simple demo in 09 is the right first proof:

> N successful signups must create exactly N user rows. Intentionally introduce a duplicate-write defect, watch the invariant fail live, replay it, and produce a verifiable receipt.

### Phase 0 — the proof slice

Build this first, before broad agent support or UI polish:

1. One demo app and one database (the 09 plan suggests Next.js + Postgres in Docker Compose).
2. One semantic actor that creates a configurable number of signups.
3. One read-only database watcher, such as a transaction tail/row-count observer.
4. One invariant declared independently of the handler implementation.
5. A deliberately injected duplicate-write bug that reliably violates it.
6. A causal chain linking the actor action to the database result.
7. A deterministic replay script that works without the LLM.
8. A local JSON receipt with content-addressed/hash-chained integrity and a clear verdict.

**Acceptance:** the same injected bug is caught and replayed consistently; the no-bug control passes; a missing/weak expectation is NOT_PROVEN, not confirmed; an unavailable dependency is BLOCKED, not passed.

### Phase 1 — useful V1 verifier

After the proof slice works:

- `init`, `doctor`, `plan`, `verify`, `retest`, `replay`, `report`, and permission commands.
- MCP tools returning structured JSON, plus portable skills and a small number of host adapters (09 prioritizes Claude Code and Cursor first).
- A versioned semantic model with workflows, roles, states, and invariant provenance.
- A reviewed scenario list generated from a small taxonomy × semantic-model cross-product.
- Local test identities for a few roles, then a role/route authorization matrix.
- DB + server/log + network/browser evidence, added incrementally with exact/inferred correlation labels.
- Failure injection for network interruption, latency, and common error responses; no live production targets.
- Replay-confirmation, root-cause deduplication, flake history, explicit coverage gaps, and four-way verdicts.
- A commit/CI ship gate that blocks only **confirmed** configured severity thresholds; NOT_PROVEN and BLOCKED are visible but never silently treated as failures or passes.

### Phase 2 — expand the capability modes carefully

- **DEBUG:** reuse the same evidence to read the trace forward and explain likely cause. Low architectural cost once evidence exists.
- **ATTACK:** expand from sandboxed role/authorization checks to documented, approved test profiles. Never target production.
- **ARCHITECT:** report measurable consequences of the existing stack. Keep preferences advisory; allow greenfield planning as a separate explicit context.
- **TEACH:** explain reports and observed project behavior from project evidence, not unsupported model memory.
- **PRODUCTION witness:** separate opt-in read-only integration; harvest minimized error classes and replay inputs, then reproduce on an isolated branch. No synthetic actors or mutations on production.
- **More stacks, hosts, and scenario packs:** add based on compatibility evidence and user demand, not on “any stack” claims alone.

### Not in the first implementation slices (retained in the full-product plan)

- Rust rewrite and 1,000–10,000 actor claims until profiling demonstrates need.
- Cloud-runner, blue/green traffic switching, automatic production hotfix or deploy actions.
- Git Guardian as an autonomous branch/rollback manager (normal Git integration is still needed for safe snapshots and CI).
- Feedback-to-test ingestion from Discord/Intercom and broad external telemetry integrations; `18`
  defines the full adapter target, including read-only production evidence.
- Multi-agent orchestration at large scale, marketplace, all mobile/desktop adapters, and always-on heavy monitoring.
- The full command-center dashboard until the core receipt and replay are compelling.

---

## 5. Important product rules for a trustworthy V1

### A. Oracle independence

Every scenario must record its expectation source: a user requirement, reviewed invariant, versioned taxonomy rule, or cited documentation. Expectations inferred only from the code or observed behavior are not proof. Label those as exploratory/observed and return **NOT_PROVEN** until an independent expectation exists.

External docs, web pages, app output, and emails are untrusted data. Store the source URL and retrieval date; never let retrieved content directly authorize a tool call. V1 should minimize the number of external MCP sources until the trust boundary and prompt-injection controls are tested.

### B. Four-way verdicts and coverage honesty

Use `PROVEN`, `FAILED`, `NOT_PROVEN`, and `BLOCKED` at scenario level. Keep run-level readiness separate. Report discovered/exercised/blocked/not-tested states and actions. A test that could not run is not a pass; an observational guess is not a confirmed bug.

### C. Default-safe permissions

Use the v0.3 modes, carried into 09: **ask**, **catalog**, and **budgeted**. Start with ask for external access and a narrowly approved catalog for sandbox-local dependencies. Log every permission-mode change, install, command, egress request, and sensitive read. Apply explicit budgets and hard stops. Treat external production telemetry as a separate grant, not part of a general “use whatever tool is available” permission.

### D. Gate only on evidence

The 16 rule is useful: advice plus a measurement can be a gate; advice without a measurement stays advisory. For destructive or release-blocking hooks, gate on replay-confirmed findings and clear configured policy—not model confidence alone. A user can inspect why a gate blocked and override according to local policy.

### E. Privacy and performance for TRACK mode

“Always on” should mean maintaining a lightweight, local, incremental model—not silently uploading or retaining every trace forever. Define what is captured, where it is stored, retention/rotation, sampling, resource budgets, how to pause it, and which collectors need approval. Make hosted telemetry opt-in and keep code, secrets, and evidence local by default.

### F. Calibrate before making reliability claims

Before public launch, test on multiple real AI-built apps with known injected invariant, failure-injection, and authorization bugs. Publish which bugs were injected, which were found, replay rates, coverage, and false-positive outcomes. Avoid unsupported claims such as “production-ready,” “zero downtime,” “zero flakiness,” or “100% green.”

---

## 6. How each older idea can complement, without bloating the first implementation slices

- **04 living code map:** keep as the semantic graph; implement a small incremental model first, not a universal parser for every language.
- **04/06 Architect:** preserve measurable architecture checks and stack-context awareness; defer autonomous stack changes and subjective file-size rules.
- **04 virtual swarm:** preserve actor personas and deterministic compilation; start with a few semantic actors and delegate high concurrency to a mature load tool.
- **04 five-layer recorder:** preserve the end-state schema; start with a small honest set of correlated layers and expand as adapters become reliable.
- **05 tool sourcing:** use adapters and mature tools; maintain a supported-version and license review for every dependency. Treat k6/AGPL and other license flags in 09 as items for legal review before redistribution.
- **06 Git Guardian:** use worktrees/snapshots as isolation infrastructure, but don’t make autonomous Git management a V1 headline feature.
- **07 dashboard and UI inspector:** keep a simple live progress view/CLI output after the proof is established; no large dashboard effort first.
- **07 feedback synthesizer:** design its output to become a reviewed scenario with provenance, but defer inbound feedback integrations until the core generator is validated.
- **07 universal SRE:** reuse adapters later for read-only evidence ingestion; keep hotfix/deploy mutation out of V1.
- **v0.3 invariant watchers, permissions, flake history, AI contracts:** retain as compatible trust features, prioritizing watchers, permission governance, and replay before AI-specific checks.
- **16/17 six modes:** make them views over the same evidence engine. TEST is the first implementation workflow; DEBUG is a natural second view; other modes are staged, not removed from the full-product plan.
- **18 tool adapters:** preserve the shared evidence and permission contract while adding BrowserTools/Chrome DevTools, Lighthouse/axe, visual-regression, DAST, production-observability, and external test-platform adapters across later workstreams.

---

## 7. Final recommendation

Adopt this hierarchy going forward:

- **Scope:** `16-testkit-product-scope-v2.md`.
- **Initial plugin architecture and ship order:** `09-testkit-plugin-architecture.md`.
- **Operating modes:** `17-testkit-capability-map-by-mode.md`.
- **Full-product tool integrations and evidence contract:** `18-testkit-tool-adapter-and-evidence-integration-plan.md`.
- **Vision and capability heritage:** `04`, `06`, and `07`.
- **Tool options and build environment:** `05`.
- **Trust, session, and security detail:** compatible parts of v0.2/v0.3 and the product brief, updated where 16/09 changes the boundary.

The first implementation should focus on **the oracle, live invariant assertion, deterministic replay, and trustworthy evidence packet**. Build a thin but real harness around that proof, then add the full planned UI-quality, active-security, runtime-observation, interoperability, and read-only production-witness adapters in sequenced workstreams. This is the development order—not a cut to the product plan. Keep every claim measurable and every untested area visible.
