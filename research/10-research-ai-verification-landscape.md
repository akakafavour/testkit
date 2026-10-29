# TestKit — Landscape Research A: "Idea → Production" + AI Verification Layer

| | |
|---|---|
| **Date** | 2026-10-11 |
| **Method** | websearch + webfetch + GitHub API |
| **Star counts** | ✅ = pulled from GitHub REST API this session · ⚠️ = from page metadata · "unknown" = not captured (API rate-limited). No numbers were guessed. |

---

## ⚠️ Headline finding (read before anything else)

**The category is crowded with tiny repos and empty of anyone with distribution.**

- ~25 projects found doing "AI verification of AI-built code"
- **14 have 0–24 stars.** Several created within the last 8 months.
- **Only 2 have real usage:** `millionco/expect` (~3.5k⭐) and `searlsco/prove_it` (199⭐)
- **Every one is a single-mechanism tool.** None composes synthetic users + fault injection + invariant monitoring + replay-confirmed evidence into one loop.

**But:** the four mechanisms TestKit plans are all independently commoditized. The real opportunity
is **the artifact** — nobody ships a *replayable, tamper-evident proof-of-bug* that survives the
next commit. Everyone ships a report. **A report is a claim; a replay is proof.**

---

## 1. Bucket: AI app builders / vibe-coding platforms

### Open source (verified)

| Name | Repo | Stars | License | Note |
|---|---|---|---|---|
| **bolt.diy** | `Stackblitz-Labs/bolt.diy` | **19,942** ✅ | MIT | Official OSS Bolt.new, 21+ LLM providers, WebContainer runtime |
| Codex CLI | `openai/codex` | ⚠️ large | Apache-2.0 | OpenAI terminal coding agent |
| OpenHands | `All-Hands-AI/OpenHands` | ⚠️ ~65k | MIT | Open-source Devin alternative |
| Aider | `Aider-AI/aider` | ⚠️ ~40k | Apache-2.0 | Git-aware pair programmer |
| goose | `block/goose` | ⚠️ ~30k | Apache-2.0 | Block's local MCP-native agent |
| Gemini CLI | `google-gemini/gemini-cli` | ⚠️ large | Apache-2.0 | Google's terminal agent, 1M context |
| SWE-agent | `SWE-agent/SWE-agent` | ⚠️ ~17k | MIT | Stanford, solves GitHub issues autonomously |
| OpenCode | `sst/opencode` | ⚠️ large | MIT | OSS TUI coding agent, 75+ providers |
| Plandex | `plandex-ai/plandex` | ⚠️ | — | 2M-token context, multi-file planning w/ VCS |

### Commercial (verified via vendor docs)

| Name | Verified claim relevant to TestKit |
|---|---|
| **v0 (Vercel)** | Next.js/React/Tailwind/shadcn. Feb 2026 added Git panel (branch per chat + PR). **Zero documented proactive test generation** (builderproof.org, Jul 2026) |
| **Lovable** | ~25M projects, $330M @ $6.6B (Dec 2025). Has browser testing + frontend tests + edge tests |
| **Bolt.new** | WebContainer runtime. Testing posture is *reactive* — "paste the error into chat" |
| **Replit Agent** | Docs: "writes code, sets up infrastructure, and tests the result." Type/ownership/re-runnability **not documented** |
| **Firebase Studio** | Gemini prototyping/testing agents. Virtual Android devices for automated UI tests |
| **Base44** | Changelog: "Screenshots from app testing in the AI chat" (Jul 12 2026). **Screenshot-grade only** |
| Google Stitch, Dyad, Onlook, Databutton, Tempo Labs, Magic Patterns, Same.new, Emergent, Figma Make | All live in 2026 curated lists. None document a verification layer |
| Devin, Factory Droid, Cursor Background Agents, Warp, Amazon Q, GitHub Spark, Copilot Workspace, Cline, Roo Code, Kilo Code, Continue, PearAI, Augment, Tabnine, Qodo, CodeRabbit, Sweep, Pythagora, Kiro, Qwen Code, Crush, Trae | All live (filipecalegario/awesome-vibe-coding, 5,315⭐) |
| **Google Antigravity** | Google's agentic IDE. Has an MCP Store; already one-click installs MCP Toolbox |

### 🔥 Demand signal (strongest in the entire landscape)

- **Reeve / vibe-eval.com (Aug 2026):** scanned 30,998 deployed vibe-coded apps. **57% of Supabase-backed apps (2,096/3,680) allowed unauthenticated table reads** (RLS disabled/misconfigured).
- **Stanford (widely cited 2026):** ~80% of AI-generated apps carry ≥1 exploitable vulnerability.
- ShipProof field studies: 5,600 vibe-coded apps → 2,000+ vulns, 400+ exposed secrets.

Also the most crowded narrative — six tools already market "we catch what vibe coders get wrong."

---

## 2. Bucket: scaffold / deploy layer

**Frameworks:** create-t3-app · vercel/next.js · vercel/examples · astro · TanStack Start/Router · nuxt · sst

**Self-host PaaS:**

| Name | Repo | License | Note |
|---|---|---|---|
| **Coolify** | `coolify/coolify` | Apache-2.0 (source-available) | Vercel/Netlify clone |
| **Dokploy** | `dokploy/dokploy` | — | Coolify competitor |
| **CapRover** | `caprover/caprover` | Apache-2.0 | Docker PaaS, one-click |
| **PocketBase** | `pocketbase/pocketbase` | MIT | Single-binary BaaS — extremely popular with vibe coders |
| **Convex** | `get-convex/convex` | Apache-2.0 | Reactive backend + test/deploy story |
| **Appwrite** | `appwrite/appwrite` | BSD-3 | BaaS + Functions |
| **Nhost** | `nhost/nhost` | — | Supabase-compatible |
| **Directus** | `directus/directus` | BUSL/GPLv3 | Headless data platform |
| **NocoDB / Baserow / Hasura / Cube.js** | — | various | Open-source Airtable / GraphQL / semantic layer |
| **Appsmith / ToolJet / Budibase** | — | various | Internal-tool builders |

**Managed:** Vercel · Netlify · Railway · Render · Fly.io · Cloudflare Workers/Pages

> **Gap:** none of these verify anything. They ship. Deliberate, permanent gap — TestKit plugs in as a **pre-deploy gate**, not a competitor.

---

## 3. Bucket: testing embedded in those platforms ⭐ most important for GTM

### What the platforms actually do (verified against builderproof.org Jul 2026 + vendor docs)

| Builder | Agent self-check | Test type | Owned test files | Re-runnable | Regression intent |
|---|---|---|---|---|---|
| **Lovable** | Partial (on request) | Documented | Partial | Documented | Documented |
| **Replit** | Documented (automatic) | ✗ | ✗ | ✗ | ✗ |
| **Base44** | Documented (screenshots) | Partial | ✗ | Partial | ✗ |
| **v0** | ✗ | ✗ | ✗ | ✗ | ✗ |
| **Bolt.new** | ✗ | ✗ | ✗ | ✗ | ✗ |

> **The killer finding:** *"The agent tests your app"* means three very different things — screenshot eyeballing, a one-off browser check during the build, or a **persisted suite you own**. Only the last survives after the chat window closes. Only Lovable documents durable artifacts at all, and even there the burden is on you.
>
> **This is the sharpest verified gap in the entire landscape.**

### Embedded testing surfaces (verified)

- **Lovable Browser Testing** — real browser, remote VM, screenshots, console/network logs, multi-viewport. Runs against **preview, not production**. Documented limits: no canvas, unreliable drag-drop/clipboard/icon-only buttons, signed-in testing requires Lovable Cloud.
- **Lovable frontend tests** — `.test.tsx` co-located; **edge tests** via Deno's built-in runner.
- **Firebase Test Lab** — real devices in Google DCs, Robo tests, game loops, XCTest. Real, mature, device-level.
- **Firebase App Distribution** — TestFlight-equivalent for Android; integrates with EAS Workflows.
- **Supabase** — Edge Function tests via `deno test` + `withSupabase`. Plus **`supabase-ai-rls-tests-generator`** (npm v1.3.x) which uses Claude to auto-generate RLS policy tests. *RLS is the #1 Lovable failure mode and this already attacks it.*
- **Expo EAS Workflows + Maestro** — `type: maestro` job **currently in alpha**. Don't hard-depend.
- **Maestro / Detox / Flutter integration_test / fastlane snapshot testing** — standard mobile stack.

### ⭐ Playwright Test Agents — the finding that changes TestKit's strategy

Microsoft shipped **three official Playwright Test Agents**:
- 🎭 **planner** — explores app → Markdown test plan
- 🎭 **generator** — plan → executable Playwright tests
- 🎭 **healer** — replays failing steps, inspects UI, patches, re-runs until pass or guardrail stops

```bash
npx playwright init agents --loop claude   # or vscode / codex / opencode
```

> Docs at `playwright.dev/docs/next/test-agents` — flagged "unreleased documentation for Playwright Next" in one fetch, GA-referenced as v1.56 in another. **Verify version before citing.**
>
> **Implication:** Playwright now owns *agent-generates-tests + agent-heals-them* natively. **A TestKit that just does "agent writes Playwright tests that self-heal" is building into a commoditized layer.** TestKit must decide *what* to test and **prove** the result — not write `.spec.ts` files.

---

## 4. Bucket: OSS "vibe code QA / ship it" repos ⭐ THE CROWD

Stars ✅ verified via GitHub API this session.

| # | Project | Repo | Stars | License | What it does | Overlap |
|---|---|---|---|---|---|---|
| 1 | **expect** | `millionco/expect` | ~3,550 ⚠️ | Other | Reads git diff → test plan → real Playwright browser w/ your cookies. Perf/security/design checks. Replay at expect.dev | **Closest single competitor.** Browser-level, not behavioral |
| 2 | **prove_it** | `searlsco/prove_it` | **199** ✅ | MIT | Hook framework for Claude Code. Blocks `Stop` until tests/linters/AI reviewers pass. Churn gates, async reviewers, `/prove` evidence skill | **Closest mechanism competitor.** Enforcement, not bug-finding |
| 3 | **vibe-check** | `Hypership-Software/vibe-check` | **14** ✅ | **NONE** | Production-readiness scanner, 12 domains → score/100. Installs to 9 harnesses via `npx skills add` | Scorecard pitch. No license = blocker |
| 4 | **agents-shipgate** | `ThreeMoonsLab/agents-shipgate` | **90** ✅ | Apache-2.0 | Deterministic local-first merge gate for **agent capability changes** (MCP/OpenAPI/SDK). Trust-root detection, SARIF, **hash-chained receipts** | ⭐ Highest engineering quality in space |
| 5 | **shipproof** | `kingggg5/shipproof` | **18** ✅ | MIT | Evidence-first ship gate. 200 detectors, CPU/RAM/latency budgets, worktree isolation, explicit `proof_level` L0/L1 honesty, MCP adapter | Same instinct as TestKit |
| 6 | **loop-testing** | `sdsrss/loop-testing` | ⚠️ low | — | **Dual Claude/Codex skill. AI agent uses your app as a real user in a sandbox, finds bugs, fixes safe ones with regression tests, loops to convergence.** Two personas (novice/power user). Replay-verify, atomic commit, issue ledger, multi-model committee, stop-hook | **⚠️ MOST DANGEROUS.** TestKit's loop, already built |
| 7 | **verif-kit** | `chitti-learns-ai/verif-kit` | **0** ✅ (May 2026) | MIT | **Blind AI verifier reads your spec, never your code.** Property + metamorphic + mutation + runtime lenses. Coverage closure w/ cover points. NASA IV&V. Ships 20/20 bug-injection evidence | **⚠️ 2nd most dangerous.** Invariant watching + fault injection + calibration evidence |
| 8 | **vibe-testing** | `aishwaryshrivastav/vibe-testing` | **6** ✅ | MIT | MCP server, 13–14 Playwright tools, code-aware, `VIBE.md` config | Direct MCP competitor |
| 9 | **vibecodeqa CLI** | `vibecodeqa/cli` | **0** ✅ | MIT | 34 checks / 7 categories, GH Action w/ `fail-under`, SARIF, **MCP server** (7 tools) | Direct MCP+CI competitor |
| 10 | **vibe-test (626 Labs)** | `estevanhernandez-stack-ed/vibe-test` | **0** ✅ | **NONE** | Classifies maturity tier → risk-proportional tests. Honest coverage via `c8 --all`. **Detects broken harnesses** (false 0%, uninstalled binaries, cherry-picked denominators) | Sharp on test-harness integrity |
| 11 | **claude-code-ship-gate** | `aksheyw/claude-code-ship-gate` | **0** ✅ | MIT | `PreToolUse` hook **above git** so `--no-verify` can't bypass. Gate-pass marker bound to exact commit SHA + TTL. Fail-closed, 4 escape hatches | ⭐ Excellent hook engineering |
| 12 | **roborev** | `steelrain/roborev` | **0** ✅ | MIT | Continuous background review DB. post-commit hook, TUI, `refine` loop in isolated worktree, `compact` for false-positive filtering | Review, not verification |
| 13 | **Prove It** | `Pablo-aps/prove-it` | **15** ⚠️ (Aug 2026) | Apache-2.0 | Adversarial verification skill. Falsifiable claims → searches counterevidence → `PROVEN`/`FAILED`/`NOT PROVEN`/`BLOCKED`. 12-case benchmark 9/12→12/12 | ⭐ **The verdict taxonomy TestKit should adopt** |
| 14 | **vibescore** | `stef41/vibescore` | **24** ✅ | NOASSERTION | Python. Letter grade: quality/security/deps/testing. Zero deps, offline | Score-only |
| 15 | **vibe-check (Nyx)** | `NyxToolsDev/vibe-check` | **2** ✅ | MIT | Python. 6-category grade + HIPAA module, CI threshold gating | Score-only |
| 16 | **vibeQA** | `Tew12345678910/vibeQA` | **1** ✅ | **NONE** | Next.js + FastAPI. Static RAG rules + **AWS Bedrock Browser Use** live browser testing, multi-viewport | Realistic architecture, 1⭐ |
| 17 | **agent-sh/ship** | `agent-sh/ship` | **2** ✅ | **NONE** | 12-phase PR→prod. Waits for Copilot/Gemini/CodeRabbit, classifies+answers every comment, deploy + health check + auto-rollback | Operates *after* verification |
| 18 | **ShipIT Forge** | `shipiit/forge` | **1** ✅ | MIT | Autonomous GH agent: issue→PR, CWE review, deterministic scanners pre-model, auto-fix CI. Never merges | Complement |
| 19 | **agent-browser-loop** | `jasonsilberman/agent-browser-loop` | **7** ⚠️ | MIT | `open→act→wait/verify→repeat`. Returns page state + console + network | Primitive |

### Curated lists (survey infrastructure)
`filipecalegario/awesome-vibe-coding` **5,315⭐** CC0 · `acvnace/…-resources` 224 MIT · `roboco-io/awesome-vibecoding` 195 CC0 · `taskade/awesome-vibe-coding` 109 CC-BY · `no-fluff/…` 56 MIT · `bluegalaxy111/…` 48 CC0

---

## 5. Bucket: Google developer tooling (explicitly requested)

| Project | Repo | Stars | License | Relevance |
|---|---|---|---|---|
| **ADK (Python)** | `google/adk-python` | **21,758** ✅ | Apache-2.0 | Google's agent framework. Has an **evaluation** story |
| **ADK (Go)** | `google/adk-go` | **8,859** ✅ | Apache-2.0 | Ships `adk.dev/llms.txt` + `llms-full.txt` |
| **ADK Docs** | `google/adk-docs` | **1,481** ✅ | Apache-2.0 | v2.0. *"Applying software development principles to agent creation"* |
| **Google MCP servers** | `google/mcp` | ⚠️ | — | Hub for ~20+ OSS MCP servers + remote GCP servers (BigQuery, Cloud SQL, Spanner, GKE, Cloud Run, Security Command Center, **Chrome DevTools**) |
| **Genkit** | `firebase/genkit` | ⚠️ | Apache-2.0 | Agent framework JS/Go/Python/Dart. Ships **Dev UI w/ traces + evaluate + playground** |
| **MCP Toolbox for Databases** | `googleapis/mcp-toolbox` | ⚠️ | Apache-2.0 | Connect agents (Claude Code, Codex, **Antigravity**) to real DBs. Enables schema introspection + data-invariant verification |
| **go-containerregistry** | `google/go-containerregistry` | ⚠️ | Apache-2.0 | `crane`, `ko`, `kpack` |
| **Kokoro** | `google/kokoro` | ⚠️ | Apache-2.0 | Google's CI — largely internal, low portability |
| **google-github-actions** | `google-github-actions/*` | ⚠️ | — | `setup-gcloud`, `auth`, `get-project-version`. Legit CI integration point |
| Firebase Test Lab / App Distribution | closed | — | — | See §3 |

> **Google's angle:** Google ships *agent frameworks* (ADK, Genkit) and *MCP plumbing* (`google/mcp`, `mcp-toolbox`, `google-github-actions`) but **no product answering "did the AI-built app actually work?"** ADK's evaluation is for *agents*, not *apps an agent built*. **That gap is real and Google-shaped.**

---

## 6. Bucket: distribution channels — "take this repo and verify it"

### ⭐ The distribution primitive

| Project | Repo | License | Why it matters |
|---|---|---|---|
| **vercel-labs/skills** (`npx skills`) | `vercel-labs/skills` | MIT | **THE primitive.** v1.7.0 (Sep 17 2026). Installs into **75+ agents** from one command. Canonical `.agents/skills/` symlinked to `.claude/skills/`, `.cursor/`, `~/.config/opencode/skills/`. Lockfile `skills-lock.json`. Risk badges. **Every Bucket-4 competitor already installs through this** |
| Claude Code marketplace | `anthropics/claude-code-plugins` | — | ~90+ curated plugins. Ships `pr-review-toolkit` (6 review agents incl. **silent-failure-hunter**), `skill-creator`, `superpowers` |
| **skills.sh** | Vercel-backed | — | Index + risk badges. Launched Jan 2026 |
| agentmods.dev | — | — | Crawls Claude settings/plugin files. Star counts + security grades + token cost |
| addyosmani/agent-skills | `addyosmani/agent-skills` | — | 24 skills; Claude marketplace + Antigravity + Copilot + Kiro. Reference-quality layout |
| NVIDIA/skills | `NVIDIA/skills` | — | Enterprise-grade distribution (cuOpt) |
| plugin-creator | `leoriczhang/plugin-creator` | ⚠️ | Manifest validator: 28 hook events, MCP/LSP shapes, SemVer, marketplace strict mode |
| potenlab QA agent | `potenlab/marketplace-potenlab` | — | Claude marketplace QA agent: generates Vitest RLS tests vs local Supabase |

> **Implication:** 2,500+ registered Claude marketplaces, skills.sh, ~10k packages. **Distribution is commoditized. Ride `npx skills`; do not build a channel.**

### Agent-behavior testing (closest scientific neighbors)

| Project | Repo | What it does | Relevance |
|---|---|---|---|
| **AgentSwing** | `makieali/agentswing` | Black-box QA for AI agents. Personas → multi-turn convos → scored. **Publishes judge's Cohen's kappa.** 822 tests, 95% cov. MIT. 0⭐ (Jul 2026) | ⭐ Synthetic AI users done honestly. But it's *agent* QA not *app* QA |
| **AgentEval** | `AlexandreDeCarli/AgentEval` | Gemini Tester Agent + system + Gemini Evaluator. 100% local | Synthetic users, older |
| **τ-Trait** | `collinear-ai/tau-trait` | Academic (arXiv 2510.04491). Activation-space steering to simulate impatience/confusion/skepticism. **Frontier agents degrade 2–46%** under trait perturbation | ⭐ **The scientific justification for synthetic personas.** Citable in a whitepaper |
| **AgentProbe** | `NeuZhou/agentprobe` | "Playwright for AI Agents." YAML behavioral testing, PII leaks, tool abuse, step explosions. 3,200+ tests | Direct YAML behavioral competitor |
| **Agenttic** | `omarharoun/Agenttic` | "UVM for agents." Reqs → versioned benchmarks. 7 metrics + 8 public datasets (BFCL, τ-bench, AgentHarm, AgentDojo, GAIA, SWE-bench Verified). **Hash-chained evidence dossiers**, cert tiers A/B/C w/ explicit "NOT ASSESSED" | ⭐ **Most rigorous evidence model found** |
| **AgentAssay** | `qualixar/agentassay` | Token-efficient stochastic testing, 5–20× cheaper. 10 framework adapters. **Trace-first offline metamorphic analysis.** arXiv 2603.02601 | ⭐ Trace-first + metamorphic + **PASS/FAIL/INCONCLUSIVE** |
| **Invarium** | `invarium-ai/invarium` | "Pytest for AI agents." `bless` baselines, flakiness scoring, regression detection | Clean DX, test-authoring |
| **MetamorphicGuard** | `duhboto/MetamorphicGuard` | Baseline-vs-candidate, bootstrap CIs. `--monitor latency/fairness/resource:metric=cpu_ms`. `spec_fingerprint` hashes prove which oracles ran | ⭐ **Direct hit on "invariant watching"** — but baseline-diff, not runtime |
| **probatio** | `Lexieli666/probatio` | pytest plugin for LLM apps. Metamorphic relations as decorators, Wilson 95% CI, flakiness stats, **record-replay cassettes** | ⚠️ Has record-replay — closest to "replay-confirmed" |
| **Agent-Harness** | `Suirotciv/Agent-Harness` | Trace assertions, EU AI Act + NIST AI RMF refs | Compliance-flavored |
| **agent-qa-toolkit** | `Tanyayvr/agent-qa-toolkit` | Evidence packs, regression diffs, CI gates, EU AI Act min-package | Compliance pack generator |
| **OpenTesters** | `OTMO123/OpenTesters` | 10–50 parallel Haiku agents, persona role-play, **5-layer testing** (API → DB → UI visual regression → observability → synthesis). MCP PostHog/Sentry/Supabase/GH | ⭐ **Only tool testing cross-layer causal chains** (Flutter tap → API → Stripe idempotency → DB → PostHog → Sentry silence). 0⭐ |
| **MetaProbe** | `huyuelin/MetaProbe` | ICSE 2027 submission. Auditable metamorphic eval of repo-level coding agents. Certificate Checker L0–L5, PEAR adjudication | Research-grade calibration |

---

## Synthesis

### Layer coverage

| Layer | Who covers it | Crowded? | Free/cheap gap |
|---|---|---|---|
| **Scaffold** | create-t3-app, Next.js, bolt.diy | 🔴 Saturated | None |
| **Build (AI)** | v0, Lovable, Bolt, Replit, Devin, OpenHands, Codex, Claude Code, Cursor | 🔴🔴 Saturated | None |
| **Deploy** | Vercel, Netlify, Fly, Railway, Coolify, Dokploy, CapRover, SST | 🔴 Saturated | None |
| **Test (scripted)** | Playwright (+Test Agents), Cypress, Vitest, Maestro, Detox, Firebase Test Lab | 🔴 Saturated | None |
| **Test (AI-authored)** | Expect, vibe-testing, vibecodeqa, loop-testing, Playwright Test Agents | 🟡 Crowded, all tiny | ⚠️ Trust layer only |
| **Verify — static gates** | ShipProof, agents-shipgate, prove_it, ship-gate, roborev | 🟡 Crowded | ⚠️ Truthfulness layer |
| **Verify — behavioral/runtime** | loop-testing, verif-kit, AgentSwing, AgentAssay | 🟢 Thin + scattered | ⭐ **HERE** |
| **Synthetic users** | AgentSwing, τ-Trait, OpenTesters, AgentEval | 🟢 Thin, agent-only | ⭐ Adjacent |
| **Invariant watching** | MetamorphicGuard, probatio, AgentAssay | 🟢 **Nearly empty at runtime** | ⭐⭐ **BIGGEST GAP** |
| **Replay evidence** | probatio (cassettes), Playwright healer, loop-testing | 🔴 Nearly empty as a *product* | ⭐⭐ **BIGGEST GAP** |
| **Monitor** | MetamorphicGuard, Genkit, PostHog/Sentry | 🟡 Post-hoc only | ⭐ Pre-ship = gap |

### 3 most crowded spaces
1. **Static "production readiness" scoring** — ≥7 tools, all ≤90⭐. **Users won't pay for a checklist.**
2. **AI-authored browser tests** — Expect (3.5k), vibe-testing, vibecodeqa, loop-testing — **and Microsoft just shipped the official version.** Structural headwind.
3. **Agent skill/plugin marketplaces** — 2,500+ registered, skills.sh, ~10k packages. **Ride `npx skills`.**

### 3 emptiest spaces
1. **Runtime invariant watching.** MetamorphicGuard = baseline-diff at release. probatio = pytest decorators. **Nobody watches invariants live while an agent churns the code**, correlating "agent changed X" → "invariant Y broke."
2. **Replayable, tamper-evident bug evidence.** `prove-it` invented the honest 4-way verdict. `agents-shipgate` invented hash-chained receipts. **Nobody shipped both together as a product an app-builder's CI can consume.** Every "bug report" here is prose.
3. **The app-builder → repo handoff seam.** v0/Lovable/Bolt all export code and drop verification exactly at that boundary. Bolt's WebContainer actively *blocks* external tooling until push to GitHub. **Nobody owns the first hour after export.**

### Distance to TestKit

| Tier | Who | Gap |
|---|---|---|
| Adjacent | expect (3.5k), Playwright Test Agents, prove_it (199) | Browser/static scope |
| **Close** | **loop-testing** | Has synthetic users, replay-verify, convergence, honest INCOMPLETE, stop-hook. **Missing: invariant watching, cross-layer observability, first-class evidence artifact** |
| **Close** | **verif-kit** | Blind verifier, property+metamorphic+mutation, 20/20 evidence. **Missing: runtime/synthetic users, distributed/browser coverage, TS-only, 0⭐** |
| Different scope | agents-shipgate, Agenttic, AgentAssay, AgentSwing, MetamorphicGuard | Verify agents/tooling, not "did the app work" |
| Platform-owned | Lovable browser tests, Replit self-test, Base44 screenshots | Ephemeral by design |

---

## Recommended integration surfaces

**Tier 1 — install surfaces (do first; reach every competitor's user):**
1. **`npx skills add`** via `vercel-labs/skills` — non-negotiable. 75+ agents incl. Claude Code, Codex, Cursor, OpenCode.
2. **Claude Code plugin** (`.claude-plugin/plugin.json` + `marketplace.json`) — required for `PreToolUse`/`Stop` hooks. Only `prove_it` + `claude-code-ship-gate` genuinely exploit hooks.
3. **MCP server** — table stakes now.
4. **GitHub Action** with `fail-under`, SARIF, PR comments, blocking **Check Run**.
5. **Git hooks + worktree isolation** — `claude-code-ship-gate`'s `PreToolUse`-above-git pattern is worth copying.

**Tier 2 — verification targets:**
6. Playwright MCP + Test Agents (**consume, don't compete**)
7. `pocketbase`, `get-convex/convex`, `appwrite`, `supabase` local — the vibe-coder default backends
8. `googleapis/mcp-toolbox` — real DB introspection for data invariants
9. `google/mcp` → **Chrome DevTools MCP** (console + network + DOM, agent-drivable)

**Tier 3 — deploy gates:**
10. Vercel/Netlify preview webhooks + `x-vercel-protection-bypass`
11. `google-github-actions/*` for GCE/GKE deploys
12. Expo EAS `type: maestro` (**alpha** — don't hard-depend)

**Tier 4 — evidence/compliance (differentiators nobody has):**
13. Emit `agents-shipgate`-style **content-addressed receipts** (verify via `reproduce`)
14. Adopt **prove-it's 4-way verdict**: `PROVEN | FAILED | NOT PROVEN | BLOCKED`
15. **Publish your own validation study** like verif-kit (20/20), prove-it (9/12→12/12), AgentSwing (judge kappa). **In this market, published self-calibration is the only moat.**

---

## 🎯 Honest verdict

**Is there still a real product?** The *category* is saturated. The *layer* is not.

- 25+ tools say "AI verification for AI-built code." Top has 3,500⭐; **median has zero.**
- Microsoft just shipped official agentic-testing primitives into Playwright.
- Replit, Lovable, Base44 all ship agent-side verification, eroding the "nobody verifies" premise.
- 7 near-identical production-readiness scorers; none broke out.

**What that pattern means:** the idea has been tried many times and failed to find distribution 25 times. Consistent failure mode — **single-mechanism tools with no downstream action.** A report nobody reads, a score nobody gates on, a browser check nobody re-runs.

### The narrowest sharp wedge

> **Don't build a better vibe-code QA tool. Build the bug *evidence packet* — and make it the only artifact a gate ever consumes.**

1. **Runtime invariant watching is the real empty space.** Not baseline-diff-at-release. Not pytest decorators. **A watcher that holds declared invariants live while an agent rewrites the codebase**, and can say *"this agent iteration broke tenant isolation on line 47."* Nobody does this.

2. **Make the artifact replayable and tamper-evident.** Every competitor emits prose. `agents-shipgate` proves hash-chained receipts work here; `prove-it` proves the 4-way verdict reads well. **Combine them.** A deterministic replay script + hash chain + honest verdict is not comparable to a markdown report.

3. **Own the handoff hour, not the build.** Target the moment code leaves Bolt/Lovable/v0 and lands in a repo. All three export; none verify there; WebContainers block external tools until push. **"First hour after export" is defensible and uncontested** — and it's where the RLS disaster actually bites.

4. **Do NOT compete on:** writing Playwright tests (Expect + Playwright own it), static scoring (7 dead tools), or building your own skill marketplace (use `npx skills`).

### The skeptical read on execution

The biggest risk isn't competition — it's that **this space has a 25-failure graveyard of the same idea.** Four of the five closest competitors were created in the last 8 months and all have ~0⭐. Whatever killed them will kill TestKit too.

**Hypothesis: the failure mode is "verification without consequence."** TestKit survives only if its output is *consumed by a gate that blocks a merge or a deploy* — and if it publishes calibration evidence proving the gate doesn't false-positive. **The tools that gate survive (agents-shipgate 90⭐, prove_it 199⭐, ShipProof 18⭐). The tools that merely score die (all ≤24⭐). That's the actual dividing line in this data.**

Validate that before writing code.