// Helpers that drive the ORDO app like a user.
import { expect, Page } from "@playwright/test";
import type { DeviceKey, OrdoServer } from "./server";

export const status = (page: Page) => page.locator(".status").innerText().then((t) => t.replace(/\n/g, " | "));
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

/** The replica as stored on the device (Base64 with prefix; older versions: hex). */
export async function storedReplica(page: Page): Promise<Buffer> {
  const text = (await page.evaluate(() => localStorage.getItem("ordo.replica"))) ?? "";
  return text.startsWith("b64:") ? Buffer.from(text.slice(4), "base64") : Buffer.from(text, "hex");
}

/** Pairs the device; creates a new incident unless one is given. Returns the incident ID. */
export async function pair(page: Page, server: OrdoServer, user: string, key: DeviceKey, incident?: string): Promise<string> {
  await page.goto(server.url);
  await page.getByLabel("Benutzer (name@server)").fill(user);
  await page.getByLabel("Geräte-ID").fill(key.id);
  await page.getByLabel("Privater Schlüssel des Geräts").fill(key.private_key);
  if (incident) await page.getByLabel("Einsatz-ID").fill(incident);
  else await page.getByRole("button", { name: "Neuen Einsatz anlegen" }).click();
  const id = await page.getByLabel("Einsatz-ID").inputValue();
  await page.getByRole("button", { name: "Koppeln" }).click();
  return id;
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
