// Shared by the field-near tests (dsn~field-test~1, dsn~large-incident~1): the tablets'
// browsers in their containers, the bundle's containers and the report.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { Browser, BrowserContext, chromium, devices, expect, Page, webkit } from "@playwright/test";
import type { OrdoServer } from "../support/server";

export const WORK = process.env.FIELD_WORK ?? "";
export const SITE = { url: "https://ordo.internal/" } as OrdoServer;
export const budgets = JSON.parse(fs.readFileSync(path.join(__dirname, "budgets.json"), "utf8"));

/** A line of the report (FIELD_WORK/report.md; run.sh adds it to the CI summary). */
export function note(line: string): void {
  console.log(line);
  if (WORK) fs.appendFileSync(path.join(WORK, "report.md"), `${line}\n`);
}

export const mb = (bytes: number) => `${(bytes / 1048576).toFixed(1)} MB`;

export const docker = (...args: string[]) => execFileSync("docker", args, { encoding: "utf8" }).trim();
export const container = (service: string) => `ordo-field-${service}-1`;

/** Connects to a tablet's browser, retrying while its container starts. */
export async function connect(kind: "android" | "ipad"): Promise<Browser> {
  const port = kind === "android" ? process.env.FIELD_ANDROID_PORT ?? "3201" : process.env.FIELD_IPAD_PORT ?? "3202";
  let url = `ws://127.0.0.1:${port}/`;
  if (kind === "android") {
    const spki = fs.readFileSync(path.join(WORK, "tls", "site-spki.txt"), "utf8").trim();
    url += `?launch-options=${encodeURIComponent(JSON.stringify({ args: [`--ignore-certificate-errors-spki-list=${spki}`] }))}`;
  }
  const end = Date.now() + 180_000;
  for (;;) {
    try {
      return await (kind === "android" ? chromium : webkit).connect(url, { timeout: 30_000 });
    } catch (error) {
      if (Date.now() > end) throw error;
      await new Promise((r) => setTimeout(r, 2000));
    }
  }
}

/** A device: its own context with the tablet's screen, and the app's WebAssembly memory kept. */
export async function device(browser: Browser, descriptor: string): Promise<Page> {
  const { defaultBrowserType: _, ...options } = devices[descriptor];
  const context: BrowserContext = await browser.newContext({ ...options, locale: "de-DE" });
  await context.addInitScript(() => {
    addEventListener("TrunkApplicationStarted", (e) => {
      (window as unknown as { ordoWasm: unknown }).ordoWasm = (e as CustomEvent).detail.wasm;
    });
  });
  return context.newPage();
}

export const wasmMemory = (page: Page) =>
  page.evaluate(() => (window as unknown as { ordoWasm?: { memory: WebAssembly.Memory } }).ordoWasm?.memory.buffer.byteLength ?? 0);

/** The JavaScript heap in use of a Chromium page. */
export async function jsHeap(page: Page): Promise<number> {
  const cdp = await page.context().newCDPSession(page);
  await cdp.send("Performance.enable");
  const { metrics } = await cdp.send("Performance.getMetrics");
  return metrics.find((m) => m.name === "JSHeapUsedSize")?.value ?? 0;
}

/** The memory of the server's containers, in bytes. */
export function serverMemory(): Record<string, number> {
  const server: Record<string, number> = {};
  for (const service of ["ordo", "dex", "caddy"]) {
    const used = docker("stats", "--no-stream", "--format", "{{.MemUsage}}", container(service)).split("/")[0].trim();
    const [, value, unit] = used.match(/^([\d.]+)\s*([KMG]i?B|B)$/) ?? [];
    const factor: Record<string, number> = { B: 1, KiB: 1024, KB: 1000, MiB: 1048576, MB: 1e6, GiB: 1073741824, GB: 1e9 };
    server[service] = Number(value) * (factor[unit] ?? 1);
  }
  return server;
}

/** How long until `tag` is listed on `page`, in ms. */
export async function arrival(page: Page, tag: string, since: number, timeout = 30_000): Promise<number> {
  await expect(page.locator(`.patient-card[data-tag='${tag}']`)).toBeVisible({ timeout });
  return Date.now() - since;
}
