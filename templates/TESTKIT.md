# This repository is TestKit-governed

TestKit is the agent's testing toolkit and operating manual. You are the operator. Do not treat a build, a manual click-through, or a green unit suite as proof that this application works.

## Where things live

| Path | Purpose |
| --- | --- |
| `.testkit/brain/architecture.md` | The contract and the numbered HARD INVARIANTS. This is the oracle. |
| `.testkit/brain/learned-rules.json` | Durable lessons from confirmed failures. Append only. |
| `.testkit/scenarios/*.json` | Executable scenarios, one invariant per file. |
| `.testkit/evidence/` | Screenshots, traces, logs, telemetry. Never commit this directory. |
| `.testkit/receipts/run_*.json` | Append-only, SHA-256 integrity-verified proof of every run. |
| `~/.testkit/brain/global-knowledge.json` | Cross-project knowledge. Read it; do not hand-edit it. |

## Non-negotiables

1. Do not mark a task done, open a release PR, or commit a feature until `testkit gate` passes.
2. Every P0/HIGH invariant needs a current-build `PASSED` receipt whose signature verifies. No receipt means not proven.
3. Never edit, delete, move, re-sign, or regenerate a receipt. History is evidence.
4. Never weaken an invariant, lower a severity, or change an expected value to make a run pass.
5. Never bypass the gate. Only a human release owner may override, in writing, and the override never turns a failure into a pass.
6. Keep `.testkit/evidence/` out of git and out of shared chat; it may contain sensitive state.

## The commands

```bash
testkit init --path .
testkit map --path .              # --agent-brief prints your mapping instructions
testkit verify --path . --scenario <id>
testkit verify --path . --fail-fast
testkit gate --path .
testkit receipt list --path .
testkit receipt verify .testkit/receipts/run_<id>.json
```

`testkitd --project . --port 4949 --interval-ms 1000` runs the local daemon when a scenario needs live telemetry. Stop it when you are done.

## The semantic map: two witnesses

TestKit and you are collaborators, and neither of you grades your own work. The codebase gets **two independent floor plans**, and the comparison between them is where errors surface.

1. **Detect.** Run `testkit map`. The engine reads the dependency manifests itself (never guess the stack) and records `.testkit/brain/stack.json`.
2. **Machine map.** If `extractor-<framework>` exists in `.testkit/extractors/` or `~/.testkit/extractors/`, TestKit runs it and writes `map.machine.json`. If none exists for this stack, **you are the mapper** — TestKit works with any framework, language, or platform from day one.
3. **Your tour.** Run `testkit map --agent-brief` and follow it. Walk the codebase yourself — every route, handler, screen, command, model, button, and what connects to what — and write `.testkit/brain/map.agent.json` to the `testkit.map/1` schema. Rules: exact project-relative paths; every file you could not interpret goes in `coverage.unresolved` with a reason; never fabricate a node.
4. **Never touch the machine map.** Do not read `map.machine.json` while drawing yours, do not correct it, do not copy from it. Two witnesses that compare notes stop being independent, and the whole check collapses.
5. **Reconcile.** Re-run `testkit map`. Agreement between the two maps is proof. Every difference is a finding: doors only the machine found mean your tour must revisit them; doors only you found mean the extractor has a blind spot to fix (or a new invariant to declare).
6. **Feed the loop.** Use the reconciliation to extend coverage in `.testkit/scenarios/` — every agreed surface with no scenario watching it is a coverage gap. A map that never turns into scenarios is decoration.

## When the gate blocks

1. Read the blocking receipt IDs from `testkit gate --json`.
2. Verify the receipt is authentic: `testkit receipt verify <file>`.
3. Reproduce the breach: `testkit verify --scenario <id>`.
4. Run the failing scenario three times. Only 3/3 is a confirmed defect; record every run ID.
5. Fix the underlying breach in application code, migrations, or configuration.
6. Re-run the scenario and confirm a new signed `PASSED` receipt.
7. Re-run `testkit gate` on the exact commit you intend to ship.

Do not proceed to step 5 by editing the scenario to match current behavior, and do not proceed to step 7 with an open `FAILED` or `BLOCKED` receipt at P0/HIGH.

## Adding coverage

Add or extend a scenario in `.testkit/scenarios/` whenever you add a user-visible behavior, a money movement, a permission boundary, or a schedule. It must name an existing invariant; if none fits, ask the Architect to add one and sign the revision first. Use `time_jump` (`1h`, `7d`, `14d`, `30d`) for expiry and drift cases; TestKit injects `TESTKIT_FAKE_NOW` automatically.

Three scenario kinds exist for verification, plus one for load. `process` spawns any program (exit code, output markers, duration budget); `http` probes any endpoint — local or deployed, including Vercel/Render previews — (status, body markers, negated markers, timing budget), carrying the injected instant as `X-Testkit-Fake-Now`. For flows the engine cannot drive — UI screens, database state, platform-specific behavior — use `kind: "agent"`: declare the `brief` and the `evidence` files (path, `must_contain` markers, `min_bytes`). You perform the flow with your own tools, write the evidence exactly where the scenario says, and then run `verify`: the engine checks the artifacts and notarizes them into the receipt. Missing evidence blocks the receipt — it means the flow was never performed. Never fabricate evidence to unblock a receipt; that is tampering under rule 3. `load` runs N concurrent virtual users against one endpoint for a fixed window and gates on error rate and p95 latency; it measures the endpoint's answers, not the server's footprint — capture `docker stats` or the provider dashboard as agent evidence next to it.

Resource budget for test runs is the project ResourceEnvelope: 95% CPU and 1536 MB RSS on a 1 vCPU / 1.5 GB host — overridable per run with `testkit verify --max-cpu / --max-rss-mb` when the Architect declares a different target. A scenario that exceeds the envelope is a finding, not a reason to raise the limit.
