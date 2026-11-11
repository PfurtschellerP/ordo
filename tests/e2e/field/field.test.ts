// Field-near test (dsn~field-test~1): the Compose bundle with an Android tablet (Chromium)
// and an iPad (WebKit), each a browser in its own container on the bundle's network. Run
// by field/run.sh, which starts everything and sets FIELD_WORK.
import { expect, test } from "@playwright/test";
import { createUhs, grantRole, loginPair, openIncident, openView, openWorkspace, register, status } from "../support/app";
import { arrival, budgets, connect, container, device, docker, jsHeap, mb, note, serverMemory, SITE, wasmMemory, WORK } from "./common";

const INCIDENT = "Feldtest";
const UHS = "UHS Feld";

// [itest->dsn~field-test~1]
test("two tablets and the Compose bundle, as in the field", async () => {
  expect(WORK, "run by field/run.sh").not.toBe("");
  const android = await connect("android");
  const ipad = await connect("ipad");
  note("# ORDO field-near test");
  note("");
  note(`Bundle: ${docker("inspect", "--format", "{{.Config.Image}}", container("ordo"))}; Android tablet: Chromium ${android.version()}; iPad: WebKit ${ipad.version()}.`);
  note("");

  // The planner (on a laptop with Chromium) plans the incident; both medics get roles.
  const dora = await device(android, "Desktop Chrome");
  await loginPair(dora, SITE, "dora@example.org", "Laptop Einsatzplanung");
  await dora.locator("#plan-name").fill(INCIDENT);
  await dora.getByRole("button", { name: "Einsatz anlegen" }).click();
  await openWorkspace(dora, "Vorbereitung");
  await openView(dora, "Personal");
  await grantRole(dora, "anna@ordo.internal", "Sanitätsdienst");
  await grantRole(dora, "ben@ordo.internal", "Sanitätsdienst");
  await openView(dora, "Einsatz");
  await dora.getByRole("button", { name: "Einsatz beginnen" }).click();
  await expect(dora.locator("#incident-state")).toHaveText("Laufend");
  await dora.context().close();

  // Ben's Android tablet: the first visit downloads the app.
  const ben = await device(android, "Galaxy Tab S4 landscape");
  await ben.goto(SITE.url);
  await expect(ben.getByRole("button", { name: "Anmelden" })).toBeVisible();
  const download = await ben.evaluate(() => {
    const entries = [...performance.getEntriesByType("navigation"), ...performance.getEntriesByType("resource")] as PerformanceResourceTiming[];
    const wasm = entries.filter((e) => e.name.endsWith(".wasm"));
    return {
      transfer: entries.reduce((sum, e) => sum + e.transferSize, 0),
      encoded: entries.reduce((sum, e) => sum + e.encodedBodySize, 0),
      decoded: entries.reduce((sum, e) => sum + e.decodedBodySize, 0),
      wasmEncoded: wasm.reduce((sum, e) => sum + e.encodedBodySize, 0),
      wasmDecoded: wasm.reduce((sum, e) => sum + e.decodedBodySize, 0),
      files: entries.length,
    };
  });
  note("## Download (PERF-5)");
  note(`- App, first visit: ${mb(download.transfer)} transferred (${download.files} files; ${mb(download.decoded)} unpacked)`);
  note(`- WebAssembly: ${mb(download.wasmEncoded)} transferred, ${mb(download.wasmDecoded)} unpacked`);
  await loginPair(ben, SITE, "ben@example.org", "Tablet Ben");
  await openIncident(ben, INCIDENT);
  await openView(ben, "Patienten");

  // Anna's iPad.
  const anna = await device(ipad, "iPad (gen 7) landscape");
  await loginPair(anna, SITE, "anna@example.org", "iPad Anna");
  await openIncident(anna, INCIDENT);
  await openView(anna, "UHS verwalten");
  await createUhs(anna, UHS, 20);
  const card = anna.locator(".load-list__item", { hasText: UHS });
  await anna.getByRole("button", { name: `Ort und Personal von ${UHS}` }).click();
  for (const person of ["anna@ordo.internal", "ben@ordo.internal"]) {
    await card.getByLabel("Name", { exact: true }).fill(person);
    await card.getByRole("button", { name: "Zum Personal hinzufügen" }).click();
  }
  await expect(card.locator(".load-list__staff")).toHaveText("Personal: Anna Schmidt, Ben Weber");
  await openView(anna, "Patienten");
  for (const page of [anna, ben]) await expect.poll(() => status(page), { message: "both tablets are connected" }).toContain("Verbunden");

  // PERF-2: a change appears on the other connected tablet within 2 s.
  note("");
  note("## Propagation between the tablets (PERF-2)");
  const times: number[] = [];
  for (let i = 1; i <= 5; i++) {
    const tag = `FT-000${i}`;
    const [from, to] = i % 2 ? [anna, ben] : [ben, anna];
    const start = Date.now();
    await register(from, tag, 2, UHS);
    times.push(await arrival(to, tag, start, budgets.propagationMs + 5_000));
  }
  const sorted = [...times].sort((a, b) => a - b);
  note(`- Five patients, each registered on one tablet and shown on the other: ${times.join(", ")} ms (median ${sorted[2]} ms, budget ${budgets.propagationMs} ms)`);
  expect(Math.max(...times), "PERF-2: within 2 s").toBeLessThanOrEqual(budgets.propagationMs);

  // The iPad loses its network: what Anna documents meanwhile arrives after reconnecting.
  note("");
  note("## Network cut");
  const network = "ordo-field_internal";
  const cut = Date.now();
  docker("network", "disconnect", network, container("tablet-ipad"));
  // [itest->dsn~connection-liveness~1]
  // The WebSocket stays open; the app notices the silence.
  await expect.poll(() => status(anna), { message: "the iPad notices", timeout: budgets.offlineNoticedMs + 10_000, intervals: [500] }).toContain("Offline");
  const noticed = Date.now() - cut;
  note(`- The iPad leaves the network: it shows "Offline" after ${noticed} ms (budget ${budgets.offlineNoticedMs} ms)`);
  expect(noticed, "a vanished network is noticed").toBeLessThanOrEqual(budgets.offlineNoticedMs);
  for (const tag of ["FT-0010", "FT-0011", "FT-0012"]) await register(anna, tag, 1, UHS);
  await expect.poll(() => status(anna), { message: "kept on the iPad" }).toContain("3 Eingaben noch nicht gesendet");
  await expect(ben.locator(".patient-card[data-tag='FT-0010']"), "nothing arrives meanwhile").toHaveCount(0);
  const back = Date.now();
  docker("network", "connect", network, container("tablet-ipad"));
  const resync = await arrival(ben, "FT-0012", back, budgets.reconnectMs);
  for (const tag of ["FT-0010", "FT-0011"]) await expect(ben.locator(`.patient-card[data-tag='${tag}']`)).toBeVisible();
  note(`- 3 patients registered offline on the iPad arrive on the Android tablet ${resync} ms after the network is back (budget ${budgets.reconnectMs} ms)`);
  await expect.poll(() => status(anna), { message: "everything is sent" }).toContain("Alles gesendet");

  // The server restarts: both tablets reconnect by themselves; nothing is lost.
  note("");
  note("## Server restart");
  const restart = Date.now();
  docker("restart", container("ordo"));
  for (const page of [anna, ben]) await expect.poll(() => status(page), { message: "reconnected", timeout: budgets.reconnectMs + 30_000 }).toContain("Verbunden");
  const reconnected = Date.now() - restart;
  await register(ben, "FT-0020", 0, UHS);
  await arrival(anna, "FT-0020", Date.now());
  for (const page of [anna, ben]) await expect(page.locator(".patient-card"), "all 9 patients on both tablets").toHaveCount(9);
  note(`- ORDO restarted; both tablets connected again after ${reconnected} ms; all 9 patients on both`);

  // PERF-5: memory on the tablets and on the server.
  note("");
  note("## Memory (PERF-5)");
  const benWasm = await wasmMemory(ben);
  const annaWasm = await wasmMemory(anna);
  const heap = await jsHeap(ben);
  note(`- Android tablet: WebAssembly memory ${mb(benWasm)}, JavaScript heap ${mb(heap)}`);
  note(`- iPad: WebAssembly memory ${mb(annaWasm)}`);
  const server = serverMemory();
  note(`- Server: ORDO ${mb(server.ordo)}, Dex ${mb(server.dex)}, Caddy ${mb(server.caddy)}`);

  expect.soft(download.transfer, "app download").toBeLessThanOrEqual(budgets.appTransferBytes);
  expect.soft(download.wasmEncoded, "WebAssembly download").toBeLessThanOrEqual(budgets.wasmTransferBytes);
  expect.soft(Math.max(benWasm, annaWasm), "WebAssembly memory").toBeLessThanOrEqual(budgets.wasmMemoryBytes);
  expect.soft(heap, "JavaScript heap").toBeLessThanOrEqual(budgets.jsHeapBytes);
  for (const service of ["ordo", "dex", "caddy"]) {
    expect.soft(server[service], `${service} memory`).toBeLessThanOrEqual(budgets.serverMemoryBytes[service]);
  }
  await android.close();
  await ipad.close();
});
