// End-to-end scenario in a real browser against a real `ordo serve`, in Firefox, Chromium
// and WebKit (Playwright). Scenarios from tests/acceptance/features/: register patients,
// required triage choice, patient record with photo, keyboard command palette, offline
// registration with reload and resend, command sees counts but no patient data, phone
// layout with bottom navigation, login timeout, revoking a lost device. Every view is
// audited (support/audit.ts).
import os from "node:os";
import path from "node:path";
import { Browser, expect, Page, test } from "@playwright/test";
import { audit, settle } from "./support/audit";
import {
  announced,
  createUhs,
  facts,
  fillForm,
  closePalette,
  openPalette,
  grantRole,
  loginPair,
  openIncident,
  openRecord,
  openView,
  qrPair,
  pathOf,
  patientRows,
  register,
  saveForm,
  selectedOption,
  status,
  recordFrames,
  storedItem,
  title,
} from "./support/app";
import { startDex } from "./support/dex";
import { OrdoServer } from "./support/server";

let server: OrdoServer;
let stopDex: () => void;
const INCIDENT = "Sanitätsdienst Stadtfest";
let anna: Page;
let ben: Page;
let phone: Page;
let dora: Page;
let shots: string;
let annaFrames: ReturnType<typeof recordFrames>;
let patientPhoto: Buffer;
let benFrames: ReturnType<typeof recordFrames>;

const shot = (page: Page, name: string, fullPage = true) => page.screenshot({ path: path.join(shots, `${name}.png`), fullPage });

async function newDevice(browser: Browser): Promise<Page> {
  const context = await browser.newContext({ viewport: { width: 1200, height: 1000 } });
  return context.newPage();
}

test.beforeAll(async ({ browser }, info) => {
  const work = path.join(process.env.ORDO_E2E_WORK ?? os.tmpdir(), `ordo-e2e-${info.project.name}`);
  shots = path.join(process.env.ORDO_E2E_SHOTS ?? path.join(__dirname, "screenshots"), info.project.name);
  server = new OrdoServer(work, 18090 + info.parallelIndex);
  server.setup(); // no seed: people log in with Dex, roles come from their groups
  stopDex = await startDex();
  await server.start();
  anna = await newDevice(browser);
  ben = await newDevice(browser);
  phone = await newDevice(browser);
  dora = await newDevice(browser);
  annaFrames = recordFrames(anna);
  benFrames = recordFrames(ben);
});

test.afterAll(async () => {
  await server?.stop();
  stopDex?.();
});

// One test with steps: later steps build on earlier ones, and soft checks (audits)
// report every finding of a run instead of stopping at the first.
test("ORDO in the browser", async () => {
  // [itest->dsn~incident-roles~1]
  await test.step("a planner plans the incident and gives roles", async () => {
    await dora.goto(server.url);
    await expect(dora.getByRole("button", { name: "Anmelden" })).toBeVisible();
    await audit(dora, "Pairing page");
    // [itest->dsn~oidc-login~1]
    await loginPair(dora, server, "dora@example.org");
    await expect(dora.locator(".incident-card"), "no incident planned yet").toHaveCount(0);
    await audit(dora, "Incident list", [1200, 320]);
    await dora.locator("#plan-name").fill(INCIDENT);
    await dora.getByRole("button", { name: "Einsatz anlegen" }).click();
    await expect.poll(() => title(dora), { message: "the planner leads the new incident" }).toContain("Einsatzleitung");
    await expect(dora.locator("#incident-name"), "the header names the incident").toHaveText(INCIDENT);
    await openView(dora, "Personal");
    await expect(dora.locator(".roster__entry"), "the planner is its commander").toHaveCount(1);
    await grantRole(dora, "anna@localhost", "Sanitätsdienst");
    await grantRole(dora, "ben@localhost", "Einsatzleitung");
    await dora.getByRole("button", { name: "Einsatz beginnen" }).click();
    await expect(dora.locator("#incident-state")).toHaveText("Laufend");
    await audit(dora, "Roster", [1200, 320]);
    await shot(dora, "roster");
  });

  await test.step("a medic registers patients at several UHS", async () => {
    await loginPair(anna, server, "anna@example.org");
    await expect(anna.locator(".incident-card"), "Anna sees the incident she has a role in").toHaveCount(1);
    await expect(anna.locator(".incident-card")).toContainText("Sanitätsdienst");
    await openIncident(anna, INCIDENT);
    await expect(anna.locator(".launcher__user"), "the login maps to an ORDO user").toHaveText("Angemeldet als anna@localhost");
    // [itest->dsn~device-storage-encryption~1]
    for (const item of ["ordo.pairing", "ordo.replica", "ordo.storage-key"]) {
      await expect.poll(() => storedItem(anna, item), { message: `${item} is stored` }).not.toBeNull();
    }
    const pairing = (await storedItem(anna, "ordo.pairing"))!;
    expect(pairing.startsWith("enc1:") && !pairing.includes("anna") && !pairing.includes("private_key"), "the device key is stored encrypted").toBe(true);
    await expect.poll(() => status(anna), { message: "medic device connects" }).toContain("Verbunden");
    await expect.poll(() => title(anna), { message: "a medic starts in Sanität – UHS" }).toContain("Sanität – UHS");

    await openView(anna, "UHS verwalten");
    await createUhs(anna, "UHS Bühne Nord", 10);
    await createUhs(anna, "UHS Eingang", 5);
    // [itest->dsn~uhs-registry~1]
    await expect(anna.locator(".load-list__item"), "two UHS are listed with their load").toHaveCount(2);

    await openView(anna, "Patienten");
    for (const [tag, category, uhs] of [["DE-0042", 0, "UHS Bühne Nord"], ["DE-0043", 1, "UHS Bühne Nord"], ["DE-0044", 1, "UHS Eingang"]] as const) {
      await register(anna, tag, category, uhs);
      await expect(anna.locator(`.patient-card[data-tag='${tag}']`)).toBeVisible();
    }
    await expect.poll(() => status(anna), { message: "registrations are sent" }).toContain("Alles gesendet");
    expect(await patientRows(anna)).toHaveLength(3);

    await register(anna, "DE-0099"); // no triage category chosen
    expect(await patientRows(anna), "registration without a triage category is not possible").toHaveLength(3);
    await anna.locator("#tag").fill("");
  });

  await test.step("the patient record documents identity, vital signs and a photo", async () => {
    await openRecord(anna, "DE-0044");
    expect(pathOf(anna), "the record has its own address").toBe("/med.uhs/patients/DE-0044");
    await anna.getByRole("button", { name: "Nachsichtung DE-0044: SK I", exact: true }).click();
    await expect.poll(() => announced(anna), { message: "re-triage is announced to screen readers" }).toContain("nachgesichtet");
    await expect.poll(() => status(anna), { message: "re-triage is sent" }).toContain("Alles gesendet");
    // [itest->dsn~transport-priority~1]
    await anna.getByRole("button", { name: "DE-0044: Transportpriorität setzen" }).click();
    await expect(anna.locator(".patient-card[data-tag='DE-0044'] .priority-badge"), "the list shows the transport priority").toHaveText(/Transportpriorität/);
    // [itest->dsn~treatment-places~1]
    await anna.locator("#place").selectOption({ label: "Platz 2" });
    await expect(anna.locator(".patient-card[data-tag='DE-0044'] .patient-card__meta"), "the list shows the treatment place").toContainText("Platz 2");

    // [itest->dsn~patient-care-record~1]
    await anna.locator("#edit-identity").click();
    await fillForm(anna, "identity", { family_name: "Mustermann", given_name: "Max", age_estimate: "40" }, [["sex", "männlich"]]);
    await saveForm(anna, "identity");
    await expect.poll(() => facts(anna, "identity"), { message: "identity is documented" }).toMatch(/Mustermann.*männlich/);

    await anna.locator("#new-vitals").click();
    await fillForm(anna, "vitals", { heart_rate: "abc" });
    await saveForm(anna, "vitals");
    await expect(anna.locator("#f-vitals-heart_rate"), "an invalid number is marked").toHaveAttribute("aria-invalid", "true");
    await expect(anna.locator("#f-vitals-heart_rate-error"), "and explained").toHaveText("Bitte eine Zahl eingeben.");
    await fillForm(anna, "vitals", { heart_rate: "112", spo2: "97", gcs_eye: "4", gcs_verbal: "5", gcs_motor: "6" });
    await saveForm(anna, "vitals");
    await expect.poll(() => facts(anna, "vitals"), { message: "vital signs with the GCS total" }).toMatch(/112 \/min.*GCS gesamt 15/);
    await anna.locator("[aria-labelledby=section-vitals] .entry").getByRole("button").click(); // Korrigieren
    await fillForm(anna, "vitals", { heart_rate: "92" });
    await saveForm(anna, "vitals");
    await expect.poll(() => facts(anna, "vitals")).toContain("92 /min");
    await expect(anna.locator("[aria-labelledby=section-vitals] .corrected summary"), "the old entry stays visible").toHaveText("1 korrigierter Eintrag");

    // Unsaved input survives switching views (UX-5).
    await anna.locator("#edit-history").click();
    await fillForm(anna, "history", { allergies: "Penicillin" });
    await openView(anna, "UHS verwalten");
    await openView(anna, "Patienten");
    await openRecord(anna, "DE-0044");
    await expect(anna.locator("#f-history-allergies"), "unsaved input is still there").toHaveValue("Penicillin");
    await saveForm(anna, "history");

    // [itest->dsn~patient-photo~1]
    const png = await anna.evaluate(() => {
      const canvas = document.createElement("canvas");
      canvas.width = 3000;
      canvas.height = 2000;
      const ctx = canvas.getContext("2d")!;
      for (let i = 0; i < 400; i++) {
        ctx.fillStyle = `hsl(${(i * 7) % 360} 70% 50%)`;
        ctx.fillRect(Math.random() * 3000, Math.random() * 2000, 300, 200);
      }
      return canvas.toDataURL("image/png").split(",")[1];
    });
    await anna.locator("#photo-upload").setInputFiles({ name: "photo.png", mimeType: "image/png", buffer: Buffer.from(png, "base64") });
    // Anna's own profile picture (ADR-0047): set in the menu, shown in her header.
    await anna.locator("#avatar-upload").setInputFiles({ name: "me.png", mimeType: "image/png", buffer: Buffer.from(png, "base64") });
    await expect(anna.locator(".app-header__user img.avatar"), "the own picture appears in the header").toBeVisible({ timeout: 20_000 });
    const photo = anna.locator("img.photo");
    await expect(photo, "a photo for identification is shown in the record").toBeVisible({ timeout: 20_000 });
    await expect.poll(() => photo.evaluate((i: HTMLImageElement) => i.complete && i.naturalWidth > 0)).toBe(true);
    patientPhoto = Buffer.from((await photo.getAttribute("src"))!.split(",")[1], "base64");
    const [width, height, bytes, prefix] = await photo.evaluate((i: HTMLImageElement) => [
      i.naturalWidth,
      i.naturalHeight,
      atob(i.src.split(",")[1]).length,
      i.src.slice(0, 23),
    ]);
    expect(Number(width), "downscaled").toBeLessThanOrEqual(800);
    expect(Number(height), "downscaled").toBeLessThanOrEqual(800);
    expect(Number(bytes), "within the size limit").toBeLessThanOrEqual(160 * 1024);
    expect(prefix, "re-encoded as JPEG").toBe("data:image/jpeg;base64,");
    await expect.poll(() => status(anna), { message: "record entries and photo are sent" }).toContain("Alles gesendet");
    await audit(anna, "Patient record");
    await shot(anna, "record");

    await anna.goBack();
    await expect.poll(() => pathOf(anna), { message: "the back button returns to the list" }).toBe("/med.uhs/patients");
    await expect(anna.locator("#record-title")).toHaveCount(0);
    await expect(anna.locator(".patient-card[data-tag='DE-0044']"), "the list shows the name").toContainText("Mustermann, Max");
    expect((await storedItem(anna, "ordo.replica"))!.startsWith("enc1:"), "the device stores its data encrypted").toBe(true);
    const sent = Buffer.concat(annaFrames.sent);
    expect(sent.length).toBeGreaterThan(0);
    expect(sent.includes("Mustermann") || sent.includes("Penicillin"), "clinical entries travel sealed, not in plain text").toBe(false);
  });

    // [itest->dsn~command-palette~1]
  await test.step("the command palette works with the keyboard alone", async () => {
    await anna.locator("#tag").focus();
    await anna.keyboard.press("Control+k");
    await expect(anna.locator("#palette"), "Ctrl+K opens the command palette").toHaveAttribute("open", "");
    await expect(anna.locator("#palette-input"), "the search field has the focus").toBeFocused();
    await anna.keyboard.type("0043");
    await expect.poll(() => selectedOption(anna), { message: "part of a triage tag number finds the patient" }).toBe("DE-0043");
    await audit(anna, "Command palette", [1200]);
    await anna.keyboard.press("Enter");
    await anna.keyboard.type("retriage minor"); // English command name in a German UI (ADR-0036)
    await expect.poll(() => selectedOption(anna), { message: "the patient's quick actions are offered" }).toBe("Sichtung ändern: SK III");
    await anna.keyboard.press("Enter");
    await expect.poll(() => announced(anna), { message: "a re-triage runs from the palette" }).toContain("DE-0043 nachgesichtet: SK III");
    await expect(anna.locator("#palette"), "the palette closes after the action").not.toHaveAttribute("open", "");
    await openPalette(anna);
    await anna.keyboard.type("vitalwerte");
    await expect(anna.locator("#palette-list [role=option]"), "patient commands only for a chosen patient").toHaveCount(0);
    await anna.keyboard.press("Escape");
    await expect(anna.locator("#palette"), "Escape closes the palette").not.toHaveAttribute("open", "");
    await expect(anna.locator("#tag"), "focus returns to where it was").toBeFocused();
    // A quick action that opens a form brings it into view, with the cursor in its first field.
    await openPalette(anna);
    await anna.keyboard.type("0043");
    await anna.keyboard.press("Enter");
    await anna.keyboard.type("add vitals");
    await expect.poll(() => selectedOption(anna)).toBe("Vitalwerte eintragen");
    await anna.keyboard.press("Enter");
    const firstField = anna.locator("[aria-labelledby=section-vitals] .editor :is(input, select, textarea)").first();
    await expect(firstField, "the form's first field has the focus").toBeFocused();
    await expect(firstField, "and is on screen").toBeInViewport();
    await anna.locator("[aria-labelledby=section-vitals] .editor").getByRole("button", { name: "Abbrechen" }).click();
    await anna.goBack();
  });

  await test.step("input survives being offline and a reload", async () => {
    // [itest->dsn~offline-app-shell~1]
    await expect.poll(() => anna.evaluate(() => !!navigator.serviceWorker.controller), { message: "the app shell is installed for offline use" }).toBe(true);
    if (test.info().project.name === "chromium") {
      // Chrome offers to install the app (PWA): manifest with name, icons, start URL.
      const cdp = await anna.context().newCDPSession(anna);
      const { installabilityErrors } = await cdp.send("Page.getInstallabilityErrors");
      expect(installabilityErrors, "Chrome can install ORDO as an app").toEqual([]);
    }
    await server.stop();
    await expect.poll(() => status(anna), { message: "loss of connection is shown" }).toContain("Offline");
    await register(anna, "DE-0045", 2, "UHS Eingang");
    await expect.poll(() => status(anna), { message: "pending input is shown" }).toContain("1 Eingabe noch nicht gesendet");
    // [itest->dsn~local-persistence~1]
    await anna.reload(); // the server is down: the app must start from the device
    await expect.poll(() => patientRows(anna), { message: "offline registration survives a reload" }).toContain("DE-0045 SK III noch nicht gesendet");
    await server.start();
    await expect.poll(() => status(anna), { message: "after reconnecting, everything is sent", timeout: 20_000 }).toContain("Alles gesendet");
    // [itest->dsn~triage-badge~1]
    const badges = await anna.locator(".triage").evaluateAll((list) =>
      list.map((b) => !!b.querySelector(".triage__shape") && (b as HTMLElement).innerText.trim().length > 0),
    );
    expect(badges.length).toBeGreaterThan(0);
    expect(badges.every(Boolean), "every triage badge has a shape and a text label").toBe(true);
    await audit(anna, "Med workspace");
    await anna.locator("#launcher-button").click();
    // [itest->dsn~views-by-role~1]
    await expect(anna.locator("#reason-command-situation"), "UHS crews do not see the situation view").toHaveText("Kein Zugriff in diesem Einsatz");
    await expect(anna.locator("#reason-org-devices"), "nor the device list").toHaveText("Kein Zugriff in diesem Einsatz");
    await audit(anna, "Launcher (medic)");
    await anna.locator("#launcher-button").click();
    await shot(anna, "med-day");
  });

  await test.step("on phones, views and menu are at the bottom", async () => {
    await anna.setViewportSize({ width: 390, height: 844 });
    await settle(anna);
    const [barBottom, viewportHeight, menuInBar] = await anna.evaluate(() => {
      const n = document.querySelector(".navigation")!.getBoundingClientRect();
      const m = document.getElementById("launcher-button")!.getBoundingClientRect();
      return [Math.round(n.bottom), innerHeight, m.top >= n.top && m.bottom <= n.bottom];
    });
    expect(barBottom, "the bar sits at the bottom").toBe(viewportHeight);
    expect(menuInBar, "the menu is in the bar").toBe(true);
    await anna.evaluate(() => window.scrollTo(0, 0));
    await shot(anna, "med-phone", false);
    await anna.locator("#launcher-button").click();
    const [sheetBottom, barTop, sheetTop] = await anna.evaluate(() => {
      const p = document.getElementById("launcher-panel")!.getBoundingClientRect();
      const n = document.querySelector(".navigation")!.getBoundingClientRect();
      return [Math.round(p.bottom), Math.round(n.top), Math.round(p.top)];
    });
    expect(sheetBottom, "the menu opens upwards above the bar").toBeLessThanOrEqual(barTop + 1);
    expect(sheetTop).toBeGreaterThan(0);
    // With larger text (WCAG 1.4.4) the bar grows; the menu must still sit above it.
    const atLargeText = await anna.evaluate(() => {
      document.documentElement.style.fontSize = "150%";
      const p = document.getElementById("launcher-panel")!.getBoundingClientRect();
      const n = document.querySelector(".navigation")!.getBoundingClientRect();
      const result = [Math.round(p.bottom), Math.round(n.top)];
      document.documentElement.style.fontSize = "";
      return result;
    });
    expect(atLargeText[0], "at 150 % text size the menu still opens above the bar").toBeLessThanOrEqual(atLargeText[1] + 1);
    await shot(anna, "launcher-phone", false);
    await anna.locator("#launcher-button").click();
    await openRecord(anna, "DE-0042");
    await shot(anna, "record-phone", false);
    await expect(anna.locator(".master"), "on phones the record is a page of its own").toBeHidden();
    await anna.goBack();
    await anna.setViewportSize({ width: 1200, height: 1000 });
    await expect(anna.locator(".app-header__user img.avatar"), "the own picture is back after offline and reload").toBeVisible();
    await anna.evaluate(() => (document.documentElement.dataset.theme = "night"));
    await settle(anna);
    await shot(anna, "med-night");
    await anna.context().close();
  });

  await test.step("command sees counts and resources, but no patient data", async () => {
    // [itest->dsn~qr-pairing~1]
    await qrPair(ben, phone, server, "ben@example.org");
    await openIncident(ben, INCIDENT);
    await phone.context().close();
    await openView(ben, "Personal");
    await expect(ben.locator(".roster__entry", { hasText: "anna@localhost" }).locator("img.avatar"), "others see Anna's picture").toBeVisible();
    await expect(ben.locator(".roster__entry", { hasText: "dora@localhost" }).locator(".avatar--initials"), "without a picture: initials").toHaveText("D");
    await openView(ben, "Patientenzahlen");
    // [itest->dsn~workspace-navigation~2]
    await expect.poll(() => title(ben), { message: "the commander starts in the first workspace they may use" }).toContain("Einsatzleitung – Lage");
    await ben.locator("#launcher-button").click();
    const uhs = ben.locator(".launcher__item", { hasText: "UHS" });
    await expect(uhs, "the UHS workspace stays in the menu, disabled").toHaveAttribute("aria-disabled", "true");
    await expect(ben.locator("#reason-med-uhs"), "with the reason").toHaveText("Kein Zugriff in diesem Einsatz");
    await audit(ben, "Launcher (commander)");
    await shot(ben, "launcher");
    await ben.keyboard.press("Escape");
    await expect(ben.locator("#launcher-panel"), "Escape closes the menu").toBeHidden();
    await expect(ben.locator("#launcher-button"), "focus returns to the menu button").toBeFocused();

    // DE-0043 was re-triaged to SK III from the palette.
    const cards = () =>
      ben.locator(".count-card").evaluateAll((list) =>
        list.map((c) =>
          [
            (c.querySelector("h3") as HTMLElement).innerText,
            ...[...c.querySelectorAll(".count-card__n")].map((n) => (n as HTMLElement).innerText),
            (c.querySelector("p") as HTMLElement).innerText,
          ].join(" "),
        ),
      );
    await expect.poll(cards, { message: "command sees counts and load per UHS" }).toEqual([
      "UHS Bühne Nord 1 0 1 0 0 0 Gesamt: 2 · Belegung: 2 / 10",
      "UHS Eingang 1 0 1 0 0 0 Gesamt: 2 · Belegung: 2 / 5",
    ]);
    await ben.setViewportSize({ width: 1700, height: 1000 });
    const table = () =>
      ben.locator("#main tbody tr").evaluateAll((rows) =>
        rows.filter((r) => (r as HTMLElement).offsetParent).map((r) => [...(r as HTMLTableRowElement).cells].map((c) => c.innerText.split("\n")[0]).join(" ")),
      );
    await expect.poll(table, { message: "where there is room, the counts are a table" }).toEqual([
      "UHS Bühne Nord 1 0 1 0 0 0 2 2 / 10",
      "UHS Eingang 1 0 1 0 0 0 2 2 / 5",
    ]);
    await audit(ben, "Command workspace (wide)", [1700]);
    const received = Buffer.concat(benFrames.received);
    expect(received.length).toBeGreaterThan(0);
    expect(
      received.includes("DE-0042") || received.includes("Mustermann") || received.includes(patientPhoto.subarray(patientPhoto.length - 400, patientPhoto.length - 336)),
      "the command device receives no triage tag number, no name and no photo",
    ).toBe(false);
    await audit(ben, "Command workspace");
    await shot(ben, "command");
  });

  await test.step("resources: add a unit and set its status, also from the keyboard", async () => {
    await openView(ben, "Einsatzmittel");
    await ben.locator("#unit-call-sign").fill("RTW 83/1");
    await ben.locator("#unit-type").fill("RTW");
    await ben.locator("#unit-organisation").fill("DRK Musterstadt");
    await ben.getByRole("button", { name: "Einsatzmittel hinzufügen" }).click();
    const status_ = ben.getByLabel("Status von RTW 83/1 setzen");
    await status_.selectOption({ label: "3 – Einsatz übernommen" });
    const card = ben.locator(".unit-card[data-unit='RTW 83/1']");
    await expect(card, "a unit is added and its status is set").toContainText("Status: 3 – Einsatz übernommen");
    await expect(card).toContainText("RTW · DRK Musterstadt");
    await expect.poll(() => status(ben), { message: "unit and status are sent" }).toContain("Alles gesendet");
    await openPalette(ben);
    await ben.keyboard.type("rtw");
    await expect.poll(() => selectedOption(ben), { message: "the palette finds a unit by call sign" }).toBe("RTW 83/1");
    await ben.keyboard.press("Enter");
    await ben.keyboard.type("on scene"); // English name in a German UI (ADR-0036)
    await ben.keyboard.press("Enter");
    await expect(card, "a status is set from the palette by its English name").toContainText("Status: 4 – Am Einsatzort");
    await openPalette(ben);
    await ben.keyboard.type("patienten");
    await expect(
      ben.locator("#palette-list [role=option][aria-disabled=true]", { hasText: "Kein Zugriff in diesem Einsatz" }).filter({ hasText: "patients" }),
      "views without access are shown disabled with the reason",
    ).toHaveCount(1);
    await closePalette(ben);
    await audit(ben, "Resources workspace");
    await shot(ben, "resources");
    await ben.evaluate(() => (document.documentElement.dataset.theme = "high-contrast"));
    await settle(ben);
    await shot(ben, "command-high-contrast");
    await ben.evaluate(() => delete document.documentElement.dataset.theme);
  });

  // [itest->dsn~access-timeout~1]
  await test.step("a device locks itself when its login has expired, also offline", async () => {
    const start = Date.now();
    await ben.clock.install({ time: start });
    await ben.reload();
    await expect(ben.locator(".app-header__title")).toBeVisible();
    // 49 hours later, without any contact to the server (default timeout: 48 hours).
    await ben.clock.setSystemTime(start + 49 * 3600_000);
    await ben.clock.fastForward("01:05");
    await expect(ben.getByRole("heading", { name: /Anmeldung abgelaufen/ }), "the device locks itself").toBeVisible();
    await expect(ben.locator(".unit-card"), "no data is shown while locked").toHaveCount(0);
    await expect(ben.getByRole("button", { name: "Anmelden" }), "the person can log in again").toBeVisible();
    await audit(ben, "Login expired", [1200]);
    await shot(ben, "login-expired");
    // Back within the login: the device unlocks; its data is still there.
    await ben.clock.setSystemTime(Date.now());
    await ben.clock.fastForward("01:05");
    await expect(ben.locator(".app-header__title"), "the device unlocks within its login").toBeVisible();
  });

  // [itest->dsn~device-revocation~1]
  await test.step("a lost device is revoked and deletes its data", async () => {
    // Ben sees only his own device.
    await ben.locator("#launcher-button").click();
    await ben.locator(".launcher__item", { hasText: "Geräte" }).click();
    await expect.poll(() => title(ben)).toContain("Organisation – Geräte");
    await expect(ben.locator(".device-card"), "a person sees their own devices").toHaveCount(1);
    await expect(ben.locator(".device-card h3")).toContainText("dieses Gerät");
    await expect(ben.locator(".device-card dt", { hasText: "Person" }), "no person shown for own devices").toHaveCount(0);

    // Dora, an administrator, sees all devices.
    await dora.locator("#launcher-button").click();
    await dora.locator(".launcher__item", { hasText: "Geräte" }).click();
    await expect.poll(() => title(dora)).toContain("Organisation – Geräte");
    await expect(dora.locator(".device-card"), "an admin sees all devices").toHaveCount(3);
    await audit(dora, "Devices");
    const tablet = dora.locator(".device-card", { hasText: "ben@localhost" });
    await tablet.getByRole("button", { name: /^Sperren/ }).click();
    await expect(dora.getByRole("button", { name: "Ja, sperren" }), "focus moves to the confirmation").toBeFocused();
    await audit(dora, "Revoke confirmation", [1200]);
    await shot(dora, "devices-revoke");
    await dora.getByRole("button", { name: "Ja, sperren" }).click();
    await expect(dora.locator(".device-card"), "the revoked tablet is gone from the list").toHaveCount(2);
    await expect.poll(() => announced(dora)).toContain("gesperrt");

    // Ben's tablet is cut off at once and deletes its key and data.
    await expect(ben.getByRole("heading", { name: "Gerät gesperrt" }), "the tablet learns it was revoked").toBeVisible();
    expect(await ben.evaluate(() => [localStorage.getItem("ordo.pairing"), localStorage.getItem("ordo.replica")]), "remote wipe").toEqual([null, null]);
    await audit(ben, "Device revoked", [1200]);
    await shot(ben, "device-revoked");
    await ben.getByRole("button", { name: "Neu koppeln" }).click();
    await expect(ben.getByRole("button", { name: "Anmelden" }), "the tablet can be paired again").toBeVisible();
  });
});
