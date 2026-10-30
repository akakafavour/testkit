# TestKit: Unified Source of Truth Specification

**Version:** 1.0.0 (Unified Master)  
**Package Standard:** Agent Plugins 1.0 (`agent-plugins.org`) + Agent Skills (`agentskills.io`) + MCP (`2026-07-28`)  
**Architecture Core:** TypeScript / Node.js Ambient Daemon (`testkitd`) + Multi-Surface Client Engine (Report, TUI, Web, CI, Agent MCP Plugin)

---

## ⚠️ Implementation Status — Read Before Citing This Document

This document is the **target architecture**. It mixes shipped behavior with design vision. As of v1.0.0, the implemented engine is:

**Shipped (Rust workspace, `crates/`):**
- `testkit init` — deterministic `.testkit/` provisioning, `.gitignore` hygiene, `TESTKIT.md` drop.
- `testkit verify` — scenario execution with four scenario kinds: `process` (spawn a program with `TESTKIT_FAKE_NOW` injected), `http` (request with `X-Testkit-Fake-Now` header; status, body markers, negated markers, timing budget), `agent` (the operator agent drives the flow and writes declared evidence artifacts; the engine verifies and notarizes them — missing evidence blocks the receipt), and `load` (concurrent virtual users against one endpoint, gated on error rate and p95 latency). Whole-run CPU/RAM telemetry, runaway detection, evidence capture, tamper-evident receipts. Filters: `--scenario <id>`, `--tag <tag>`, `--limit <n>` for incremental ramping; per-run envelope overrides `--max-cpu` / `--max-rss-mb`; `--fail-fast`.
- Per-service resource envelopes: each scenario can declare its own budget (frontend vs backend tracked separately), with run-level flags as fallback and hosting-plan presets (edge 128 MB, Render free 512 MB, serverless-function 1 GB, $12-server 1.5 GB, high-throughput 8 GB).
- `testkit gate` — receipt ledger evaluation with 4-way verdicts (`PASSED` / `FAILED` / `NOT_PROVEN` / `BLOCKED`) and P0/HIGH blocking.
- `testkit receipt list | latest | verify` — ledger inspection and integrity verification.
- `testkitd` — a directory-watch daemon that logs `.testkit/` change events (no AST map, no warm harnesses yet).
- Receipts carry a **SHA-256 integrity hash** (tamper-evident), not an asymmetric signature.
- A runnable worked example (`examples/demo/`) and a GitHub Actions CI matrix (Linux + Windows, `cargo test --workspace`).

**Vision, not yet implemented (do not assume it exists):**
- The TypeScript/Node daemon architecture, semantic AST map, and 8-layer watcher fabric (§4).
- AST feature extraction, state-machine synthesis, and the combinatorial scenario engine (§8).
- Automatic breaking-point search across envelopes, distributed/multi-node load generation, and baseline regression comparison across receipts.
- Compute tiers A/B/C, remote runners, and hybrid split execution (§6) — remote targets work today only as plain scenario URLs and `ssh` process scenarios, without remote telemetry.
- Third-party service doubling, seed-data injection, git worktree isolation (§9).
- Production SRE witness and telemetry ingestion (§12) — real-time log connections are not built.
- Numeric targets quoted here (2–8 ms/scenario, 10,000+ scenarios) are design goals, not measured results.

When making claims about TestKit to users or in marketing, cite only the **Shipped** list, or label the rest as roadmap.

---

## 1. Executive Summary

> **"TestKit is the agent's testing toolkit and operating manual; the agent is the operator."**

A widespread fallacy in AI testing is imagining an "autonomous QA agent" that magically understands your business domain, invents test suites out of thin air, and roams around finding bugs on its own. That model produces flaky, superficial tests that merely reflect whatever code was generated.

The correct architectural paradigm is **the Tool + Skill Model (The Remotion Analogy)**:
- **Remotion** provides the engine and CLI (`@remotion/cli`, Chromium renderer, timeline clock). The **Remotion Skill** teaches an AI agent how to author video compositions. The **AI Agent** is the operator that understands the user's intent, writes the composition, executes the CLI, and fixes visual errors.
- **TestKit** operates identically:
  1. **The TestKit Engine & Daemon (`testkit` CLI + `testkitd`):** The deterministic mechanical substrate. It handles project initialization, process lifecycle observation, full-stack trace interception (UI, API, DB WAL, host CPU/RAM), time-travel clock virtualization, seed-matrix execution (2–8ms/scenario), and cryptographic receipt stamping.
  2. **The TestKit Skill (`SKILL.md` / `TESTKIT.md`):** The operating manual that equips any AI coding agent with the domain discipline of a Principal Systems & Reliability Engineer—teaching it *when and how* to declare invariants, probe edge cases, seed realistic volumes, and interpret receipts.
  3. **The Coding / ShipKit Agents (The Operators):** The human-prompted intelligence. The *Architect* declares the hard invariants and $12-server resource envelopes; the *Builder* implements the code; the *Tester/Security* agents invoke TestKit's verification tools to stress-test the implementation and fix any violations.

Rather than making unprovable claims of "zero defects" or "finding every bug," TestKit provides **honest, mathematically quantified proof of software reliability**: exactly what was discovered, what was exercised, what was verified, and what remains unreached.

---

## 2. Core Operating Principles

1. **The Remotion Model: Tool vs. Operator:**
   TestKit is NOT an autonomous QA entity that invents tests in a vacuum. It is an **instrumentation engine and operating manual** operated by coding agents:
   - **The Engine (`testkit` CLI / `testkitd`):** Performs mechanical execution, state isolation, process telemetry, clock virtualization, and receipt generation.
   - **The Skill (`SKILL.md`):** Equips the agent with testing methodology, invariant design patterns, and seed failure templates.
   - **The Agent:** Retains cognitive ownership—planning the matrices, invoking the toolkit, analyzing receipts, and implementing fixes.
2. **The Daemon Owns the Evidence (testkitd Core Architecture):**
   The core is an ambient, low-footprint daemon (`testkitd`) running locally. It maintains the semantic AST map, tails DB Write-Ahead Logs, intercepts server logs, and manages an append-only causal evidence store. The Agent Plugin is **one client surface** among many (HTML report, TUI, Web dashboard, CI action).
3. **"Watch Once, Read Many" Economic Model:**
   Instrumenting a system (debuggers, proxies, DB connection pools) is expensive to do on-demand. `testkitd` starts once (`npx testkit init`), remains near-zero CPU when idle, and ensures the semantic map and observation harness are always warm. Debugging a failure requires zero re-runs—the causal trace is already captured.
4. **ARCHITECT Comes First (Day-0 Lifecycle Foundation):**
   TestKit does not start by testing code that already exists. It starts at raw idea: requirement extraction, tech stack selection, data modeling, API contract definition, and independent invariant declarations.
5. **The Assertion Oracle (Spec vs Behavior):**
   If an AI generates both the code and the test, they share the exact same blind spot. TestKit anchors all expectations **externally** in requirements, documentation, and declared invariants—never from observing running code.
6. **Hardware Sympathy & Adaptive Algorithmic Verification ("The Envelope Test"):**
   AI agents write syntax without understanding physical hardware constraints or algorithmic scaling. TestKit enforces physical hardware realities and deep computer science primitives:
   - **No Hardcoded Defaults:** The resource envelope is completely adaptive to any environment:
     - Constrained edge/serverless ($128\text{ MB RAM}$, micro-timeouts).
     - Single-node VPS ($1\text{ vCPU}, 1.5\text{ GB RAM}$ baseline).
     - High-throughput nodes ($4\text{+ cores}, 8\text{+ GB RAM}$).
     - The AI is never boxed into one fixed rule; it is given the freedom to explore architectures while remaining bound to the declared invariant.
   - **Deep Engineering Verification (Trees, Sorting, Graphs, Indexing):**
     - **B-Trees & LSM-Trees:** Flags $O(N)$ sequential table scans on disk; proves $O(\log N)$ indexed query plans.
     - **Graph & Dependency Order:** Enforces Topological Sort on async dependency DAGs and workers to prevent deadlocks.
     - **Search & Prefixes:** Enforces Tries and Inverted Indexes for high-frequency autocomplete/search rather than naive in-memory array scans.
     - **Latency Hierarchy:** RAM access (~100 ns) vs. SSD I/O (~100 µs) vs. Network (~150 ms).
     - **Single-Tick Event-Loop Write Batching:** Flags sequential disk sync stalls; prescribes transacted multi-row batching.
7. **Compute Sovereignty & Hybrid Split Execution:**
   Testing is never artificially gated by user-tier caps (10k, 100k, 1M, or 1B virtual users are limited only by compute). TestKit provides a 3-tier execution plane:
   - **Tier A (Local In-Place / Capped Docker):** Strict CPU (e.g. 1.0 core) and RAM (e.g. 1.5 GB) envelopes so lightweight laptops stay cool.
   - **Tier B (Bring Your Own Compute / VPS):** Connect any self-hosted VPS (Hetzner, DigitalOcean, AWS) via SSH/Docker (`testkit runner connect`) with 100% free compute sovereignty.
   - **Tier C (TestKit Managed Cloud):** Ephemeral cloud sandboxes authenticated like GitHub CLI (`testkit login`) for zero-setup, on-demand scale.
   - **Hybrid Split Mode:** Lightweight UI/component checks run locally on the developer's laptop, while heavy concurrent traffic simulations, DB replicas, and DAST scans run simultaneously in the cloud/VPS, unified into one causal receipt.
8. **MCP-Agnostic Research & Tool Grounding:**
   TestKit hooks into external documentation and research MCPs:
   - **Tier 0:** Direct `llms.txt` retrieval.
   - **Tier 1:** **Context7 MCP** for version-pinned, injection-scanned first-party docs.
   - **Tier 2:** **DeepWiki MCP** for runtime behavioral patterns.
   - **Tier 3:** **Firecrawl / Web Search** for edge packages and live references.
   TestKit synthesizes this research to recommend specific stacks with architectural rationale and failure-mode analysis.
9. **Strict Separation of Concerns & Powers:**
   - **Application Code:** The user's coding agent (Claude Code, Cursor, Devin) writes and modifies application code. TestKit never directly edits application source files.
   - **Verification & Staging:** TestKit plans checks, synthesizes evidence, generates reproduction scripts, and stages sandboxed worktrees.
   - **Human Authority:** Critical production operations (database migrations, live traffic routing, git resets) require explicit human sign-off unless specifically delegated by policy.
10. **Deterministic Proof & 4-Way Verdicts:**
    Findings are compiled into deterministic scripts that replay without LLM tokens. Verdicts are strictly 4-way: `PROVEN`, `FAILED`, `NOT_PROVEN`, or `BLOCKED`.
11. **Honest Quantified Proof (No "Zero-Blind-Spot" Fallacy):**
    No test system can test infinite permutations or find every bug. TestKit rejects fraudulent claims of "100% bug-free". Instead, it maintains an immutable Evidence Ledger that mathematically accounts for:
    - **Discovered States:** What the static and semantic AST revealed.
    - **Exercised States:** What the combinatorial matrix ran.
    - **Proven Invariants:** What passed deterministic assertions.
    - **Unreached / Unprovable States:** What was blocked by missing mocks, third-party sandboxes, or undeclared contracts.
12. **Tested Adapter Ecosystem (Not Magic Universal Heuristics):**
    Support across runtimes is anchored in explicit, tested adapters:
    - **Tier 1 (Native Built-in):** TypeScript/JavaScript (Node, Bun, Next.js), Python (FastAPI, Django), SQL (Postgres, SQLite), Web (Playwright / CDP).
    - **Tier 2 (Adapter Extensions):** Go, Rust, Java/Kotlin, Mobile (Maestro/Appium), NoSQL (MongoDB change streams, DynamoDB streams, Redis keyspace).
    - **Tool Orchestrators:** Ingests and unifies results from existing linters (ESLint, Biome), type-checkers (`tsc`, `pyright`), unit suites (Vitest, pytest), and OpenAPI/Postman collections.
13. **Tri-Tier Governance (Hard Gates vs. Advisory vs. Human Approval):**
    - **Tier 1: Hard Gates (Automated Blocking):** Catastrophic security risks (hardcoded secrets, raw SQL injection, IDOR), breaking P0 invariant regressions, and unindexed full-table scans on production-sized tables.
    - **Tier 2: Advisory Findings (Evidence-Based Recommendations):** File complexity/bloat suggestions, Redis caching recommendations, database partitioning advice, and serverless cold start warnings. These educate the developer without halting progress.
    - **Tier 3: Human-in-the-Loop (Mandatory Authorization):** Irreversible actions—including production deployments, production schema cutovers, and local git worktree resets—require human confirmation.

---

## 3. The End-to-End Lifecycle Pipeline

### 0. Mechanical Grounding (`testkit init`)
Before any planning, ideation, or architecture begins, the project workspace is mechanically grounded. **`testkit init` is purely deterministic and non-conversational (identical in spirit to `git init`):**
- **Zero Prompts / Zero Interviews:** It does not ask questions, classify ideas, or author rules. It runs in under 1 second.
- **Provisions Groundwork Hierarchy (`.testkit/`):**
  - `.testkit/brain/`: Reserved location for future architecture specs, invariants, and learned rules.
  - `.testkit/evidence/`: Capture destination for runtime traces, logs, memory/CPU snapshots, and visual assets.
  - `.testkit/receipts/`: Append-only storage for cryptographically signed test run certificates (`run_XXXX.json`).
  - `.testkit/scenarios/`: Executable scenario storage.
- **Git Hygiene Defense:** Automatically configures `.gitignore` to track `.testkit/brain/` rules while keeping heavy ephemeral dumps (`.testkit/evidence/`) out of version control.
- **Agent Operating Manual (`TESTKIT.md` / `AGENTS.md` Directive):** Drops a universal directive informing any coding agent (Claude Code, Cursor, OpenCode, Codex) that the repository is TestKit-governed, where rules live, and that commits are gated by verified receipts.
- **Spawns Ambient Daemon (`testkitd`):** Launches the low-footprint background observer (~50MB RAM, ~0% idle CPU) to keep AST and process harnesses warm.

```
  ┌────────────────────────────────────────────────────────────────────────┐
  │ 0. MECHANICAL GROUNDING (testkit init)                                 │
  │    Input: Empty folder or existing repo                                │
  │    Actions: Provision .testkit/, gitignore, TESTKIT.md, start testkitd │
  │    Output: Fully initialized workspace (Zero conversational prompts)    │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 1. INCEPTION & AUTO-DETECT                                             │
  │    Input: Raw idea or existing repo                                    │
  │    Actions: Classify scope, profile stack, research via Context7/Web   │
  │    Output: brief.md, plan.md, profile.json                             │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 2. ARCHITECT FIRST (Day 0)                                             │
  │    Input: brief.md, plan.md, research context                          │
  │    Actions: Stack selection, data model, API contracts (OpenAPI)       │
  │             ⭐ Formally declare independent invariants & gate policies │
  │    Output: architecture.md                                             │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │ [Gate: User signs off on Architecture]
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 3. CODING AGENT IMPLEMENTATION                                         │
  │    Coding agent (Claude Code / Cursor / Codex) builds features         │
  │    TestKit daemon tracks changes incrementally via Tree-Sitter AST     │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │ [Hook: on commit / PR]
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 4. INDEPENDENT VERIFICATION & RED-TEAMING (Day 1+)                     │
  │    Actions: Bounded semantic actors, Playwright/Maestro UI validation, │
  │             Live 8-layer watchers, failure injection (Toxiproxy),      │
  │             Security matrix (IDOR, RLS, secret leakage)                │
  │    Output: qa-report.md + tamper-evident replay receipts               │
  └───────────────────────────────────┬────────────────────────────────────┘
                                      │ [Fix loop with Coding Agent]
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 5. DEPLOY REHEARSAL & SHIP-GATE                                        │
  │    Actions: Migrations up/rollback rehearsal, env parity verification, │
  │             post-deploy smoke test in sandbox, CI/CD blocking gate     │
  │    Output: Verified deployment artifact + Production Witness hook      │
  └────────────────────────────────────────────────────────────────────────┘
```

---

## 4. Cross-Platform Live Watcher Architecture (iOS, Android, Web, CLI, Desktop)

The 4-watcher model (HTTP, Console, Server Log, DB) expands into a universal 8-layer observation fabric tailored to the target platform:

| Watcher | Web / API Target | iOS / macOS Target | Android Target | Desktop / CLI Target |
|---|---|---|---|---|
| **1. UI / Accessibility** | DOM / CDP accessibility tree | XCUITest / Accessibility API | UIAutomator / Accessibility tree | Win32 UIA / macOS AX tree |
| **2. Client Console / Logs** | Browser DevTools console / errors | `os_log` / Unified Logging system | Android `Logcat` stream | Terminal `stderr` / process stream |
| **3. Network & Transport** | CDP Fetch/XHR / HAR capture | URLSession proxy / CFNetwork capture | OkHttp interceptor / Network proxy | Sockets / Local proxy |
| **4. Server / Runtime Logs** | Node/Go/Python stdout/stderr | API server logs | API server logs | Process stdout/stderr |
| **5. Data & Storage** | Postgres WAL / SQLite / Redis | CoreData / SQLite / Keychain | Room / SQLite / EncryptedSharedPreferences | App local storage / SQLite |
| **6. Device & OS State** | Viewport / online-offline state | Lifecycle events (background/foreground, push notifications, deep links) | Activity lifecycle, intent filters, orientation, battery | OS signals / exit codes |
| **7. Background Queues** | BullMQ / Celery / Cron jobs | BackgroundTasks framework | WorkManager / JobScheduler | System cron / daemon threads |
| **8. Fault & Chaos** | Toxiproxy (HTTP toxics) | Network link conditioner / throttle | Bad network emulator / ADB throttle | Process kill / socket drops |

### 1. Sub-Millisecond Multi-Signal Correlator (The Flight Data Recorder)
To eliminate AI hallucination, TestKit binds every observed layer onto a single **microsecond-resolution monotonic clock** (`performance.now()`):
- **Unified Causal Graph:** When an anomaly occurs, TestKit correlates the UI click, the outgoing HTTP request/payload, the backend route execution, the Postgres SQL query, and the exact DOM render state down to the millisecond.
- **Microsecond Timestamp Anchoring:** If a user action causes a 500 white-screen crash, TestKit captures the exact DOM state and camera snapshot at that precise millisecond, proving the causal chain from UI click to DB deadlock.

### 2. Native Modern Browser & Agentic Hooks (`/browser`, CDP & Playwright)
Modern coding agents possess native browser tools and subagents (e.g. `/browser`, Chrome DevTools Protocol, Playwright). TestKit integrates natively with these tools:
- **Instant Event-Driven Screen Capture:** Rather than polling or slow manual screen scraping, TestKit hooks into the CDP event loop. When an uncaught exception, visual overlap, or network failure triggers, TestKit snaps a high-resolution screenshot in under 15 milliseconds.
- **Parallel Multi-Actor Browser Emulation:** Coordinates parallel browser sessions (e.g. Actor Alice blocking Actor Bob) to test social features, permissions, and group messaging rules concurrently without cross-contamination.

### 3. Before-and-After Differential Verification (Proving the Healed State)
When a coding agent applies a code fix, TestKit does not just run a generic test. It executes a **Differential Verification Pass**:
- **Replay Against the Crime Scene:** Re-runs the exact failing scenario using the same pre-seeded inputs, timestamps, and mock states that triggered the initial bug.
- **Visual & State Delta Check:** Verifies that the previous error screen (e.g. raw 500 white screen) has been replaced by the expected compliant UI state (e.g. clean 403 error dialog) with side-by-side screenshot diffs (`before_fix.png` vs. `after_fix.png`).
- **Regression Invariant Guard:** Re-runs the full invariant suite to verify that the agent's fix did not inadvertently break other features. Only when both the delta is proven and zero regressions exist is the run receipt signed as `PROVEN_RESOLVED`.

### 4. Output Pipeline & Clean Repository Organization (`.testkit/`)
All test runs, captured evidence, and run receipts are cleanly contained inside a single, dedicated directory within the repository root: **`.testkit/`**, which is automatically added to `.gitignore` so no transient evidence clutters git history.

```
.testkit/                      <-- (Automatically managed in .gitignore)
├── brain/                     <-- Project-specific learned rules & architectural memory
│   └── learned-rules.json     <-- Anti-spaghetti patterns & repo-specific do's/don'ts
├── evidence/                  <-- Target-adaptive proof artifacts
│   ├── screenshots/           <-- UI Targets Only: before_fix.png, after_fix.png
│   ├── terminal/              <-- CLI/Daemon Targets: stderr logs, POSIX exit codes
│   └── traces/                <-- API/DB Targets: HAR dumps, SQL transaction tails
├── receipts/                  <-- Cryptographic run receipts: run_XXXX.json
└── report.html                <-- Optional lightweight, simple human summary
```

#### A. Target-Adaptive Evidence (Not Everything Has a UI)
TestKit adapts its evidence capture to the exact platform being tested:
- **Graphical Targets (Web, Desktop, iOS, Android):** Captures pixel-perfect screenshots and view hierarchy trees (`evidence/screenshots/`).
- **Headless & Backend Targets (APIs, Serverless, Microservices):** Captures HTTP status codes, JSON response diffs, latency histograms, and SQL query execution plans (`evidence/traces/`).
- **CLI & System Daemons (Terminal tools, Windows Services, Background Jobs):** Captures standard error streams (`stderr`), stdout logs, OS signals, process exit codes (`0` vs `1`), and memory consumption graphs (`evidence/terminal/`).

#### B. The Agent Brain: Eliminating Spaghetti Code Across Any Project
To prevent AI agents from generating sprawling 1,500-line spaghetti files and repeating past mistakes, TestKit maintains a **Two-Tier Brain Memory**:
1. **Local Project Brain (`.testkit/brain/learned-rules.json`):**
   - Records repository-specific architecture patterns (e.g. *"This repo uses modular service classes in `src/services/`, never write inline database mutations in controllers"*).
   - Injected into the coding agent's prompt via hooks before new code is typed.
2. **Global Developer Brain (`~/.testkit/brain/global-knowledge.json`):**
   - Persists clean code wisdom across **all projects on the developer's machine**.
   - If an agent learns on Project A that a certain library causes memory leaks or that a specific async pattern triggers race conditions, that lesson is automatically applied when starting a brand new Project B tomorrow.
   - **Result:** The agent writes modular, clean, single-responsibility code from Day 1, with zero spaghetti.

- **Automatic Git Protection:**
  - TestKit automatically verifies and injects `.testkit/` into `.gitignore` upon initialization, guaranteeing that screenshots, logs, and temp fixtures are never accidentally committed.

---

## 5. The Multi-Layer Verification Matrix

To systematically maximize test coverage across system layers, TestKit orchestrates specialized checks across 9 core dimensions:

1. **Functional Usability & Workflows:** Core user journeys from signup to retention, across roles.
2. **Cross-Actor & Concurrency:** Multi-tenant isolation, IDOR, simultaneous edits, race conditions.
3. **Data Integrity & Invariants:** Zero silent orphan rows, soft-delete consistency, idempotency on retries.
4. **Visual & Layout Integrity:** Percy/Chromatic-style baseline screenshot diffs, clipping, obscuration.
5. **Accessibility (a11y):** Axe-core automated WCAG compliance and keyboard/touch-target navigation.
6. **Performance & Budgets:** Core Web Vitals (LCP, CLS, FID), native frame drops, memory leaks, p99 latency.
7. **Security & Red-Teaming:** Scoped DAST (ZAP / hackbrowser-mcp), unauthenticated RLS read scans, secret detection.
8. **AI-Feature Contracts (if app embeds LLMs):** Deterministic schema enforcement, latency/cost budgets, secret non-leakage, and provider-outage fallbacks.
9. **Deploy Rehearsal:** Migration reversibility, environment-variable parity, and pre-deploy smoke checks.

---

## 6. Real-World Infrastructure & System Design Stress Testing

TestKit bridges the gap between an AI prototype and a production-grade system by simulating real-world traffic patterns, capacity bottlenecks, and deployment architectures:

### 1. Scaling Physics: The "$12 Server Benchmark" & Hardware Limits
Novice builders and AI coding agents reflexively throw Kubernetes, microservices, and 5 separate cloud databases at a problem before optimizing a single node. TestKit enforces **Mechanical Sympathy** and vertical optimization:
- **The $12 Server Reality:** A basic 1-CPU, 2GB RAM VPS running NGINX, app server, and Postgres can easily support 5,000+ concurrent active users (equivalent to 30,000+ Daily Active Users) if architected cleanly:
  - *Tier 0: Query Plan Optimization:* Eliminating sequential table scans ($O(N) \rightarrow O(\log N)$) keeps RAM under 1 GB and DB CPU under 15%.
  - *Tier 1: In-Process Caching:* Caching high-read endpoints in application memory doubles throughput (e.g., from 2,500 to 4,000 users).
  - *Tier 2: Reverse-Proxy Caching (NGINX/Caddy):* Moving cache as close to the user as possible bypasses the application runtime entirely, pushing concurrency past 5,000 on a $12 box.
  - *Tier 3: Single-Tick Event-Loop Write Batching:* Grouping all disk writes occurring within a single event-loop tick into one database commit transaction (the Fortran/SQLite principle) unlocks up to 38,000 concurrent users by eliminating blocking SSD sync stalls ($100\ \mu\text{s}$ per transaction).
- **The System Design 101 Law:** 
  > *"Every component adds complexity: a cache can go stale, a queue can deliver twice, replicas lag, sharding complicates queries, and microservices introduce network failures. Only add a component when an actual physical bottleneck forces it."*
  TestKit acts as the gatekeeper against premature architecture: it proves an existing node is bottlenecked before allowing the agent to introduce queues, replicas, or distributed services.

### 2. Compute Sovereignty: 3-Tier Execution & Hybrid Split Testing
Testing is never gated by arbitrary user limits (10k, 100k, 10M, or 1B virtual users). TestKit provides three execution tiers:
- **Tier 1: Local In-Place & Resource-Capped Docker:**
  - For lightweight machines (e.g. MacBook Air, 8GB RAM), TestKit enforces strict resource caps (e.g., 1.0 CPU, 1.5 GB RAM) so laptop fans never spin and batteries never drain.
- **Tier 2: Bring Your Own Compute / VPS (BYOC):**
  - Developers can connect their own self-hosted VPS (Hetzner, DigitalOcean, AWS, home lab) via SSH/Docker:
    ```bash
    testkit runner connect user@my-vps-ip
    ```
  - Grants 100% compute sovereignty, unlimited concurrent virtual users, and zero vendor lock-in.
- **Tier 3: TestKit Managed Cloud (Zero-Setup SaaS):**
  - Authenticated seamlessly like the GitHub CLI or NPM:
    ```bash
    testkit login
    ```
  - Spins up ephemeral cloud sandboxes with realistic production database replicas and load generators, shutting them down immediately after the run.
- **Hybrid Split Execution:**
  - TestKit uniquely splits workloads across environments simultaneously:
    - **Frontend / UI checks** run locally on the developer's laptop with instant hot-reload and Chrome DevTools/CDP capture.
    - **Heavy backend load (50,000 virtual users), DB replicas, and DAST scans** run concurrently in the cloud or remote VPS.
    - TestKit's daemon synchronizes both into a **single, unified causal evidence receipt**.

### 3. Deployment Topology & Zero-Downtime Rehearsal
- **Deployment Strategy Selection:** Determines whether the architecture requires:
  - **Blue/Green Deployment:** Traffic flips atomically from old version to new version once health checks pass.
  - **Canary Rollouts:** 5% of traffic routed to new build to monitor error rates and latency before 100% rollout.
  - **Rolling Updates:** Sequential container updates with graceful connection draining.
- **Database Migration Compatibility (Expand & Contract):**
  - Verifies that new database schema migrations are backwards-compatible with running versions of the app during rollouts (preventing downtime during mid-deploy traffic).

### 3. Distributed System Stress & Failure Scenarios
- **Connection Pool Sizing:** Asserts that database connection pools (e.g. PgBouncer, Prisma, Hikari) match expected concurrency under load without dropping queries.
- **Cache Invalidation & Thundering Herds:** Simulates cache misses across thousands of concurrent actors to test whether the origin database collapses (stampede mitigation).
- **Rate-Limiting & Backpressure:** Verifies that token buckets, leaky buckets, and 429 backoff headers protect internal microservices from cascade failures.
- **Queue Saturation & Dead-Letter Handling:** Tests queue drain rates, consumer concurrency, poison-pill messages, and worker heartbeat timeouts.

### 4. Polyglot Persistence & Database Workload Fit (Beyond Vibe-Coded Postgres)
Vibe-coders default lazily to a single Postgres instance for everything. When traffic shifts, this creates fatal bottlenecks. TestKit profiles workload characteristics to recommend and enforce polyglot architectures:
- **Hot Ephemeral Caching (Redis/Valkey):** Automatically recommends and gates in-memory key-value caching when read-to-write ratios exceed 85% on session states, rate limits, or cart data.
- **High-Velocity Analytics & Event Logs (ClickHouse/Timescale):** Flags heavy `GROUP BY` and historical aggregation queries on transactional OLTP tables; prescribes offloading to columnar or time-series engines.
- **Vector Embeddings & Semantic Search (pgvector vs Dedicated Vector DB):** Recommends when in-database vector search (e.g. `pgvector`) suffices (<100k vectors) versus when scale demands dedicated vector stores (Qdrant, Pinecone, Milvus) with filtered HNSW indexing.
- **Automated Partitioning & Sharding Triggers:**
  - *Declarative Partitioning:* Automatically generates SQL partitioning schemes (`RANGE` by timestamp or `LIST` by tenant ID) when tables exceed 10 million rows.
  - *Horizontal Sharding:* Prescribes horizontal sharding architectures (Citus, Vitess, or tenant-isolated schemas) when single-node write IOPS saturate.

### 5. Serverless, Lambda & Edge Execution Physics
Serverless runtimes (AWS Lambda, Vercel Functions, Cloudflare Workers) suffer from unique runtime physics that AI code consistently breaks:
- **Connection Pool Exhaustion Prevention:** Verifies that database instances and connection pools are declared in outer container scope for warm reuse, and requires PgBouncer, AWS RDS Proxy, or serverless HTTP database drivers.
- **Cold Start Latency & Import Pruning:** Analyzes the dependency graph of serverless bundles. Rejects monolithic SDK imports (e.g. full AWS SDK) in favor of modular tree-shakeable packages.
- **Execution Timeout & Decoupling:** Flags synchronous long-running operations (batch CSV imports, video transcoding, external mass mailing) inside synchronous HTTP handlers. Mandates background queues (SQS, Inngest, Trigger.dev, BullMQ) returning `202 Accepted`.
- **Warm Container State Containment:** Detects mutable global arrays or caches that grow unbounded across multiple warm invocations, preventing Out-Of-Memory (`OOM 502`) crashes.

### 6. Real-Time, WebSocket & Streaming Event Integrity
- **Zombie Connection Pruning & Heartbeats:** Verifies bidirectional WebSocket connections implement ping/pong frames and client reconnection backoff.
- **Reconnection Storm Resilience:** Simulates server restarts with 10,000 active socket connections to ensure the gateway doesn't collapse under simultaneous reconnections (jittered exponential backoff enforcement).
- **Stream Backpressure:** Verifies that Server-Sent Events (SSE) and HTTP chunked streams respect downstream consumer backpressure without buffering unbounded memory chunks on the server.

### 7. Modern Fullstack & Rendering Safety (React 19, RSC & Mobile Lifecycles)
- **Server/Client Component Boundary Security:** Scans React Server Components (RSC) to verify server-only secrets, database clients, or internal tokens never cross the `"use client"` boundary into client bundles.
- **Hydration Determinism:** Flags non-deterministic functions (`Date.now()`, `Math.random()`, `window.innerWidth`) in initial SSR renders that cause React hydration mismatches.
- **Infinite Effect Loop Guards:** AST-scans React `useEffect`, `useCallback`, and `useMemo` hooks for unmemoized object/array dependencies that trigger runaway re-render loops.
- **Mobile Lifecycle & Process Death:** For iOS/Android apps, simulates background process kills by the OS to verify that navigation state, forms, and draft data cleanly rehydrate from local persistent storage.

### 8. Cross-Platform Operating System & Environmental Discrepancies
- **File System Case-Sensitivity Traps:** Identifies file and module imports whose casing doesn't match the disk file name (e.g. `import User from './user'` when file is `User.ts`), which passes silently on macOS/Windows but crashes Docker Linux in CI/production.
- **Line-Ending Cryptographic Verification:** Enforces uniform LF line endings on files and payloads used in cryptographic HMAC/signature generation, preventing silent verification failures between Windows (`\r\n`) and Linux (`\n`).
- **Timestamp Precision Drift:** Enforces explicit microsecond vs millisecond timestamp handling between Postgres (`timestamptz`) and JavaScript `Date`, preventing phantom race conditions in time-based entity comparisons.

### 9. Multi-Region Global Data Sovereignty & Geo-Routing Physics
When applications scale across global regions (US, EU, APAC), legal boundaries and distributed physics create catastrophic failure points:
- **Legal Data Sovereignty & Residency (GDPR / Schrems II / HIPAA):** Scans routing logic, database connection factories, and backup pipelines to verify that EU tenant data (PII) is pinned strictly to EU-located clusters (e.g., `eu-central-1`) and never leaks to US regions during failovers.
- **Multi-Master Conflict Resolution (Split-Brain Prevention):** When active-active databases run in multiple regions (e.g. DynamoDB Global Tables, CockroachDB, Spanner), simultaneous writes in London and Tokyo must not suffer "Last-Write-Wins" silent data loss. TestKit verifies Conflict-free Replicated Data Types (CRDTs), vector clocks, or deterministic merge handlers.
- **Geo-DNS Failover Drift:** Simulates regional data center outages (e.g. AWS `us-east-1` down) to verify Anycast/Route53 failover routes to the nearest healthy region without dropping active sessions or violating data residency covenants.

---

## 7. Anti-AI Slop & Clean Code Quality Enforcement (Educating the Vibe-Coder)

AI coding agents left unchecked produce "AI Slop": bloated 1,500-line single files, duplicated utility logic across 10 folders, silent try-catch blocks that swallow errors, hallucinated mock data, hardcoded secrets, and over-abstracted wrappers.

TestKit acts as an automated **Principal Staff Engineer gate** that eliminates AI slop and enforces clean, production-grade code that any human engineer or team can maintain:

### 1. Concrete Measurable Anti-Slop AST Rules (Gate vs. Advisory)
TestKit mechanically rejects AI slop using enforceable code metrics and native AST rules:
- **`code/god-object-ban`:** Hard limit on single-file lines (>350 lines with multiple responsibilities triggers an architectural refactor finding).
- **`code/silent-failure-ban`:** Rejects empty `catch (e) {}` blocks, fallback stubs, or placeholder mocks that fake a passing test while doing nothing.
- **`async/floating-promise-check`:** Rejects missing `await` on database, network, and async calls that cause unhandled rejections and lost server logs.
- **`concurrency/atomic-increment-check`:** Rejects non-atomic read-then-write updates on financial balances and counters (`balance: user.balance + amount`); requires atomic database increments or transactional locks.
- **`serverless/connection-pool-check`:** Rejects database client initializations inside exported serverless handlers.
- **`serverless/unbounded-memory-check`:** Flags mutable global state outside serverless handlers that causes memory leaks across warm container invocations.
- **`serverless/bundle-size-check`:** Flags monolithic package imports that cause multi-second cold starts.
- **`compliance/geo-data-sovereignty-check`:** Asserts that regional tenant queries strictly enforce regional database routing and block cross-border PII egress.
- **`database/multi-region-conflict-check`:** Verifies conflict resolution policies (CRDTs, vector clocks) on distributed multi-region tables to prevent split-brain data loss.
- **`security/pii-log-redaction`:** Scans logging statements (`console.log`, logger calls) for unredacted passwords, card numbers, Bearer tokens, and social security numbers.
- **`types/strict-interface-integrity`:** Rejects `any` proliferation in TypeScript; enforces schema validation (Zod/TypeBox) at application boundaries.
- **`deps/package-hallucination-audit`:** Verifies that every installed npm/pip package exists on the official public registry, preventing supply-chain slopsquatting.

### 2. Informing and Elevating the Vibe-Coder
Instead of giving vague complaints, TestKit teaches the builder in plain English with measurable consequences:
- *Slop:* AI puts sorting and heavy filtering directly inside a React component render.
  *TestKit Finding:* *"Sort runs on every render pass. Under 5,000 items, p99 UI freeze is 1.8s. Move sort into a database query or useMemo hook. Here is the exact fix."*
- *Slop:* AI writes 3 endpoints that call Stripe without an idempotency key.
  *TestKit Finding:* *"Write path missing idempotency key. A network retry will double-bill the user. Refactor endpoint to require `Idempotency-Key` header."*
- *Slop:* AI leaves Supabase tables open with `public` SELECT policies.
  *TestKit Finding:* *"RLS policy on `orders` grants SELECT to `anon`. Anyone on the internet can read customer records without signing in. Here is the hardened policy."*

### 3. Universal Handover Quality
The output of a TestKit-governed project is not disposable prototype spaghetti—it is clean, maintainable software with documented contracts, strict typing, passing invariant tests, and clear architectural separation that any engineering team can take over on Day 1.

---

## 8. Total Semantic Feature Discovery & The Combinatorial Scenario Engine

How does TestKit ensure that **every feature and state transition** is mapped and tested without writing tests manually or hallucinating coverage?

TestKit solves this through a three-stage pipeline: **Deterministic AST Extraction**, **State-Machine Synthesis**, and the **Mathematical Cross-Product Scenario Engine**.

```
  ┌────────────────────────────────────────────────────────────────────────┐
  │ 1. AST Feature Extraction (Tree-Sitter / Language Servers)             │
  │    • Discovers all routes, RPCs, views, forms, mutations, DB models    │
  └───────────────────────────────────┬────────────────────────────────────┘
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 2. Semantic Graph & State-Machine Synthesis                            │
  │    • Workflow: Open conversation → Type text → Tap send                │
  │    • Declared States: [Success, Offline, Timeout, RateLimited, 500]    │
  │    • Undeclared States: Flagged immediately as architectural blindspot │
  └───────────────────────────────────┬────────────────────────────────────┘
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 3. Combinatorial Cross-Product Expansion (The Mathematical Grid)       │
  │    [Every Workflow]  ×  [Every Actor Role]  ×  [Every Failure Mode]    │
  │    ×  [Boundary Inputs]  ×  [Bug-Pattern Taxonomy]                     │
  └───────────────────────────────────┬────────────────────────────────────┘
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 4. Honest Quantified Coverage Matrix                                   │
  │    • "84 total states discovered: 71 explored, 13 unreached"           │
  │    • Exact provenance for every test; zero hallucinated passes         │
  └────────────────────────────────────────────────────────────────────────┘
```

### 1. Extracting What the App Can Actually Do (Zero LLM Tokens)
TestKit's local engine parses the AST across any language (TypeScript, Python, Go, Rust, Swift, Kotlin):
- **Routes & Handlers:** Every GET/POST endpoint, server action, or screen route.
- **Form Schemas:** Every input field, validation rule (Zod, Pydantic, Marshmallow), and required parameter.
- **Database Schema & Relations:** Every table, enum, primary/foreign key, and RLS policy.
- **Client Components & Interactivity:** Every button, modal, form submit, and tab transition.

### 2. State-Machine Discovery (Declared vs. Undeclared States)
For each discovered workflow, TestKit constructs a state machine:
- **Happy Path:** Expected success response + UI change + DB write.
- **Expected Failure States:** What happens if the network drops? What if input is empty? What if the user is unauthenticated?
- **The "Undeclared State" Finding:** If an endpoint handles 200 OK and 400 Bad Request, but has no handling for a database timeout or 429 rate limit, TestKit immediately flags: *"Workflow `checkout` has no declared failure handling for Stripe timeout."*

### 3. The Combinatorial Cross-Product Expansion
To test every possible scenario without humans having to invent them, TestKit calculates the mathematical cross-product:
$$\text{Scenarios} = \text{Workflows} \times \text{Roles} \times \text{States} \times \text{Bug Patterns} \times \text{Boundary Inputs}$$

1. **Roles:** (Admin $\times$ Standard User $\times$ Other Tenant Member $\times$ Anonymous Visitor)
2. **Boundary Inputs:**
   - 4,000-character strings, emojis, RTL Arabic/Hebrew text, SQL injection strings, null bytes, negative numbers, floats where ints are expected.
3. **Bug-Pattern Taxonomy:**
   - Auth races, session fixation, token expiration mid-flow, double-click submissions, browser back-button resubmits, orphan data leaks, WebSocket reconnection storms.
4. **Execution & Honest Accounting:**
   - Every scenario compiles to a deterministic script.
   - TestKit reports **honest coverage**: *"84 states discovered in your code: 71 verified green, 3 invariant failures, 10 blocked by missing Stripe test keys."* It never claims "100% bug free"—it quantifies mathematically what was and was not proven.

### 4. High-Throughput Edge-Case Fuzzing: 1,000+ Scenarios in Minutes
How can a single developer's laptop execute 1,000+ complex edge-case scenarios in under 3 minutes without overheating or crashing?

- **The Semantic Speed Engine:** 
  - 95% of generated edge-case scenarios do **not** boot a full graphical browser or simulate human typing delays.
  - They execute as **compiled in-process HTTP/RPC/Actor calls** with pre-seeded authentication states, completing in **2 to 8 milliseconds per scenario**.
  - A batch of 1,000 edge-case mutations (e.g. boundary fuzzing, malformed payloads, role probes, idempotency races) executes concurrently in lightweight asynchronous Node worker pools.
- **Selective UI Confirmation:**
  - Only the 5% of critical journeys that require rendering verification (CSS layout, keyboard focus, DOM accessibility) boot headless Playwright/Maestro instances.
- **Automated Flake & Noise Suppression:**
  - If an edge case fails, TestKit automatically replays it 3 times against clean state snapshots.
  - If it fails 3/3 times $\rightarrow$ **Confirmed Bug**.
  - If it passes intermittently $\rightarrow$ flagged with a **Flake Score** and prioritized for race-condition analysis rather than blocking the developer with noise.

---

## 9. The Missing Edge-Case Armor: The Final 5 Pillars

To ensure 100% complete coverage where vibe-coded apps fail in the wild, TestKit includes the final five operational pillars:

### 1. Hermetic Third-Party Service Doubling & Webhook Replays (Preventing Double-Debit & Double-Credit)
- **The Problem:** Vibe-coders hook up Stripe, Twilio, Resend, or OAuth, and tests fail because they don't have active API keys or want to avoid incurring sandbox fees. Furthermore, webhook retries often cause fatal **double-debiting** (charging twice) or **double-crediting** (granting tokens twice).
- **TestKit Solution:** Automated stateful test doubles (WireMock / MSW). TestKit automatically simulates:
  - Rapid double-click payments and concurrent webhook delivery to verify idempotency keys prevent **double-debits** and **double-credits**.
  - Stripe subscription lifecycle webhooks (`customer.subscription.deleted`, `invoice.payment_failed`).
  - Webhook delivery delays, signature verification failures, and replay attacks.

### 2. Realistic Seed Data Volumes (Catching Scale Bugs Locally)
- **The Problem:** An app with 5 rows in SQLite feels fast. At 50,000 rows, pagination breaks, search freezes the server, and memory spikes.
- **TestKit Solution:** Local synthetic seed-data generator. Injects realistic volume distributions (e.g. 10,000 users, 50,000 orders, soft-deleted rows, Unicode names) into local Docker/Testcontainers databases to catch $O(N)$ query bottlenecks before production.

### 3. Time-Travel & Scheduled Cron Verification (Simulating Paid/Premium Features)
- **The Problem:** How do you test premium feature locks, subscription upgrades, 14-day free trials, usage quota resets, or scheduled billing crons without waiting real weeks?
- **TestKit Solution:** System clock virtualization & state injection:
  - **Simulate Premium & Paid Tiers:** Instantly toggle synthetic users between Free, Pro, and Enterprise states to verify feature gates (e.g. "Does a free user see the paywall? Does upgrading unlock premium features immediately?").
  - **Fast-Forward Time:** Fast-forwards system time by 1 hour, 7 days, 14 days, or 30 days in milliseconds:
    - Verifies trial expiration: Did the user downgrade gracefully when the trial lapsed?
    - Verifies quota resets: Did monthly API usage reset on the 1st of the month?
    - Verifies recurring billing: Did the cron charge the card exactly once (no double-billing)?
    - Confirms edge cases: Leap Years and Daylight Saving Time (DST) transitions.

### 4. Sandboxed Git Rollback & Safe Reversion Guardrails
- **The Problem:** A coding agent introduces a breaking change or database migration that corrupts state during autonomous attempts.
- **TestKit Solution:** Isolated Git worktree checkpoints:
  - *Automated CI / Headless Mode:* TestKit isolates test executions in a dedicated, ephemeral git worktree branch. If an agent's change breaks core P0 invariants and fails to self-heal within 2 iterations, TestKit resets the scratch worktree to the last verified green commit.
  - *Interactive Local Development:* TestKit generates an explicit reproduction receipt and advisory rollback patch, requiring user or policy confirmation before modifying any local git branches. TestKit never destructively touches a developer's working tree unprompted.

### 5. Tool & MCP-Agnostic CI/CD Merge Gate (The PR Enforcer)
- **The Problem:** Developers test locally, but colleagues or AI agents merge code directly into `main` without verification.
- **TestKit Solution:** Native CI/CD enforcement through open tools and MCPs:
  - Integrates seamlessly with **GitHub Actions**, **GitHub Agentic Workflows (`gh-aw`)**, GitLab CI, and the **GitHub MCP Server**.
  - Evaluates the signed cryptographic run receipt (`run_XXXX.json`).
  - Blocks PR merge if any P0 invariants fail, untested schema changes exist, or security vulnerabilities are detected.
  - Posts an automated, structured PR comment with replay reproduction commands and line-level proof.

---

## 10. The Unbounded Bug-Pattern Seed Library

### The Principle: No Ceiling, No Checklist

> **TestKit does NOT maintain a finite list of bugs to check against. It maintains an ever-growing library of *failure dimensions* that feed the combinatorial cross-product engine from Section 8.**

Any attempt to enumerate "all the bugs" would produce a list that is always incomplete — production systems fail in ways no one predicted. Instead, TestKit treats bug patterns as **seed dimensions**: starter fuel that multiplies across the app's own semantic graph to generate thousands of unique, context-specific scenarios dynamically.

$$\text{Scenarios} = \text{App Semantic Graph} \times \text{Seed Dimensions} \times \text{Roles} \times \text{States} \times \text{Boundary Inputs}$$

The named patterns below are **examples of seeds, not limits.** The engine is unbounded.

### How the Seed Library Works

```
  ┌────────────────────────────────────────────────────────────────────────┐
  │ 1. SEED DIMENSIONS (Starter Fuel)                                      │
  │    • Data integrity, auth chaos, async failures, file/storage,         │
  │      communication, concurrency, caching, payment, lifecycle ...       │
  │    • Each seed is a reusable failure-injection template                 │
  └───────────────────────────────────┬────────────────────────────────────┘
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 2. APP SEMANTIC GRAPH (from Section 8 AST Extraction)                  │
  │    • Every route, mutation, workflow, role, state machine discovered   │
  │    • TestKit knows YOUR app's specific surface area                     │
  └───────────────────────────────────┬────────────────────────────────────┘
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 3. COMBINATORIAL CROSS-PRODUCT EXPLOSION                               │
  │    • Each seed dimension × each discovered workflow × each role        │
  │      × each state × boundary inputs = 10,000+ unique scenarios        │
  │    • Scenarios are GENERATED, not hand-written or enumerated           │
  └───────────────────────────────────┬────────────────────────────────────┘
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 4. OPEN-ENDED DISCOVERY                                                │
  │    • Engine finds bugs that are NOT in the seed library                 │
  │    • Novel failures auto-classify into new seed dimensions             │
  │    • Library grows with every project, every community contribution    │
  └────────────────────────────────────────────────────────────────────────┘
```

### Example Seed Dimensions (Starter Set — Not Exhaustive)

These are illustrative categories that ship as default seeds. They are **not a ceiling**:

| Seed Dimension | What It Injects | Example |
|---|---|---|
| Data Integrity | Partial failures, orphan records, stale caches | 7 of 10 batch records succeed — is state consistent? |
| Auth & Session Chaos | Token expiry mid-flow, multi-tab identity drift, OAuth drops | Password changed on desktop — is mobile session revoked? |
| Async & Queue Failures | Poison-pill loops, out-of-order delivery, stuck locks | Malformed job crashes worker in infinite restart cycle |
| File & Storage | Upload interruption, missing asset fallbacks, path traversal | Connection drops at 90% of upload — are temp chunks cleaned? |
| Communication | Provider outages, expired tokens, replay attacks | Email API returns 500 — does user see a clear retry message? |
| Payment & Billing | Double-debit, double-credit, subscription lifecycle | Rapid double-click payment — does idempotency key prevent duplicate charge? |
| Concurrency & Race Conditions | Simultaneous edits, lock contention, thundering herds | Two users edit the same document within 50ms |
| Time & Scheduling | Clock skew, DST transitions, leap years, cron drift | Trial expires at midnight — does the feature gate activate cleanly? |
| Data Sovereignty & Multi-Region | Cross-border PII leakage, multi-master split-brain, geo-failover | User in Frankfurt routed to US database during failover — does residency violate GDPR? |

### The Library Is Never "Done"

1. **Auto-Discovery:** When the combinatorial engine finds a novel failure pattern that doesn't match any existing seed dimension, it **auto-classifies the pattern as a new seed** and adds it to the local library for future runs.

2. **Community-Contributed Seeds:** The seed library is a pluggable directory of failure-injection templates. Teams can contribute domain-specific seeds (e.g., healthcare HIPAA compliance patterns, fintech settlement edge cases, IoT device timeout patterns) that expand the engine's reach for specific verticals.

3. **No Upper Bound:** The engine's output is bounded only by the app's own semantic surface area multiplied by the seed dimensions. A simple CRUD app might generate 500 scenarios. A complex SaaS with 40 workflows, 5 roles, and 8 failure dimensions generates 10,000+ scenarios — all in minutes, all deterministic, all reproducible.

> **The goal is not to name every bug. The goal is to build an engine that can *find* every bug — including the ones nobody thought to name.**

---

## 11. The Production Reality Manifesto: What Vibe-Coding Actually Requires

A vibe-coder typing prompts into an AI coding agent creates the illusion of speed. But shipping software that survives real customers, real money, and real adversaries requires navigating over 100 discrete operational realities that AI coding agents naturally ignore.

**TestKit exists to embody this exact engineering discipline, automatically:**

### The Checklist of Real-World Software Truths

| Domain | What the AI Forgets | What TestKit Automatically Solves / Enforces |
|---|---|---|
| **Architecture & Database** | • Improvised table-by-table schemas<br>• Full-table scans on 5M rows<br>• Migrations that lock tables at 2 PM on Tuesday<br>• Floats used for money (`0.1 + 0.2 != 0.3`)<br>• Connection pool exhaustion during traffic spikes | • Day-0 schema design before code is written<br>• Performance index checks & $O(\log N)$ query auditing<br>• Zero-downtime expand-and-contract migration rehearsal<br>• Enforced integer/currency storage rules<br>• Sized connection pools & PgBouncer tuning |
| **Auth, Access & Enterprise** | • Broken logins on `user+tag@email.com` or apostrophes<br>• Missing session revocation when passwords change<br>• IDOR: User A reading User B's records via URL ID<br>• Inability to support SAML/SSO or SCIM offboarding<br>• No audit trail on "log in as user" admin features | • Internationalized RFC email & Unicode boundary testing<br>• Multi-device token revocation & session-expiry verification<br>• Automated 4-role authorization & tenant-isolation matrix<br>• External OIDC/SAML enterprise provider compatibility<br>• Cryptographic audit logging on all privileged actions |
| **Payments & Billing** | • Duplicate debits when a user clicks "Pay" twice<br>• Duplicate token grants on webhook retries<br>• Broken pagination on page 400<br>• Failure to prorate subscriptions or calculate local VAT<br>• Invoices missing PO numbers for enterprise procurement | • Idempotency key requirement on all mutation endpoints<br>• Stateful webhook deduplication and replay testing<br>• Limit-offset vs keyset cursor pagination validation<br>• Time-travel trial expiration & proration tests<br>• Flexible billing metadata contract assertions |
| **Security & Compliance** | • Hardcoded API keys in git history or client bundles<br>• Phantom packages installed from hallucinated repos<br>• Over-permissive CORS (`*`) and missing CSP headers<br>• Zero preparation for SOC 2, HIPAA BAA, or GDPR deletions<br>• Prompt injection in fields that reach LLMs | • Pre-commit secret scanning (above git)<br>• Lockfile verification & CVE dependency auditing<br>• Automated security headers & strict CORS checking<br>• PII redaction & GDPR cascade-delete verification<br>• Prompt injection fuzzing & LLM egress boundary guards |
| **Global Sovereignty & Multi-Region** | • PII routed to non-compliant geographical jurisdictions (GDPR/Schrems II)<br>• Split-brain data loss in multi-master regions<br>• Cascading geo-failover drops | • Region-pinned tenant routing & database residency checks<br>• CRDT / vector-clock multi-master conflict validation<br>• Geo-DNS & Anycast failover resilience testing |
| **DevOps & Infrastructure** | • "Works on my laptop" breaking in staging<br>• Friday afternoon deploys with zero rollback path<br>• Cron jobs running twice when a second server is added<br>• Unchecked cloud loops (\$1,382 runaway agent bills)<br>• SPF/DKIM/DMARC missing, sending password resets to spam | • Byte-identical local vs CI environment fidelity checks<br>• Rehearsed one-click rollback & blue/green traffic flips<br>• Distributed lock checks (`SETNX`) on background crons<br>• Bounded step & token budgets per actor run<br>• Automated DNS, email authentication & header verification |
| **Resilience & Chaos** | • Background workers crashing in infinite restart loops<br>• Server freezing when third-party APIs slow down<br>• 4,000 duplicate error alerts at 3 AM<br>• Backups taken that have never been tested for restore<br>• Memory leaks that crash servers after 48 hours | • Dead-letter queue (DLQ) & poison-pill injection testing<br>• Backpressure, circuit breakers & timeout verification<br>• Deduplicated root-cause issue triage<br>• Migration up AND rollback verification in sandboxes<br>• Heap profiling & continuous load-ramp saturation checks |
| **Code Health & Handover** | • 2,000-line God-files that no human can read<br>• Empty `catch(e) {}` blocks hiding real bugs<br>• Zero documentation for the next engineer<br>• The "bus factor of one" where only the AI knows the system | • Anti-slop mechanical file & complexity gates<br>• Elimination of silent failure stubs & fake greens<br>• Auto-maintained living semantic architecture map<br>• Handover-ready codebases with clean contracts & tests |

> **"A vibe-coder with TestKit is no longer just playing with a toy prompt—they are protected by the accumulated hard-won scars of a Senior Systems Architect, DevOps Engineer, and Security Auditor, running locally on their machine."**

---

## 12. Post-Launch Production Management: The Continuous Self-Healing Loop

TestKit does not stop when you click "Deploy". After your application is live with real users, TestKit transitions into an **always-on, read-only Production SRE Witness & Copilot**:

```
  ┌────────────────────────────────────────────────────────────────────────┐
  │ 1. READ-ONLY PRODUCTION OBSERVATION                                    │
  │    • Ingests live telemetry (Sentry, OpenTelemetry, Datadog, Logs)     │
  │    • NEVER sends synthetic chaos or test traffic into live production   │
  └───────────────────────────────────┬────────────────────────────────────┘
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 2. ANOMALY CAPTURE & LOCAL ISOLATION                                   │
  │    • Live user crash detected (e.g. "TypeError on Checkout at 3:14 AM") │
  │    • TestKit extracts the exact payload & state (PII redacted)         │
  │    • Automatically spins up a fresh local git worktree branch          │
  └───────────────────────────────────┬────────────────────────────────────┘
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 3. LOCAL DETERMINISTIC REPRODUCTION SCRIPT                             │
  │    • Converts the live production incident into a local test scenario  │
  │    • Replays locally in Docker sandbox: Proves bug is 100% reproducible│
  └───────────────────────────────────┬────────────────────────────────────┘
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 4. COPILOT GUIDANCE TO CODING AGENT                                    │
  │    • "Here is the exact line of failure, causal graph, and fix plan"   │
  │    • Coding agent (Claude Code / Cursor) applies the fix               │
  └───────────────────────────────────┬────────────────────────────────────┘
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 5. RE-VERIFY & EXPANDED REGRESSION TEST                                │
  │    • TestKit re-runs the reproduction script: Confirms bug is GONE      │
  │    • Runs the full 1,000+ invariant regression suite: Zero side effects│
  └───────────────────────────────────┬────────────────────────────────────┘
  ┌───────────────────────────────────▼────────────────────────────────────┐
  │ 6. ZERO-DOWNTIME RE-DEPLOYMENT & ONGOING SENTINEL                      │
  │    • Rehearses migration rollback & executes blue/green deployment     │
  │    • Incident test case permanently added to local regression suite    │
  │    • That bug can NEVER happen in production again                     │
  └────────────────────────────────────────────────────────────────────────┘
```

### The 4 Real-World Management Roles TestKit Plays in Production

1. **"Tell You What To Do" (Advisory Copilot Mode):**
   - When a production alert fires, TestKit analyzes the stack trace and explains it to the vibe-coder in plain language:
     - *"Customer #492 in Germany hit an unhandled currency exception on checkout because their locale uses commas instead of periods for decimals. Here is why it happened and the exact code patch to fix it."*
2. **"Do It & Test It Again" (Autonomous Fix Loop):**
   - Automatically passes the exact line-level proof to your coding agent.
   - The coding agent edits the code.
   - TestKit intercepts the change in a sandbox, runs the reproduction script to confirm the fix, and runs 1,000+ regression tests to ensure the fix didn't break two other features.
3. **Adaptive Deployment Strategy (Stack & Scale-Appropriate):**
   - TestKit is **never dogmatic about deployment methods**. It recommends and executes the exact deployment topology suited to your architecture:
     - **Major Cloud Providers (AWS, GCP, Azure):**
       - **AWS:** ECS / Fargate rolling updates, AWS Lambda container & zip deploys, App Runner, EC2 blue/green via ALB.
       - **GCP:** Cloud Run revisions with traffic splitting, GKE cluster rollouts, App Engine.
       - **Azure:** Azure Container Apps, App Service staging slots with atomic slot swap.
     - **Serverless & Edge Platforms:** Vercel, Cloudflare Pages/Workers, Netlify immutable preview builds promoted to production.
     - **Modern PaaS Services:** Render web services, Railway, Fly.io, Dokku, Coolify, or custom VPS via SSH/Docker.
     - **Deployment Topology Selection:** Chooses between atomic blue/green flips, percentage-based canary rollouts (5% → 25% → 100%), rolling updates, or direct revision switches based on your traffic volume and statefulness.
4. **Universal Telemetry, Log & Database Ingestion (Connect to Anything):**
   - TestKit connects natively to **whatever observability stack or data plane you run**:
     - **Managed APM & Error Trackers:** Sentry, Datadog, PostHog, Bugsnag, New Relic.
     - **Cloud & Host Log Streams:** AWS CloudWatch, GCP Cloud Logging, Vercel Log Streams, Render Log Drains, Railway Observability, Cloud Run logs.
     - **Live Database & Cache Telemetry:**
       - **Postgres / MySQL:** Monitors slow-query logs (`pg_stat_statements`), deadlocks, connection pool saturation, and unindexed tables.
       - **Managed DB Platforms:** Supabase, Neon, PlanetScale, AWS RDS / Aurora performance insights.
       - **Redis & Queues:** Memory fragmentation, key eviction rates, queue depth, and stuck jobs.
     - **Custom & On-Prem Ingestion:** Generic HTTP webhook receivers, Syslog drains, Grafana Loki, or OpenTelemetry (OTel) collectors.
   - This creates an **unbroken closed loop**: Live Logs / Crashes / Slow DB Queries → Local Reproduction Script → Agent Fixes Code → TestKit Re-tests Locally → Adaptive Re-deploy.
5. **Permanent Regression Immunization:**
   - The captured production bug is compiled into a permanent `.testkit/scenarios/` script committed to git.
   - That specific production bug is now an immutable gate—it is physically impossible for future AI iterations or code changes to reintroduce it.








