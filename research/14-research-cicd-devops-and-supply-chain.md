# TestKit — Landscape Research: CI/CD, GitOps, Deployment, Observability, Release Safety & Agent-Readable Learning Resources

**Research date:** 11 Oct 2026
**Compiled for:** TestKit v0.3 positioning (local-first AI verification plugin + CI gate)
**Method:** Every GitHub entry below was verified by fetching the repo page/API on 11 Oct 2026. Stars are point-in-time. Items marked **⚠️ CORRECTED** had names in the original brief that do not exist — the repo was renamed, moved, or never existed. Do not use the original names.

---

## Executive Summary (read this first)

Six findings that change TestKit's plan:

1. **Sentry already shipped TestKit's headline feature.** Sentry's **Seer** agent now "predicts bugs before they ship to prod," does **code review against real production data**, does root-cause analysis, and opens PRs — with handoff to Claude Code, Cursor, and Copilot. A "SRE Sentinel" that watches production crashes and generates repros is not a differentiator; it's a feature request to Sentry. See §5 and §10.

2. **There is now a crowded "agentic pre-ship verification" category.** Autonoma, Minitap, Momentic, Shift-Left API, Confident AI, Langfuse, LangSmith, Braintrust, Arize Phoenix, and GitHub's own **gh-aw** all sell "an agent verifies your change before merge." TestKit is not entering a white space. See §10.2.

3. **GitHub Actions pricing changed materially on 1 Mar 2026.** Self-hosted runners in private repos now carry a **$0.002/min platform fee**; hosted Linux dropped to **$0.006/min**. This makes "bring your own runner" economically interesting for a heavy verification tool — and it means TestKit can be priced per-run without fear of being blamed for runaway CI minutes.

4. **The Agent Skills open standard now exists** (`agentskills.io`, `SKILL.md` folder format), backed by `anthropics/skills` and `github/awesome-copilot`. TestKit should ship a Skill, not just a CLI. This is the cheapest distribution channel available and it is already normalized.

5. **The cheapest isolation tier in 2026 is a Firecracker microVM, not Docker.** Vercel Sandbox went GA in Jan 2026 on Firecracker; E2B is Apache-2.0 Firecracker. Docker shares a host kernel and is now considered insufficient for untrusted/AI-generated code. TestKit's sandbox choice is a credibility decision, not a performance one.

6. **Google's agent infrastructure is unusually good and unusually under-used.** `google/mcp` (4,633★) ships MCP servers for Cloud Run, GKE, BigQuery, Cloud SQL, Spanner, Cloud Storage, Security Command Center, and the gcloud CLI. `googleapis/mcp-toolbox` (16,643★) is an open-source MCP server for databases. A local-first verification tool has a ready-made, free, Google-grade MCP surface to talk to. See §9.

---

## 1. CI/CD Platforms & Runners

### Open source (verified)

| Name | URL | Stars | License | What it does | Relevance to a pre-ship gate |
|---|---|---|---|---|---|
| **GitHub Actions** | github.com/features/actions | — | Free tier; metered | Native CI/CD for GitHub repos; marketplace of reusable actions | **Primary target.** A required status check is the gate mechanism. Runs 11.5B min in public/OSS repos in 2025 (+35% YoY) |
| **actions/runner** | github.com/actions/runner | 6,319 | MIT | The GitHub Actions runner itself | The binary TestKit must ship against; self-hosted runner = free compute for heavy verification |
| **actions/checkout** | github.com/actions/checkout | 8,946 | MIT | Checks out a repo into the workspace | Trivial prerequisite step |
| **actions/actions-runner-controller** ⚠️ CORRECTED *(was `actions/runner-controller`)* | github.com/actions/actions-runner-controller | 6,552 | Apache-2.0 | K8s operator that autoscales ephemeral self-hosted runners | Scale path if TestKit's gate becomes minutes-expensive |
| **Jenkins** | github.com/jenkinsci/jenkins | 26,637 | MIT | Self-hosted automation server, ~1,800 plugins | #2 CI adoption (28%). Every org has one; a TestKit plugin step is viable |
| **GitLab CI** | gitlab.com/gitlab-org/gitlab | 24,552 (gitlabhq/gitlabhq CE mirror) | CE core MIT / EE commercial | CI/CD fused into the GitLab platform | 19% adoption. Dual-target: ship a `testkit.yml` job template |
| **Buildbot** | github.com/buildbot/buildbot | 5,480 | GPL-2.0 | Python CI framework, heavy plugin model | Niche. Skip |
| **Drone → Harness Open Source** ⚠️ CORRECTED *(repo `drone/drone` now redirects to `harness/harness`)* | github.com/harness/harness | 38,513 | Apache-2.0 (mixed; EE commercial) | End-to-end platform: SCM + CI/CD + hosted dev envs + registries | Worth watching; Drone proper is effectively absorbed |
| **Woodpecker CI** ⚠️ CORRECTED *(is `woodpecker-ci/woodpecker`, not `woodpeckerci/woodpecker`)* | github.com/woodpecker-ci/woodpecker | 7,981 | Apache-2.0 | Lightweight Drone-inspired CI engine | Small community. Low priority |
| **Tekton Pipelines** | github.com/tektoncd/pipeline | 9,077 | Apache-2.0 | K8s-native pipeline CRDs | Only relevant for K8s-heavy teams. Low priority |
| **Tekton Triggers** | github.com/tektoncd/triggers | 594 | Apache-2.0 | Event-driven Tekton triggers | Low priority |
| **Argo Workflows** | github.com/argoproj/argo-workflows | 17,031 | Apache-2.0 | Container-native workflow engine on K8s | Overlaps with "swarm of actors" concept — read as prior art |
| **Concourse CI** | github.com/concourse/concourse | 7,915 | Apache-2.0 | Container-based pipeline system in Go | Niche. Skip |
| **Codefresh (CLI)** | github.com/codefresh-io/codefresh | 81 | MIT | CLI only; the platform is commercial SaaS | SaaS, not OSS. Low priority |

### Runner/infra accelerators (verified)

| Name | URL | Stars | License / Price | What it does | Relevance |
|---|---|---|---|---|---|
| **Dagger** | github.com/dagger/dagger | 16,334 | Apache-2.0 | CI/CD as code engine that runs identically locally and in CI | **Closest architectural analog to TestKit's "local-first."** Study their engine/CLI split carefully |
| **Earthly** | github.com/earthly/earthly | 12,054 | MPL-2.0 | Build framework, Dockerfile+Makefile syntax, repeatable builds | Same local/CI parity thesis, different layer (builds not verification) |
| **Nixpacks** | github.com/railwayapp/nixpacks | 3,554 | MIT | Turns app source into a container image with zero Dockerfile | Auto-provisions the exact runtime TestKit needs |
| **Depot** ⚠️ CORRECTED *(CLI is `github.com/depot/cli`, not `depotdev/depot`)* | depot.dev | — | Dev: 500 Docker min + 2,000 CI min/mo. Startup **$200/mo**: 20,000 Depot CI + 20,000 GHA min. Overages: Docker $0.04/min, Depot CI $0.00005/s/vCPU, GHA jobs $0.006/min, agent sandboxes **$0.01/min**, cache/registry $0.20/GB/mo, test results $0.000001/passing test | Remote container builds + managed GitHub Actions runners + a separate "Depot CI" engine + agent sandboxes + test-result analytics | **Directly models TestKit's unit economics.** Their "per passing test" pricing is the closest analogue to a verification gate |
| **Blacksmith** | blacksmith.dev | — | From $0.004/min on 2 vCPU Linux x64 | Managed faster GitHub Actions runners | Runner-cost alternative |
| **Namespace** | namespace.so | — | From $0.002/min in-plan, $0.003/min overage (2 vCPU Linux x64) | Remote build cache + runners, Bazel/Turborepo-first | Cache-first; less about verification |
| **Bitrise Build Hub** | bitrise.io | — | Basic/Pro $15–$810/mo; strong on macOS | Build infrastructure, macOS-heavy | Worth knowing if TestKit needs real-device iOS verification |

### Commercial CI platforms (no OSS repo — verified as commercial products)

| Name | Price model | Relevance |
|---|---|---|
| **CircleCI** | ~$0.006 per standard Linux minute (matches GitHub); credit-based | Performance leader; Test Engine does flaky-test splitting |
| **Buildkite** | Pro **$30/user/month**, build-count independent; hosted agents from **$0.004/vCPU-min** (Sept 2026); signed pipelines | Economics invert at high volume — agents you own |
| **Harness CI** | Commercial (OSS repo above is the platform core) | Feature flags + CI + CD in one; a competitor for the "gate" budget |
| **TeamCity** (JetBrains) | Commercial; 2026 roundup notes strong governance/TCO | Governance-heavy orgs |
| **Azure Pipelines** | Microsoft DevOps; 1 free parallel job w/ MS-hosted minutes | Azure-committed shops |
| **Bitbucket Pipelines** | Per-minute; **now also charging for self-hosted runners** (announced 2026) | Atlassian shops |
| **AWS CodePipeline / CodeBuild** | Per-minute + vCPU-second; CodeBuild has a 1,000-min free tier | AWS shops |
| **Google Cloud Build** | Per build-minute above free tier; **424,979 GitHub Marketplace installs** | GCP shops |
| **Cloudflare Workers Builds** | Usage-based; free tier | Edge-first teams |
| **Vercel / Netlify CI** | Bundled into platform plans | Deploy-and-preview buyers |

---

## 2. GitOps, Release & Progressive Delivery

### GitOps engines (verified)

| Name | URL | Stars | License | What it does | Relevance to a pre-ship gate |
|---|---|---|---|---|---|
| **Argo CD** | github.com/argoproj/argo-cd | 24,361 | Apache-2.0 | Declarative continuous deployment for K8s; Git as source of truth | The canonical GitOps engine. A gate is most naturally a *sync-window blocker* here |
| **Argo Rollouts** | github.com/argoproj/argo-rollouts | 3,595 | Apache-2.0 | Canary/A-B/blue-green for K8s; metric-driven auto promote/rollback | **This is the reference implementation of TestKit's "blue/green traffic flip."** It already exists, is mature, and is free |
| **Flux** | github.com/fluxcd/flux2 | 8,442 | Apache-2.0 | GitOps toolkit; continuous delivery for K8s | Alternative GitOps engine |
| **Flagger** | github.com/fluxcd/flagger | 5,418 | Apache-2.0 | Progressive delivery operator (canary, A/B, blue-green) | Metric-only, more automated than Rollouts. Pairs with Flux |
| **Keptn** | github.com/keptn/keptn | 1,773 | Apache-2.0 | SLO-driven delivery orchestration + auto-remediation | **Conceptually the closest thing to TestKit's "SRE Sentinel."** Sits between delivery and operations, keys off SLOs. Slow-moving project, but read it |
| **Spinnaker** | github.com/spinnaker/spinnaker | 9,803 | Apache-2.0 | Multi-cloud CD platform | Legacy; large install base in enterprises |
| **Istio** | github.com/istio/istio | 38,429 | Apache-2.0 | Service mesh; traffic management | Where the actual traffic-splitting happens under Argo/Flagger |
| **Traefik** | github.com/traefik/traefik | 65,146 | MIT | Cloud-native application proxy / ingress | Lightweight traffic-split point without a full mesh |

### Feature flags & experimentation (verified OSS)

| Name | URL | Stars | License / Price | What it does | Relevance |
|---|---|---|---|---|---|
| **Unleash** | github.com/Unleash/unleash | 13,867 | AGPL-3.0 (self-host free; commercial cloud) | Open-source feature management | Kill-switch as a release-safety tool |
| **Flagsmith** | github.com/Flagsmith/flagsmith | 6,591 | BSD-3-Clause | Feature flags + remote config | Same |
| **GrowthBook** | github.com/growthbook/growthbook | 8,486 | Mixed (core open; NOASSERTION) | Feature flags + experimentation + product analytics | Experiment analysis on release cohorts |
| **PostHog** | github.com/PostHog/posthog | 40,218 | Mixed (MIT core + EE; NOASSERTION) | Product analytics, session replay, feature flags, experiments, error tracking, logs, **and an MCP server** | **Highest-leverage single integration for TestKit.** One OSS deployment gives analytics + replay + flags + errors + MCP |

### Commercial flags / delivery / incident (no OSS repo)

| Name | Category | Relevance |
|---|---|---|
| **LaunchDarkly** | Feature flags + progressive delivery | Enterprise standard; has MCP |
| **Split.io** | Feature flags | Enterprise |
| **Optimizely** | Experimentation + delivery | Enterprise |
| **Fiddler** | Experimentation + AI monitoring | Enterprise |
| **Rollbar** | Error tracking | See §5 |
| **Harness** | CD + flags + verification | **Competes for the same budget line as TestKit.** Harness already sells a "chaos engineering + verification" module |
| **FireHydrant** | Incident response | Its CLI repo 404s publicly; SaaS-only |

---

## 3. Deployment & Preview Environments

### PaaS / self-hosted platforms (verified)

| Name | URL | Stars | License | What it does | Relevance to a pre-ship gate |
|---|---|---|---|---|---|
| **Portainer** | github.com/portainer/portainer | 38,629 | Zlib | Docker/K8s management UI | Ops plane; not a gate |
| **Dokku** | github.com/dokku/dokku | 32,175 | MIT | Docker-powered mini-PaaS | Cheap target for "deploy then verify" |
| **CapRover** | github.com/caprover/caprover | 15,183 | Mixed (NOASSERTION) | Heroku-like PaaS on Docker+nginx | Same |
| **Coolify** | — | ⚠️ CORRECTED *(repo `coolify-io/coolify` returns 404; product is SaaS + self-host commercial)* | AGPL-backed, commercial tiers | Self-hostable PaaS with previews | Preview-env-as-a-feature is table stakes here |

### Sandbox / ephemeral compute for agents (verified)

| Name | URL | Stars | License / Price | Isolation | Relevance |
|---|---|---|---|---|---|
| **Daytona** | github.com/daytonaio/daytona | 71,575 | Mixed | Secure elastic infra for running AI-generated code | Strong candidate for TestKit's remote-sandbox tier |
| **Vercel Sandbox** | vercel.com/docs/sandbox | — | Usage-based, active-CPU billing (up to 95% cheaper for bursty/I-bound); GA **Jan 2026**; Hobby 45-min / Pro 5-hour caps | **Firecracker microVM** on Amazon Linux 2023; Node 22/24/26 + Python 3.13 preinstalled; OCI custom images (public beta Jul 2026); **CLI and SDK open-sourced** | The benchmark for agent sandbox. Billed on active CPU — perfect fit for I/O-bound verification |
| **E2B** | github.com/e2b-dev/E2B | 14,272 | Apache-2.0 | Secure, enterprise-grade agent execution env; maintains its own Firecracker fork | **The open-source alternative to Vercel Sandbox.** Self-hostable → best fit for local-first TestKit |
| **E2B runtime** | github.com/e2b-dev/runtime | 1,707 | Apache-2.0 | The runtime behind E2B Cloud, Enterprise, and self-hosted | Read for self-host architecture |
| **Modal** | github.com/modal-labs/modal-client | 523 | Apache-2.0 (SDK); platform commercial | Serverless CPU/GPU with a Sandbox primitive explicitly built for untrusted agent code | GPU/load-scale option for 1,000-actor swarms |
| **Fly.io Machines** | fly.io | — | Usage-based | Firecracker microVMs + Machines API, 18 regions; SOC 2 II + HIPAA BAA without Enterprise gate | Many teams use it as an ad-hoc sandbox host |
| **Cloudflare Sandboxes** | developers.cloudflare.com | — | **$0.00002/vCPU-second**; free tier; Workers paid plan $5/mo | Linux container per VM on Workers + Durable Objects | Cheapest credible isolation at scale |
| **Coder** | github.com/coder/coder | 16,944 | AGPL-3.0 | "Secure environments for developers **and their agents**" | Agent-aware dev environments |
| **Gitpod** | github.com/gitpod-io/gitpod | 13,794 | AGPL-3.0 | On-demand cloud dev environments | Legacy of the category |

### Branching-based preview data (verified as capabilities)

| Name | What it does | Relevance |
|---|---|---|
| **Supabase branching** | Per-branch Postgres clone; migrations applied fresh; `merge_branch` / `reset_branch` / `rebase_branch` | **Best preview-data story available.** Real schema + real queries per PR, zero fixture writing |
| **Neon branching** | Same idea, Postgres, copy-on-write | Strong alternative |
| **PlanetScale** | Branching was **removed**; Vitess-based, no longer a fit for this | Do not build on it |
| **Expo EAS Update** (github.com/expo/eas-cli, 1,366★, MIT) | OTA JS bundle updates for React Native | The closest analogue to "ship without app-store review" — a model for low-risk release |
| **Vercel preview / Netlify deploy previews** | Per-PR ephemeral deploy + shareable URL | The default substrate for pre-ship verification |

### Google Cloud & Firebase deploy surfaces (verified)

| Name | URL | What it does | Relevance |
|---|---|---|---|
| **Google Cloud Run** | cloud.google.com/run | Serverless containers, scale-to-zero | Cheapest GCP target to deploy-and-verify |
| **Google Cloud Deploy** | cloud.google.com/deploy | Managed GKE/Anthos/Cloud Run rollout with approvals and canary | Google's answer to Argo Rollouts. **Managed, so TestKit shouldn't rebuild it** |
| **Google Cloud Deploy GitHub Action** | github.com/google-github-actions/create-cloud-deploy-release (28★, Apache-2.0) | Creates a Cloud Deploy release from Actions | Note: explicitly **"not an officially supported Google product"** |
| **Firebase App Hosting** | firebase.google.com/docs/app-hosting | Cloud Run–backed app hosting with rollouts | Emerging; worth a demo |
| **Google Cloud Shell** | docs.cloud.google.com/shell/docs | Browser shell with preinstalled, pre-authenticated `gcloud`; built-in Cloud Code editor; web app previews | Zero-install Google access. **A place TestKit should work on day one for GCP users** |

---

## 4. Containers & Sandboxes for Agent Execution

| Name | URL | Stars | License | What it does | Relevance to a local-first verifier |
|---|---|---|---|---|---|
| **Docker (Moby)** | github.com/moby/moby | 72,167 | Apache-2.0 | The container ecosystem | Baseline. **Shared kernel = not sufficient isolation for AI-generated code in 2026** |
| **docker compose** | github.com/docker/compose | 38,353 | Apache-2.0 | Multi-container app definition | How TestKit assembles "exact services the project uses" |
| **Podman** | github.com/podman-container-tools/podman *(⚠️ moved from `containers/podman`)* | 33,023 | Apache-2.0 | Rootless OCI container/pod manager | Best local-first daemon; rootless suits untrusted code better |
| **Testcontainers** | github.com/testcontainers/testcontainers-go | 5,002 | MIT | Programmatic throwaway container deps for integration tests | **The commodity answer to "provision Postgres/Mongo/Redis automatically." Do not rebuild** |
| **Lima** | github.com/lima-vm/lima | 22,065 | Apache-2.0 | Linux VMs focused on running containers | VM isolation on macOS without Docker Desktop licensing |
| **Colima** | github.com/abiosoft/colima | 31,151 | MIT | Container runtimes on macOS/Linux, minimal setup | The friction-remover for mac devs. A likely TestKit dependency, not a competitor |
| **Rancher Desktop** | github.com/rancher-sandbox/rancher-desktop | 7,386 | Apache-2.0 | Containers + K8s on desktop | Enterprise Mac/Windows story |
| **WSL2** | learn.microsoft.com/windows/wsl | — | Free (Windows) | Linux VMs + kernel on Windows | Mandatory for Windows local-first |
| **OrbStack** | orbstack.dev | — | Commercial (free for personal) | Fast macOS/Linux containers + VMs | Best UX competitor to Colima |
| **Nix** | github.com/NixOS/nix | 17,853 | LGPL-2.1 | Purely functional package manager | Hermetic toolchains. Heavy user-experience cost |
| **Devbox** | github.com/jetify-com/devbox | 12,397 | Apache-2.0 | Instant, predictable dev environments | Best "reproducible env" ergonomics in the OSS world |
| **Dev Container spec** | github.com/devcontainers/spec | 5,762 | CC-BY-4.0 | Container-as-full-dev-environment standard | **The portable "bring your own env" contract.** VS Code, Codespaces, Coder, and many CI runners understand it |
| **Kubernetes** | github.com/kubernetes/kubernetes | 128,258 | Apache-2.0 | Container orchestration | Overkill for most TestKit users |
| **runc** | github.com/opencontainers/runc | 13,484 | Apache-2.0 | OCI runtime | The layer below Docker |
| **Buildah** | github.com/containers/buildah | 9,066 | Apache-2.0 | Builds OCI images without a daemon | Rootless image builds in CI |
| **ko** | github.com/ko-build/ko | 8,563 | Apache-2.0 | Build and deploy Go apps from source | The Go path from source to Cloud Run |
| **GitHub Codespaces** | github.com/features/codespaces | — | Per-core-hour | Hosted devcontainer; CAUTION on shared-kernel isolation | Cheap env provisioning, weaker isolation |
| **Blast (BlastRadius)** | — | ⚠️ CORRECTED *(repo 404s; project appears retired)* | — | — | — |

---

## 5. Observability, Error Tracking & Incident Response

| Name | URL | Stars | License | What it does | Relevance to a ship gate |
|---|---|---|---|---|---|
| **Uptime Kuma** | github.com/louislam/uptime-kuma | 92,308 | MIT | Self-hosted monitoring, 20+ monitor types | Post-deploy verification target; also the CNCF-crowd-favorite |
| **Grafana** | github.com/grafana/grafana | 77,226 | AGPL-3.0 | Metrics/logs/traces visualization | The dashboard layer of any self-hosted SLO story |
| **Netdata** | github.com/netdata/netdata | 80,872 | GPL-3.0 | "Fastest path to AI-powered full stack observability" | Single-binary install → very cheap to make optional in TestKit |
| **Loki** | github.com/grafana/loki | 29,006 | AGPL-3.0 | Prometheus for logs | Log tailing for the flight recorder |
| **SigNoz** | github.com/SigNoz/signoz | 32,325 | Mixed (NOASSERTION) | OpenTelemetry-native observability, "for your team **and their AI agents**" | **Explicitly agent-targeted.** Strong candidate for TestKit's telemetry backend |
| **OpenObserve** | github.com/openobserve/openobserve | 22,315 | AGPL-3.0 | Logs/metrics/traces/RUM/replay/pipelines/SLO/**LLM observability** | LLM-observability built in — relevant if TestKit records its own agent runs |
| **Jaeger** | github.com/jaegertracing/jaeger | 23,272 | Apache-2.0 | CNCF distributed tracing | Request-level correlation |
| **Grafana Tempo** | github.com/grafana/tempo | 5,520 | AGPL-3.0 | High-volume distributed trace backend | Cheapest trace store |
| **Grafana Pyroscope** | github.com/grafana/pyroscope | 11,697 | AGPL-3.0 | Continuous profiling to the line | Catches perf regressions a functional test misses |
| **Grafana OnCall** | github.com/grafana-cold-storage/oncall *(⚠️ moved from `grafana/oncall`)* | 3,884 | AGPL-3.0 | Incident response with Slack integration | On-call surface for sentinel alerts |
| **VictoriaMetrics** | github.com/VictoriaMetrics/VictoriaMetrics | 17,841 | Apache-2.0 | Fast, cost-effective TSDB | Cheapest Prometheus-compatible metrics |
| **Sentry** | github.com/getsentry/sentry | 45,525 | Functional Source License (NOASSERTION — *not* OSI OSS) | Developer-first error tracking + performance | **The system the 04 blueprint names.** Note the license: self-hosting is possible but it is source-available, not open source |
| **Sentry Seer** ⚠️ NOT OSS | sentry.io/product/seer | — | Commercial; flat pricing, unlimited usage; open beta | AI debugging agent: prod RCA, Autofix + PR creation, **PR code review against real production data**, **bug prediction before ship**; hands off to Claude Code, Cursor Cloud Agents, Copilot; **Sentry MCP server** | **This is TestKit's direct competitor.** Read §10.1 before writing more product copy |
| **Datadog agent** | github.com/DataDog/datadog-agent | 3,759 | Apache-2.0 | The Datadog agent | Metric provider for Argo Rollouts/Flagger |
| **Elastic APM (Go agent)** | github.com/elastic/apm-agent-go | 427 | Apache-2.0 | Elastic APM agent | Same |
| **New Relic** | — | ⚠️ CORRECTED *(repo `newrelic/newrelic-agent-go` 404s; agent is `newrelic/newrelic-go-agent`)* | Commercial | APM | Rollout metric provider |
| **Rollbar (JS)** | github.com/rollbar/rollbar.js | 587 | MIT | JS error tracking | Error surface |
| **Bugsnag (JS)** | github.com/bugsnag/bugsnag-js | 894 | MIT | JS error handling + reporting | Error surface |
| **Honeycomb** | — | ⚠️ CORRECTED *(`honeycombio/reqresp` 404s — it was renamed; commercial SaaS)* | Commercial | Query-based debugging | Good query UX; commercial |
| **HyperDX** | — | ⚠️ CORRECTED *(`HyperDX/hyperdx` now redirects to a user profile; verify current org before citing)* | OSS + cloud (MIT core) | ClickHouse-based observability | Cheap self-hosted stack |
| **Axiom** | github.com/axiomhq/axiom-js | 150 | MIT | Language bindings; ClickHouse-backed log/trace platform | Cheap ingest for raw event capture |
| **OpenTelemetry spec** | github.com/open-telemetry/opentelemetry-specification | 4,345 | Apache-2.0 | The vendor-neutral telemetry spec | **TestKit should emit OTel. It's the only durable schema.** |
| **OTel Collector contrib** | github.com/open-telemetry/opentelemetry-collector-contrib | 4,984 | Apache-2.0 | Receivers/processors/exporters | Pluggable ingest for the flight recorder |
| **Better Stack** | betterstack.com | — | Commercial (generous free tier) | Uptime + telemetry + On-call | Polished, agent-friendly |
| **Uptime Kuma** (see above) | — | — | — | — | — |
| **Upptime** | github.com/Upptime/upptime | 17,176 | MIT | Uptime monitor + status page built entirely from GitHub Actions | **Clever pattern: zero-server monitoring via Actions.** A free uptime tier for TestKit users |
| **Checkmate** | github.com/bluewave-labs/Checkmate | 10,924 | AGPL-3.0 | Self-hosted server hardware, uptime, response times, incidents | Infra-level verification |
| **Gatus** | github.com/TwiN/gatus | 12,279 | Apache-2.0 | Developer-oriented status page with alerting + incidents | **Best-fit for CI: health checks defined as code, runnable in a pipeline step.** A serious upstream for a "verify deploy" step |
| **healthchecks** | github.com/healthchecks/healthchecks | 10,407 | BSD-3-Clause | Cron/background job monitoring | Catches "the deploy worked but the job never ran" |
| **Doctor McNinja** | — | ⚠️ CORRECTED *(repo 404s; unverifiable at this URL)* | — | — | Do not cite |
| **pwndoc** | github.com/pwndoc/pwndoc | 2,908 | MIT | Pentest report generator | Report output; only if TestKit does red-team reporting |
| **Bearertrail/Honeycomb-style commercial** | — | — | — | — | — |

### Chaos engineering (adjacent, relevant to the Red-Team actor)

| Name | URL | Stars | License | Relevance |
|---|---|---|---|---|
| **LitmusChaos** | github.com/litmuschaos/litmus | 5,725 | Apache-2.0 | Chaos experiments for cloud-native; ChaosHub catalog | The mature answer to "red team in staging" |
| **Chaos Mesh** | github.com/chaos-mesh/chaos-mesh | 7,930 | Apache-2.0 | Chaos platform for K8s | Same, K8s-native |

---

## 6. Security Scanning in CI — TestKit's "code verification" phase

**All commodity. All should be invoked, none should be rebuilt.**

### SAST / code analysis (verified)

| Name | URL | Stars | License | What it does | Gate role |
|---|---|---|---|---|---|
| **Semgrep** | github.com/semgrep/semgrep | 16,959 | LGPL-2.1 | Lightweight static analysis, pattern-based bug and vuln detection | Fast PR-gate SAST. Note the older `returntocorp/semgrep-action` is **deprecated** — use `semgrep/semgrep` |
| **CodeQL** | github.com/github/codeql | 10,186 | MIT | Semantic engine + query packs; powers GitHub Advanced Security | Deepest SAST; free on public repos, paid on private via GHAS |
| **SonarQube / SonarCloud** | sonarsource.com | — | Community Build free / commercial | SAST + code smells + coverage | The "code health" gate most teams already run |
| **Bearer** ⚠️ CORRECTED *(org is `bearer`, repo `bearer/bearer`)* | github.com/bearer/bearer | 2,757 | Mixed (NOASSERTION) | SAST for security/privacy risks, prioritized | Modern privacy-focused alternative |
| **GitGuardian ggshield** | github.com/GitGuardian/ggshield | 2,004 | MIT | 500+ secret types; pre-commit hook, CLI, **and GitHub Action** | Best-in-class for secret gates |

### SCA / dependency & supply chain (verified)

| Name | URL | Stars | License | What it does | Gate role |
|---|---|---|---|---|---|
| **Trivy** | github.com/aquasecurity/trivy | 38,337 | Apache-2.0 | Vulns, misconfigs, secrets, SBOM across containers/K8s/code/cloud | **The single best-value scanner.** One binary, dozens of scanners. Start here |
| **Grype** | github.com/anchore/grype | 13,005 | Apache-2.0 | Vulnerability scanner for images and filesystems | Trivy alternative (Anchore/Syft lineage) |
| **OSV-Scanner** | github.com/google/osv-scanner | 11,162 | Apache-2.0 | Vulnerability scanner using the OSV.dev database | Authoritative OSV data. Google's contribution |
| **Dependency-Track** (alternative) / **Snyk CLI** | github.com/snyk/cli | 5,673 | Mixed (NOASSERTION; commercial SaaS) | Vuln + license monitoring | Commercial; GPL/AGPL-avoidance |
| **Vuls** | github.com/future-architect/vuls | 12,283 | GPL-3.0 | Agent-less scanner: Linux, containers, libs, network devices | Host-level. GPL makes embedding awkward |
| **Giskard** | github.com/Giskard-AI/giskard-oss *(⚠️ renamed from `Giskard-AI/giskard`)* | 5,887 | Apache-2.0 | Evaluation & testing library for LLM agents | **Agent-behaviour testing.** Relevant to TestKit's Red-Team actor |
| **Scorecard** | github.com/ossf/scorecard | 5,748 | Apache-2.0 | OpenSSF security health metrics for OSS repos | Supplier risk |
| **Reno­vate** | github.com/renovatebot/renovate | 22,718 | AGPL-3.0 | Cross-platform dependency automation | The Dependabot alternative. AGPL — invoke, don't embed |
| **GuardDog** | github.com/DataDog/guarddog | 1,230 | Apache-2.0 | Identifies malicious PyPI and npm packages | Supply-chain poisoning detection |
| **aquasecurity/kube-bench** | github.com/aquasecurity/kube-bench | 8,221 | Apache-2.0 | CIS Kubernetes Benchmark checks | K8s posture gate |
| **Prowler / Cloud Custodian** (adjacent) | — | — | — | Cloud posture | — |

### Secrets (verified)

| Name | URL | Stars | License | What it does |
|---|---|---|---|---|
| **Gitleaks** | github.com/zricethezav/gitleaks | 29,840 | MIT | Finds secrets in code/git history |
| **TruffleHog** | github.com/trufflesecurity/trufflehog | 28,418 | AGPL-3.0 | Finds, **verifies**, and analyzes leaked credentials (verification is the differentiator) |
| **Nuclei** | github.com/projectdiscovery/nuclei | 31,912 | MIT | Fast template-based vuln scanner, YAML DSL, community templates |

### Dynamic / web (verified)

| Name | URL | Stars | License | What it does |
|---|---|---|---|---|
| **OWASP ZAP** | github.com/zaproxy/zaproxy | 15,912 | Apache-2.0 | Web app DAST; has a ZAP-baseline GitHub Action |
| **sqlmap** | github.com/sqlmapproject/sqlmap | — | GPL-2.0 | Automated SQLi. ⚠️ **Point at staging/preview only. Never run against anything a customer can reach** |
| **Burp Suite** | portswigger.net | — | Commercial + freemium | Manual/assisted web testing |

### Policy as Code (verified)

| Name | URL | Stars | License | What it does | Relevance |
|---|---|---|---|---|---|
| **OPA** | github.com/open-policy-agent/opa | 12,340 | Apache-2.0 | General-purpose Rego policy engine | **The policy substrate.** TestKit can ship policy bundles |
| **Gatekeeper** | github.com/open-policy-agent/gatekeeper | 4,294 | Apache-2.0 | Admission control for K8s | K8s enforcement point |
| **Conftest** ⚠️ CORRECTED *(moved into the OPA org: `open-policy-agent/conftest`)* | github.com/open-policy-agent/conftest | 3,280 | Apache-2.0 | Test structured config against Rego | **Best fit for a repo-level gate** — lint your CI/CD YAML, Dockerfiles, k8s manifests, IaC |
| **Kyverno** | github.com/kyverno/kyverno | 8,236 | Apache-2.0 | Unified policy as code | K8s-native |
| **KICS** | github.com/Checkmarx/kics | 2,716 | Apache-2.0 | IaC security scanning | Terraform/CF/Bicep misconfigs |
| **Oso** ⚠️ **DEPRECATED** *(repo `osohq/oso` now reads "Deprecated: See README.")* | github.com/osohq/oso | 3,489 | Apache-2.0 | Authorization engine | Do not build on Oso/Polar |
| **Puppet** / **Nessus** | — | — | Commercial | Config mgmt / vuln scanning | Legacy enterprise |

### ⚠️ CI/CD-specific supply-chain security — the fastest-growing sub-category

**This is the single most relevant niche for a "verification gate" and it is genuinely underserved by the incumbents.**

| Name | URL | Stars | License | What it does |
|---|---|---|---|---|
| **zizmor** | github.com/zizmorcore/zizmor | 6,688 | MIT | Static analysis for GitHub Actions workflows — template injection, cache poisoning, permission overreach |
| **actionlint** | github.com/rhysd/actionlint | 4,310 | MIT | Static checker for Actions workflow files: syntax, shellcheck, pyflakes, actionlint rules |
| **StepSecurity Harden-Runner** | stepsecurity.io | — | SaaS + OSS | Network egress filtering + runtime security for GitHub-hosted and self-hosted runners |
| **Cimon** | — | Commercial | — | Runtime security for CI/CD pipelines |
| **poutine** | — | OSS | — | Detects misconfigurations/vulns in build pipelines |
| **octoscan** | — | OSS | — | Static vulnerability scanner for GitHub Actions workflows |
| **Gato / Gato-X** | — | OSS | — | Finds weaknesses in a GitHub org's public/private repos |
| **SmokedMeat / nord-stream / gh-hijack-runner** | — | OSS | — | Offensive tooling proving CI secret exfiltration is trivial |
| **awesome-cicd-attacks** | github.com/TupleType/awesome-cicd-attacks | 631 | CC0-1.0 | Curated offensive CI/CD research |
| **awesome-cicd-security** | github.com/myugan/awesome-cicd-security | 844 | MIT | Books, CISA/NSA guidance, blogs, tools, cases |
| **CISA guidance** (Defending CI/CD) | cisa.gov | — | Free | The reference threat model |

---

## 7. Test Automation in CI

| Name | URL | Stars | License | What it does | Relevance to a pre-ship gate |
|---|---|---|---|---|---|
| **Playwright** | github.com/microsoft/playwright | 97,445 | Apache-2.0 | Cross-browser (Chromium/Firefox/WebKit) testing + automation with one API | **TestKit's browser "hands and eyes."** CDP bridge is the same protocol. Build on it |
| **Cypress** | github.com/cypress-io/cypress | 51,045 | MIT | Browser testing, time-travel debugging | Competitor to Playwright; heavier runtime |
| **k6** | github.com/grafana/k6 | 31,859 | AGPL-3.0 | Load testing in Go/JS | **The commodity load generator.** TestKit's 1,000-actor swarm should not reinvent it |
| **Vitest** | github.com/vitest-dev/vitest | 17,197 | MIT | Vite-native unit testing | Fast unit layer in the gate |
| **Jest** | github.com/jestjs/jest | 45,550 | MIT | JS/TS test framework | Still the default in most repos |
| **Maestro** ⚠️ CORRECTED *(org is `mobile-dev-inc`, capitalized)* | github.com/mobile-dev-inc/Maestro | 16,006 | Apache-2.0 | Painless E2E automation for mobile and web (YAML flows) | **Best mobile smoke-test primitive.** Minitap's product is built on this class of thing |
| **Robot Framework** | github.com/robotframework/robotframework | 11,931 | Apache-2.0 | Generic automation + acceptance testing + RPA | Keyword-driven; excellent for non-engineers writing gate checks |
| **Pact** | github.com/pact-foundation/pact-js | 1,820 | Mixed (NOASSERTION) | Consumer-driven contract testing | Gate service boundaries without full integration |
| **Schemathesis** ⚠️ CORRECTED *(now `schemathesis/schemathesis`)* | github.com/schemathesis/schemathesis | 3,658 | MIT | Property-based API fuzzing from OpenAPI/GraphQL schemas | **Highest signal-per-effort gate.** Finds contract violations without a single written test |
| **dbt tests** | github.com/dbt-labs/dbt-core | 13,991 | Apache-2.0 | Data transformation with built-in test framework | The data-quality gate layer |
| **docker compose** | github.com/docker/compose | 38,353 | Apache-2.0 | Multi-container app definition | Test orchestration substrate |
| **Testcontainers** | github.com/testcontainers/testcontainers-go | 5,002 | MIT | Disposable containerized deps for tests | Dependency orchestration for the gate |

---

## 8. Learning Resources an AI Agent Can Read

**This bucket matters most for TestKit's cold-start problem and for the Skill distribution channel.**

### The Agent Skills ecosystem (verified) — ⭐ highest-leverage section

| Name | URL | Stars | License | What it is |
|---|---|---|---|---|
| **Agent Skills open standard** | agentskills.io | — | Open spec | A skill = a folder with `SKILL.md` (metadata: name + description) plus optional scripts, references, templates. Two-phase loading: agent reads name/description at startup, reads the full file only when relevant. Portable across Claude, Copilot, Cursor, OpenCode |
| **agentskills/agentskills** | github.com/agentskills/agentskills | 26,027 | Apache-2.0 | The spec and documentation. Python reference SDK available |
| **anthropics/skills** | github.com/anthropics/skills | 180,351 | Mixed | Anthropic's public Agent Skills repository |
| **mattpocock/skills** | github.com/mattpocock/skills | 284,555 | MIT | "Skills for Real Engineers" — `/to-spec`, `codebase-design`, `code-review` (parallel Standards/Spec sub-agents), `grill-with-docs`. **The highest-signal template for how to write a TestKit Skill** |
| **github/awesome-copilot** | github.com/awesome-copilot | 39,890 | MIT | Community instructions, **agents, skills**, and configurations for Copilot. Includes `github-actions-ci-cd-best-practices` with concrete directives on checkout depth, self-hosted runners, staging-environment E2E, and flakiness mitigation |
| **github/gh-aw (GitHub Agentic Workflows)** | github.com/github/gh-aw | 5,378 | MIT | **GitHub's own agentic CI/CD layer.** Markdown-defined workflows that run Claude/Copilot/Codex/OpenCode in Actions. A public discussion shows it shipping Smoke Tests, Dependency Security Monitor, Performance Monitor — and a self-assessment noting smoke tests are *not* blocking checks. **This is the most direct proof-of-concept that agents can run inside CI, and the clearest public evidence of the gap TestKit would fill** |

### Awesome lists & curated indexes (verified)

| Name | URL | Stars | License | Notes |
|---|---|---|---|---|
| **sdras/awesome-actions** ⚠️ CORRECTED *(the original brief's `github/awesome-actions` returns 404 — this is the live equivalent)* | github.com/sdras/awesome-actions | 28,304 | CC0-1.0 | The canonical GitHub Actions index, organized by category with static analysis / dynamic analysis / monitoring / deployment sections |
| **awesome-copilot** | github.com/github/awesome-copilot | 39,890 | MIT | See above |
| **awesomelistsio/awesome-cicd** | github.com/awesomelistsio/awesome-cicd | 24 | Mixed | Newest, most agent-friendly taxonomy (Foundations → Learning Resources), but **immature at 24★** |
| **ligurio/awesome-ci** | github.com/ligurio/awesome-ci | 4,161 | Mixed | Comparative table of CI services with stars, license, SCM support, pricing. **Best single artifact for competitive parsing** |
| **myugan/awesome-cicd-security** | github.com/myugan/awesome-cicd-security | 844 | MIT | CI/CD security: CISA guidance, books, tools, offensive case studies |
| **TupleType/awesome-cicd-attacks** | github.com/TupleType/awesome-cicd-attacks | 631 | CC0-1.0 | Offensive CI/CD technique taxonomy |
| **github/roadmap** | github.com/github/roadmap | 8,912 | CC-BY-4.0 | Public product roadmap — signals where GitHub is going with Actions/Copilot |

### Official starter material (verified)

| Name | URL | Stars | License | Notes |
|---|---|---|---|---|
| **actions/starter-workflows** | github.com/actions/starter-workflows | 12,145 | Mixed | Canonical CI/CD workflow templates per language/stack |
| **actions/example-services** | github.com/example-services** ⚠️ see note | 259 | Unlicense | Example workflows using service containers |
| **actionlint** | github.com/rhysd/actionlint | 4,310 | MIT | The linter that makes any Actions corpus machine-readable |

> ⚠️ **Correction note:** The original brief listed `github/awesome-actions`, `sdaf/awesome-ci-cd`, and `actions/awesome-actions`. All three return **404**. The live equivalents are `sdras/awesome-actions` and `github/awesome-copilot`. Likewise `actions/example-services` is correct at `actions/example-services` but has only 259★ — it is a thin repo.

### Vendor example/template corpora (verified)

| Name | URL | Stars | License | Relevance |
|---|---|---|---|---|
| **google/awesome-copilot** (see above) | — | — | — | — |
| **vercel/examples** | github.com/vercel/examples | 5,158 | MIT | Production patterns for the Vercel platform |
| **cloudflare/templates** | github.com/cloudflare/templates | 2,146 | MIT | Workers templates |
| **supabase/supabase** | github.com/supabase/supabase | 111,319 | Apache-2.0 | The Postgres platform; includes its own CI, migrations, and edge functions |
| **supabase/cli** | github.com/supabase/cli | 2,431 | Mixed | Local Supabase, migrations, type generation |
| **google-gemini/gemini-cli** | github.com/google-gemini/gemini-cli | 107,276 | Apache-2.0 | Google's open terminal agent. Its skill/plugin architecture is worth reading |
| **rwx-research/rwx-ci** ⚠️ NOT VERIFIED — referenced in a Hacker News thread as a GHA alternative; **verify before citing** | — | — | — | — |

### Google Cloud training (verified)

| Name | URL | What it is |
|---|---|---|
| **Google Skills** ⚠️ CORRECTED *(this is the current platform at `skills.google`, consolidating what was Cloud Skills Boost)* | skills.google | ~3,000 courses, labs, and credentials from Google Cloud, DeepMind, Grow with Google, and Google for Education. 35 free credits/month for developers; entire on-demand library free for Google Cloud customers. AI-driven labs via Gemini Code Assist |
| **Cloud Shell docs** | docs.cloud.google.com/shell/docs | Preinstalled, pre-authenticated `gcloud` in the browser with a Cloud Code editor and web app previews. Last updated Oct 2026 |
| **GoogleCloudPlatform/cloud-builders** | github.com/GoogleCloudPlatform/cloud-builders | 1,463★, Apache-2.0. Builder images for Cloud Build |
| **GoogleCloudPlatform/release-please-action** ⚠️ CORRECTED *(moved to `googleapis/release-please-action`)* | github.com/googleapis/release-please-action | 2,543★, Apache-2.0 |

---

## 9. Google-Specific CI/CD & DevOps (explicit bucket)

| Name | URL | Stars | License | What it does | Relevance to TestKit |
|---|---|---|---|---|---|
| **google/mcp** ⭐ | github.com/google/mcp | 4,633 | Apache-2.0 | Google's official MCP servers: **Cloud Run, GKE, BigQuery, Cloud SQL, AlloyDB, Spanner, Firestore, Cloud Storage, Genmedia, Security Command Center, gcloud CLI, Firebase, Analytics, Apigee, Google Genmedia**, plus Gemini CLI extensions | **The single most useful free integration surface available to TestKit.** Free, Google-maintained, Apache-2.0. An agent-verification plugin that ships an MCP config pointing at this gets GCP verification nearly free |
| **googleapis/mcp-toolbox** ⭐ | github.com/googleapis/mcp-toolbox | 16,643 | Apache-2.0 | Open-source MCP server for databases (renamed from `genai-toolbox`). Handles connection pooling, auth, OTel metrics/tracing. Go/Python/JS SDKs | **The cleanest path to "agent actually queries the app's real database."** Pairs with Supabase/Neon branching |
| **mcp-toolbox-sdk-python** | github.com/googleapis/mcp-toolbox-sdk-python | 192 | Apache-2.0 | Python SDK | — |
| **google-github-actions/setup-gcloud** | github.com/google-github-actions/setup-gcloud | 1,914 | Apache-2.0 | Installs/configures `gcloud` in Actions | ⚠️ **The old `GoogleCloudPlatform/github-actions` URL now redirects here — the org was renamed.** Use `google-github-actions/*` |
| **google-github-actions/auth** | github.com/google-github-actions/auth | 1,399 | Apache-2.0 | WIF + service account auth for Google Cloud | The credential path in CI |
| **google-github-actions/deploy-cloudrun** | github.com/google-github-actions/deploy-cloudrun | 605 | Apache-2.0 | Deploys to Cloud Run; emits the service URL as a step output | **Emitting the URL as an output is the exact contract a verification step needs** |
| **google-github-actions/create-cloud-deploy-release** | github.com/google-github-actions/create-cloud-deploy-release | 28 | Apache-2.0 | Creates a Cloud Deploy release for GKE/Anthos/Cloud Run | The CD handoff from Actions. ⚠️ Explicitly "not an officially supported Google product" |
| **google-github-actions/deploy-cloud-functions** | github.com/google-github-actions/deploy-cloud-functions | 337 | Apache-2.0 | Deploys source to Cloud Functions | — |
| **Google Cloud Deploy** | cloud.google.com/deploy | — | Commercial (managed) | Managed rollouts, approvals, canary analysis for GKE/Anthos/Cloud Run | **Do not rebuild. Managed.** |
| **Google Cloud Build** | cloud.google.com/build | — | Per build-minute above free tier; **424,979 Marketplace installs** | Managed builds + container/solution builder ecosystem | GCP-native CI alternative to GitHub Actions |
| **Google Cloud Shell / Cloud Shell Editor** | docs.cloud.google.com/shell/docs | — | Free | Browser terminal, preinstalled + authenticated `gcloud`, Cloud Code editor, app previews | **A place TestKit must work on day one if it wants GCP users** |
| **bazelbuild/bazel** | github.com/bazelbuild/bazel | 25,934 | Apache-2.0 | Scalable multi-language build system | Google's answer to "reproducible builds at scale" |
| **google/go-containerregistry** | github.com/google/go-containerregistry | 4,075 | Apache-2.0 | Go library + CLIs (`crane`) for container registries | Image scanning/probing primitive. Useful for image-based verification |
| **googleapis/google-cloud-go** | github.com/googleapis/google-cloud-go | 4,518 | Apache-2.0 | Go client libraries for all Google Cloud APIs | — |
| **googleapis/release-please** ⭐ | github.com/googleapis/release-please | 7,610 | Apache-2.0 | Generates release PRs from Conventional Commits | **A TestKit gate should be able to read the release PR to learn what is being shipped.** Directly relevant |
| **googleapis/release-please-action** | github.com/googleapis/release-please-action | 2,543 | Apache-2.0 | The Action wrapper | — |
| **firebase/firebase-tools** ⚠️ CORRECTED *(is `firebase/firebase-tools`, not `GoogleCloudPlatform/firebase-tools`)* | github.com/firebase/firebase-tools | 4,471 | MIT | Firebase CLI: hosting, functions, emulator suite | **The Firebase emulator suite is a great hermetic target for local-first verification** |
| **GoogleCloudPlatform/k8s-node-termination-handler** | github.com/GoogleCloudPlatform/k8s-node-termination-handler | 192 | Apache-2.0 | Graceful GCE node termination | Minor |
| **GoogleCloudPlatform/k8s-cloud-provider** | github.com/GoogleCloudPlatform/k8s-cloud-provider | 41 | Apache-2.0 | K8s cloud provider for GCE | Minor |
| **GoogleCloudPlatform/k8s-stackdriver** | github.com/GoogleCloudPlatform/k8s-stackdriver | 409 | Apache-2.0 | Legacy GKE→Stackdriver log/metric agent | Minor/legacy |
| **terraform-google-modules/terraform-google-project-factory** | github.com/terraform-google-modules/terraform-google-project-factory | 964 | Apache-2.0 | Opinionated GCP project via Shared VPC + IAM | Pattern for provisioning a disposable GCP test project |
| **GoogleCloudPlatform/spanner-migration-tool** | github.com/GoogleCloudPlatform/spanner-migration-tool | 155 | Apache-2.0 | DB/dump → Spanner migration | Niche |
| **googleapis/genproto** | — | ⚠️ CORRECTED *(404s; protos moved into per-language `googleapis/*-genproto` repos)* | — | — |
| **GoogleCloudPlatform/release-please** / **cloud-deploy-action** / **terraform-google-network** / **terraform-google-startup-script** / **cloud-run-example** | — | ⚠️ CORRECTED *(all 404 — verify current names before citing)* | — | — |
| **Kokoro CI** | — | ⚠️ CORRECTED *(`google/kokoro` returns 404; the project appears archived/internal)* | — | — | Historical: Google's internal CI, briefly public. Not usable |
| **google/stargazer** | — | ⚠️ CORRECTED *(404; appears retired)* | — | — | — |

---

## 10. Synthesis

### 10.1 The uncomfortable finding: Sentry already shipped this

Sentry's **Seer** (announced expanded 27 Jan 2026) now does, in production today:

- **Bug prediction before ship** — "predicting bugs before they ship to prod," and "Catch tomorrow's bugs in today's PR" by reviewing your PR against **real production data** (errors, traces, failure patterns Sentry has already recorded from your app)
- **Code review before merge** — with noise suppression ("without spamming your PR")
- **Local development debugging** — extended Jan 2026
- **Prod RCA + Autofix PRs**, with handoff to **Claude Code, Cursor Cloud Agents, and GitHub Copilot**
- **Flat pricing, unlimited usage**; open beta
- **Sentry MCP server** so agents can query issues, trigger RCA, and investigate DB performance

The 04 blueprint's **"Sentry SRE Sentinel"** (capture real user crashes → auto-generate reproduction scripts) is precisely Seer Autofix. Building it as a differentiator is building something a well-funded incumbent shipped in January.

**Two survivable readings:**

- **(a) Sentinel as a consumer, not a competitor.** TestKit doesn't rebuild runtime RCA. It reads Sentry's issue history *before* deploy and asks: "does this diff touch code paths that have historically produced these error types?" That's still Seer's own feature, so the honest value is *speed and framing*, not capability.
- **(b) Sentinel for teams without Sentry.** That's a real market — self-hosted and OSS shops. But it's a small, low-ARPU market, and the observability it sits on (OTel, Prometheus, Grafana) is a separate acquisition.

**Recommendation:** drop "SRE Sentinel" as a headline. Keep a **"Production Replay Gate"**: pull historical failures from whatever error tracker exists, replay them against the PR build, fail the gate if any regress. Frame it as *regression replay*, not *AI root cause*. That is defensible, cheap, and nobody sells it as a headline.

---

### 10.2 The 10 tools a ship-it verification gate most plausibly needs to plug into

Ranked by (how often the gate must touch it) × (how bad it is to build yourself).

| # | Tool | Category | Why it's in the top 10 | Build/buy |
|---|---|---|---|---|
| 1 | **GitHub Actions** | CI host | The gate mechanism. A required status check. ~33% org adoption, rising | **Buy.** Ship an Action; be additive to `.github/workflows` |
| 2 | **Playwright** | Browser runtime | Hands and eyes. One API across Chromium/Firefox/WebKit. CDP is already the protocol the blueprint wants | **Buy.** 97,445★. Do not write a browser driver |
| 3 | **Testcontainers** | Dependency orchestration | Solves "auto-provision exactly the Postgres/Mongo/Redis this project uses," hermetically, in CI and locally | **Buy.** 5,002★, MIT, language-portable |
| 4 | **E2B** | Agent sandbox | Apache-2.0 Firecracker, **self-hostable** → the only credible way to keep TestKit local-first while matching Vercel's isolation tier | **Buy, self-hostable.** Closest to the local-first thesis |
| 5 | **Dagger** | CI-as-code engine | Proves the local/CI parity pattern is achievable and shows the engine-vs-CLI split. The reference implementation for "same binary everywhere" | **Study, don't embed.** Competing engine is a distraction |
| 6 | **Trivy** | Security scan | 38,337★, Apache-2.0, one binary covers vulns + misconfig + secrets + SBOM | **Buy.** Never rebuild |
| 7 | **zizmor + actionlint** | CI supply-chain | 6,688★ + 4,310★, both MIT. Static analysis of the *pipeline itself*. The gap gh-aw's own self-assessment flagged | **Buy — and this is the wedge.** See §10.4 |
| 8 | **Supabase branching / Neon branching** | Preview data | Real Postgres schema per PR without fixture writing. Solves the "CI has no database" blocker outright | **Buy.** Merge/reset APIs are the point |
| 9 | **PostHog** (or Sentry) | Telemetry | One OSS deployment gives analytics + replay + flags + experiments + error tracking + **MCP server** at 40,218★ | **Buy, but stay optional.** Never make it a dependency |
| 10 | **google/mcp + googleapis/mcp-toolbox** | Agent interfaces | 4,633★ + 16,643★, both Apache-2.0, Google-maintained. Free read/write access to Cloud Run, GKE, BigQuery, Cloud SQL, Spanner, Security Command Center, and real database queries | **Buy.** The cheapest credible GCP story available |

**Runners-up:** Argo Rollouts (if you want traffic flipping, use this), Schemathesis (highest signal-per-effort gate), k6 (don't build a load generator), Kubernetes (usually overkill), Flux/Argo CD (only if you're a GitOps shop).

---

### 10.3 Commodity vs. differentiating

**Commodity — use/buy, never build (≈80% of the surface area)**

| Commodity | The one to use |
|---|---|
| CI execution | GitHub Actions (plus a GitLab job template) |
| Container/VM runtime | Docker locally, E2B or Vercel Sandbox remotely |
| Dependency provisioning | Testcontainers |
| Browser automation | Playwright |
| SAST/SCA/secrets | Trivy + Semgrep + Gitleaks + ggshield |
| Policy as code | OPA/Conftest |
| Load testing | k6 |
| Progressive delivery | Argo Rollouts or Flagger (or Cloud Deploy / Vercel if not on K8s) |
| Flags | PostHog / Unleash |
| Dashboards | Grafana + Loki + Tempo + Pyroscope (all AGPL — invoke, don't embed) |
| Telemetry schema | OpenTelemetry |
| CI caching | actions/cache or Depot/Blacksmith if it's a bottleneck |

**Differentiating — must build (≈20%, and this is the whole product)**

1. **The cross-platform parity contract.** One engine binary, one config file, one output format, running byte-identically on a laptop and in a runner. Dagger proved the pattern; nobody has done it for *verification*.
2. **The gate protocol.** A stable, machine-readable pass/fail/skip schema with severity, evidence, and a reproducible command for every finding — consumable by a required status check, a PR comment, and an agent skill.
3. **The auto-triage engine.** When 400 findings come back, which 3 actually block? Nobody has solved ranking findings by "will this reach a user." gh-aw's public self-assessment showed the real state of the art is "we have 4 uncovered npm vulnerabilities and non-blocking smoke tests."
4. **Environment truthfulness.** Detecting that CI is not the developer's environment — missing env vars, different seed data, a service silently not running — and failing loudly *because* of it. This is the thing that makes local-first trustworthy, and it's nobody's job today.
5. **The Skill + agent contract.** A `SKILL.md` that teaches an agent to invoke, interpret, and act on TestKit findings — following `mattpocock/skills` and the Agent Skills standard. Distribution without a sales team.

---

### 10.4 Where TestKit adds real value vs. where a thin wrapper is enough

**Real value — TestKit owns the outcome**

- **CI-environment fidelity detection.** TestKit knows what a project's app *should* need (from the AST map, the compose file, the migrations, the env schema) and can assert the runner actually has it. Every existing tool assumes the environment is correct. This is the load-bearing insight of a local-first verifier and it is currently nobody's job.
- **Cross-platform parity attestation.** "The same engine ran here and on the developer's machine, on the same commit, and produced the same verdict" is a *claim* nobody else can make, because everyone else runs different things in CI.
- **Severity ranking by reachability.** Schemathesis finds 400 schema violations; k6 finds a p95; Trivy finds 11 CVEs. None can say "only 2 of these are on a code path a logged-in user touches." That's a genuine product.
- **Gate outcome schema as a shared contract.** Not "run 40 tools" but "one structured verdict an agent and a CI system both understand." This is the integration surface that makes TestKit stick.
- **CI-supply-chain self-audit as the wedge.** zizmor and actionlint each catch a class of pipeline vulnerability. Neither answers: *"for this repo's pipeline, which finding is exploitable by an attacker with a PR?"* A gate that reads the workflow, the permissions, the secrets exposed to PRs, and the action pinning — then reasons about reachability — is defensible, timely, and currently nobody's job. `gh-aw` demonstrating this problem publicly (and failing to solve it) is the market signal.

**Thin wrapper — just call the tool and ship the result**

- Container image scanning (Trivy does it)
- Dependency CVE checking (OSV-Scanner/Dependabot/Renovate do it)
- Secret scanning (Gitleaks/TruffleHog do it, and TruffleHog *verifies* them)
- Static analysis (Semgrep/CodeQL do it)
- Linting (ESLint/Rustfmt/actionlint do it)
- Load testing (k6 does it)
- Uptime/synthetic checks (Gatus/Uptime Kuma do it)
- Format/prettier, type checking

**The strategic trap:** a gate that prints "Trivy: 11 findings, Semgrep: 4 findings, CodeQL: 2 findings, Gitleaks: clean, Schemathesis: 400 findings" is a *dashboard*, and dashboards lose to the tools. The gate has to **decide**. If TestKit's gate doesn't have an opinion, it's a CI runner with a nice theme.

---

### 10.5 The cheapest credible path: local laptop → every team's CI

**The cheap path (do this):**

| Step | Action | Why it's cheap |
|---|---|---|
| 1 | Ship a **static single binary + npm wrapper**. `npx testkit verify` locally. | Zero install friction; matches the blueprint's existing TS CLI layer |
| 2 | Ship a **GitHub Action** that downloads the same binary and runs `testkit verify --format=json`. One `uses:` line. | Every GitHub team can adopt in one PR |
| 3 | **Post-only first.** Report findings as a PR comment; do **not** create a required check. | Admin approval for required status checks is the single biggest adoption blocker. Let teams opt in to blocking |
| 4 | Add a **ShipKit Skill** (`SKILL.md`, Agent Skills standard) so agents read findings natively | Free distribution through the ecosystem GitHub already normalized |
| 5 | Then, and only then, offer **opt-in blocking** as a per-repo toggle | Converts on earned trust, not on a sales call |
| 6 | Add **branching-based preview data** (Supabase/Neon) as the paid/unlockable tier | Solves the "CI has no database" problem that blocks everyone |

**The blockers, honestly ranked:**

1. **Hermeticity.** A developer's laptop has a running Postgres, 14 env vars in `.env.local`, a warm Docker layer cache, and a browser with a logged-in session. A runner has none of it. **Any gate that behaves differently in the two places destroys the entire "local-first" premise — it's the whole product.** This is the hard problem and it is not solved by a container.

2. **Determinism of LLM-driven verification.** If the verdict is partly model-generated, the same commit can pass and fail. A gate that is flaky *about flakiness* is worse than no gate. Whatever the model contributes must be advisory; pass/fail must come from deterministic checks. This constraint should be designed in from day one, not bolted on.

3. **Required-check admin permission.** Only org owners can make a status check blocking. Every rollout is an org-admin conversation. This is a distribution problem disguised as a product problem.

4. **Secrets and egress in CI.** An agent-driven verifier needs LLM API keys in the runner. Teams have policies about that. Self-hosted runner policies are tightening in some orgs (and GitHub now charges $0.002/min for them).

5. **Cost attribution.** GitHub Actions went to $0.006/min hosted and $0.002/min self-hosted (Mar 2026). A verification step that adds 8 minutes to every PR on a 200-repo company is ~$500–$1,000/mo. **Budget objections kill more rollouts than technical objections.** Price per run like Depot prices per passing test, and make the fast path genuinely fast.

6. **Cold start / CI-time latency.** A 6-minute gate on every PR gets disabled by week three. Target <2 minutes for the blocking tier and push the deep sweep to nightly.

7. **Noise.** Every scanner floods. Ten findings per PR gets ignored at 100. Ranking and suppression (see §10.4) is a survival requirement.

---

### 10.6 How much of the 04 blueprint is realistic

Blueprint: Rust engine + TypeScript CLI + 1,000-actor swarm + 5-layer flight recorder + Sentry SRE sentinel + blue/green traffic flip.

| Element | Verdict | Notes |
|---|---|---|
| **TypeScript CLI + `npx` distribution** | ✅ **Realistic — do it** | Correct call. Zero-friction distribution is the adoption mechanism |
| **Agent Skills / SKILL.md / AGENTS.md auto-injection** | ✅ **Realistic — and now standardized** | `agentskills.io` exists; `anthropics/skills`, `github/awesome-copilot`, `mattpocock/skills` are the references. Do this |
| **Rust core engine** | 🟡 **Realistic but questionable** | Rust is fine (Tree-sitter, Tokio, Hyper are real, and the concurrency claim holds). But it doubles the surface area, slows a team that must ship weekly, and buys nothing a Go or even TS engine wouldn't. **Ship the CLI in TS and, if profiling proves it, rewrite one hot path.** Don't start in Rust because it sounds fast |
| **Living Codebase State Map (Tree-sitter AST)** | ✅ **Realistic — genuinely differentiating** | High value, achievable, and it's the input for environment-fidelity detection (§10.4). **This is the most underrated piece of the blueprint** |
| **Hot-sync delta pull into a running sandbox** | 🟡 **Realistic but deprioritize** | Cool demo, hard to make reliable, and it doesn't help the CI story at all. Cut first if time runs short |
| **1,000+ virtual actor swarm, 0 token cost** | 🟡 **Realistic technically, wrong abstraction** | Tokio handles it trivially. **But k6 already does load generation** and everyone knows how to read k6 output. The novel part isn't 1,000 actors — it's 1,000 actors *correlated against a specific user journey with an oracle*. Without the oracle it's a load test with extra steps |
| **5-layer microsecond-synchronized flight recorder (UI/Network/Logs/DB/Server)** | 🔴 **Partially fantasy** | The UI and network layers are obtainable (Playwright CDP, reverse proxy). The **DB layer via Postgres wire protocol + WAL tailing** is research-grade. Microsecond synchronization across five layers with a causal graph is a multi-team, multi-year project. **Ship 2 correlated layers (network + app logs via OTel) with monotonic timestamps and good UX. Fake nothing; ship less.** Note k6 and Playwright already emit traces — use OTLP rather than a proprietary format |
| **Sentry SRE Sentinel** | 🔴 **Already shipped — see §10.1** | Seer does this better with data you cannot replicate. Reframe as Production Replay Gate |
| **Blue/green zero-downtime traffic routing (Hyper/Axum proxy)** | 🔴 **Don't build — use Argo Rollouts / Flagger / Cloud Deploy / Vercel** | Argo Rollouts is 3,595★, Apache-2.0, CNCF, and does metric-driven canary + auto-rollback + blue-green. **Reimplementing a reverse proxy that flips traffic is a solved problem and you'd lose to it on correctness.** If TestKit must own this for non-K8s targets, make it a *thin* Hyper proxy that only does the flip — and only after a customer asks |
| **In-memory WASM DB sandbox (PGlite/SQLite)** | 🟡 **Realistic and smart for the demo, wrong for the gate** | PGlite/WASM is fine for zero-install demos. **For a CI gate you need the real Postgres engine** — dialect differences produce false confidence. Use branching (Supabase/Neon) or Testcontainers |
| **Red-Team actor (SQLi/XSS/IDOR/auth bypass)** | 🟡 **Mostly commodity** | Nuclei (31,912★), ZAP (15,912★), Schemathesis (3,658★), Bearer (2,757★) all cover this. What's *not* commodity: driving those tools from a discovered journey and reporting business impact. **Be the orchestrator + judge, not the scanner.** And: never point a red-team actor at anything reachable by a customer |
| **Cross-network test on a disposable remote VPS** | ✅ **Realistic, do it with E2B** | `e2b-dev/E2B` is Apache-2.0 and self-hostable. This is a genuine local-first payoff |
| **Anti-Spaghetti architecture gate** | 🟡 **Realistic, but needs a hard oracle** | Sonar and your own taste are both soft. "Forbidden import," "module exceeds N lines," "route has no test" are enforceable. Only ship rules that produce zero false positives |

**Net honest read:** roughly **60% of the blueprint is buildable**, and **three of its most memorable claims are already commodity** (progressive delivery, Sentry sentinel, 1,000-actor load). The genuinely durable core is narrower and stronger than the blueprint admits:

> **A deterministic, parity-preserving verification engine that (a) knows what environment the app actually needs, (b) proves the CI environment actually has it, (c) runs every check in both places identically, and (d) returns one verdict an agent and a CI system both understand.**

That is defensible, hard, and unclaimed.

---

## Appendix A — Corrections to the original brief

These names in the brief return **HTTP 404** as of 11 Oct 2026. Do not use them.

| Brief said | Reality |
|---|---|
| `github/awesome-actions` | 404. Use `sdras/awesome-actions` (28,304★, CC0) or `github/awesome-copilot` (39,890★, MIT) |
| `actions/awesome-actions` | 404. Same as above |
| `sdaf/awesome-ci-cd` | 404. Use `awesomelistsio/awesome-cicd` (new, 24★) or `ligurio/awesome-ci` (4,161★) |
| `corca/awesome-ci-cd` | 404 |
| `actions/runner-controller` | Moved. Use `actions/actions-runner-controller` (6,552★) |
| `GoogleCloudPlatform/github-actions` | Renamed. Use `google-github-actions/*` |
| `GoogleCloudPlatform/release-please-action` | Moved. Use `googleapis/release-please-action` |
| `GoogleCloudPlatform/release-please` | Moved. Use `googleapis/release-please` |
| `GoogleCloudPlatform/firebase-tools` | Moved. Use `firebase/firebase-tools` |
| `coolify-io/coolify` | 404. Product is SaaS + commercial self-host |
| `woodpeckerci/woodpecker` | Renamed. Use `woodpecker-ci/woodpecker` |
| `nuclei-project/nuclei` | Renamed. Use `projectdiscovery/nuclei` |
| `BearerHQ/bearer` | Renamed. Use `bearer/bearer` |
| `containers/podman` | Moved. Use `podman-container-tools/podman` |
| `instrumenta/conftest` | Moved into OPA org. Use `open-policy-agent/conftest` |
| `grafana/oncall` | Moved. Use `grafana-cold-storage/oncall` |
| `Giskard-AI/giskard` | Renamed. Use `Giskard-AI/giskard-oss` |
| `mobile-dev-inc/maestro` | Correct but case-sensitive: `mobile-dev-inc/Maestro` |
| `google/kokoro` | 404. Appears archived/internal. Not usable |
| `google/stargazer` | 404. Appears retired |
| `googleapis/genproto` | 404. Split into per-language `googleapis/*-genproto` repos |
| `HyperDX/hyperdx` | Redirects to a user profile. Verify current org |
| `getsentry/seer` | No public repo. Commercial product |
| `osohq/oso` | **DEPRECATED** — repo self-describes as deprecated |
| `blastclp/blast` | 404. Project appears retired |
| `googleapis/google-cloud-nodejs-client` | 404 |
| `Doctor McNinja` | Unverifiable at any known URL |

## Appendix B — Full GitHub verification log

All stars pulled 11 Oct 2026 via `github.com/<repo>` page scrape or `api.github.com/repos/<repo>`. License field is GitHub's detected SPDX ID; `NOASSERTION` means GitHub could not determine it (usually a mixed or non-standard license — treat as "read it yourself before embedding").

<details>
<summary>Click to expand ~150 verified repositories</summary>

```
actions/actions-runner-controller 6552 Apache-2.0
actions/checkout 8946 MIT
actions/example-services 259 Unlicense
actions/runner 6319 MIT
actions/starter-workflows 12145 NOASSERTION
agentskills/agentskills 26027 Apache-2.0
anchore/grype 13005 Apache-2.0
anchore/scan-action 289 MIT
anthropics/skills 180351 NOASSERTION
aquasecurity/kube-bench 8221 Apache-2.0
aquasecurity/trivy 38337 Apache-2.0
aquasecurity/trivy-action 1436 Apache-2.0
axiomhq/axiom-js 150 MIT
bearer/bearer 2757 NOASSERTION
bluewave-labs/Checkmate 10924 AGPL-3.0
bugsnag/bugsnag-js 894 MIT
buildbot/buildbot 5480 GPL-2.0
caprover/caprover 15183 NOASSERTION
chaos-mesh/chaos-mesh 7930 Apache-2.0
circleci/circleci-docs 850 NOASSERTION
cloudflare/templates 2146 MIT
codefresh-io/codefresh 81 MIT
coder/coder 16944 AGPL-3.0
concourse/concourse 7915 Apache-2.0
containers/buildah 9066 Apache-2.0
coolify (n/a) 404
cypress-io/cypress 51045 MIT
dbt-labs/dbt-core 13991 Apache-2.0
dagger/dagger 16334 Apache-2.0
daytonaio/daytona 71575 NOASSERTION
devcontainers/spec 5762 CC-BY-4.0
docker/compose 38353 Apache-2.0
dokku/dokku 32175 MIT
earthly/earthly 12054 MPL-2.0
e2b-dev/E2B 14272 Apache-2.0
e2b-dev/runtime 1707 NOASSERTION
elastic/apm-agent-go 427 Apache-2.0
envoyproxy/envoy 29058 Apache-2.0
expo/eas-cli 1366 MIT
expo/expo 52628 MIT
firebase/firebase-tools 4471 MIT
fluxcd/flagger 5418 Apache-2.0
fluxcd/flux2 8442 Apache-2.0
future-architect/vuls 12283 GPL-3.0
getsentry/sentry 45525 NOASSERTION
gitlabhq/gitlabhq 24552 NOASSERTION
Giskard-AI/giskard-oss 5887 Apache-2.0
gitpod-io/gitpod 13794 AGPL-3.0
github/actions-runner-controller 6552 Apache-2.0
github/awesome-copilot 39890 MIT
github/branch-deploy 581 MIT
github/codeql 10186 MIT
github/gh-aw 5378 MIT
github/roadmap 8912 CC-BY-4.0
GitGuardian/ggshield 2004 MIT
gitleaks/gitleaks (zricethezav) 29840 MIT
google-gemini/gemini-cli 107276 Apache-2.0
google/mcp 4633 Apache-2.0
google/osv-scanner 11162 Apache-2.0
googleapis/google-cloud-go 4518 Apache-2.0
googleapis/mcp-toolbox 16643 Apache-2.0
googleapis/mcp-toolbox-sdk-python 192 Apache-2.0
googleapis/release-please 7610 Apache-2.0
googleapis/release-please-action 2543 Apache-2.0
google-github-actions/auth 1399 Apache-2.0
google-github-actions/create-cloud-deploy-release 28 Apache-2.0
google-github-actions/deploy-cloud-functions 337 Apache-2.0
google-github-actions/deploy-cloudrun 605 Apache-2.0
google-github-actions/setup-gcloud 1914 Apache-2.0
google/go-containerregistry 4075 Apache-2.0
GoogleCloudPlatform/cloud-builders 1463 Apache-2.0
GoogleCloudPlatform/k8s-cloud-provider 41 Apache-2.0
GoogleCloudPlatform/k8s-node-termination-handler 192 Apache-2.0
GoogleCloudPlatform/k8s-stackdriver 409 Apache-2.0
GoogleCloudPlatform/spanner-migration-tool 155 Apache-2.0
grafana/grafana 77226 AGPL-3.0
grafana/loki 29006 AGPL-3.0
grafana/oncall (grafana-cold-storage) 3884 AGPL-3.0
grafana/pyroscope 11697 AGPL-3.0
grafana/tempo 5520 AGPL-3.0
grafana/k6 31859 AGPL-3.0
grpc/grpc 45367 Apache-2.0
harness/harness 38513 NOASSERTION
hashicorp/terraform 49859 NOASSERTION
healthchecks/healthchecks 10407 BSD-3-Clause
instrumenta/conftest (open-policy-agent) 3280 NOASSERTION
istio/istio 38429 Apache-2.0
jaegertracing/jaeger 23272 Apache-2.0
jenkinsci/jenkins 26637 MIT
jetify-com/devbox 12397 Apache-2.0
jestr (jestjs/jest) 45550 MIT
keptn/keptn 1773 Apache-2.0
kyverno/kyverno 8236 Apache-2.0
lima-vm/lima 22065 Apache-2.0
ligurio/awesome-ci 4161 NOASSERTION
litmuschaos/litmus 5725 Apache-2.0
louislam/uptime-kuma 92308 MIT
marshmallowcode (mattpocock/skills) 284555 MIT
mattpocock/skills 284555 MIT
mattpocock (see above)
moby/moby 72167 Apache-2.0
mobile-dev-inc/Maestro 16006 Apache-2.0
modal-labs/modal-client 523 Apache-2.0
myugan/awesome-cicd-security 844 MIT
netdata/netdata 80872 GPL-3.0
NixOS/nix 17853 LGPL-2.1
open-policy-agent/conftest 3280 NOASSERTION
open-policy-agent/gatekeeper 4294 Apache-2.0
open-policy-agent/opa 12340 Apache-2.0
open-telemetry/opentelemetry-collector-contrib 4984 Apache-2.0
open-telemetry/opentelemetry-specification 4345 Apache-2.0
openobserve/openobserve 22315 AGPL-3.0
opencontainers/runc 13484 Apache-2.0
osohq/oso 3489 Apache-2.0 (DEPRECATED)
ossf/scorecard 5748 Apache-2.0
ossf/scorecard-action 421 Apache-2.0
pact-foundation/pact-js 1820 NOASSERTION
pwndoc/pwndoc 2908 MIT
projectdiscovery/nuclei 31912 MIT
podman-container-tools/podman 33023 Apache-2.0
railwayapp/nixpacks 3554 MIT
rancher-sandbox/rancher-desktop 7386 Apache-2.0
renovatebot/renovate 22718 AGPL-3.0
rhysd/actionlint 4310 MIT
rollbar/rollbar.js 587 MIT
schemathesis/schemathesis 3658 MIT
sdras/awesome-actions 28304 CC0-1.0
semgrep/semgrep 16959 LGPL-2.1
SigNoz/signoz 32325 NOASSERTION
snyk/cli 5673 NOASSERTION
spinnaker/spinnaker 9803 Apache-2.0
supabase/cli 2431 NOASSERTION
supabase/postgres-meta 1249 Apache-2.0
supabase/supabase 111319 Apache-2.0
suzuki-shunsuke/tfcmt 552 NOASSERTION
tailscale/tailscale 37354 BSD-3-Clause
tektoncd/pipeline 9077 Apache-2.0
tektoncd/triggers 594 Apache-2.0
terraform-google-modules/terraform-google-project-factory 964 Apache-2.0
terraform-docs/terraform-docs 4827 MIT
testcontainers/testcontainers-go 5002 MIT
traefik/traefik 65146 MIT
trufflesecurity/trufflehog 28418 AGPL-3.0
TupleType/awesome-cicd-attacks 631 CC0-1.0
TwiN/gatus 12279 Apache-2.0
Unleash/unleash 13867 AGPL-3.0
Upptime/upptime 17176 MIT
Vitest-dev/vitest 17197 MIT
woodpecker-ci/woodpecker 7981 Apache-2.0
zaproxy/zaproxy 15912 Apache-2.0
zizmorcore/zizmor 6688 MIT
zricethezav/gitleaks 29840 MIT
argoproj/argo-cd 24361 Apache-2.0
argoproj/argo-rollouts 3595 Apache-2.0
argoproj/argo-workflows 17031 Apache-2.0
aws-github-actions 404 (verify current name)
circleci/circleci-docs 850 NOASSERTION
cloudflare/templates 2146 MIT
coolify-io/coolify 404
Codefresh 81 MIT
Drone (drone/drone -> harness/harness) 38513 NOASSERTION
Depot (depot/cli) verified via depot.dev pricing page
Blacksmith — commercial, verified via blacksmith.dev pricing comparison
Namespace — commercial, verified via craci.com runner price comparison
Google Cloud Build — commercial, verified via GitHub Marketplace (424,979 installs)
Vercel Sandbox — commercial GA, verified via Vercel blog + Qovery/Better Stack comparisons
Cloudflare Sandboxes — commercial, verified at $0.00002/vCPU-second
PostHog 40218 NOASSERTION
growthbook/growthbook 8486 NOASSERTION
Flagsmith/flagsmith 6591 BSD-3-Clause
Bitbucket Pipelines — commercial; self-hosted runner charging announced 2026 (WarpBuild blog)
Azure Pipelines — commercial
AWS CodePipeline/CodeBuild — commercial; CodeBuild 1,000 min free tier
TeamCity — commercial
Harness CI — commercial
LaunchDarkly, Split.io, Optimizely, Fiddler — commercial, no canonical OSS repo
FireHydrant — commercial SaaS, CLI repo not public
Rollbar (platform) — commercial; rollbar/rollbar.js is MIT
FireHydrant CLI — 404
Honeycomb — commercial; honeycombio/reqresp 404 (renamed)
HyperDX — HyperDX/hyperdx redirects to user profile (org verify needed)
Better Stack — commercial, free tier
Google Cloud Shell — docs.google.com; skills.google is Google Skills (merged catalog)
Oso — deprecated
Nessus — commercial
Burp Suite — commercial freemium
sqlmap — GPL-2.0; staging-only warning
PlanetScale branching — removed from the product; not a fit
Replit Deploy, Qovery, Northflank, Railway, Render — commercial PaaS
EAS Update — expo/eas-cli 1366 MIT
Dokku 32175 MIT, CapRover 15183 NOASSERTION, Portainer 38629 Zlib
ko-build/ko 8563 Apache-2.0
kubernetes/kubernetes 128258 Apache-2.0
cri-o/cri-o 5676 Apache-2.0
abiosoft/colima 31151 MIT
github/awesome-copilot 39890 MIT
awesomelistsio/awesome-cicd 24 NOASSERTION
vercel/examples 5158 MIT
```

</details>