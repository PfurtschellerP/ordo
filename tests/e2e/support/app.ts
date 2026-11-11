// Helpers that drive the ORDO app like a user.
import { expect, Page } from "@playwright/test";
import type { DeviceKey, OrdoServer } from "./server";

/** The connection state, as the connection button names it ("Verbindung: Verbunden. Alles gesendet"). */
export const status = (page: Page) => page.locator("#connection-button").getAttribute("aria-label").then((t) => t ?? "");
export const title = (page: Page) => page.locator(".app-header__title").innerText();
export const announced = (page: Page) => page.locator("body > [role=status]").innerText();
export const pathOf = (page: Page) => decodeURIComponent(new URL(page.url()).pathname);

/** Patient cards: "tag category state". */
export const patientRows = (page: Page) =>
  page.locator(".patient-card").evaluateAll((cards) =>
    cards.map((c) => {
      const el = c as HTMLElement;
      const badge = (el.querySelector(".triage") as HTMLElement).innerText.split("\n")[0];
      return `${el.dataset.tag} ${badge} ${(el.querySelector(".patient-card__state") as HTMLElement).innerText}`;
    }),
  );

/** The documented values of one record section, as text. */
export const facts = (page: Page, section: string) =>
  page.locator(`[aria-labelledby=section-${section}] .facts`).evaluateAll((list) =>
    list.map((f) => (f as HTMLElement).innerText.replace(/\n/g, " ")).join(" / "),
  );

/** An item as stored on the device (encrypted: "enc1:" and Base64, ADR-0041). */
export const storedItem = (page: Page, name: string) => page.evaluate((n) => localStorage.getItem(n), name);

/** Records the WebSocket frames a device sends and receives, to check what travels. */
export function recordFrames(page: Page): { sent: Buffer[]; received: Buffer[] } {
  const frames = { sent: [] as Buffer[], received: [] as Buffer[] };
  const bytes = (payload: string | Buffer) => (typeof payload === "string" ? Buffer.from(payload) : payload);
  page.on("websocket", (ws) => {
    ws.on("framesent", (f) => frames.sent.push(bytes(f.payload)));
    ws.on("framereceived", (f) => frames.received.push(bytes(f.payload)));
  });
  return frames;
}

/** Logs in at Dex's login form. */
async function dexLogin(page: Page, email: string, password = "password"): Promise<void> {
  await page.locator("#login").fill(email);
  await page.locator("#password").fill(password);
  await page.locator("#submit-login").click();
}

/** Opens the pairing page. */
async function startPairing(page: Page, server: OrdoServer): Promise<void> {
  await page.goto(server.url);
  await expect(page.getByRole("button", { name: "Anmelden" })).toBeVisible();
}

/** Pairs the device by logging in with the organisation's account (Dex); the app then
 *  lists the person's incidents (ADR-0046). */
export async function loginPair(page: Page, server: OrdoServer, email: string, deviceName?: string, password?: string): Promise<void> {
  await startPairing(page, server);
  // ORG-8: a name is suggested from the browser; a test may choose its own.
  if (deviceName) await page.locator("#device-name").fill(deviceName);
  await page.getByRole("button", { name: "Anmelden" }).click();
  await dexLogin(page, email, password);
  await expect(page.getByRole("heading", { name: "ORDO – Einsätze" })).toBeVisible();
}

/** Pairs a shared device by QR code: a logged-in phone opens the code and approves. */
export async function qrPair(tablet: Page, phone: Page, server: OrdoServer, email: string): Promise<void> {
  await startPairing(tablet, server);
  await tablet.getByRole("button", { name: "Mit QR-Code koppeln" }).click();
  await expect(tablet.locator(".pairing-qr__code")).toBeVisible();
  const address = await tablet.locator("#pair-qr-link").getAttribute("href");
  await phone.goto(address!);
  await phone.locator("#submit-login").click(); // the code is filled in from the QR address
  await dexLogin(phone, email);
  await expect(tablet.getByRole("heading", { name: "ORDO – Einsätze" })).toBeVisible({ timeout: 30_000 });
}

/** Opens an incident from the incident list. */
export async function openIncident(page: Page, name: string): Promise<void> {
  await page.getByRole("button", { name: `Öffnen: ${name}` }).click();
  await expect(page.locator(".app-header__title")).toBeVisible();
}

/** Grants a role in the open incident (view "Personal" of the Einsatzleitung). */
export async function grantRole(page: Page, user: string, role: string): Promise<void> {
  await page.locator("#roster-person").fill(user);
  await page.locator("#roster-role").selectOption({ label: role });
  await page.getByRole("button", { name: "Rolle vergeben" }).click();
  await expect(page.locator(".roster__entry", { hasText: user }).filter({ hasText: role })).toBeVisible();
}

/** Opens a workspace from the menu (launcher), e.g. "Vorbereitung" (ADR-0061). */
export async function openWorkspace(page: Page, label: string): Promise<void> {
  await page.locator("#launcher-button").click();
  await page.locator("#launcher-panel").getByRole("button", { name: label, exact: true }).click();
  // Wait until the switch has happened, so a following view is chosen within it.
  await expect(page.getByRole("heading", { level: 1 })).toHaveText(new RegExp(`– ${label}$`));
}

export async function openView(page: Page, label: string): Promise<void> {
  await page.getByRole("navigation", { name: "Ansichten" }).getByRole("link", { name: label, exact: true }).click();
}

export async function createUhs(page: Page, name: string, capacity: number): Promise<void> {
  await page.locator("#uhs-name").fill(name);
  await page.locator("#uhs-capacity").fill(String(capacity));
  await page.getByRole("button", { name: "UHS anlegen" }).click();
  await expect(page.locator(".load-list__name", { hasText: name })).toBeVisible();
}

/** Registers a patient; `category` is the index of the triage category, or none. */
export async function register(page: Page, tag: string, category?: number, uhs?: string): Promise<void> {
  if (uhs) await page.locator("#uhs").selectOption({ label: uhs });
  await page.locator("#tag").fill(tag);
  if (category !== undefined) await page.locator("input[name=category]").nth(category).check();
  await page.getByRole("button", { name: "Registrieren" }).click();
}

export async function openRecord(page: Page, tag: string): Promise<void> {
  await page.locator(`.patient-card[data-tag='${tag}']`).click();
  await expect(page.locator("#record-title")).toHaveText(`Patient ${tag}`);
}

/** Fills an open editor: text values by field ID, choices by option label. */
export async function fillForm(page: Page, form: string, values: Record<string, string>, choices: [string, string][] = []) {
  for (const [field, value] of Object.entries(values)) await page.locator(`#f-${form}-${field}`).fill(value);
  for (const [field, label] of choices) await page.locator(`#f-${form}-${field}`).getByLabel(label, { exact: true }).check();
}

export async function saveForm(page: Page, form: string): Promise<void> {
  await page.locator(`#section-${form}`).locator("xpath=ancestor::section[1]").locator("form.editor button[type=submit]").click();
}

/** The label of the selected palette option. */
export const selectedOption = (page: Page) =>
  page.locator("#palette-list [aria-selected=true] .palette__label").innerText();

/** Opens the command palette with Ctrl+K and waits until its search field has the focus. */
export async function openPalette(page: Page): Promise<void> {
  await page.keyboard.press("Control+k");
  await expect(page.locator("#palette")).toHaveAttribute("open", "");
  await expect(page.locator("#palette-input")).toBeFocused();
}

/** Closes the command palette with Escape. */
export async function closePalette(page: Page): Promise<void> {
  await page.keyboard.press("Escape");
  await expect(page.locator("#palette")).not.toHaveAttribute("open", "");
}
