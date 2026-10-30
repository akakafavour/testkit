// Minimal demo API under TestKit verification. No dependencies.
//
// Time-travel contract: TestKit sends the injected instant in the
// `X-Testkit-Fake-Now` header; this server reads it instead of the wall clock,
// so `time_jump` scenarios are deterministic. A real app would store the
// trial end in its database at signup; here the trial end is fixed once at
// server start, and the clock the app believes in moves with the header.
import { createServer } from "node:http";

const TRIAL_DAYS = 14;
const trialEnd = Date.now() + TRIAL_DAYS * 24 * 60 * 60 * 1000;

function anchorNow(request) {
  const injected = request.headers["x-testkit-fake-now"];
  if (injected) {
    const parsed = new Date(String(injected));
    if (!Number.isNaN(parsed.getTime())) return parsed;
  }
  return new Date();
}

const server = createServer((request, response) => {
  const url = new URL(request.url, "http://localhost");
  const now = anchorNow(request);

  if (url.pathname === "/api/health") {
    response.writeHead(200, { "content-type": "application/json" });
    response.end(JSON.stringify({ ok: true, now: now.toISOString() }));
    return;
  }

  if (url.pathname === "/api/trial") {
    if (now.getTime() >= trialEnd) {
      response.writeHead(403, { "content-type": "application/json" });
      response.end(JSON.stringify({ error: "trial_expired" }));
      return;
    }
    response.writeHead(200, { "content-type": "application/json" });
    response.end(
      JSON.stringify({
        plan: "trial",
        active: true,
        trial_ends_at: new Date(trialEnd).toISOString(),
      }),
    );
    return;
  }

  response.writeHead(404, { "content-type": "application/json" });
  response.end(JSON.stringify({ error: "not_found" }));
});

const port = Number(process.env.PORT ?? 8931);
server.listen(port, "127.0.0.1", () => {
  console.log(`mock-server listening on http://127.0.0.1:${port}`);
});
