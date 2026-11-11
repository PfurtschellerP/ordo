// Field-near test, large incident (dsn~large-incident~1, PERF-3): the load device fills an
// incident to 100 units, 50 sectors, 500 patients and 10,000 log entries over the sync
// protocol; then two tablets that have never seen it join and work in it.
import { spawn } from "node:child_process";
import path from "node:path";
import readline from "node:readline";
import { expect, Page, test } from "@playwright/test";
import { grantRole, loginPair, openIncident, openView, register } from "../support/app";
import { arrival, budgets, connect, device, jsHeap, mb, note, serverMemory, SITE, wasmMemory, WORK } from "./common";

const INCIDENT = "Großeinsatz";
const PATIENTS = 500;
const EVENTS = 10_000;
const repo = path.resolve(__dirname, "..", "..", "..");

/** Logs in at Dex's form (the address of a device pairing fills in the code). */
async function approve(page: Page, address: string, email: string): Promise<void> {
  await page.goto(address);
  await page.locator("#submit-login").click();
  await page.locator("#login").fill(email);
  await page.locator("#password").fill("password");
  await page.locator("#submit-login").click();
}

interface Loaded {
  written: number;
  confirmed: number;
  ms: number;
  byType: Record<string, number>;
}

/** Runs the load device; `onVerify` approves its pairing. */
function load(onVerify: (address: string) => Promise<void>): Promise<Loaded> {
  const bin = process.env.FIELD_LOAD_BIN ?? path.join(repo, "target", "debug", "ordo-load");
  const port = process.env.FIELD_ORDO_PORT ?? "3203";
  const child = spawn(bin, ["--server", `http://127.0.0.1:${port}`, "--incident", INCIDENT, "--profile", path.join(repo, "profiles", "de"), "--name", "Lastgerät", "--patients", String(PATIENTS), "--events", String(EVENTS)], { stdio: ["ignore", "pipe", "pipe"] });
  let errors = "";
  child.stderr.on("data", (d) => (errors += d));
  return new Promise((resolve, reject) => {
    let result: Loaded | undefined;
    readline.createInterface({ input: child.stdout }).on("line", (line) => {
      if (line.startsWith("verify ")) onVerify(line.slice(7)).catch(reject);
      if (line.startsWith("done ")) result = JSON.parse(line.slice(5));
    });
    child.on("exit", (code) => (code === 0 && result ? resolve(result) : reject(new Error(`ordo-load exited with ${code}: ${errors}`))));
  });
}

/** How long a fresh tablet takes to show the whole incident, in ms. */
async function join(page: Page, email: string, name: string): Promise<number> {
  await loginPair(page, SITE, email, name);
  const start = Date.now();
  await openIncident(page, INCIDENT);
  await openView(page, "Patienten");
  await expect(page.locator(`.patient-card[data-tag='GE-${String(PATIENTS).padStart(4, "0")}']`)).toBeVisible({ timeout: 120_000 });
  await expect(page.locator(".patient-list .patient-card")).toHaveCount(PATIENTS);
  return Date.now() - start;
}

// [itest->dsn~large-incident~1]
test("a large incident: 500 patients and 10,000 log entries on two tablets", async () => {
  expect(WORK, "run by field/run.sh").not.toBe("");
  const android = await connect("android");
  const ipad = await connect("ipad");
  note("");
  note("## Large incident (PERF-3)");

  // The planner plans it; the load device's person leads and treats, the medics treat.
  const dora = await device(android, "Desktop Chrome");
  await loginPair(dora, SITE, "dora@example.org", "Laptop Großeinsatz");
  await dora.locator("#plan-name").fill(INCIDENT);
  await dora.getByRole("button", { name: "Einsatz anlegen" }).click();
  await openView(dora, "Führung");
  await grantRole(dora, "lea@ordo.internal", "Einsatzleitung");
  await grantRole(dora, "lea@ordo.internal", "Arzt/Ärztin");
  await grantRole(dora, "anna@ordo.internal", "Sanitätsdienst");
  await grantRole(dora, "ben@ordo.internal", "Sanitätsdienst");
  await dora.getByRole("button", { name: "Einsatz beginnen" }).click();
  await expect(dora.locator("#incident-state")).toHaveText("Laufend");
  await dora.context().close();

  // The load device pairs by QR code (Lea approves on her phone) and writes the incident.
  const phone = await device(android, "Pixel 7");
  const loaded = await load((address) => approve(phone, address, "lea@example.org"));
  await phone.context().close();
  note(`- The load device wrote ${loaded.written} events (${Object.entries(loaded.byType).map(([t, n]) => `${n} ${t}`).join(", ")}) in ${(loaded.ms / 1000).toFixed(1)} s, ${Math.round(loaded.written / (loaded.ms / 1000))} events/s through the server`);
  expect(loaded.confirmed, "the server confirmed every event").toBe(EVENTS);

  // Two tablets that have never seen the incident join it.
  const ben = await device(android, "Galaxy Tab S4 landscape");
  const benJoin = await join(ben, "ben@example.org", "Tablet Ben groß");
  const anna = await device(ipad, "iPad (gen 7) landscape");
  const annaJoin = await join(anna, "anna@example.org", "iPad Anna groß");
  note(`- A tablet joins and shows all ${PATIENTS} patients: Android ${benJoin} ms, iPad ${annaJoin} ms (budget ${budgets.large.joinMs} ms)`);

  // The incident log of the Android tablet.
  const logStart = Date.now();
  await ben.locator("#launcher-button").click();
  await ben.locator(".launcher__item", { hasText: "Einsatztagebuch" }).click();
  await expect.poll(() => ben.locator(".log-entry").count(), { timeout: 120_000, intervals: [250] }).toBeGreaterThanOrEqual(EVENTS);
  const logMs = Date.now() - logStart;
  const entries = await ben.locator(".log-entry").count();
  note(`- The incident log shows ${entries} entries on the Android tablet after ${logMs} ms (budget ${budgets.large.logMs} ms)`);
  await ben.goBack();
  await expect(ben.locator(".patient-list .patient-card")).toHaveCount(PATIENTS);

  // PERF-2 in the large incident.
  const times: number[] = [];
  for (let i = 1; i <= 5; i++) {
    const tag = `GL-000${i}`;
    const [from, to] = i % 2 ? [anna, ben] : [ben, anna];
    const start = Date.now();
    await register(from, tag, 2, "UHS 1");
    times.push(await arrival(to, tag, start, budgets.propagationMs + 10_000));
  }
  const sorted = [...times].sort((a, b) => a - b);
  note(`- Five patients registered in it, each shown on the other tablet: ${times.join(", ")} ms (median ${sorted[2]} ms, budget ${budgets.propagationMs} ms)`);

  // Memory with the large incident.
  const benWasm = await wasmMemory(ben);
  const annaWasm = await wasmMemory(anna);
  const heap = await jsHeap(ben);
  const server = serverMemory();
  note(`- Android tablet: WebAssembly memory ${mb(benWasm)}, JavaScript heap ${mb(heap)}; iPad: WebAssembly memory ${mb(annaWasm)}`);
  note(`- Server: ORDO ${mb(server.ordo)}, Dex ${mb(server.dex)}, Caddy ${mb(server.caddy)}`);

  expect.soft(Math.max(benJoin, annaJoin), "joining the large incident").toBeLessThanOrEqual(budgets.large.joinMs);
  expect.soft(logMs, "the incident log").toBeLessThanOrEqual(budgets.large.logMs);
  expect.soft(Math.max(...times), "PERF-2 in the large incident").toBeLessThanOrEqual(budgets.propagationMs);
  expect.soft(Math.max(benWasm, annaWasm), "WebAssembly memory").toBeLessThanOrEqual(budgets.large.wasmMemoryBytes);
  expect.soft(heap, "JavaScript heap").toBeLessThanOrEqual(budgets.large.jsHeapBytes);
  expect.soft(server.ordo, "ORDO memory").toBeLessThanOrEqual(budgets.large.ordoMemoryBytes);
  await android.close();
  await ipad.close();
});
