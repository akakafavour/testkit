# TestKit

**Full Lifecycle AI Software Intelligence.** Mechanical grounding, independent invariants, hardware sympathy, and tamper-evident receipts for software written by agents.

```
       [ Human Intent / Product Idea ]
                      │
                      ▼
            [ testkit init (< 1s) ] ───► Zero-prompt mechanical setup (.testkit/ + TESTKIT.md)
                      │
                      ▼
            [ Architect Agent ] ───────► Declares hard invariants & hardware budgets in architecture.md
                      │
                      ▼
            [ Coding Agent ] ──────────► Implements application features
                      │
                      ▼
            [ testkit verify ] ────────► Runs scenarios, probes CPU/RAM, records tamper-evident evidence
                      │
                      ▼
            [ Signed Receipts ] ───────► .testkit/receipts/run_receipt-*.json (SHA-256 integrity hash)
                      │
                      ▼
            [ testkit gate ] ──────────► Hard pre-commit/CI blocker: Zero open P0 bugs, zero slop
```

TestKit is the agent's testing toolkit and operating manual. **The agent is the operator**, not an autonomous QA system: it does not invent requirements out of thin air, chase hallucinated bugs, or edit your source to fake passing tests.

---

## The Mental Model: The Remotion Analogy

Remotion does not make videos for you. It hands you a programmable toolkit and primitives (`Composition`, `Sequence`, `useCurrentFrame()`) that turn a vague task into a mechanical one. You still direct the video. TestKit does the exact same for software verification:

| Remotion Primitives | TestKit Primitives | Role in Verification |
| --- | --- | --- |
| `Composition`, `Sequence` | Scenario files in `.testkit/scenarios/*.json` | Declarative, reproducible test definitions |
| `useCurrentFrame()` | System clock virtualization (`time_jump: 14d`) | Fast-forward trial expiry, crons, billing — TestKit injects `TESTKIT_FAKE_NOW`; the app must read it to see the jump |
| The Render Pipeline | `testkit verify` → `.testkit/receipts/` | Scenario execution & tamper-evident receipts |
| You direct the video | The Agent authors scenarios and drives the UI | The Mind directs; the Engine provides the substrate |

Three coordinated surfaces:
1. **The Native Rust CLI (`testkit`)**: On-demand, zero-daemon execution for scaffolding, scenario verification, whole-run telemetry sampling, and pre-commit gating.
2. **The 5 Agent Skills (`skills/`)**: The operating manual teaching any agent (`architect`, `verify`, `triage`, `replay`, `ship-gate`) how to think like a Principal Systems Engineer.
3. **The 4 Agent Personas (`agents/`)**: Specialized roles (`testkit-architect`, `testkit-tester`, `testkit-security`, `testkit-orchestrator`) that eliminate prompt bloat through strict separation of duties.

---

## Why Existing AI Code Fails (and How TestKit Fixes It)

1. **The Self-Grading Fallacy:** When the same AI writes both the code and the test, it tests what it *built*, completely missing its own blind spots. TestKit anchors all expectations **externally** in `architecture.md` invariants authored *before* code exists.
2. **The Cloud-Lazy Trap:** When an app gets slow, naive AI tells you to upgrade to a $200/mo AWS server. TestKit enforces **Hardware Sympathy**—it catches $O(N)$ full-table scans, memory leaks, and unbatched disk syncs right on your local machine.
3. **Context Overload & Skill Dilution:** Dumping 10 skills into one prompt causes hallucinations and instruction conflicts. TestKit enforces isolated agent-to-skill binding where each specialist loads only its assigned task.
4. **Source Tampering:** Standard agents often modify test assertions to force a failing test to turn green. TestKit enforces an iron rule: **TestKit never edits an invariant to make a test pass.** The agent must fix the application source code.

---

## Algorithmic Discipline: Engine vs. Operator

TestKit separates what the **engine mechanically checks** from what the **agent is trained to enforce**. The Rust binary never claims to reason about your algorithms; it measures, records, and gates. The reasoning lives in `skills/` and `agents/`, which train the operator agent to apply computer-science fundamentals before and after each run:

**The engine checks mechanically (every run, no LLM involved):**
- Process exit codes, stdout/stderr expectations, and durations.
- HTTP status codes and response-body expectations.
- Whole-run CPU/RAM telemetry against the declared resource envelope, with runaway-CPU-loop and monotonic-memory-leak detection.
- Receipt integrity: any edit to a receipt invalidates its SHA-256 hash.

**The operating manual trains the agent to enforce (via `skills/`):**
- **B-Trees & LSM-Trees:** Flag $O(N)$ sequential table scans on disk; demand $O(\log N)$ indexed query plans.
- **Prefix Tries & Inverted Indexes:** Enforce optimal prefix trees for search and autocomplete instead of linear array scans.
- **Topological Sorting:** Verify async worker dependency DAGs to eliminate deadlocks and race conditions.
- **Hardware Latency Hierarchy:** Assert the physical realities of RAM access (~100ns) vs. SSD I/O (~100µs) vs. Network (~150ms).
- **Adaptive Resource Ramps:** No rigid defaults. The agent can ramp testing budgets (e.g. 200MB → 500MB → 1GB RAM) across runs — or override the envelope per run with `verify --max-cpu / --max-rss-mb` — to discover exact breaking points and memory creep under load.

---

## Repository Architecture

```text
testkit/
├── Cargo.toml                      # Clean Rust workspace config
├── package.json                    # OpenAgent / npm plugin distribution manifest
├── plugin.json                     # Agent Plugins 1.0 standard manifest
├── hooks/
│   └── hooks.json                  # Lifecycle interception hooks (pre-commit gate, session exit check)
├── agents/                         # Persona-driven system definitions
│   ├── testkit-architect.md        # System design & hard invariant author
│   ├── testkit-tester.md           # Scenario executor & evidence gatherer
│   ├── testkit-security.md         # Red-teaming & auth boundary auditor
│   └── testkit-orchestrator.md     # Phase coordinator & gatekeeper
├── skills/                         # AgentSkills 1.0 specifications
│   ├── architect/SKILL.md          # Invariant definition & .testkit/brain/ rules
│   ├── verify/SKILL.md             # Invariant execution & progressive ramps
│   ├── triage/SKILL.md             # Root-cause clustering & agent dispatch
│   ├── replay/SKILL.md             # 3/3 deterministic reproduction & delta checks
│   └── ship-gate/SKILL.md          # Receipt validation & PR/Deploy blocker
├── templates/
│   └── TESTKIT.md                  # Standard manual dropped into repos on `testkit init`
├── examples/
│   └── demo/                       # Runnable demo: all three scenario kinds + the ramp
└── crates/
    ├── testkit-cli/                # Native binary CLI (init, verify, gate, receipt)
    ├── testkit-core/               # Engine core (telemetry, time-travel, receipts, evidence)
    └── testkit-daemon/             # Optional ambient background observer
```

---

## Installation & Setup

Requirements: Rust **1.77+**, `git`, Node.js **18+** (for plugin integration).

### Build from Source
```sh
cargo build --release
```

Outputs native binaries in `target/release/testkit` (`.exe` on Windows).

Install directly to your `PATH`:
```sh
cargo install --path crates/testkit-cli
# Or link via npm for agent plugins:
npm run install:local
```

---

## CLI Reference

TestKit runs as an on-demand, standalone command-line suite (zero background daemon required):

### 1. `testkit init`
Provisions the workspace in <1s with zero interactive prompts (identical to `git init`).
```sh
testkit init
# Or initialize a specific directory:
testkit init --path ./my-project
```
Creates `.testkit/` (`brain/`, `evidence/`, `receipts/`, `scenarios/`), configures `.gitignore`, and drops `TESTKIT.md`.

### 2. `testkit verify`
Executes scenarios against declared invariants and profiles host hardware usage (CPU/RAM) for the whole run.
```sh
# Run the entire scenario suite
testkit verify

# Filter by scenario ID or invariant tag
testkit verify --scenario trial-expiry-blocks-api
testkit verify --tag INV-012

# Override the hardware envelope for this run
testkit verify --max-rss-mb 256 --max-cpu 80

# Incremental ramp: run the first N scenario files
testkit verify --limit 10
testkit verify --limit 1000

# Stop at the first failing scenario
testkit verify --fail-fast
```

## Scenario Kinds

Three kinds cover any app without per-app drivers. The agent is the universal
adapter: it drives what the engine cannot, and the engine notarizes the proof.

**`process`** — spawn any program (CLI, worker, test binary, migration script)
with `TESTKIT_FAKE_NOW` injected:

```json
{
  "id": "trial-cli-honors-fake-now",
  "invariant": "INV-TRIAL-002",
  "kind": "process",
  "severity": "HIGH",
  "time_jump": "30d",
  "process": {
    "program": "node",
    "args": ["tools/trial-cli.js"],
    "expected_exit_code": 1,
    "stdout_contains": ["EXPIRED"],
    "max_duration_ms": 5000
  }
}
```

**`http`** — probe any API or web app (status, body markers, negated markers,
timing budget), with the injected instant sent as `X-Testkit-Fake-Now`:

```json
{
  "id": "trial-expiry-blocks-api",
  "invariant": "INV-TRIAL-002",
  "kind": "http",
  "severity": "P0",
  "time_jump": "30d",
  "http": {
    "url": "http://127.0.0.1:8931/api/trial",
    "expected_status": [403],
    "expected_body_contains": ["trial_expired"],
    "expected_body_not_contains": ["stack"],
    "max_duration_ms": 2000
  }
}
```

**`agent`** — the operator agent performs the flow with its own tools (browser
automation for UI, `psql` for database state, shell for files) and writes the
declared evidence artifacts; `verify` then checks them and notarizes them into
the receipt. Missing evidence blocks the receipt rather than failing it — the
flow was never performed, which is a different fact from the app misbehaving:

```json
{
  "id": "agent-checkout-flow-notarized",
  "invariant": "INV-TRIAL-003",
  "kind": "agent",
  "severity": "HIGH",
  "agent": {
    "brief": "Drive the checkout flow end to end and write the transcript to .testkit/scratch/checkout-transcript.md.",
    "evidence": [
      {
        "path": ".testkit/scratch/checkout-transcript.md",
        "must_contain": ["ORDER CONFIRMED"],
        "min_bytes": 200
      }
    ]
  }
}
```

**`load`** — N concurrent virtual users hammer one endpoint for a fixed
window; the gate is the error rate and the tail latency. This is how you find
the breaking point and *keep it as an invariant*: once p95 holds at 20
concurrent users today, the scenario blocks any change that makes it 4
seconds tomorrow.

```json
{
  "id": "trial-api-holds-20-users",
  "invariant": "INV-PERF-001",
  "kind": "load",
  "severity": "HIGH",
  "load": {
    "url": "http://127.0.0.1:8931/api/trial",
    "concurrency": 20,
    "duration_seconds": 30,
    "max_error_rate_percent": 1.0,
    "max_p95_ms": 500
  }
}
```

The generator measures the endpoint's *answers*, not its host's resources:
the server-side footprint (RAM, CPU) comes from the platform the app runs on
(`docker stats`, the provider dashboard) captured as `agent` evidence next to
the load receipt.

## Per-service budgets

Real apps have parts with different homes — a Next.js frontend on Vercel, a
Go backend on a $6 VPS, Supabase Postgres behind both. Each scenario can
declare its own `envelope`, so every part is tracked against the plan it
actually runs on, not one blanket number:

```json
{
  "id": "backend-uses-vps-budget",
  "invariant": "INV-PERF-002",
  "kind": "process",
  "envelope": { "max_cpu_percent": 95.0, "max_rss_mb": 512 },
  "process": { "program": "./server", "args": ["--port", "8080"] }
}
```

Run-level flags (`--max-cpu`, `--max-rss-mb`) still apply as the fallback for
everything that doesn't override. Built-in presets exist in `telemetry.rs`:
`edge_serverless` (128 MB), `render_free` (512 MB), `vercel_function`
(1 GB), the $12-server default (1.5 GB), and `high_throughput` (8 GB) —
always verify the number against the provider's current plan page.

## Testing against your real hosts

Scenarios hit URLs, so your deployed app is already a target — no code needed:

- **Vercel / Render / Fly previews:** point the scenario `url` at
  `https://my-app.vercel.app/api/trial`. The `X-Testkit-Fake-Now` header
  travels with the request; your deployed app honors the same time-travel
  contract the local one does.
- **Supabase / hosted Postgres:** `http` scenarios against PostgREST/edge
  functions, or `process` scenarios running `psql` assertions against the
  connection string from your env.
- **A VPS you own:** `process` scenarios whose program is `ssh` —
  `{"program": "ssh", "args": ["my-vps", "pg_dump -- ..."]}` — runs the
  command remotely; agent evidence captures the remote's `docker stats` and
  logs, since the engine's telemetry watcher only sees local processes.
- **Your laptop can't handle it?** Run the verify pass inside GitHub
  Codespaces or on the VPS itself; receipts compare across environments
  because the telemetry and evidence travel inside the receipt.

The honest boundary: when the target is remote, TestKit verifies *behavior*
(status, body, latency as seen from here) and notarizes evidence you supply
about *footprint* — it cannot read the remote host's RAM from outside.

## Advisory, not dogma

TestKit's gates are quiet until your receipts say otherwise. A vibecoder with
5,000 users on a $12 VPS does not need blue-green deployment, sharding, or
Kafka — and TestKit will never block a merge for *not having* them. The
escalation is evidence-driven:

1. Your load receipts hold at your current envelope: nothing fires, nothing
   is suggested. The system is green and stays boring.
2. The same receipts start showing `RAM_CEILING` violations, p95 creep, or
   error-rate budget breaches at unchanged traffic: the gate blocks, and
   *that* is the moment the advisory tier speaks — "your node is saturated at
   4,900 users; here is the cheapest next step," which is usually a cache or
   an index, not a second server.
3. Only when receipts prove a single node cannot hold the traffic does the
   advice become "add a component" — and the added complexity itself becomes
   the next set of invariants to gate.

A runnable version of the process/http/agent/load kinds lives in
[`examples/demo/`](examples/demo/README.md), together with the incremental
ramp workflow (`--limit 10` → `--limit 100` → full suite) used to pretest an
app scale-first.

### 3. `testkit receipt`
Inspects, parses, and validates receipts.
```sh
# List every receipt in the ledger with signature status
testkit receipt list

# Show the most recent receipt and verify its integrity
testkit receipt latest

# Verify a specific receipt file's SHA-256 integrity hash
testkit receipt verify .testkit/receipts/run_receipt-2026-10-29-0001.json
```

### 4. `testkit gate`
Evaluates receipts to enforce quality gates in pre-commit hooks and CI/CD pipelines.
```sh
# Standard gate: exits with 1 if any open P0/HIGH failures exist, 0 if clean
testkit gate

# Machine-readable JSON verdict
testkit gate --json

# Quiet mode for shell hooks (pre-commit, CI)
testkit gate --quiet

# Advisory/warning mode (prints open issues without blocking exit code)
testkit gate --warn-only
```

---

## Tamper-Evident Receipts

A receipt is an immutable, canonical JSON document carrying a SHA-256 evidence hash (over the whole `.testkit/evidence/` tree) and a SHA-256 integrity hash over its own canonical payload. **This is an integrity hash, not an asymmetric signature**: there is no signing key, so a receipt proves it has not been modified since it was written — editing a result or timestamp invalidates verification immediately — but it does not by itself prove who wrote it. Receipts live in `.testkit/receipts/run_receipt-YYYY-MM-DD-NNNN.json`:

```json
{
  "id": "receipt-2026-10-29-0001",
  "status": "PASSED",
  "severity": "LOW",
  "timestamp": "2026-10-29T17:16:44.812345600Z",
  "duration_ms": 812,
  "hardware_metrics": {
    "peak_cpu_percent": 37.5,
    "peak_ram_mb": 214
  },
  "invariants_checked": [
    { "name": "INV-012", "passed": true, "detail": "API rejected the expired trial with 403" }
  ],
  "violations": [],
  "evidence_hash": "8f1c9e2a4b7d6c5e8a1b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f",
  "signature": "2ab7c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2"
}
```

- **`PASSED`**: All invariant expectations held.
- **`FAILED`**: An expectation was violated (crime scene captured).
- **`NOT_PROVEN`**: Claim could not be established (unparsable result, timeout).
- **`BLOCKED`**: Precondition failed (missing dependency, unreachable service, hardware envelope breached).

---

## The Two-Tier Brain

TestKit maintains a clean separation between local project truth and cross-project developer wisdom:

| Tier | Location | Scope & Purpose | Git Status |
| --- | --- | --- | --- |
| **Project Brain** | `.testkit/brain/architecture.md` | Invariants, API contracts, and domain rules for THIS repo. | **Tracked** |
| **Project Learned**| `.testkit/brain/learned-rules.json` | Project-specific patterns distilled from past receipts. | **Tracked** |
| **Project Evidence**| `.testkit/evidence/` | Raw screenshots, network traces, CPU/RAM profiles. | **Ignored** (`.gitignore`) |
| **Global Brain** | `~/.testkit/brain/global-knowledge.json` | Cross-project engineering wisdom (anti-patterns, memory leak lessons). | **Local to Machine** |

---

## OpenAgent / Plugin Integration

TestKit natively supports the **Agent Plugins 1.0** and **Agent Skills 1.0** specifications, seamlessly integrating with **Claude Code, OpenCode, Cursor, and Antigravity**.

`hooks/hooks.json` automatically enforces lifecycle discipline:
- **`PreToolUse` (Bash)**: Calls `testkit gate --quiet` to prevent shell commands (including commits) if unresolved P0/HIGH receipts exist.
- **`Stop`**: Calls `testkit gate --warn-only --quiet` to warn if sessions conclude with active test failures.

Statuses are 4-way and deterministic: `PASSED`, `FAILED`, `NOT_PROVEN`, `BLOCKED`. The gate blocks on `FAILED` or `BLOCKED` receipts at `P0`/`HIGH` severity, and on any receipt whose integrity hash fails to verify.

---

## License
MIT. See `Cargo.toml` for workspace metadata.
