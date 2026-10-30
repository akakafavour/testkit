// Minimal demo CLI under TestKit verification. No dependencies.
//
// Time-travel contract: TestKit injects TESTKIT_FAKE_NOW into the environment
// of every process scenario. This CLI reads it instead of the wall clock.
// The trial end is fixed once per machine at first run (stored in a marker
// file next to this script) so a 30-day jump deterministically expires it.
const fs = require("node:fs");
const path = require("node:path");

const TRIAL_DAYS = 14;
const MARKER = path.join(__dirname, "trial-start.txt");

let trialStart;
if (fs.existsSync(MARKER)) {
  trialStart = new Date(fs.readFileSync(MARKER, "utf8").trim());
} else {
  trialStart = new Date();
  fs.writeFileSync(MARKER, trialStart.toISOString());
}
const trialEnd = trialStart.getTime() + TRIAL_DAYS * 24 * 60 * 60 * 1000;

const injected = process.env.TESTKIT_FAKE_NOW;
const now = injected ? new Date(injected) : new Date();
if (Number.isNaN(now.getTime())) {
  console.error(`invalid TESTKIT_FAKE_NOW: ${injected}`);
  process.exit(2);
}

if (now.getTime() >= trialEnd) {
  console.log("EXPIRED");
  process.exit(1);
}
console.log("ACTIVE");
