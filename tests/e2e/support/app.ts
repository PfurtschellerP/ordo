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
/** Takes a person into the incident with a role (ADR-0076): their account, which fills in
 * the name, and the role; with access to the register, they are registered as a helper too. */
export async function grantRole(page: Page, user: string, role: string): Promise<void> {
  const form = page.locator("form.helper-form");
  await form.locator("#helper-account").fill(user);
  await form.locator("#helper-account").blur();
  await form.getByRole("group", { name: "Rollen im Einsatz" }).getByLabel(role, { exact: true }).check();
  await form.getByRole("button", { name: /^(Aufnehmen|Rolle vergeben)$/ }).click();
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

/** Sets up a UHS on *Vorbereitung – Einsatzmittel*, as the incident command (ADR-0086). */
export async function createUhs(page: Page, name: string, capacity: number): Promise<void> {
  await page.locator("#uhs-name").fill(name);
  await page.locator("#uhs-capacity").fill(String(capacity));
  await page.getByRole("button", { name: "UHS anlegen" }).click();
  await expect(page.locator(".uhs-card__name", { hasText: name })).toBeVisible();
}

/** Puts `people` (account names) on a UHS's staff list in its dialog, as the incident command (UHS-10). */
export async function addUhsStaff(page: Page, name: string, people: string[]): Promise<void> {
  await page.getByRole("button", { name: `Ort und Personal von ${name}` }).click();
  const details = page.locator("#uhs-details-dialog");
  for (const person of people) {
    await details.getByLabel("Name", { exact: true }).fill(person);
    await details.getByRole("button", { name: "Zum Personal hinzufügen" }).click();
  }
  await expect(page.locator(".uhs-card", { hasText: name }).locator(".uhs-card__staff")).toHaveText(`Personal: ${people.length} ${people.length === 1 ? "Person" : "Personen"}`);
  await page.keyboard.press("Escape");
}

/** Registers a patient; `category` is the index of the triage category, or none; at a UHS with places, on `place` (a label) or on none. */
/** Shows a part of the patient list column (chapter 8.6.5, "Tabs"): "list", "register" or "staff". */
export async function patientsPart(page: Page, part: "list" | "register" | "staff"): Promise<void> {
  const tab = page.locator(`#patients-parts-tab-${part}`);
  if (await tab.isVisible()) await tab.click();
}

export async function register(page: Page, tag: string, category?: number, uhs?: string, place?: string): Promise<void> {
  await patientsPart(page, "register");
  if (uhs) await page.locator("#uhs").selectOption({ label: uhs });
  await page.locator("#tag").fill(tag);
  if (category !== undefined) await page.locator("input[name=category]").nth(category).check();
  const places = page.locator("#register-place");
  if (await places.isVisible()) await places.selectOption({ label: place ?? "Kein Platz" });
  await page.getByRole("button", { name: "Registrieren" }).click();
}

export async function openRecord(page: Page, tag: string, tab = "Dokumentation"): Promise<void> {
  await patientsPart(page, "list");
  await page.locator(`.patient-card[data-tag='${tag}']`).click();
  await expect(page.locator("#record-title")).toHaveText(`Patient ${tag}`);
  await recordTab(page, tab);
}

/** Chooses a tab of a page in tabs (dsn page-tabs), e.g. Personal: Helfende, Führungsstruktur, Geräte. */
export async function pageTab(page: Page, name: string): Promise<void> {
  const tab = page.locator("#main [role=tablist]").first().getByRole("tab", { name, exact: true });
  if ((await tab.getAttribute("aria-selected")) !== "true") await tab.click();
  await expect(tab).toHaveAttribute("aria-selected", "true");
}

/** Chooses a tab of the open record (EPCR-14): Übersicht, Verlauf, Dokumentation, Übergabe. */
export async function recordTab(page: Page, name: string): Promise<void> {
  // Where the record is narrow (beside the list, on phones), "Dokumentation" reads "Doku".
  const label = name === "Dokumentation" ? /^Doku(mentation)?$/ : name;
  const tab = page.getByRole("tab", { name: label, exact: true });
  if ((await tab.getAttribute("aria-selected")) !== "true") await tab.click();
  await expect(tab).toHaveAttribute("aria-selected", "true");
}

/** Shows the record tab that holds `selector`, if another tab is chosen. */
export async function showTabOf(page: Page, selector: string): Promise<void> {
  const tab = await page
    .locator(selector)
    .first()
    .evaluate((el) => el.closest("[role=tabpanel]")?.getAttribute("aria-labelledby") ?? null);
  if (tab && (await page.locator(`#${tab}`).getAttribute("aria-selected")) !== "true") await page.locator(`#${tab}`).click();
}

/** Opens the collapsed inputs of a record section (e.g. "section-triage"), if closed. */
export async function openSection(page: Page, section: string): Promise<void> {
  await showTabOf(page, `#${section}-open`);
  const button = page.locator(`#${section}-open`);
  if ((await button.getAttribute("aria-expanded")) !== "true") await button.click();
  await expect(page.locator(`#${section}-inputs`)).toBeVisible();
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
