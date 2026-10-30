# TestKit Demo: one tiny app, all three scenario kinds

A minimal example showing how TestKit verifies a real app without any per-app
drivers. The "app" is two Node scripts (no dependencies):

- `tools/mock-server.mjs` — an HTTP API with a `/api/health` endpoint and a
  `/api/trial` endpoint that honors `X-Testkit-Fake-Now` (the time-travel
  contract: TestKit injects the header, the app must read it).
- `tools/trial-cli.js` — a CLI that reads `TESTKIT_FAKE_NOW` and prints
  `ACTIVE` or `EXPIRED` against a 14-day trial anchored at the injected now.

The scenarios in `scenarios/` are copied into a project's
`.testkit/scenarios/` directory:

| Scenario | Kind | What it proves |
| --- | --- | --- |
| `01-health.json` | `http` | Health endpoint answers 200 fast, and never leaks internal error strings |
| `02-trial-active.json` | `http` | The trial API serves an active trial at the anchor instant |
| `03-trial-expired.json` | `http` + `time_jump: 30d` | After a 30-day jump the API rejects with 403 `trial_expired` |
| `04-trial-cli.json` | `process` + `time_jump: 30d` | The CLI honors `TESTKIT_FAKE_NOW` and prints `EXPIRED` |
| `05-agent-checkout.json` | `agent` | The operator agent drove a checkout flow and notarized the transcript |

## Run it

```sh
testkit init --path ./demo          # or run inside a copy of this folder
cp scenarios/*.json .testkit/scenarios/

# Terminal 1: the app under test
node tools/mock-server.mjs

# Terminal 2: verify, ramping up
testkit verify --limit 2            # smoke: first two scenarios
testkit verify --tag INV-TRIAL      # one invariant family
testkit verify                      # everything
```

The `agent` scenario is different: before running `verify`, the operator agent
performs the flow described in its `brief` (drive the UI, exercise the API,
query the database — with whatever tools the environment has), writes the
declared evidence files, and then `verify` notarizes them into a tamper-evident
receipt. Missing evidence blocks the receipt instead of failing it: the flow
was never performed, which is a different fact from the app misbehaving.

## The incremental ramp

Start small and grow only when green — every step writes receipts, so progress
is measurable:

```sh
testkit verify --limit 10        # first 10 scenario files
testkit verify --limit 100       # first 100
testkit verify --limit 1000      # first 1k
testkit verify                   # full suite
```

Pair `--limit` with `--tag` to ramp one invariant family at a time, with
`--max-rss-mb / --max-cpu` to tighten the hardware envelope as you optimize,
and with `--fail-fast` while debugging. Scale bugs show up as receipts: ramp
the same scenarios under a bigger envelope and compare `duration_ms` and
`peak_ram_mb` across receipts — a scenario that jumps from 8ms to 4s between
the 100-row and 10k-row mark is an O(N²) you just caught with zero profilers.
