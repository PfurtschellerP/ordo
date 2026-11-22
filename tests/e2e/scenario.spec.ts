// End-to-end scenario in a real browser against a real `ordo serve`, in Firefox, Chromium
// and WebKit (Playwright). Scenarios from tests/acceptance/features/: register patients,
// required triage choice, patient record with photo, keyboard command palette, offline
// registration with reload and resend, command sees counts but no patient data, phone
// layout with bottom navigation, login timeout, revoking a lost device. Every view is
// audited (support/audit.ts).
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { Browser, expect, Locator, Page, test } from "@playwright/test";
import { audit, settle } from "./support/audit";
import {
  pageTab,
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
  openSection,
  recordTab,
  showTabOf,
  openView,
  openWorkspace,
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
import { DEX_URL, ORDO_PORT, startDex, UPSTREAM } from "./support/dex";
import { PLACE, startGeocoder } from "./support/geocoder";
import { screenPhoto } from "./support/screen-photo";
import { OrdoServer } from "./support/server";

let server: OrdoServer;
let stopDex: () => void;
let stopGeocoder: () => void;
const INCIDENT = "Sanitätsdienst Stadtfest";
let anna: Page;
let ben: Page;
let phone: Page;
let dora: Page;
let petra: Page;
let shots: string;
let annaFrames: ReturnType<typeof recordFrames>;
let patientPhoto: Buffer;
let benFrames: ReturnType<typeof recordFrames>;

const shot = (page: Page, name: string, fullPage = true) => page.screenshot({ path: path.join(shots, `${name}.png`), fullPage });

/** Content Security Policy violations reported by any page of the scenario (SEC-18). */
const cspViolations: string[] = [];

async function newDevice(browser: Browser): Promise<Page> {
  const context = await browser.newContext({ viewport: { width: 1200, height: 1000 } });
  // The page's own violation events, in every browser. Only the app's code counts, not
  // axe-core: it reads stylesheets through <style> elements the policy rightly blocks, and
  // marks its run in the page (see `runAxe`), so the decision does not depend on timing.
  await context.exposeFunction("ordoE2eCsp", (line: string) => cspViolations.push(line));
  await context.addInitScript(() => {
    const w = window as unknown as { ordoE2eTooling?: number; ordoE2eCsp: (line: string) => void };
    document.addEventListener("securitypolicyviolation", (e) => {
      if (!w.ordoE2eTooling) {
        w.ordoE2eCsp(`${e.documentURI} ${e.effectiveDirective} ${e.blockedURI} ${e.sourceFile}:${e.lineNumber}`);
      }
    });
  });
  const page = await context.newPage();
  return page;
}

test.beforeAll(async ({ browser }, info) => {
  const work = path.join(process.env.ORDO_E2E_WORK ?? os.tmpdir(), `ordo-e2e-${info.project.name}`);
  shots = path.join(process.env.ORDO_E2E_SHOTS ?? path.join(__dirname, "screenshots"), info.project.name);
  server = new OrdoServer(work, ORDO_PORT + info.parallelIndex);
  server.setup(); // no seed: people log in with Dex, roles come from their groups
  stopDex = await startDex();
  const geocoder = await startGeocoder();
  stopGeocoder = geocoder.stop;
  server.geocoder = geocoder.url;
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
  stopGeocoder?.();
});

// One test with steps: later steps build on earlier ones, and soft checks (audits)
// report every finding of a run instead of stopping at the first.
test("ORDO in the browser", async ({ browser }) => {
  // [itest->dsn~incident-roles~1]
  await test.step("a planner plans the incident and gives roles", async () => {
    await dora.goto(server.url);
    await expect(dora.getByRole("button", { name: "Anmelden" })).toBeVisible();
    await expect
      .poll(() => dora.evaluate(async () => (await document.fonts.ready, document.fonts.check('700 16px "Atkinson Hyperlegible Next"'))), {
        message: "the app's own typeface is served with it, not left to the device",
      })
      .toBe(true);
    await audit(dora, "Pairing page");
    // [itest->dsn~licence-notices~2]
    // LIC-1: before pairing, the pairing screen links to the licences and the source code.
    await expect(dora.getByRole("contentinfo"), "no footer under every screen").toHaveCount(0);
    await dora.getByRole("link", { name: "Lizenzen und Quellcode" }).click();
    await expect(dora.getByRole("heading", { level: 1 })).toHaveText("Lizenzen und Quellcode");
    await expect(dora.locator(".notices__source"), "the source code of this build").toHaveAttribute("href", /^https:\/\/github\.com\/PfurtschellerP\/ordo/);
    await expect(dora.locator(".notices__part").filter({ hasText: "MapLibre GL JS" }).getByRole("link", { name: "BSD-3-Clause" })).toHaveAttribute("href", "/map/vendor/maplibre-gl.LICENSE.txt");
    const licence = await dora.request.get(`${server.url}/LICENSES/AGPL-3.0-or-later.txt`);
    expect(await licence.text(), "ORDO's licence text is served").toContain("GNU AFFERO GENERAL PUBLIC LICENSE");
    await expect(dora.locator(".notices__crates .notice").first(), "the crates' notices, made at build time").toBeAttached({ timeout: 15_000 });
    await expect(dora.locator(".notices__crates"), "with the licence texts as the crates ship them").toContainText("leptos");
    await audit(dora, "Licence notices");
    await dora.getByRole("link", { name: "Zurück" }).click();
    await expect(dora.getByRole("button", { name: "Anmelden" }), "back where the person came from").toBeVisible();
    // [itest->dsn~component-gallery~1]
    // DS-1, ADR-0094: the component gallery, before pairing, with the real components.
    await dora.goto(`${server.url}gallery`);
    await expect(dora.getByRole("heading", { level: 1 })).toHaveText("Component gallery");
    for (const part of ["Foundations", "Actions", "Fields", "Status and messages", "Containers", "Lists and values", "Sanität", "Einsatz"]) {
      await expect(dora.getByRole("heading", { level: 2, name: part, exact: true }), `the part ${part}`).toBeVisible();
    }
    await expect(dora.locator("#gallery-icons li"), "every icon with its meaning").not.toHaveCount(0);
    await expect(dora.locator("#gallery-patient-cards .patient-card"), "the package's own building blocks").toHaveCount(5);
    await dora.getByRole("button", { name: "Ort und Personal: UHS Eingang" }).click();
    await expect(dora.getByRole("dialog", { name: "UHS Eingang bearbeiten" })).toBeVisible();
    await dora.keyboard.press("Escape");
    await expect(dora.getByRole("dialog")).toHaveCount(0);
    await expect(dora.getByRole("button", { name: "Ort und Personal: UHS Eingang" }), "the focus returns").toBeFocused();
    await dora.getByRole("button", { name: "Entfernen: Sammelstelle" }).click();
    await expect(dora.locator(".undo-bar"), "a removal offers to undo").toContainText("Sammelstelle entfernt.");
    await dora.getByRole("button", { name: "Rückgängig" }).click();
    await expect(dora.locator(".undo-bar"), "undone, the bar goes").not.toContainText("Sammelstelle");
    await dora.locator("#gallery-theme").selectOption("night");
    await expect(dora.locator("html"), "the switch shows the theme").toHaveAttribute("data-theme", "night");
    await shot(dora, "gallery-night");
    await dora.locator("#gallery-theme").selectOption("high-contrast");
    await shot(dora, "gallery-high-contrast");
    await dora.locator("#gallery-theme").selectOption("day");
    await shot(dora, "gallery");
    await audit(dora, "Component gallery");
    await dora.getByRole("link", { name: "Zurück" }).click();
    await expect(dora.getByRole("button", { name: "Anmelden" })).toBeVisible();
    await expect(dora.locator("html"), "the device's own theme again: the gallery stores none").not.toHaveAttribute("data-theme", /.+/);
    // [itest->dsn~oidc-login~1]
    await expect(dora.locator("#device-name"), "a device name is suggested from the browser").toHaveValue(/^\w+ auf \w+$/);
    await loginPair(dora, server, "dora@example.org", "Laptop Einsatzplanung");
    await expect(dora.locator(".incident-card"), "no incident planned yet").toHaveCount(0);
    await expect(dora.getByRole("button", { name: "Abmelden" }), "logging out also works without an incident").toBeVisible();
    await audit(dora, "Incident list", [1200, 320]);
    await dora.locator("#plan-name").fill(INCIDENT);
    await dora.getByRole("button", { name: "Einsatz anlegen" }).click();
    await expect.poll(() => title(dora), { message: "a new incident opens where it is prepared: the menu follows the work" }).toContain("Einsatzleitung – Vorbereitung");
    await openWorkspace(dora, "Führung");
    // [itest->dsn~workspaces-by-phase~5]
    // ADR-0067: the overview is a grid of tiles: map, Disposition, Personal, UHS, quick links.
    await expect(dora.locator(".bento > .tile"), "the command opens on its overview of tiles").toHaveCount(5);
    await expect(dora.locator("#overview-state"), "with the incident's state").toHaveText("Geplant");
    await expect(dora.locator("#overview-persons"), "counts only").toHaveText("0");
    await expect(dora.locator(".tile--dispatch"), "Disposition keeps its name").toContainText("Keine offenen Aufträge.");
    const tileMap = dora.locator(".tile--map .situation__map");
    await expect(tileMap, "the map tile loads").not.toHaveAttribute("data-map", "loading", { timeout: 30_000 });
    if ((await tileMap.getAttribute("data-map")) === "ready") {
      // Without WebGL (some headless browsers) there is no map to move.
      await expect(
        dora.locator(".tile--map .maplibregl-cooperative-gestures"),
        "the map tile moves with two fingers; one scrolls the page",
      ).toHaveCount(1);
    }
    await audit(dora, "Command overview");
    await shot(dora, "command-overview");
    // … and once paired, under Administration – Info, for everyone.
    await openWorkspace(dora, "Info");
    await expect(dora.locator("#info-app"), "the app's version").toContainText("Protokolle: Abgleich");
    await expect(dora.locator("#info-server"), "the server's, asked from it").toContainText(/0\.0\.0, (Build|lokaler Build)/);
    await expect(dora.locator(".notices__source"), "with the source code of this build").toHaveAttribute("href", /^https:\/\/github\.com\/PfurtschellerP\/ordo/);
    await audit(dora, "Info");
    await openWorkspace(dora, "Zugriff");
    await expect(dora.locator(".views__item"), "administration: accounts, permissions, devices, login").toHaveText(["Konten", "Berechtigungen", "Geräte", "Anmeldung"]);
    await shot(dora, "administration", false);
    await expect(dora.locator("#incident-name"), "the header names the incident").toHaveText(INCIDENT);
    // [itest->dsn~map-region~2]
    // MAP-8: the planner cuts the incident's map region (ADR-0062).
    await openWorkspace(dora, "Vorbereitung");
    // The map region is prepared with the map, on Vorbereitung – Führung & Raum (ADR-0095).
    await openView(dora, "Führung & Raum");
    // [itest->dsn~command-and-space~2]
    // Narrower than 84rem the page is in tabs (ADR-0097); the region is the last one.
    await expect(dora.locator(".command-space .tabs__long"), "structure, map and region").toHaveText(["Führungsstruktur", "Lagekarte", "Kartengrundlage"]);
    await pageTab(dora, "Kartengrundlage");
    await expect(dora.locator("#map-region-title")).toHaveText("Kartengrundlage");
    const region = dora.locator(".map-region__state");
    await expect(region).toHaveText("Für diesen Einsatz ist noch keine Kartengrundlage erstellt.");
    // [itest->dsn~address-search~1]
    // MAP-9: the centre from an address (ADR-0066), searched only when asked.
    await dora.locator("#region-address").fill("Rathausplatz Augsburg");
    const regionSearch = dora.locator(".map-region .address-search");
    await regionSearch.getByRole("button", { name: "Suchen" }).click();
    await expect(regionSearch.locator(".address-search__status"), "the server asked its geocoder").toHaveText("1 Ort gefunden. Bitte wählen.");
    await expect(regionSearch.getByText("© OpenStreetMap-Mitwirkende"), "credited").toBeVisible();
    await dora.getByRole("button", { name: PLACE.label }).click();
    await expect(dora.locator("#region-lat"), "the place fills the centre").toHaveValue("48.37000");
    await expect(dora.locator("#region-lon")).toHaveValue("10.90000");
    await dora.locator("#region-address").fill("Nirgendwo");
    await regionSearch.getByRole("button", { name: "Suchen" }).click();
    await expect(regionSearch.locator(".address-search__status")).toHaveText("Nichts gefunden. Bitte anders schreiben, zum Beispiel mit dem Ort.");
    await dora.locator("#region-lat").fill("48,37");
    await dora.locator("#region-lon").fill("10,9");
    await dora.locator("#region-radius").fill("10");
    await dora.getByRole("button", { name: "Kartengrundlage erstellen" }).click();
    await expect(region, "the server cuts it and reports when it is ready").toHaveText(/^Kartengrundlage bereit \(\d+,\d MB\)\. Umkreis 10 km um 48\.37000, 10\.90000\./, { timeout: 30_000 });
    await audit(dora, "Map region");
    await openView(dora, "Personal");
    await expect(dora.locator(".roster__entry"), "the planner is its commander").toHaveCount(1);
    await grantRole(dora, "anna@localhost", "Sanitätsdienst");
    await grantRole(dora, "ben@localhost", "Einsatzleitung");
    await grantRole(dora, "petra@localhost", "Arzt/Ärztin");
    // A second role for someone registered already: the same card, both roles (ADR-0076).
    await grantRole(dora, "petra@localhost", "Notfallsanitäter/in");
    await expect(dora.locator(".roster__entry[data-user='petra@localhost']"), "registered once").toHaveCount(1);
    await expect(dora.locator(".roster__entry[data-user='petra@localhost'] .roster__role")).toHaveCount(2);
    // [itest->dsn~display-names~2]
    await expect(dora.locator(".roster__entry", { hasText: "petra@localhost" }).locator(".roster__person"), "named from her local account before her first login").toHaveText("Dr. Petra Lang");
    await audit(dora, "Roster", [1200, 320]);
    await openView(dora, "Einsatz");
    // [itest->dsn~incident-details~1]
    // ORG-13: a description and the planned times, kept with the incident.
    await dora.locator("#incident-description").fill("Stadtfest in der Altstadt, drei Tage, etwa 20 000 Gäste.");
    await dora.locator("#incident-starts").fill("2030-05-01T10:00");
    await dora.locator("#incident-ends").fill("2030-04-30T10:00");
    await dora.getByRole("button", { name: "Angaben speichern" }).click();
    await expect(dora.locator("[aria-labelledby=incident-details-title] [role=status]"), "an end before the start is refused").toHaveText("Das Ende liegt vor dem Beginn, oder ein Datum ist unvollständig.");
    await dora.locator("#incident-ends").fill("2030-05-03T23:00");
    await dora.getByRole("button", { name: "Angaben speichern" }).click();
    await expect(dora.locator("#incident-times")).toHaveText("01.05.2030, 10:00 bis 03.05.2030, 23:00");
    await dora.getByRole("button", { name: "Einsatz beginnen" }).click();
    await expect(dora.locator("#incident-state")).toHaveText("Laufend");
    // Paused overnight and resumed the next morning.
    await dora.getByRole("button", { name: "Einsatz unterbrechen" }).click();
    await expect(dora.locator("#incident-state"), "paused, still running").toHaveText("Laufend – unterbrochen");
    await audit(dora, "Incident paused");
    await dora.getByRole("button", { name: "Einsatz fortsetzen" }).click();
    await expect(dora.locator("#incident-state")).toHaveText("Laufend");
    await shot(dora, "roster");
    // [itest->dsn~uhs-registry~2]
    // [itest->dsn~uhs-resources~2]
    // UHS-1, ADR-0086: the incident command sets up the UHS, among the resources.
    await openView(dora, "Einsatzmittel");
    await createUhs(dora, "UHS Bühne Nord", 10);
    await createUhs(dora, "UHS Eingang", 5);
    await expect(dora.locator(".uhs-card"), "two UHS are listed").toHaveCount(2);
    // [itest->dsn~uhs-details~4]
    const eingang = dora.locator(".uhs-card", { hasText: "UHS Eingang" });
    await dora.getByRole("button", { name: "Ort und Personal von UHS Eingang" }).click();
    // One dialog for the list (ADR edit dialog): it stays open while the list updates.
    const details = dora.locator("#uhs-details-dialog");
    await expect(details.locator(".uhs-on-map"), "its place is on the situation map").toHaveText("Auf der Lagekarte: noch nicht gesetzt");
    await details.getByLabel("Name", { exact: true }).fill("anna@localhost");
    await details.getByRole("button", { name: "Zum Personal hinzufügen" }).click();
    await expect(eingang.locator(".uhs-card__staff"), "and assigned staff; the dialog stays open").toHaveText("Personal: 1 Person");
    await expect(details.locator(".crew-list"), "by name").toContainText("Anna Schmidt");
    // A UHS can be renamed (and here back again), in the same dialog.
    await details.getByLabel("Name der UHS").fill("UHS Tor");
    await details.getByRole("button", { name: "Speichern" }).click();
    await expect(details.locator("[role=status]"), "saving says so").toHaveText("Gespeichert.");
    const tor = dora.locator(".uhs-card", { hasText: "UHS Tor" });
    await expect(tor.locator(".uhs-card__name"), "renamed").toHaveText("UHS Tor");
    await expect(dora.getByRole("dialog", { name: "UHS Tor bearbeiten" }), "the dialog follows its item").toBeVisible();
    await details.getByLabel("Name der UHS").fill("UHS Eingang");
    await details.getByRole("button", { name: "Speichern" }).click();
    await expect(eingang.locator(".uhs-card__name")).toHaveText("UHS Eingang");
    // [itest->dsn~edit-dialog~3]
    // Editing happens in a dialog, so the list does not shift; Escape closes it and the
    // focus goes back to its button.
    const editButton = dora.getByRole("button", { name: "Ort und Personal von UHS Eingang" });
    await expect(dora.getByRole("dialog", { name: "UHS Eingang bearbeiten" })).toBeVisible();
    await dora.keyboard.press("Escape");
    await expect(dora.getByRole("dialog")).toHaveCount(0);
    await expect(editButton).toBeFocused();
    // [itest->dsn~uhs-closing~1]
    // UHS-17: a UHS set up by mistake is closed again; it leaves the list, the log keeps it.
    await createUhs(dora, "UHS Irrtum", 2);
    await dora.getByRole("button", { name: "Ort und Personal von UHS Irrtum" }).click();
    await dora.getByRole("button", { name: "UHS auflösen …" }).click();
    await dora.getByRole("button", { name: "Ja, auflösen" }).click();
    await expect(dora.locator(".uhs-card"), "closed").toHaveCount(2);
    await expect(dora.getByRole("dialog")).toHaveCount(0);
  });

  await test.step("a medic registers patients at several UHS", async () => {
    await loginPair(anna, server, "anna@example.org");
    await expect(anna.locator(".incident-card"), "Anna sees the incident she has a role in").toHaveCount(1);
    await expect(anna.locator(".incident-card")).toContainText("Sanitätsdienst");
    await openIncident(anna, INCIDENT);
    // [itest->dsn~display-names~2]
    await expect(anna.locator(".launcher__user"), "the login maps to an ORDO user, shown by the provider's name").toHaveText("Angemeldet als Anna Schmidt (anna@localhost)");
    // [itest->dsn~device-storage-encryption~1]
    for (const item of ["ordo.pairing", "ordo.storage-key"]) {
      await expect.poll(() => storedItem(anna, item), { message: `${item} is stored` }).not.toBeNull();
    }
    const pairing = (await storedItem(anna, "ordo.pairing"))!;
    expect(pairing.startsWith("enc1:") && !pairing.includes("anna") && !pairing.includes("private_key"), "the device key is stored encrypted").toBe(true);
    await expect.poll(() => status(anna), { message: "medic device connects" }).toContain("Verbunden");
    await expect.poll(() => title(anna), { message: "a medic starts in Sanität – UHS" }).toContain("Sanität – UHS");

    // ADR-0086: the UHS module neither sets up nor changes a UHS.
    await expect(anna.getByRole("navigation", { name: "Ansichten" }).getByRole("link", { name: "UHS verwalten" }), "no UHS management in the UHS module").toHaveCount(0);

    await openView(anna, "Patienten");
    // [itest->dsn~triage-reference~1]
    await anna.getByRole("button", { name: "Sichtungshilfe" }).click();
    const aid = anna.locator("#triage-algorithm-help-panel");
    await expect(aid, "the profile's algorithms, with their sources").toContainText("PRIOR");
    await expect(aid.getByRole("link", { name: /Offizielle Quelle/ }).first()).toHaveAttribute("href", /bbk\.bund\.de/);
    await expect(aid, "the person decides the category").toContainText("ORDO schlägt keine vor");
    await anna.getByRole("button", { name: "Sichtungshilfe" }).click();
    await anna.locator("#triage-algorithm").selectOption({ label: "PRIOR (Vorsichtung)" });
    // [itest->dsn~uhs-staff~3]
    await expect(anna.locator("#working-uhs option:checked"), "UHS-9: the UHS whose staff list names her").toHaveText("UHS Eingang");
    await anna.locator("#working-uhs").selectOption({ label: "Alle UHS" });
    await expect(anna.locator("#uhs option"), "UHS-10: she registers only where she is on the staff list").toHaveText(["UHS wählen", "UHS Eingang"]);
    await register(anna, "DE-0044", 1, "UHS Eingang");
    await expect(anna.locator(".patient-card[data-tag='DE-0044']")).toBeVisible();
    // Nobody is on the staff list of UHS Bühne Nord yet: a physician works at every UHS.
    petra = await newDevice(browser);
    await loginPair(petra, server, "petra@example.org");
    await openIncident(petra, INCIDENT);
    await openView(petra, "Patienten");
    for (const [tag, category] of [["DE-0042", 0], ["DE-0043", 1]] as const) {
      await register(petra, tag, category, "UHS Bühne Nord");
      await expect(petra.locator(`.patient-card[data-tag='${tag}']`)).toBeVisible();
    }
    await expect.poll(() => status(petra), { message: "Petra's registrations are sent" }).toContain("Alles gesendet");
    for (const tag of ["DE-0042", "DE-0043"]) await expect(anna.locator(`.patient-card[data-tag='${tag}']`), "the medic reads them").toBeVisible();
    // UHS-9: the person chooses the UHS they work at; the list and registration follow.
    await anna.locator("#working-uhs").selectOption({ label: "UHS Eingang" });
    await expect(anna.locator(".patient-card"), "only the chosen UHS's patients").toHaveCount(1);
    await expect(anna.locator("#uhs option:checked"), "new patients go there").toHaveText("UHS Eingang");
    await anna.locator("#working-uhs").selectOption({ label: "Alle UHS" });
    await expect(anna.locator(".patient-card")).toHaveCount(3);
    await expect.poll(() => status(anna), { message: "registrations are sent" }).toContain("Alles gesendet");
    expect(await patientRows(anna)).toHaveLength(3);

    await register(anna, "DE-0099"); // no triage category chosen
    expect(await patientRows(anna), "registration without a triage category is not possible").toHaveLength(3);
    // [itest->dsn~unique-triage-tag~2]
    // PAT-12: a card already registered, however typed, is not registered again.
    await register(anna, "de 0042", 1, "UHS Eingang");
    await expect(anna.locator("#tag"), "the number is marked").toHaveAttribute("aria-invalid", "true");
    await expect(anna.locator("#tag-in-use"), "and says where the patient is").toContainText(/Die Karte DE-0042 ist schon registriert: UHS Bühne Nord, seit \d\d:\d\d\./);
    await expect(anna.locator("#tag-in-use").getByRole("link", { name: "Akte öffnen" })).toBeVisible();
    expect(await patientRows(anna), "nothing registered").toHaveLength(3);
    await anna.locator("#tag").fill("");
  });

  await test.step("the patient record documents identity, vital signs and a photo", async () => {
    await openRecord(anna, "DE-0044");
    expect(pathOf(anna), "the record has its own address").toBe("/patient/DE-0044");
    await expect(anna.locator("[aria-labelledby=section-triage] .history li").first(), "the algorithm used is recorded").toContainText("nach PRIOR");
    await openSection(anna, "section-triage");
    await anna.getByRole("button", { name: "Nachsichtung DE-0044: SK I", exact: true }).click();
    await expect.poll(() => announced(anna), { message: "re-triage is announced to screen readers" }).toContain("nachgesichtet");
    await expect.poll(() => status(anna), { message: "re-triage is sent" }).toContain("Alles gesendet");
    // [itest->dsn~transport-priority~1]
    await anna.getByRole("button", { name: "DE-0044: Transportpriorität setzen" }).click();
    await expect(anna.locator(".patient-card[data-tag='DE-0044'] .priority-badge"), "the list shows the transport priority").toHaveText(/Transportpriorität/);
    // [itest->dsn~treatment-places~3]
    await anna.locator("#place").selectOption({ label: "Platz 2" });
    await expect(anna.locator(".patient-card[data-tag='DE-0044'] .patient-card__meta"), "the list shows the treatment place").toContainText("Platz 2");
    // [itest->dsn~place-plan~3]
    // UHS-14: working at one UHS, *Patienten* shows who lies where, one list with the plan.
    await expect(anna.locator("#place optgroup"), "the place by triage category first (here: none)").toHaveAttribute("label", "ohne Kategorie");
    await anna.locator("#working-uhs").selectOption({ label: "UHS Eingang" });
    const plan = anna.locator("[aria-labelledby=board-title] .place-plan");
    await expect(anna.locator("#board-title")).toHaveText("Patienten");
    await expect(plan.locator(".place-tile[data-place='p2'] .patient-card"), "the patient's own card on their place").toHaveAttribute("data-tag", "DE-0044");
    await expect(plan.locator(".place-tile--free"), "the others are free").toHaveCount(4);
    await expect(plan.locator(".place-plan__summary")).toHaveText("4 von 5 frei");
    // Patients without a place are listed below the places; a free place takes one of them.
    await anna.locator("#place").selectOption({ label: "Kein Platz" });
    const waiting = plan.locator(".place-plan__waiting");
    await expect(waiting, "waiting for a place").toContainText("Ohne Platz");
    await expect(waiting.locator(".patient-card"), "nobody is lost").toHaveCount(1);
    await expect(waiting.locator(".patient-card")).toHaveAttribute("data-tag", "DE-0044");
    await expect(plan.locator(".place-plan__summary")).toHaveText("5 von 5 frei · 1 ohne Platz");
    await plan.getByRole("button", { name: "Platz 2 belegen" }).click();
    await plan.getByRole("list", { name: "Platz 2 belegen" }).getByRole("button", { name: /DE-0044/ }).click();
    await expect(plan.locator(".place-tile[data-place='p2'] .patient-card"), "put on the free place").toHaveAttribute("data-tag", "DE-0044");
    await expect(waiting, "nobody waits").toHaveCount(0);
    await expect(anna.locator(".patient-card[data-tag='DE-0044'] .patient-card__place"), "the card names the place").toHaveText("Platz 2");
    // Registration proposes the first free place of the chosen category.
    await anna.locator("input[name=category]").nth(2).check();
    await expect(anna.locator("#register-place"), "a free place proposed").toHaveValue("p1");
    await audit(anna, "Place plan");
    await anna.locator("#working-uhs").selectOption({ label: "Alle UHS" });
    await expect(plan, "places only for one UHS; otherwise the list of all").toHaveCount(0);
    await expect(anna.locator(".patient-card[data-tag='DE-0044']")).toBeVisible();

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

    // [itest->dsn~open-choice~1]
    // [itest->dsn~pupil-findings~1]
    // EPCR-12, EPCR-13: an other answer as free text; the pupils as choices.
    await anna.locator("#new-findings").click();
    await fillForm(anna, "findings", { "breathing-other": "Schnappatmung" }, [
      ["breathing", "Atemnot"],
      ["pupils_equal", "anisokor (seitenungleich)"],
      ["pupil_size_right", "weit"],
      ["pupil_light_right", "keine"],
      ["pupil_consensual", "fehlt"],
    ]);
    await audit(anna, "Findings form");
    await saveForm(anna, "findings");
    await expect
      .poll(() => facts(anna, "findings"), { message: "the other answer after the options, and the pupils" })
      .toMatch(/Atemnot, Schnappatmung.*anisokor \(seitenungleich\).*weit.*keine.*fehlt/);
    // [itest->dsn~epcr-diagnosis~1]
    // EPCR-11: a diagnosis with how certain it is, in the record only.
    await anna.locator("#new-diagnosis").click();
    await fillForm(anna, "diagnosis", { text: "Hypoglykämie" }, [["certainty", "Verdachtsdiagnose"]]);
    await saveForm(anna, "diagnosis");
    await expect.poll(() => facts(anna, "diagnosis"), { message: "a diagnosis is documented" }).toMatch(/Verdachtsdiagnose.*Hypoglykämie/);

    // Unsaved input survives switching views (UX-5).
    await anna.locator("#edit-history").click();
    await fillForm(anna, "history", { allergies: "Penicillin" });
    await openView(anna, "Behandlungsplätze");
    await openView(anna, "Patienten");
    await openRecord(anna, "DE-0044");
    await expect(anna.locator("#f-history-allergies"), "unsaved input is still there").toHaveValue("Penicillin");
    await saveForm(anna, "history");

    // [itest->dsn~record-overview~3]
    // EPCR-14: the record opens on its overview; the header keeps the essentials in view.
    await openView(anna, "Behandlungsplätze");
    await openView(anna, "Patienten");
    await openRecord(anna, "DE-0044", "Übersicht");
    const header = anna.locator(".record__header");
    await expect(header.locator(".record__facts"), "the profile's header fields").toContainText("Mustermann");
    await expect(header.locator(".record__facts"), "allergies always in view").toContainText("Allergien");
    await expect(header.locator(".record__where"), "where the patient is").toContainText("UHS Eingang · Platz 2");
    const vitalsTile = anna.locator("[aria-labelledby=tile-vitals]");
    await expect(vitalsTile.locator("h3"), "the latest entry, with how long ago").toContainText(/zuletzt \d\d:\d\d \((gerade eben|vor \d+ Min\.)\)/);
    await expect(vitalsTile.locator("dl"), "short labels from the profile, the corrected value").toContainText("HF92 /min");
    await expect(vitalsTile.locator("dl")).toContainText("GCS15");
    await expect(anna.locator("[aria-labelledby=tile-diagnosis]")).toContainText("Hypoglykämie");
    await audit(anna, "Record overview");
    // A quick action opens its form in *Dokumentation*.
    await anna.locator(".quick-actions").getByRole("button", { name: "Neuer Eintrag: Vitalwerte" }).click();
    await expect(anna.getByRole("tab", { name: /^Doku(mentation)?$/ })).toHaveAttribute("aria-selected", "true");
    await expect(anna.locator("[aria-labelledby=section-vitals] form.editor")).toBeVisible();
    await anna.locator("[aria-labelledby=section-vitals]").getByRole("button", { name: "Abbrechen" }).click();
    // [itest->dsn~record-timeline~1]
    // EPCR-15: one timeline, newest first, filtered by kind; a correction stays visible.
    await recordTab(anna, "Verlauf");
    const lines = anna.locator(".timeline__line");
    await expect(lines.first(), "newest first").toBeVisible();
    await anna.locator(".timeline__filters").getByRole("button", { name: "Vitalwerte" }).click();
    await expect(lines, "two entries of vital signs").toHaveCount(2);
    await expect(anna.locator(".timeline__line--corrected"), "the corrected one struck through").toHaveCount(1);
    await expect(lines.first(), "the correction names the entry it corrects").toContainText("korrigiert den Eintrag von");
    await anna.locator(".timeline__filters").getByRole("button", { name: "Alles" }).click();
    await expect(lines.filter({ hasText: "Erfasst: SK" }), "the registration too").toHaveCount(1);
    await audit(anna, "Record timeline");
    await recordTab(anna, "Dokumentation");

    // [itest->dsn~person-suggestions~1]
    await anna.locator("#new-medication").click();
    const givenBy = anna.locator("#f-medication-given_by");
    await expect(givenBy, "a person field suggests the incident's people").toHaveAttribute("list", "f-medication-given_by-people");
    const suggested = anna.locator("#f-medication-given_by-people option");
    await expect(suggested.first(), "the person using the device first").toHaveAttribute("value", "anna@localhost");
    await expect(anna.locator("#f-medication-given_by-people option[value='petra@localhost']"), "and the others on the roster").toHaveCount(1);
    await givenBy.fill("Dr. Extern");
    await expect(givenBy, "any other name can be typed").toHaveValue("Dr. Extern");
    await anna.locator("[aria-labelledby=section-medication]").getByRole("button", { name: "Abbrechen" }).click();

    // [itest->dsn~patient-photo~3]
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
    await expect(anna.locator("#profile-button img.avatar"), "the own picture appears in the header").toBeVisible({ timeout: 20_000 });
    const photo = anna.locator("img.photo");
    await expect(photo, "a photo for identification is shown in the record").toBeVisible({ timeout: 20_000 });
    await expect(photo, "the taking device shows the full photo at once").toHaveAttribute("data-photo", "full");
    await expect.poll(() => photo.evaluate((i: HTMLImageElement) => i.complete && i.naturalWidth > 0)).toBe(true);
    patientPhoto = Buffer.from((await photo.getAttribute("src"))!.split(",")[1], "base64");
    const [width, height, bytes, prefix] = await photo.evaluate((i: HTMLImageElement) => [
      i.naturalWidth,
      i.naturalHeight,
      atob(i.src.split(",")[1]).length,
      i.src.slice(0, 23),
    ]);
    expect(Number(width), "downscaled").toBeLessThanOrEqual(1600);
    expect(Number(height), "downscaled").toBeLessThanOrEqual(1600);
    expect(Number(bytes), "within the size limit").toBeLessThanOrEqual(1024 * 1024);
    expect(prefix, "re-encoded as JPEG").toBe("data:image/jpeg;base64,");
    await expect.poll(() => status(anna), { message: "record entries and photo are sent" }).toContain("Alles gesendet");
    // PAT-8: a wrong photo is removed again; the one before shows.
    const firstSrc = (await photo.getAttribute("src"))!;
    const wrong = await anna.evaluate(() => {
      const canvas = document.createElement("canvas");
      canvas.width = 400;
      canvas.height = 300;
      const ctx = canvas.getContext("2d")!;
      ctx.fillStyle = "#336";
      ctx.fillRect(0, 0, 400, 300);
      return canvas.toDataURL("image/png").split(",")[1];
    });
    await anna.locator("#photo-upload").setInputFiles({ name: "wrong.png", mimeType: "image/png", buffer: Buffer.from(wrong, "base64") });
    await expect(photo, "the new photo shows").not.toHaveAttribute("src", firstSrc, { timeout: 20_000 });
    await anna.getByRole("button", { name: "Foto entfernen – Patient DE-0044" }).click();
    await expect(photo, "removed: the photo before shows again").toHaveAttribute("src", firstSrc);
    await expect.poll(() => status(anna), { message: "the removal is sent" }).toContain("Alles gesendet");

    // [itest->dsn~transport-request~1]
    await showTabOf(anna, "[aria-labelledby=section-transport]");
    const transport = anna.locator("[aria-labelledby=section-transport]");
    await transport.getByLabel("RTW – Rettungswagen").check();
    await transport.getByRole("button", { name: "Transport anfordern" }).click();
    await expect(transport.locator("[role=alert]"), "type and priority are needed").toHaveText("Bitte Transportmittel und Dringlichkeit wählen.");
    await transport.getByLabel("Sofort", { exact: true }).check();
    await transport.getByLabel("Infektiös", { exact: true }).check();
    await transport.getByRole("button", { name: "Transport anfordern" }).click();
    await expect(transport.locator(".transport-card__status"), "the request is shown on the record").toHaveText("Angefordert");
    await expect(transport.locator(".transport-card")).toContainText("RTW · Sofort · Infektiös");
    await expect(transport.locator("form"), "no second request while one is open").toHaveCount(0);

    // [itest->dsn~planned-transports~1]
    // At UHS Bühne Nord, by the physician: Anna only reads its patients (UHS-10).
    await openRecord(anna, "DE-0042");
    await expect(anna.locator(".record__read-only"), "another UHS's record is read-only").toContainText("UHS Bühne Nord");
    await anna.goBack();
    await expect(anna.locator("#record-title")).toHaveText("Patient DE-0044");
    await openRecord(petra, "DE-0042");
    await showTabOf(petra, "[aria-labelledby=section-transport]");
    const later = petra.locator("[aria-labelledby=section-transport]");
    await later.getByLabel("KTW – Krankentransportwagen").check();
    await later.getByLabel("Normal", { exact: true }).check();
    await petra.locator("#transport-planned").fill("2030-05-01T18:30");
    await later.getByRole("button", { name: "Transport anfordern" }).click();
    await expect(later.locator(".transport-card__planned"), "a transport is planned for later").toHaveText("Geplant für 01.05.2030, 18:30");
    await petra.goBack();

    // [itest->dsn~medical-orders~2]
    // [itest->dsn~order-permission~2]
    // [itest->dsn~recorded-orders~1]
    const annaOrderForm = anna.locator(".order-form");
    await openSection(anna, "section-orders");
    await expect(annaOrderForm.getByRole("button", { name: "Anordnen", exact: true }), "a medic gives no orders").toHaveCount(0);
    await expect(annaOrderForm, "she records spoken ones only").not.toContainText("Wer ordnet an?");
    await expect(annaOrderForm.getByRole("button", { name: "Anordnung erfassen" })).toBeVisible();
    await openRecord(petra, "DE-0044");
    await openSection(petra, "section-orders");
    await petra.locator("#order-substance").fill("Paracetamol");
    await petra.locator("#order-dose").fill("1 g");
    await petra.getByRole("button", { name: "Anordnen" }).click();
    await expect(petra.locator(".order-form [role=alert]"), "an incomplete order is not saved").toHaveText("Bitte die Anordnung vollständig angeben.");
    await petra.locator("#order-route").selectOption({ label: "i.v. – intravenös" });
    await petra.getByRole("button", { name: "Anordnen" }).click();
    await expect(petra.locator(".order-card .order-card__what"), "the physician gives an order").toHaveText("Paracetamol 1 g i.v.");
    await expect(petra.locator(".order-card .order-card__status")).toContainText("Angeordnet");
    await expect.poll(() => status(petra), { message: "the order is sent" }).toContain("Alles gesendet");
    await petra.context().close();

    await openView(anna, "Anordnungen");
    const order = anna.locator(".order-card");
    await expect(order, "the crew sees the open order with patient and UHS").toContainText("DE-0044 · UHS Eingang");
    await expect(order).toContainText("Dr. Petra Lang");
    await order.getByRole("button", { name: "Übernehmen: Paracetamol 1 g i.v." }).click();
    await expect(order.locator(".order-card__status")).toContainText("Übernommen");
    await audit(anna, "Orders");
    // ORDER-2: Anna reports it, Paul gave it.
    await expect(order.getByLabel("Durchgeführt von"), "the reporting person by default").toHaveValue("anna@localhost");
    await order.getByLabel("Durchgeführt von").fill("Paul Beispiel");
    await order.getByRole("button", { name: "Durchgeführt: Paracetamol 1 g i.v." }).click();
    await expect(anna.locator("#main"), "done orders leave the open list").toContainText("Keine offenen Anordnungen.");
    await openView(anna, "Patienten");
    await openRecord(anna, "DE-0044");
    await expect(anna.locator("[aria-labelledby=section-orders] .order-card__status"), "the record keeps it").toContainText("Durchgeführt");
    await anna.locator("[aria-labelledby=section-orders] summary").click();
    await expect(anna.locator("[aria-labelledby=section-orders] .order-card__performed"), "who carried it out").toHaveText("Durchgeführt von Paul Beispiel");
    const steps = anna.locator("[aria-labelledby=section-orders] .history-list li");
    await expect(steps, "with every step").toHaveCount(3);
    await expect(steps.nth(2), "apart from who reported it").toHaveText(/^\d\d:\d\d Durchgeführt \(durch Paul Beispiel\) · Anna Schmidt$/);
    const doneAt = (await steps.nth(2).innerText()).slice(0, 5);
    // [itest->dsn~order-documentation~3]
    await anna.getByRole("button", { name: "In Dokumentation übernehmen: Paracetamol 1 g i.v." }).click();
    await expect(anna.locator("#f-medication-drug"), "the done order fills a new medication entry").toHaveValue("Paracetamol");
    await expect(anna.locator("#f-medication-dose")).toHaveValue("1");
    await expect(anna.locator("#f-medication-dose_unit").getByLabel("g", { exact: true })).toBeChecked();
    await expect(anna.locator("#f-medication-route").getByLabel("i.v.", { exact: true })).toBeChecked();
    await expect(anna.locator("#f-medication-ordered_by"), "with who ordered it").toHaveValue("petra@localhost");
    await expect(anna.locator("#f-medication-ordered_by"), "taken from the order, not typed").toHaveJSProperty("readOnly", true);
    await expect(anna.locator("#f-medication-given_by"), "and who gave it, not who documents it").toHaveValue("Paul Beispiel");
    await expect(anna.locator("#f-medication-time"), "when it was reported done").toHaveValue(doneAt);
    await saveForm(anna, "medication");
    await expect(anna.locator("[aria-labelledby=section-medication] .entry"), "the crew saves it").toContainText("Paracetamol");
    await expect(anna.locator("[aria-labelledby=section-medication] .entry"), "who ordered it, from the order").toContainText("Dr. Petra Lang (Anordnung");

    // [itest->dsn~recorded-orders~1]
    // ORDER-6: Petra orders by radio; Anna records it, carries it out and documents it,
    // Petra confirms it later on her device.
    await openSection(anna, "section-orders");
    await anna.locator("#order-given-by").fill("petra@localhost");
    await annaOrderForm.getByLabel("per Funk").check();
    await annaOrderForm.getByLabel("Wiederholt und bestätigt (Closed Loop)").check();
    await anna.locator("#order-substance").fill("Novaminsulfon");
    await anna.locator("#order-dose").fill("1 g");
    await anna.locator("#order-route").selectOption({ label: "i.v. – intravenös" });
    await annaOrderForm.getByRole("button", { name: "Anordnung erfassen" }).click();
    const spoken = anna.locator("[aria-labelledby=section-orders] .order-card").filter({ hasText: "Novaminsulfon" });
    await expect(spoken.locator(".order-card__spoken"), "who gave it, how, and that it was read back").toHaveText("Mündlich angeordnet von Dr. Petra Lang, per Funk, wiederholt und bestätigt");
    await expect(spoken.locator(".order-card__status"), "apart from who recorded it").toContainText("von Anna Schmidt");
    await expect(spoken.locator(".order-card__unconfirmed")).toHaveText("Noch nicht bestätigt");
    await spoken.getByRole("button", { name: "Durchgeführt: Novaminsulfon 1 g i.v." }).click();
    // [itest->dsn~order-documentation~3]
    await spoken.getByRole("button", { name: "In Dokumentation übernehmen: Novaminsulfon 1 g i.v." }).click();
    await expect(anna.locator("#f-medication-ordered_by"), "the giver of the spoken order").toHaveValue("petra@localhost");
    await saveForm(anna, "medication");
    await expect(anna.locator("[aria-labelledby=section-medication] .entry").filter({ hasText: "Novaminsulfon" })).toContainText("Dr. Petra Lang (Anordnung");
    await expect.poll(() => status(anna), { message: "the spoken order is sent" }).toContain("Alles gesendet");
    const petraLater = await newDevice(browser);
    await loginPair(petraLater, server, "petra@example.org");
    await openIncident(petraLater, INCIDENT);
    await openView(petraLater, "Patienten");
    await openRecord(petraLater, "DE-0044");
    await petraLater.getByRole("button", { name: "Anordnung bestätigen: Novaminsulfon 1 g i.v." }).click();
    await expect(petraLater.locator(".order-card__confirmed"), "the giver confirms it").toContainText("von Dr. Petra Lang");
    await expect.poll(() => status(petraLater), { message: "the confirmation is sent" }).toContain("Alles gesendet");
    await petraLater.context().close();
    await expect(spoken.locator(".order-card__confirmed"), "the crew sees the confirmation").toContainText("Bestätigt");

    // [itest->dsn~attachment-transfer~1]
    // Another device of Anna's gets the preview with the record, then the full photo from
    // the server (an encrypted attachment, ADR-0033).
    const second = await newDevice(browser);
    await loginPair(second, server, "anna@example.org");
    await openIncident(second, INCIDENT);
    await openView(second, "Patienten");
    await openRecord(second, "DE-0044");
    const secondPhoto = second.locator("img.photo");
    await expect(secondPhoto, "the other device downloads the full photo").toHaveAttribute("data-photo", "full", { timeout: 30_000 });
    const sameImage = await secondPhoto.getAttribute("src");
    expect(sameImage!.split(",")[1] === patientPhoto.toString("base64"), "it is the same photo").toBe(true);
    await second.context().close();
    await audit(anna, "Patient record");
    await shot(anna, "record");

    await anna.goBack();
    await expect.poll(() => pathOf(anna), { message: "the back button returns to the list" }).toBe("/med.uhs/patients");
    await expect(anna.locator("#record-title")).toHaveCount(0);
    await expect(anna.locator(".patient-card[data-tag='DE-0044']"), "the list shows the name").toContainText("Mustermann, Max");
    // [itest->dsn~device-storage~1]
    const storageKind = await anna.evaluate(() => document.documentElement.dataset.storage);
    const storageError = await anna.evaluate(() => document.documentElement.dataset.storageError);
    console.log(`device storage in ${test.info().project.name}: ${storageKind}${storageError ? ` (${storageError})` : ""}`);
    if (test.info().project.name === "webkit" && storageKind === "local") {
      // Playwright's WebKit may lack OPFS in workers; the app then keeps localStorage.
      test.info().annotations.push({ type: "device storage", description: `fallback to localStorage: ${storageError}` });
    } else {
      expect(storageKind, "the device uses its database on OPFS").toBe("opfs");
      expect(await storedItem(anna, "ordo.replica"), "the incident data is no longer in localStorage").toBeNull();
      const opfs = await anna.evaluate(async () => {
        const names: string[] = [];
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        for await (const name of (await navigator.storage.getDirectory() as any).keys()) names.push(name);
        return names;
      });
      expect(opfs.length, "it is in the device database on the browser's file system (OPFS)").toBeGreaterThan(0);
    }
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
    await anna.keyboard.type("0044");
    await expect.poll(() => selectedOption(anna), { message: "part of a triage tag number finds the patient" }).toBe("DE-0044");
    await audit(anna, "Command palette", [1200]);
    await anna.keyboard.press("Enter");
    await anna.keyboard.type("retriage minor"); // English command name in a German UI (ADR-0036)
    await expect.poll(() => selectedOption(anna), { message: "the patient's quick actions are offered" }).toBe("Sichtung ändern: SK III");
    await anna.keyboard.press("Enter");
    await expect.poll(() => announced(anna), { message: "a re-triage runs from the palette" }).toContain("DE-0044 nachgesichtet: SK III");
    await expect(anna.locator("#palette"), "the palette closes after the action").not.toHaveAttribute("open", "");
    await openPalette(anna);
    await anna.keyboard.type("vitalwerte");
    await expect(anna.locator("#palette-list [role=option]"), "patient commands only for a chosen patient").toHaveCount(0);
    await anna.keyboard.press("Escape");
    await expect(anna.locator("#palette"), "Escape closes the palette").not.toHaveAttribute("open", "");
    await expect(anna.locator("#tag"), "focus returns to where it was").toBeFocused();
    // A patient of another UHS: only opening the record is offered (UHS-10).
    await openPalette(anna);
    await anna.keyboard.type("0043");
    await anna.keyboard.press("Enter");
    await anna.keyboard.type("add vitals");
    await expect(anna.locator("#palette-list [role=option]"), "nothing to write where she may only read").toHaveCount(0);
    await anna.keyboard.press("Escape");
    // A quick action that opens a form brings it into view, with the cursor in its first field.
    await openPalette(anna);
    await anna.keyboard.type("0044");
    await anna.keyboard.press("Enter");
    await anna.keyboard.type("add vitals");
    await expect.poll(() => selectedOption(anna)).toBe("Vitalwerte eintragen");
    await anna.keyboard.press("Enter");
    const firstField = anna.locator("[aria-labelledby=section-vitals] .editor :is(input, select, textarea)").first();
    await expect(firstField, "the form's first field has the focus").toBeFocused();
    await expect(firstField, "and is on screen").toBeInViewport();
    await anna.locator("[aria-labelledby=section-vitals] .editor").getByRole("button", { name: "Abbrechen" }).click();
    await anna.goBack();

    // [itest->dsn~list-keys~1]
    // Ctrl+J and Ctrl+K move in the palette, where letters are typed (ADR-0051).
    await openPalette(anna);
    await shot(anna, "palette-all");
    const overlap = await anna.locator("#palette-list [role=option]").evaluateAll((options) =>
      options.slice(1).some((o, i) => o.getBoundingClientRect().top < options[i].getBoundingClientRect().bottom - 1));
    expect(overlap, "a long list scrolls; its options never overlap").toBe(false);
    await anna.keyboard.type("DE-00");
    await shot(anna, "palette");
    const first = await selectedOption(anna);
    await anna.keyboard.press("Control+j");
    await expect.poll(() => selectedOption(anna), { message: "Ctrl+J moves down" }).not.toBe(first);
    await anna.keyboard.press("Control+k");
    await expect.poll(() => selectedOption(anna), { message: "Ctrl+K moves up, and keeps the palette open" }).toBe(first);
    await expect(anna.locator("#palette")).toHaveAttribute("open", "");
    await closePalette(anna);
    // j and k move through a list while it has the focus.
    const card = (tag: string) => anna.locator(`.patient-card[data-tag='${tag}']`);
    await card("DE-0042").focus();
    await anna.keyboard.press("j");
    await expect(card("DE-0043"), "j: the next patient").toBeFocused();
    await anna.keyboard.press("ArrowDown");
    await expect(card("DE-0044"), "or Arrow Down").toBeFocused();
    await anna.keyboard.press("j");
    await expect(card("DE-0044"), "at the end the focus stays").toBeFocused();
    await anna.keyboard.press("k");
    await expect(card("DE-0043"), "k: the previous one").toBeFocused();
    await anna.locator("#tag").focus();
    await anna.keyboard.type("jk");
    await expect(anna.locator("#tag"), "in a text field j and k are typed").toHaveValue("jk");
    await anna.locator("#tag").fill("");

    // [itest->dsn~vim-keys~2]
    // UX-15 (ADR-0090): vim keys for the whole app, turned on in the menu on this device.
    await anna.locator("#profile-button").click();
    await anna.locator("#keymap").selectOption({ label: "Vim-Tasten" });
    await anna.keyboard.press("Escape");
    await anna.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());
    const cards = anna.locator("#main [data-keys=list] .patient-card");
    // ADR-0091: h j k l go to the nearest element that way, across the whole page.
    await cards.first().focus();
    await anna.keyboard.press("j");
    await expect(cards.nth(1), "j: the element below").toBeFocused();
    await expect(cards.nth(1), "lit up for a moment").toHaveClass(/keys-flash/);
    await anna.keyboard.press("k");
    await expect(cards.first(), "k: the one above").toBeFocused();
    await anna.keyboard.press("?");
    const help = anna.locator("#keys-dialog");
    await expect(help, "? shows the keys").toBeVisible();
    await expect(help).toContainText("Vim-Tasten sind auf diesem Gerät an.");
    await audit(anna, "Key help");
    await anna.keyboard.press("Escape");
    await expect(help).toBeHidden();
    await expect(cards.first(), "and the focus comes back").toBeFocused();
    await anna.keyboard.press("G");
    const lastElement = await anna.evaluate(() => document.activeElement?.outerHTML ?? "");
    await anna.keyboard.press("g");
    await anna.keyboard.press("g");
    const firstElement = await anna.evaluate(() => document.activeElement?.outerHTML ?? "");
    expect(firstElement, "gg and G: the first and the last element of the page").not.toBe(lastElement);
    expect(
      await anna.evaluate(() => [...document.querySelectorAll(".keys-flash")].map((e) => e === document.activeElement)),
      "it is lit up, and only it",
    ).toEqual([true]);
    await anna.keyboard.press(":");
    await expect(anna.locator("#palette"), ": opens the palette").toHaveAttribute("open", "");
    await closePalette(anna);
    const views = anna.locator(".views .views__item");
    await anna.keyboard.press("g");
    await anna.keyboard.press("2");
    await expect(views.nth(1), "g2: the second view").toHaveAttribute("aria-current", "page");
    await anna.keyboard.press("g");
    await anna.keyboard.press("1");
    await expect(views.nth(0), "g1: the first").toHaveAttribute("aria-current", "page");
    await openView(anna, "Patienten");
    await anna.locator("#tag").focus();
    await anna.keyboard.press("Escape");
    await expect(anna.locator("#tag"), "Esc leaves a text field").not.toBeFocused();
    await anna.locator("#profile-button").click();
    await anna.locator("#keymap").selectOption({ label: "Standard" });
    await anna.keyboard.press("Escape");
  });

  await test.step("input survives being offline and a reload", async () => {
    // [itest->dsn~offline-app-shell~2]
    await expect.poll(() => anna.evaluate(() => !!navigator.serviceWorker.controller), { message: "the app shell is installed for offline use (the first install fetches every file)", timeout: 30_000 }).toBe(true);
    await expect
      .poll(() => anna.evaluate(async () => (await caches.keys()).filter((k) => k.startsWith("ordo-app-shell")).map((k) => /^ordo-app-shell-[0-9a-f]{16}$/.test(k))), {
        message: "one cache, for the version of the app that runs",
      })
      .toEqual([true]);
    if (test.info().project.name === "chromium") {
      // Chrome offers to install the app (PWA): manifest with name, icons, start URL.
      const cdp = await anna.context().newCDPSession(anna);
      const { installabilityErrors } = await cdp.send("Page.getInstallabilityErrors");
      expect(installabilityErrors, "Chrome can install ORDO as an app").toEqual([]);
    }
    await server.stop();
    await expect.poll(() => status(anna), { message: "loss of connection is shown" }).toContain("Offline");
    await expect(anna.locator(".connection"), "offline has its own icon and colour").toHaveClass(/connection--offline/);
    const saved = () => anna.evaluate(() => Number(document.documentElement.dataset.saved ?? 0));
    const before = await saved();
    await register(anna, "DE-0045", 2, "UHS Eingang");
    await expect.poll(() => status(anna), { message: "pending input is shown" }).toContain("1 Eingabe noch nicht gesendet");
    // The write to device storage ends a moment after the change shows.
    await expect.poll(saved, { message: "saved on the device" }).toBeGreaterThan(before);
    // [itest->dsn~local-persistence~2]
    await anna.reload(); // the server is down: the app must start from the device
    await expect.poll(() => patientRows(anna), { message: "offline registration survives a reload" }).toContain("DE-0045 SK III noch nicht gesendet");
    if ((await anna.evaluate(() => document.documentElement.dataset.storage)) === "opfs") {
      // [itest->dsn~attachment-transfer~1]
      await openRecord(anna, "DE-0044");
      await expect(anna.locator("img.photo"), "the full photo is kept on the device").toHaveAttribute("data-photo", "full");
      await anna.goBack();
      await expect(anna.locator("#record-title")).toHaveCount(0);
    }
    await server.start();
    await expect.poll(() => status(anna), { message: "after reconnecting, everything is sent", timeout: 20_000 }).toContain("Alles gesendet");
    // [itest->dsn~triage-badge~1]
    const badges = await anna.locator(".triage").evaluateAll((list) =>
      list.map((b) => !!b.querySelector(".triage__shape") && (b as HTMLElement).innerText.trim().length > 0),
    );
    expect(badges.length).toBeGreaterThan(0);
    expect(badges.every(Boolean), "every triage badge has a shape and a text label").toBe(true);
    await audit(anna, "Med workspace");

    // [itest->dsn~patient-pathway~2]
    // [itest->dsn~patient-handover~1]
    // EPCR-17 (ADR-0087): a move to another UHS is a handover; the receiver takes it.
    await openRecord(anna, "DE-0045", "Übergabe");
    const handover = anna.locator("form.handover-form");
    await expect(anna.locator(".flow__step--next"), "the way ends with where next").toContainText("an wen übergeben");
    await handover.getByLabel(/^UHS oder Behandlungsplatz/).check();
    const north = await anna.locator("#handover-target option", { hasText: "UHS Bühne Nord" }).getAttribute("value");
    await anna.locator("#handover-target").selectOption(north!);
    await anna.locator("#handover-note").fill("Schmerzen im linken Arm");
    await expect(handover.locator("#handover-gets")).toHaveText("Was UHS Bühne Nord bekommt");
    await audit(anna, "Handover");
    await shot(anna, "handover");
    await anna.getByRole("button", { name: "An UHS Bühne Nord übergeben" }).click();
    const open = anna.locator("#handover-open");
    await expect(open, "both sides see it open").toContainText("Übergabe an UHS Bühne Nord offen");
    await expect(open).toContainText("Schmerzen im linken Arm");
    await expect(anna.locator(".record__read-only"), "until taken, the UHS still writes").toHaveCount(0);
    await expect(anna.locator(".patient-card[data-tag='DE-0045']"), "and the list says so").toContainText("Übergabe an UHS Bühne Nord offen");
    await expect.poll(() => status(anna), { message: "the handover is sent" }).toContain("Alles gesendet");
    // Nobody is on Bühne Nord's staff list: the physician takes the patient for it.
    const physician = await newDevice(browser);
    await loginPair(physician, server, "petra@example.org");
    await openIncident(physician, INCIDENT);
    await openView(physician, "Patienten");
    await openRecord(physician, "DE-0045", "Übersicht");
    // [itest->dsn~take-with-place~1]
    // UHS-18: taking proposes a free place kept for the category; here none is chosen.
    const takePlace = physician.locator("#handover-take-place");
    await expect(takePlace, "a free place is proposed").not.toHaveValue("");
    await takePlace.selectOption({ label: "Kein Platz" });
    await physician.getByRole("button", { name: "Übernehmen für UHS Bühne Nord: Patient DE-0045" }).click();
    await expect(physician.locator("#handover-open"), "taken: no longer open").toHaveCount(0);
    await expect.poll(() => status(physician), { message: "the taking is sent" }).toContain("Alles gesendet");
    await physician.context().close();
    await expect(open, "taken on the other device too").toHaveCount(0);
    await expect(anna.locator(".patient-card[data-tag='DE-0045']"), "the patient is where they were taken").toContainText("UHS Bühne Nord");
    await showTabOf(anna, "[aria-labelledby=section-stay]");
    const way = anna.locator("[aria-labelledby=section-stay] .history li");
    await expect(way, "registered, handed over, taken").toHaveCount(3);
    await expect(way.nth(1), "the way, with time and person").toContainText("Übergabe an UHS Bühne Nord");
    await expect(way.nth(2)).toContainText("Übernommen: UHS Bühne Nord");
    await expect(anna.locator(".record__read-only"), "taken: the other UHS's staff writes it now").toContainText("UHS Bühne Nord");
    // [itest->dsn~handover-document~1]
    // EPCR-20 (ADR-0089): the handover document, made on the server, as PDF/A-3.
    await recordTab(anna, "Übergabe");
    const saving = anna.waitForEvent("download");
    await anna.getByRole("button", { name: "PDF herunterladen: Übergabedokument Patient DE-0045" }).click();
    const pdf = await saving;
    expect(pdf.suggestedFilename()).toBe("Uebergabe-DE-0045.pdf");
    const bytes = fs.readFileSync((await pdf.path())!);
    expect(bytes.subarray(0, 5).toString(), "a PDF").toBe("%PDF-");
    expect(bytes.includes("pdfaid:part"), "PDF/A").toBe(true);
    expect(bytes.includes("fhir-bundle-DE-0045.json"), "with the FHIR bundle attached").toBe(true);
    await expect(anna.locator("#document-state")).toHaveText("Heruntergeladen.");

    // [itest->dsn~uhs-overview~3]
    // UHS-8: the places of every UHS, to see which is full and where else a patient could go.
    await openView(anna, "Behandlungsplätze");
    const unit = (name: string) => anna.locator(".places__unit", { hasText: name });
    await expect(unit("UHS Eingang").locator(".places__figure"), "free places in large figures").toHaveText("4 frei von 5");
    await expect(unit("UHS Eingang").locator(".places__boxes"), "a box per place, with a text alternative").toHaveAttribute("aria-label", "ohne Kategorie: 1 von 5 belegt, 4 frei");
    await expect(unit("UHS Eingang").locator(".places__box--taken")).toHaveCount(1);
    await expect(unit("UHS Eingang"), "open orders and transports").toContainText("Offene Anordnungen: 0 · Transporte: 1 offen, 0 geplant");
    await expect(unit("UHS Bühne Nord").locator(".places__waiting"), "patients without a place").toHaveText("3 ohne Platz");
    await expect(unit("UHS Bühne Nord")).toContainText("Transporte: 0 offen, 1 geplant");
    const allUnits = anna.locator(".places__all");
    await expect(allUnits.locator(".places__figure"), "and all units added up").toHaveText("14 frei von 15");
    await expect(allUnits.locator(".places__waiting")).toHaveText("3 ohne Platz");
    await audit(anna, "Treatment places");
    await shot(anna, "treatment-places");
    await openView(anna, "Patienten");
    await anna.locator("#launcher-button").click();
    // [itest->dsn~views-by-role~1]
    await expect(anna.locator("#reason-command-lead"), "UHS crews do not see the situation view").toHaveText("Kein Zugriff in diesem Einsatz");
    await expect(anna.locator("#reason-command-prepare"), "nor its preparation").toHaveText("Kein Zugriff in diesem Einsatz");
    await expect(anna.locator("#reason-admin-access"), "nor the administration").toHaveText("Kein Zugriff in diesem Einsatz");
    await expect(anna.locator("#launcher-panel").getByRole("button", { name: "Einsatztagebuch", exact: true }), "the incident log is for everyone").not.toHaveAttribute("aria-disabled", "true");
    await audit(anna, "Launcher (medic)");
    await anna.locator("#launcher-button").click();
    await shot(anna, "med-day");
  });

  await test.step("on phones, views and menu are at the bottom", async () => {
    await anna.setViewportSize({ width: 390, height: 844 });
    await settle(anna);
    const [barBottom, viewportHeight, menuInBar] = await anna.evaluate(() => {
      const n = document.querySelector(".navigation")!.getBoundingClientRect();
      const m = document.getElementById("launcher-bar-button")!.getBoundingClientRect();
      return [Math.round(n.bottom), innerHeight, m.top >= n.top && m.bottom <= n.bottom];
    });
    expect(barBottom, "the bar sits at the bottom").toBe(viewportHeight);
    expect(menuInBar, "the menu is in the bar").toBe(true);
    // [itest->dsn~header-layout~2]
    await expect(anna.getByRole("button", { name: "Menü – Arbeitsbereich wechseln" }), "exactly one menu button: the one in the bar").toHaveCount(1);
    await expect(anna.locator("#launcher-button"), "the header's menu button is hidden on phones").toBeHidden();
    const menuAtEnd = () =>
      anna.evaluate(() => {
        const m = document.getElementById("launcher-bar-button")!.getBoundingClientRect();
        const v = [...document.querySelectorAll(".views__item")].map((e) => e.getBoundingClientRect());
        return [v.every((r) => r.right <= m.left + 1), v.every((r) => r.left >= m.right - 1)];
      });
    expect(await menuAtEnd(), "right-handed: the menu at the right end").toEqual([true, false]);
    // [itest->dsn~compact-navigation~2]
    await expect(anna.locator(".navigation .views__item .icon svg"), "icons above the labels in the bar").toHaveCount(3); // Patienten, Anordnungen, Behandlungsplätze
    const [iconTop, labelTop] = await anna.evaluate(() => {
      const item = document.querySelector(".navigation .views__item")!;
      return [item.querySelector(".icon")!.getBoundingClientRect().top, item.querySelector(".views__label")!.getBoundingClientRect().top];
    });
    expect(iconTop, "the icon sits above the label").toBeLessThan(labelTop);
    await expect(anna.locator("#connection-panel"), "the connection is an icon, its words behind it").toBeHidden();
    const connection = anna.locator("#connection-button");
    await expect(connection).toHaveAccessibleName("Verbindung: Verbunden. Alles gesendet");
    await connection.click();
    await expect(anna.locator("#connection-panel"), "it opens the full state").toContainText("Verbunden");
    await anna.keyboard.press("Escape");
    await expect(anna.locator("#connection-panel")).toBeHidden();
    await expect(connection, "focus returns to it").toBeFocused();
    const headerRows = await anna.evaluate(() => {
      const box = (s: string) => document.querySelector(s)!.getBoundingClientRect();
      const [connection, person, search] = ["#connection-button", "#profile-button", "#palette-button"].map(box);
      return [connection.top < person.bottom && connection.bottom > person.top, search.top >= person.bottom];
    });
    expect(headerRows, "heading, connection and person in one row, the search below").toEqual([true, true]);
    await anna.locator("#profile-button").click();
    await anna.locator("#hand").selectOption({ label: "linker Hand" });
    await expect(anna.locator("html"), "left-handed use is kept on the device").toHaveAttribute("data-hand", "left");
    await anna.keyboard.press("Escape");
    await expect(anna.locator("#profile-panel")).toBeHidden();
    expect(await menuAtEnd(), "left-handed: the menu at the left end").toEqual([false, true]);
    await shot(anna, "med-phone-left", false);
    await anna.locator("#profile-button").click();
    await anna.locator("#hand").selectOption({ label: "rechter Hand" });
    await anna.keyboard.press("Escape");
    expect(await menuAtEnd()).toEqual([true, false]);
    await anna.evaluate(() => window.scrollTo(0, 0));
    // Feedback of 2026-11-22: a label on two lines ("Behandlungsplätze") keeps the bar even.
    const iconTops = await anna.evaluate(() =>
      [...document.querySelectorAll(".navigation :is(.views__item, .launcher__button) > .icon")].map((i) => Math.round(i.getBoundingClientRect().top)));
    expect(new Set(iconTops).size, `the bar's icons in one row: ${iconTops}`).toBe(1);
    await shot(anna, "med-phone", false);
    await anna.locator("#launcher-bar-button").click();
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
    await anna.locator("#launcher-bar-button").click();
    // Feedback of 2026-11-22: on phones a category's boxes sit below it, with a gap.
    await openView(anna, "Behandlungsplätze");
    const strip = anna.locator(".places__strip").first();
    const [badgeBox, boxesBox] = [await strip.locator(".triage").first().boundingBox(), await strip.locator(".places__boxes").boundingBox()];
    expect(boxesBox!.y - (badgeBox!.y + badgeBox!.height), "a gap between the category and its boxes").toBeGreaterThanOrEqual(6);
    await shot(anna, "places-phone", false);
    await openView(anna, "Patienten");
    // [itest->dsn~record-overview~3]
    // The record on phones (concept 2026-11-19, feedback of 2026-11-22): short tab labels;
    // the bottom bar stays as on every page, and the quick actions open from a floating
    // button within thumb reach.
    await openRecord(anna, "DE-0044", "Übersicht");
    await expect(anna.getByRole("tab", { name: "Doku", exact: true }), "a short label for Dokumentation").toBeVisible();
    await expect(anna.locator(".navigation .views"), "the views stay in the bar").toBeVisible();
    await expect(anna.locator("#launcher-bar-button"), "and the menu").toBeVisible();
    await expect(anna.locator(".connection"), "the connection state stays (UX-12)").toBeVisible();
    const tabs = (await anna.getByRole("tab", { name: "Übersicht", exact: true }).boundingBox())!;
    expect(tabs.y + tabs.height, "the tabs on the first screen").toBeLessThan(420);
    const capture = anna.locator("#quick-sheet-button");
    const sheet = anna.locator("#quick-sheet-panel");
    await expect(capture, "one floating button").toHaveText("Erfassen");
    const [captureBox, barBox] = [await capture.boundingBox(), await anna.locator(".navigation").boundingBox()];
    expect(captureBox!.y + captureBox!.height, "above the bar").toBeLessThanOrEqual(barBox!.y);
    expect(captureBox!.x + captureBox!.width, "on the thumb's side").toBeGreaterThan(300);
    await expect(sheet).toBeHidden();
    await capture.click();
    await expect(sheet.getByRole("button", { name: "Übergeben" }), "the handover within thumb reach").toBeVisible();
    await expect(sheet.locator("button"), "the forms marked quick, and the handover").toHaveCount(4);
    await expect(anna.locator(".quick-actions--page").getByRole("button", { name: "Übergeben" }), "not twice").toBeHidden();
    await anna.keyboard.press("Escape");
    await expect(sheet, "Escape closes it").toBeHidden();
    await expect(capture, "and the focus returns").toBeFocused();
    const backLink = (await anna.getByRole("link", { name: /Alle Patienten/ }).boundingBox())!;
    expect(backLink.x, "a 16px gutter").toBeGreaterThanOrEqual(16);
    await audit(anna, "Record on a phone");
    await shot(anna, "record-phone", false);
    await shot(anna, "record-phone-full");
    await capture.click();
    await shot(anna, "record-phone-sheet", false);
    await sheet.getByRole("button", { name: "Übergeben" }).click();
    await expect(sheet, "choosing an action closes the sheet").toBeHidden();
    await expect(anna.getByRole("tab", { name: "Übergabe", exact: true }), "and opens its part").toHaveAttribute("aria-selected", "true");
    await expect(anna.locator(".master"), "on phones the record is a page of its own").toBeHidden();
    await anna.goBack();
    await anna.setViewportSize({ width: 1200, height: 1000 });
    await expect(anna.locator("#profile-button img.avatar"), "the own picture is back after offline and reload").toBeVisible();
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
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Personal");
    // [itest->dsn~person-suggestions~1]
    await expect(ben.locator("#helper-account"), "taking a person in suggests the people the server knows").toHaveAttribute("list", "helper-account-people");
    await expect(ben.locator("#helper-account-people option[value='dora@localhost']"), "with a device, or a role anywhere").toHaveCount(1);
    await expect(ben.locator(".roster__entry", { hasText: "anna@localhost" }).locator("img.avatar"), "others see Anna's picture").toBeVisible();
    await expect(ben.locator(".roster__entry", { hasText: "dora@localhost" }).locator(".avatar--initials"), "without a picture: initials").toHaveText("D");
    // [itest->dsn~incident-devices~1]
    // [itest->dsn~page-tabs~3]
    await expect(ben.getByRole("tab"), "Helfende and Geräte; the command structure is on Führung & Raum").toHaveText(["Helfende", "Geräte"]);
    await pageTab(ben, "Geräte");
    // The incident command sees the devices of the incident's people, and blocks one of
    // Anna's two tablets, which she no longer uses (ORG-12).
    const inUse = ben.locator("section", { has: ben.locator("#incident-devices-title") });
    await expect(inUse.locator("h2")).toHaveText("Geräte im Einsatz");
    const linkOf = (card: Locator) => card.locator("div", { has: ben.locator("dt", { hasText: "Verbindung" }) }).locator("dd");
    await expect(linkOf(inUse.locator(".device-card", { hasText: "dieses Gerät" })), "his own tablet is connected").toHaveText("verbunden");
    const annas = inUse.locator(".device-card", { hasText: "Anna Schmidt" });
    await expect(annas, "with the person").toHaveCount(2);
    const old = annas.first();
    const oldName = (await old.locator("h3").textContent()) ?? "";
    await expect(linkOf(old), "Anna's tablets are switched off").toHaveText("nicht verbunden");
    await audit(ben, "Incident devices");
    await old.getByRole("button", { name: /^Sperren/ }).click();
    await expect(ben.getByRole("button", { name: "Ja, sperren" }), "focus moves to the confirmation").toBeFocused();
    await ben.getByRole("button", { name: "Ja, sperren" }).click();
    await expect(inUse.locator(".device-card h3", { hasText: oldName }), "the blocked tablet is gone from the list").toHaveCount(0);
    await expect(annas, "her other one stays").toHaveCount(1);
    await expect.poll(() => announced(ben)).toContain("gesperrt");
    await openWorkspace(ben, "Führung");
    // [itest->dsn~workspace-navigation~3]
    await expect.poll(() => title(ben), { message: "the commander starts in the first workspace they may use" }).toContain("Einsatzleitung – Führung");
    await ben.locator("#launcher-button").click();
    const uhs = ben.locator(".launcher__item", { hasText: "UHS" });
    await expect(uhs, "the UHS workspace stays in the menu, disabled").toHaveAttribute("aria-disabled", "true");
    await expect(ben.locator("#reason-med-uhs"), "with the reason").toHaveText("Kein Zugriff in diesem Einsatz");
    await audit(ben, "Launcher (commander)");
    await shot(ben, "launcher");
    await ben.keyboard.press("Escape");
    await expect(ben.locator("#launcher-panel"), "Escape closes the menu").toBeHidden();
    await expect(ben.locator("#launcher-button"), "focus returns to the menu button").toBeFocused();
    // [itest->dsn~header-layout~2]
    const header = await ben.evaluate(() => {
      const box = (s: string) => document.querySelector(s)!.getBoundingClientRect();
      const [menu, title, search, status, person, bar] = ["#launcher-button", ".app-header__title", "#palette-button", "#connection-button", "#profile-button", ".app-header"].map(box);
      return {
        order: menu.right <= title.left && title.right <= search.left && search.right <= status.left && status.right <= person.left,
        centred: Math.abs((search.left + search.right) / 2 - (bar.left + bar.right) / 2) < 2,
      };
    });
    expect(header, "header: menu, heading, search in the middle, connection, person").toEqual({ order: true, centred: true });
    await expect(ben.getByRole("button", { name: "Menü – Arbeitsbereich wechseln" }), "exactly one menu button on wide screens").toHaveCount(1);
    await ben.locator("#launcher-button").click();
    await expect(ben.locator("#launcher-panel"), "the main menu lists workspaces only").not.toContainText("Gerät entkoppeln");
    await expect(ben.locator("#launcher-panel #theme")).toHaveCount(0);
    await ben.locator("#main").click({ position: { x: 5, y: 5 } });
    await expect(ben.locator("#launcher-panel"), "a click outside closes the menu").toBeHidden();
    await ben.locator("#connection-button").click();
    await expect(ben.locator("#connection-panel")).toBeVisible();
    await ben.locator(".app-header__title").click();
    await expect(ben.locator("#connection-panel"), "so it does the connection state").toBeHidden();
    await ben.locator("#profile-button").click();
    await ben.locator("#profile-panel").getByText("Angemeldet als").click();
    await expect(ben.locator("#profile-panel"), "a click inside keeps it open").toBeVisible();
    await ben.locator("#profile-button").click();
    await ben.locator("#profile-button").click();
    await expect(ben.locator("#profile-panel"), "the person's menu holds their settings").toContainText("Angemeldet als Ben Weber (ben@localhost)");
    // ORG-7: a person may choose their own name; clearing it returns to the provider's.
    await expect(ben.locator("#display-name")).toHaveValue("Ben Weber");
    await ben.locator("#display-name").fill("Ben W.");
    await ben.getByRole("button", { name: "Namen speichern" }).click();
    await expect(ben.locator(".profile-menu__name"), "shown at once").toHaveText("Ben W.");
    await ben.getByRole("button", { name: "Namen aus der Anmeldung verwenden" }).click();
    await expect(ben.locator(".profile-menu__name"), "the provider's name again").toHaveText("Ben Weber");
    await expect(ben.locator("#profile-panel #theme")).toBeVisible();
    await expect(ben.locator("#my-roles"), "the roles show only in the profile menu").toHaveText("Rollen: Einsatzleitung");
    // [itest->dsn~compact-navigation~2]
    await expect(ben.locator(".views__item .icon svg"), "every view has an icon in the rail").toHaveCount(4);
    const connection = ben.locator(".connection");
    await expect(connection, "the same connection button as on phones").toHaveClass(/connection--synced/);
    await expect(ben.locator("#connection-button .icon"), "coloured like its state, in addition to its shape").toHaveCSS(
      "color",
      await ben.evaluate(() => {
        const probe = document.createElement("span");
        probe.style.color = "var(--ordo-color-feedback-success-text)";
        document.body.append(probe);
        const color = getComputedStyle(probe).color;
        probe.remove();
        return color;
      }),
    );
    await audit(ben, "Profile menu");
    await ben.keyboard.press("Escape");
    await expect(ben.locator("#profile-panel")).toBeHidden();
    await expect(ben.locator("#profile-button"), "focus returns to the person's button").toBeFocused();

    // The overview counts the patients; the situation map shows them per UHS.
    // [itest->dsn~command-treatment-places~1]
    // [itest->dsn~server-derived-connectors~2]
    // UHS-16: the free places per category, from the patient counts alone.
    const placesTile = ben.locator(".tile--uhs");
    await expect(placesTile.locator(".places__all .places__figure"), "the overview adds up the places of all UHS").toHaveText("14 frei von 15");
    await expect(placesTile.locator(".places__all .places__waiting")).toHaveText("3 ohne Platz");
    await expect(placesTile.locator(".places__unit", { hasText: "UHS Eingang" }).locator(".places__box--taken"), "the taken place, not who lies there").toHaveCount(1);
    await shot(ben, "command-places");
    await openView(ben, "Lagekarte");
    // DE-0044 was re-triaged to SK III from the palette; DE-0045 moved to Bühne Nord.
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
      "UHS Bühne Nord 1 1 1 0 0 0 Gesamt: 3 · Belegung: 3 / 10 · Entlassen: 0",
      "UHS Eingang 0 0 1 0 0 0 Gesamt: 1 · Belegung: 1 / 5 · Entlassen: 0",
    ]);
    await ben.setViewportSize({ width: 1700, height: 1000 });
    const table = () =>
      ben.locator("#main tbody tr").evaluateAll((rows) =>
        rows.filter((r) => (r as HTMLElement).offsetParent).map((r) => [...(r as HTMLTableRowElement).cells].map((c) => c.innerText.split("\n")[0]).join(" ")),
      );
    await expect.poll(table, { message: "where there is room, the counts are a table" }).toEqual([
      "UHS Bühne Nord 1 1 1 0 0 0 3 3 / 10 0",
      "UHS Eingang 0 0 1 0 0 0 1 1 / 5 0",
    ]);
    await audit(ben, "Command workspace (wide)", [1700]);
    const received = Buffer.concat(benFrames.received);
    expect(received.length).toBeGreaterThan(0);
    expect(
      received.includes("DE-0042") ||
        received.includes("Mustermann") ||
        received.includes("Hypoglyk") ||
        received.includes(patientPhoto.subarray(patientPhoto.length - 400, patientPhoto.length - 336)),
      "the command device receives no triage tag number, no name, no diagnosis and no photo",
    ).toBe(false);
    await audit(ben, "Command workspace");
    await shot(ben, "command");
  });

  await test.step("resources: add a unit and set its status, also from the keyboard", async () => {
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Einsatzmittel");
    // [itest->dsn~unit-register~3]
    await ben.locator("#unit-call-sign").fill("RTW 83/1");
    await ben.locator("#unit-type").fill("RTW");
    await ben.locator("#unit-organisation").fill("DRK Musterstadt");
    await ben.locator("#unit-service").selectOption("rescue");
    await ben.locator("#unit-leaders").fill("0");
    await ben.locator("#unit-sub-leaders").fill("1");
    await ben.locator("#unit-crew").fill("1");
    await ben.getByRole("button", { name: "Einsatzmittel hinzufügen" }).click();
    const card = ben.locator(".unit-card[data-unit='RTW 83/1']");
    await expect(card, "a unit is added").toContainText("Status: –");
    // [itest->dsn~workspaces-by-phase~5]
    await expect(card.getByLabel("Status von RTW 83/1 setzen"), "planned here; its status is set in Führung").toHaveCount(0);
    await expect(card).toContainText("RTW · DRK Musterstadt");
    await expect(card, "with service and strength").toContainText("Rettungsdienst · Stärke 0/1/1/2");
    // [itest->dsn~uhs-as-unit~1]
    // UHS-12: both UHS count as units, UHS Eingang with its staff (Anna), Bühne Nord unreported.
    await expect(ben.locator(".strength-on-scene__total"), "strength on scene").toHaveText("3 Einsatzmittel · Stärke 0/1/2/3 · ohne Stärkemeldung: 1");
    await ben.getByRole("button", { name: "Besatzung von RTW 83/1 (0)" }).click();
    await ben.locator("#crew-rtw-83-1").fill("anna@localhost");
    await ben.locator("#crew-dialog").getByLabel("Notfallsanitäter/in").check();
    await ben.getByRole("button", { name: "Zur Besatzung hinzufügen" }).click();
    // Anna is in the register since she got her role (ADR-0076).
    // [itest->dsn~double-booking~1]
    await expect(ben.locator("#crew-dialog .assigned-elsewhere"), "UNIT-8: someone on a UHS's staff is not booked twice").toContainText(
      "anna@localhost ist schon bei UHS Eingang eingeteilt.",
    );
    await expect(ben.locator("#crew-dialog .crew-list"), "not added yet").toHaveCount(0);
    await audit(ben, "Assigned elsewhere");
    await ben.locator("#crew-dialog .assigned-elsewhere").getByRole("button", { name: "Abbrechen" }).click();
    await ben.locator("#crew-rtw-83-1").fill("ben@localhost");
    await ben.getByRole("button", { name: "Zur Besatzung hinzufügen" }).click();
    await expect(ben.locator("#crew-dialog .crew-list"), "a crew member with qualifications; the panel stays open").toContainText("Ben Weber · Notfallsanitäter/in");
    await ben.keyboard.press("Escape");

    // [itest->dsn~tasks~7]
    const strip = ben.locator(".tasks-strip");
    const bell = ben.locator("#attention-button");
    await expect(bell, "the bell in the header says when something needs attention").toHaveAccessibleName("Hinweise: keine");
    await openWorkspace(ben, "Führung");
    await expect(strip, "no extra line about tasks on the pages of Führung").toHaveCount(0);
    await openView(ben, "Disposition");
    await expect(ben.locator("#tasks-title"), "the page of all tasks").toHaveText("Offene Aufträge (0)");
    // ADR-0070: the units' status is set and their orders given in Führung – Disposition.
    const board = ben.locator(".unit-status__item[data-unit='RTW 83/1']");
    await board.getByLabel("Status von RTW 83/1 setzen").selectOption({ label: "3 – Einsatz übernommen" });
    await expect(board, "the status is set on Disposition").toContainText("Status: 3 – Einsatz übernommen");
    await expect(board.locator(".unit-status__tasks")).toHaveText("keine offenen Aufträge");
    await board.getByRole("button", { name: "Auftrag geben: RTW 83/1" }).click();
    await expect(ben.locator("#task-unit"), "the form opens for the unit").toHaveValue("unit:RTW 83/1");
    await ben.locator("#task-template").selectOption({ label: "Lagemeldung" });
    await expect(ben.locator("#task-text"), "a template fills in the text").toHaveValue("Lagemeldung abgeben");
    await ben.locator("#task-follow-up").fill("0");
    await expect(ben.locator("#task-follow-up"), "the follow-up is typed in before the task is given").toHaveValue("0");
    await expect(ben.locator("#task-text")).toHaveValue("Lagemeldung abgeben");
    await ben.getByRole("button", { name: "Auftrag anlegen" }).click();
    const task = ben.locator(".task-card");
    await expect(task).toContainText("RTW 83/1 – Lagemeldung abgeben");
    await expect(ben.locator("#give-task-panel"), "the form closes once the Auftrag is created").toBeHidden();
    await expect(ben.locator("#give-task-button"), "the focus goes back to its button").toBeFocused();
    await expect(task.locator(".task-card__follow-up"), "a due follow-up says so").toContainText("Fällig: nachfragen seit");
    await expect(board.locator(".unit-status__tasks"), "the unit shows its task").toHaveText("1 offener Auftrag: Lagemeldung abgeben");
    await audit(ben, "Units on Disposition");
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Einsatzmittel");
    await expect(card, "the status set in Führung shows where units are planned").toContainText("Status: 3 – Einsatz übernommen");
    // A due follow-up rings the bell on every page; it leads to Disposition.
    await expect(bell).toHaveAccessibleName("Hinweise: 1");
    await expect(bell.locator(".attention__count"), "the number as text").toHaveText("1");
    await bell.click();
    const attention = ben.locator("#attention-panel");
    await expect(attention.getByRole("link"), "what needs attention, by name").toHaveText("Rückfrage fällig: RTW 83/1 – Lagemeldung abgeben");
    await audit(ben, "Attention", [1200, 320]);
    await attention.getByRole("link").click();
    await expect(ben.locator("#tasks-title")).toHaveText("Offene Aufträge (1)");
    await task.getByRole("button", { name: "In Arbeit: Lagemeldung abgeben" }).click();
    await expect(task.locator(".task-card__status")).toContainText("In Arbeit");
    await task.getByRole("button", { name: /^In 10 Minuten nachfragen/ }).click();
    await expect(task.locator(".task-card__follow-up"), "postponed").toContainText("Nachfragen um");
    await expect(task.locator(".task-card__countdown"), "with the time left, counted down").toHaveText(/^noch (9:[0-5]\d|10:00) min$/);
    await audit(ben, "Tasks");
    await task.getByRole("button", { name: "Erledigt: Lagemeldung abgeben" }).click();
    await expect(ben.locator("#tasks-title"), "done tasks leave the list").toHaveText("Offene Aufträge (0)");
    await expect.poll(() => status(ben), { message: "unit and status are sent" }).toContain("Alles gesendet");
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Einsatzmittel");
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
    // Released units leave the strength and stay in the history; they can come back.
    await ben.getByRole("button", { name: "RTW 83/1 aus dem Einsatz entlassen" }).click();
    await expect(ben.locator(".strength-on-scene__total")).toHaveText("2 Einsatzmittel · Stärke 0/0/1/1 · ohne Stärkemeldung: 1");
    await ben.locator(".released-list summary").click();
    await expect(ben.locator(".released-list summary")).toHaveText("Entlassene Einsatzmittel (1)");
    await ben.getByRole("button", { name: "RTW 83/1 wieder im Einsatz" }).click();
    await expect(card, "back in the incident, with its status").toContainText("4 – Am Einsatzort");
    // [itest->dsn~uhs-resources~2]
    // UHS-11: the UHS are resources too; the command assigns their staff.
    const uhsCard = ben.locator(".uhs-card[data-uhs]", { hasText: "UHS Eingang" });
    await expect(uhsCard.locator(".uhs-card__staff")).toHaveText("Personal: 1 Person");
    await uhsCard.getByRole("button", { name: "Ort und Personal von UHS Eingang" }).click();
    await expect(ben.locator("#uhs-details-dialog .crew-list"), "people by name, not by user ID").toContainText("Anna Schmidt");
    await ben.locator("#uhs-details-dialog").getByLabel("Name", { exact: true }).fill("petra@localhost");
    await ben.locator("#uhs-details-dialog").getByRole("button", { name: "Zum Personal hinzufügen" }).click();
    await expect(uhsCard.locator(".uhs-card__staff")).toHaveText("Personal: 2 Personen");
    await expect.poll(() => status(ben), { message: "the staff list is sent" }).toContain("Alles gesendet");
    await settle(ben);
    await expect(uhsCard.locator(".uhs-card__staff"), "and accepted by the server for the command").toHaveText("Personal: 2 Personen");
    await expect(ben.locator(".strength-on-scene__total"), "its staff count").toHaveText("3 Einsatzmittel · Stärke 0/1/3/4 · ohne Stärkemeldung: 1");
    // [itest->dsn~uhs-as-unit~1]
    // The UHS as a unit: its organisation, service and strength, counted instead of its staff.
    await ben.locator("#uhs-details-dialog").getByLabel("Organisation", { exact: true }).fill("DRK Musterstadt");
    await ben.locator("#uhs-details-dialog").getByLabel("Dienst", { exact: true }).selectOption("medical");
    await ben.locator("#uhs-details-dialog").getByLabel("Führer", { exact: true }).fill("0");
    await ben.locator("#uhs-details-dialog").getByLabel("Unterführer", { exact: true }).fill("1");
    await ben.locator("#uhs-details-dialog").getByLabel("Mannschaft", { exact: true }).fill("3");
    // [itest->dsn~uhs-capacity-by-category~2]
    // UHS-13: places kept per triage category, within the capacity; the rest for any.
    const uhsDialog = ben.locator("#uhs-details-dialog");
    await uhsDialog.getByLabel("SK I", { exact: true }).fill("1");
    await uhsDialog.getByLabel("SK II", { exact: true }).fill("1");
    await uhsDialog.getByLabel("SK III", { exact: true }).fill("9");
    await expect(uhsDialog.locator(".hint", { hasText: "Zurzeit" }), "the split, as typed").toContainText("Zurzeit 11 Plätze nach Kategorie: mehr als die Kapazität von 5.");
    await expect(uhsDialog.getByLabel("SK III", { exact: true }), "marked while typed").toHaveAttribute("aria-invalid", "true");
    await expect(uhsDialog.getByLabel("SK IV", { exact: true }), "an empty field is not").not.toHaveAttribute("aria-invalid", "true");
    // [itest->dsn~edit-dialog~3]
    // Fertig saves what was typed (feedback of 2026-11-22); a failed check keeps it open.
    await uhsDialog.getByRole("button", { name: "Fertig" }).click();
    await expect(uhsDialog.getByRole("alert"), "more places than the capacity are refused").toHaveText(/Die Plätze nach Kategorie \(11\) sind mehr als die Kapazität \(5\)/);
    await expect(uhsDialog, "the dialog stays open").toBeVisible();
    await uhsDialog.getByLabel("SK III", { exact: true }).fill("2");
    await expect(uhsDialog.locator(".hint", { hasText: "Zurzeit" })).toContainText("Zurzeit 4 nach Kategorie, 1 ohne Kategorie.");
    await expect(uhsDialog.getByLabel("SK III", { exact: true })).not.toHaveAttribute("aria-invalid", "true");
    await shot(ben, "edit-dialog");
    await uhsDialog.getByRole("button", { name: "Fertig" }).click();
    await expect(uhsDialog, "saved and closed").toBeHidden();
    await uhsCard.getByRole("button", { name: "Ort und Personal von UHS Eingang" }).click();
    await expect(uhsDialog.getByLabel("SK I", { exact: true }), "the places per category were saved").toHaveValue("1");
    await expect(uhsDialog.getByLabel("SK II", { exact: true })).toHaveValue("1");
    await expect(uhsDialog.getByLabel("SK III", { exact: true })).toHaveValue("2");
    await expect(uhsDialog.getByLabel("Kapazität (Patienten)", { exact: true }), "the capacity stays as typed").toHaveValue("5");
    await expect(uhsDialog.getByLabel("Organisation", { exact: true }), "and the rest of the form").toHaveValue("DRK Musterstadt");
    await expect(uhsDialog.getByRole("alert"), "no message").toHaveCount(0);
    await ben.keyboard.press("Escape");
    await expect(uhsDialog).toBeHidden();
    await expect(ben.locator(".strength-on-scene__total"), "the UHS with its strength").toHaveText("3 Einsatzmittel · Stärke 0/2/4/6 · ohne Stärkemeldung: 1");
    await expect(ben.locator(".strength-on-scene"), "under its organisation").toContainText("DRK Musterstadt: 2 Einsatzmittel");
    // [itest->dsn~detailed-strength~1]
    // UNIT-7: the incident command chooses the detailed notation of DV 102 (2024).
    const total = ben.locator(".strength-on-scene__total");
    const notation = async (detailed: boolean) => {
      await openView(ben, "Einsatz");
      await ben.locator("#incident-strength-detailed").setChecked(detailed);
      await ben.getByRole("button", { name: "Angaben speichern" }).click();
      await expect(ben.locator("[aria-labelledby=incident-details-title] [role=status]")).toHaveText("Angaben zum Einsatz gespeichert.");
      await openView(ben, "Einsatzmittel");
    };
    await notation(true);
    await expect(total, "five numbers").toHaveText("3 Einsatzmittel · Stärke 0/0/2/4/6 · ohne Stärkemeldung: 1");
    await uhsCard.getByRole("button", { name: "Ort und Personal von UHS Eingang" }).click();
    await expect(ben.locator("#uhs-details-dialog").getByLabel("Zugführer", { exact: true }), "the forms name the detailed parts").toHaveValue("0");
    await ben.locator("#uhs-details-dialog").getByLabel("Führer von Verbänden", { exact: true }).fill("1");
    await ben.locator("#uhs-details-dialog").getByLabel("Ärzte", { exact: true }).fill("1");
    await ben.locator("#uhs-details-dialog").getByRole("button", { name: "Speichern" }).click();
    await expect(total, "with the special functions after it").toHaveText("3 Einsatzmittel · Stärke 1/0/2/4/7 (davon 1 × Ärzte) · ohne Stärkemeldung: 1");
    await ben.keyboard.press("Escape");
    await expect(ben.locator("#unit-unit-leaders"), "the unit form asks for leaders of formations too").toBeVisible();
    await audit(ben, "Detailed strength");
    await notation(false);
    await expect(total, "back to four numbers: both leaders as Führer").toHaveText("3 Einsatzmittel · Stärke 1/2/4/7 · ohne Stärkemeldung: 1");
    // [itest->dsn~double-booking~1]
    // UNIT-8: two devices without a connection book the same person twice; it is marked.
    await server.stop();
    await expect.poll(() => status(ben), { message: "both are offline" }).toContain("Offline");
    await expect.poll(() => status(dora)).toContain("Offline");
    await openWorkspace(dora, "Vorbereitung");
    await openView(dora, "Einsatzmittel");
    const doraCard = dora.locator(".unit-card[data-unit='RTW 83/1']");
    await doraCard.getByRole("button", { name: /^Besatzung von RTW 83\/1/ }).click();
    await dora.locator("#crew-rtw-83-1").fill("max@localhost");
    await dora.locator("#crew-dialog").getByRole("button", { name: "Zur Besatzung hinzufügen" }).click();
    // [itest->dsn~crews-from-register~2]
    await expect(dora.locator("#crew-dialog [role=alert]", { hasText: "Helferregister" }), "HELP-3: crews come from the helper register").toContainText(
      "max@localhost ist nicht im Helferregister.",
    );
    await dora.locator("#crew-dialog").getByRole("button", { name: "Erfassen und hinzufügen: max@localhost" }).click();
    await expect(dora.locator("#crew-dialog .crew-list")).toContainText("max@localhost");
    await dora.keyboard.press("Escape");
    await uhsCard.getByRole("button", { name: "Ort und Personal von UHS Eingang" }).click();
    // The same account on the staff list: the helper registered on the other device.
    await ben.locator("#uhs-details-dialog").getByLabel("Name", { exact: true }).fill("max@localhost");
    await ben.locator("#uhs-details-dialog").getByRole("button", { name: "Zum Personal hinzufügen" }).click();
    await expect(uhsCard.locator(".uhs-card__staff"), "this device does not know of the other booking").toHaveText("Personal: 3 Personen");
    await ben.keyboard.press("Escape");
    await server.start();
    await expect.poll(() => status(dora), { message: "both send their changes", timeout: 20_000 }).toContain("Alles gesendet");
    await expect.poll(() => status(ben), { timeout: 20_000 }).toContain("Alles gesendet");
    await expect(total, "the total is marked").toHaveText("3 Einsatzmittel · Stärke 1/2/4/7* · ohne Stärkemeldung: 1");
    await expect(ben.locator(".strength-on-scene__several"), "and the person named with both posts").toHaveText(
      "* Mehrfach eingeteilt, eventuell doppelt gezählt: max@localhost (RTW 83/1, UHS Eingang)",
    );
    await openWorkspace(ben, "Führung");
    await expect(ben.locator(".tile__several"), "the overview marks it too").toContainText("max@localhost (RTW 83/1, UHS Eingang)");
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Einsatzmittel");
    // Moving them back to the unit takes them off the UHS's staff list.
    await card.getByRole("button", { name: /^Besatzung von RTW 83\/1/ }).click();
    await ben.locator("#crew-rtw-83-1").fill("max@localhost");
    await ben.getByRole("button", { name: "Zur Besatzung hinzufügen" }).click();
    await expect(ben.locator("#crew-dialog .assigned-elsewhere"), "found in the register, by account").toContainText("max@localhost ist schon bei UHS Eingang eingeteilt.");
    await ben.locator("#crew-dialog").getByRole("button", { name: "Hierher umsetzen: max@localhost zu RTW 83/1" }).click();
    await expect(ben.locator(".strength-on-scene__several"), "booked once again").toHaveCount(0);
    await expect(total).toHaveText("3 Einsatzmittel · Stärke 1/2/4/7 · ohne Stärkemeldung: 1");
    await expect(uhsCard.locator(".uhs-card__staff")).toHaveText("Personal: 2 Personen");
    await expect(ben.locator("#crew-dialog .crew-list li"), "on the crew once").toHaveCount(2);
    await ben.keyboard.press("Escape");
    await audit(ben, "Resources workspace");
    await shot(ben, "resources");
    await ben.evaluate(() => (document.documentElement.dataset.theme = "high-contrast"));
    await settle(ben);
    await shot(ben, "command-high-contrast");
    await ben.evaluate(() => delete document.documentElement.dataset.theme);
  });

  // [itest->dsn~command-structure~1]
  await test.step("helper register: personal data only the incident command reads", async () => {
    // [itest->dsn~helper-register~3]
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Personal");
    const helpers = ben.locator(".helpers");
    await expect(helpers.locator(".helper-list [data-helper]", { hasText: "max@localhost" }), "HELP-1: those registered from a crew").toHaveCount(1);
    // [itest->dsn~incident-roles~1]
    // HELP-6 (ADR-0076): one list of the incident's people with their roles; those who got a
    // role are registered with it, named from their account.
    const petra = helpers.locator(".helper-list .roster__entry[data-user='petra@localhost']");
    await expect(petra.locator(".roster__person")).toHaveText("Dr. Petra Lang");
    await expect(petra.locator(".roster__role", { hasText: "Arzt/Ärztin" })).toHaveCount(1);
    await expect(petra.locator(".roster__role", { hasText: "Notfallsanitäter/in" })).toHaveCount(1);
    await expect(helpers.locator(".helper-list .roster__entry[data-user='dora@localhost']"), "and those with a role who are not registered").toContainText("Angaben ergänzen");
    const form = ben.locator(".helper-form");
    // Editing shows every role ticked that the person has; unticking one withdraws it.
    await petra.getByRole("button", { name: "Bearbeiten: Dr. Petra Lang" }).click();
    const petraRoles = form.getByRole("group", { name: "Rollen im Einsatz" });
    await expect(petraRoles.getByLabel("Arzt/Ärztin")).toBeChecked();
    await expect(petraRoles.getByLabel("Notfallsanitäter/in")).toBeChecked();
    await petraRoles.getByLabel("Notfallsanitäter/in").uncheck();
    await form.getByRole("button", { name: "Speichern", exact: true }).click();
    await expect(petra.locator(".roster__role", { hasText: "Notfallsanitäter/in" }), "withdrawn by unticking").toHaveCount(0);
    await expect(petra.locator(".roster__role", { hasText: "Arzt/Ärztin" })).toHaveCount(1);
    await petra.getByRole("button", { name: "Bearbeiten: Dr. Petra Lang" }).click();
    await form.getByRole("group", { name: "Rollen im Einsatz" }).getByLabel("Notfallsanitäter/in").check();
    await form.getByRole("button", { name: "Speichern", exact: true }).click();
    await expect(petra.locator(".roster__role", { hasText: "Notfallsanitäter/in" }), "and granted again by ticking").toHaveCount(1);
    await form.getByRole("button", { name: "Aufnehmen", exact: true }).click();
    await expect(form.locator("[role=alert]"), "a name is needed").toHaveText("Bitte einen Namen angeben.");
    await ben.locator("#helper-name").fill("Lena Berger");
    await ben.locator("#helper-organisation").fill("DRK Musterstadt");
    await ben.locator("#helper-birth-date").fill("1990-05-01");
    await form.getByRole("group", { name: "Qualifikationen" }).getByLabel("Notfallsanitäter/in").check();
    await form.getByRole("group", { name: "Rollen im Einsatz" }).getByLabel("Notfallsanitäter/in").check();
    await ben.locator("#helper-account").fill("lena@localhost");
    await ben.locator("#helper-phone").fill("0170 76543x1");
    await form.getByRole("button", { name: "Aufnehmen", exact: true }).click();
    await expect(form.locator("[role=alert]"), "a phone number is checked").toHaveText("Die Telefonnummer ist keine gültige Nummer mit Vorwahl, z. B. 0171 1234567.");
    await expect(ben.locator("#helper-phone")).toBeFocused();
    await ben.locator("#helper-phone").fill("0170 7654321");
    await ben.locator("#helper-postcode").fill("5511");
    await form.getByRole("button", { name: "Aufnehmen", exact: true }).click();
    await expect(form.locator("[role=alert]"), "and a postcode, by the profile's form").toHaveText("Die Postleitzahl hat nicht die Form des Landes, z. B. 55116.");
    await ben.locator("#helper-postcode").fill("55116");
    await ben.locator("#helper-street").fill("Rheinstraße 4");
    await ben.locator("#helper-town").fill("Mainz");
    await ben.locator("#helper-diet").selectOption({ label: "andere …" });
    await ben.locator("#helper-diet-other").fill("halal");
    await form.getByRole("button", { name: "Aufnehmen", exact: true }).click();
    const lena = helpers.locator(".helper-list [data-helper]", { hasText: "Lena Berger" });
    await expect(lena.locator(".person-card__duty"), "a line for duty and organisation").toHaveText("DRK Musterstadt");
    await expect(lena.locator(".person-card__quals abbr"), "qualifications as short chips").toHaveText(["NotSan"]);
    await expect(lena.locator(".person-card__quals abbr"), "the full term as tooltip").toHaveAttribute("title", "Notfallsanitäter/in");
    await expect(lena, "HELP-2: linked to an account").toContainText("lena@localhost");
    await expect(lena.locator(".roster__role"), "HELP-6: with the role granted in the same form").toContainText("Notfallsanitäter/in");
    await lena.getByRole("button", { name: "Dienst beginnen: Lena Berger" }).click();
    await expect(lena, "on duty from now").toContainText("im Dienst seit");
    // [itest->dsn~duty-pause~1]
    // HELP-8: a pause, and back on duty.
    await lena.getByRole("button", { name: "Dienst pausieren: Lena Berger" }).click();
    await expect(lena.locator(".person-card__duty"), "paused since now").toContainText("in Pause seit");
    await lena.getByRole("button", { name: "Dienst fortsetzen: Lena Berger" }).click();
    await expect(lena.locator(".person-card__duty")).toContainText("im Dienst seit");
    await lena.getByRole("button", { name: "Bearbeiten: Lena Berger" }).click();
    await expect(ben.locator("#helper-phone"), "the form holds the helper, the number in international form").toHaveValue("+491707654321");
    await expect(ben.locator("#helper-diet-other")).toHaveValue("halal");
    await ben.locator("#helper-phone").fill("0170 1111111");
    await form.getByRole("button", { name: "Speichern", exact: true }).click();
    await expect(ben.locator("#helper-form-title"), "the form is new again").toHaveText("Person aufnehmen");
    await lena.getByRole("button", { name: "Bearbeiten: Lena Berger" }).click();
    await expect(ben.locator("#helper-phone"), "changed").toHaveValue("+491701111111");
    await form.getByRole("button", { name: "Abbrechen" }).click();
    // [itest->dsn~registration-card~2]
    // HELP-5: a photo of a Meldekarte fills the form; the command checks and saves. The card
    // is photographed on a screen, with its pixel grid and moiré (ADR-0073).
    const photo = ben.locator("#helper-card-photo");
    const card = await screenPhoto(ben, fs.readFileSync(path.join(__dirname, "fixtures", "registration-card.png")));
    await photo.setInputFiles({ name: "karte.jpg", mimeType: "image/jpeg", buffer: card });
    await expect(form.locator(".helper-card [role=status]")).toHaveText("Meldekarte von Jonas Sommer gelesen. Bitte prüfen und erfassen.");
    await expect(ben.locator("#helper-name")).toHaveValue("Jonas Sommer");
    await expect(ben.locator("#helper-birth-date")).toHaveValue("1995-03-14");
    await expect(ben.locator("#helper-street")).toHaveValue("Bahnhofstraße 5");
    await expect(ben.locator("#helper-postcode")).toHaveValue("86150");
    await expect(ben.locator("#helper-town")).toHaveValue("Augsburg");
    await expect(ben.locator("#helper-phone")).toHaveValue("+491701234567");
    await expect(ben.locator("#helper-organisation")).toHaveValue("BRK Bereitschaft Mitte, Augsburg-Stadt");
    await expect(ben.locator("#helper-duty-start")).toHaveValue("2027-06-20T08:30");
    await expect(ben.locator("#helper-diet"), "the card's wording in the profile's list").toHaveValue("vegetarisch");
    await audit(ben, "Registration card read");
    await form.getByRole("button", { name: "Aufnehmen", exact: true }).click();
    await expect(helpers.locator(".helper-list [data-helper]", { hasText: "Jonas Sommer" })).toContainText("BRK Bereitschaft Mitte, Augsburg-Stadt");
    // [itest->dsn~helper-register~3]
    await expect(helpers.locator(".helper-diets"), "diets counted for catering").toContainText("1 × vegetarisch");
    // [itest->dsn~guest-access~4]
    // HELP-7 (ADR-0077, ADR-0083): Jonas has no account; a role gives him access to this
    // incident by a one-time QR code, until his duty ends or, without an end, until the
    // incident is closed, and never as Einsatzleitung.
    const jonas = helpers.locator(".helper-list [data-helper]", { hasText: "Jonas Sommer" });
    await jonas.getByRole("button", { name: "Bearbeiten: Jonas Sommer" }).click();
    const jonasRoles = form.getByRole("group", { name: "Rollen im Einsatz" });
    await expect(jonasRoles.getByLabel("Einsatzleitung"), "without an account, no Einsatzleitung").toBeDisabled();
    await jonasRoles.getByLabel("Sanitätsdienst").check();
    await ben.locator("#helper-duty-end").fill("2020-06-20T20:00");
    await form.getByRole("button", { name: "Speichern", exact: true }).click();
    await expect(form.locator("[role=alert]"), "an end of duty that has passed gives no access").toContainText("Das Dienstende ist vorbei.");
    await ben.locator("#helper-duty-end").fill("");
    await form.getByRole("button", { name: "Speichern", exact: true }).click();
    const pass = ben.locator(".guest-pass");
    await expect(pass, "the QR code for his phone, without an end of duty").toContainText("Für Jonas Sommer: mit der Kamera jedes Geräts scannen, das Jonas Sommer nutzt. Gilt bis zum Einsatzende.");
    await expect(pass.locator("img")).toHaveAttribute("src", /^data:image\/svg\+xml;base64,/);
    const link = (await pass.locator("a.guest-pass__link").getAttribute("href"))!;
    expect(link, "a code of 256 bits").toMatch(/\/\?guest=[A-Za-z0-9_-]{43}$/);
    await expect(jonas, "his access ends with the incident").toContainText("Gast-Zugang bis zum Einsatzende");
    await expect(jonas.locator(".roster__role")).toContainText("Sanitätsdienst");
    await audit(ben, "Guest code");
    await pass.getByRole("button", { name: "Fertig" }).click();
    await expect(pass).toHaveCount(0);
    // His phone pairs with it and opens the incident with his role …
    const guestPhone = await newDevice(browser);
    await guestPhone.goto(link);
    await expect.poll(() => title(guestPhone), { message: "the guest's phone opens the incident as a medic", timeout: 30_000 }).toContain("Sanität – UHS");
    // … and so does his tablet, with the same code (ADR-0084).
    const guestTablet = await newDevice(browser);
    await guestTablet.goto(link);
    await expect.poll(() => title(guestTablet), { message: "one code for all his devices", timeout: 30_000 }).toContain("Sanität – UHS");
    // Ending his duty ends his access at once, on every device.
    await jonas.getByRole("button", { name: "Dienst beginnen: Jonas Sommer" }).click();
    await jonas.getByRole("button", { name: "Dienst beenden: Jonas Sommer" }).click();
    for (const device of [guestPhone, guestTablet]) {
      await expect(device.getByRole("heading", { name: /Zugang beendet/ }), "the guest's devices lock").toBeVisible({ timeout: 20_000 });
      await device.context().close();
    }
    await photo.setInputFiles({ name: "photo.jpg", mimeType: "image/jpeg", buffer: patientPhoto });
    await expect(form.locator(".helper-card [role=status]"), "a photo without a card").toHaveText(/^Keine Karte erkannt\./);
    await expect(ben.locator("#helper-name"), "the form stays as it was").toHaveValue("");
    await audit(ben, "Helper register");
    await expect.poll(() => status(ben), { message: "the register is sent" }).toContain("Alles gesendet");
    expect(
      Buffer.concat(annaFrames.received).includes("Lena Berger") || Buffer.concat(annaFrames.received).includes("+491701111111"),
      "a medic's device never receives the helpers' data",
    ).toBe(false);
    expect(Buffer.concat(benFrames.sent).includes("+491701111111"), "and it travels sealed").toBe(false);
  });

  await test.step("command structure: the medical branch, a handover, units and UHS", async () => {
    await openWorkspace(ben, "Vorbereitung");
    // [itest->dsn~workspaces-by-phase~5]
    await expect(ben.locator("nav.views .views__label:not(.views__label--short)"), "Vorbereitung in the order of the work (ADR-0095)").toHaveText(["Einsatz", "Personal", "Einsatzmittel", "Führung & Raum"]);
    await openView(ben, "Führung & Raum");
    await expect(ben.locator("#structure-title")).toHaveText("Führungsstruktur");
    const create = async (kind: string, name: string, parent?: string) => {
      await ben.locator("#position-kind").selectOption({ label: kind });
      await ben.locator("#position-name").fill(name);
      if (parent) await ben.locator("#position-parent").selectOption({ label: parent });
      await ben.getByRole("button", { name: "Funktion anlegen" }).click();
    };
    await create("Einsatzleitung", "Stadtfest");
    await create("Sanitätseinsatzleitung", "SanEL", "Einsatzleitung Stadtfest");
    const sanel = ben.locator(".position", { hasText: "Sanitätseinsatzleitung SanEL" }).last();
    await expect(ben.locator(".position-tree--root > .position > .position-tree .position__title"), "a tree, the medical command under the incident command").toHaveText("Sanitätseinsatzleitung SanEL");
    const lead = async (who: string) => {
      await ben.locator("#lead-position").selectOption({ label: "Sanitätseinsatzleitung SanEL" });
      await ben.locator("#lead-person").fill(who);
      await ben.getByRole("button", { name: "Leitung festlegen oder übergeben" }).click();
    };
    await lead("ben@localhost");
    await lead("dora@localhost");
    await expect(sanel.locator(".position__leader")).toHaveText("Leitung: Dora Klein");
    await sanel.locator(".history-list summary").click();
    await expect(sanel.locator(".history-list li").last(), "the handover is logged with both people").toContainText("Übergabe: Ben Weber → Dora Klein");
    await ben.locator("#assign-uhs").selectOption({ label: "UHS Eingang" });
    await ben.locator("#assign-position").selectOption({ label: "Sanitätseinsatzleitung SanEL" });
    await ben.getByRole("button", { name: "UHS zuordnen" }).click();
    // The UHS is the next level of the tree, below its position.
    await expect(sanel.locator(":scope > .position-tree > .position--member[data-member='uhs']")).toHaveText("UHS EingangUHS");
    // [itest->dsn~org-chart~2]
    // CMD-3: an org chart on wide screens: the positions under one side by side, below it.
    await create("Einsatzabschnitt", "Nord", "Einsatzleitung Stadtfest");
    const card = async (title: string) => (await ben.locator(".position__card", { hasText: title }).boundingBox())!;
    const [top, left, right] = [await card("Einsatzleitung Stadtfest"), await card("Sanitätseinsatzleitung SanEL"), await card("Einsatzabschnitt Nord")];
    expect(Math.abs(left.y - right.y), "subordinates side by side").toBeLessThan(2);
    expect(left.y, "below their leading position").toBeGreaterThan(top.y + top.height);
    expect(left.x + left.width, "left of each other").toBeLessThanOrEqual(right.x);
    await expect(ben.getByRole("region", { name: "Führungsstruktur als Organigramm" }), "a wide chart scrolls by keyboard").toHaveAttribute("tabindex", "0");
    await audit(ben, "Command structure");
    await shot(ben, "command-structure");
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Einsatzmittel");
    await ben.getByLabel("Funktion von RTW 83/1").selectOption({ label: "Sanitätseinsatzleitung SanEL" });
    await openWorkspace(ben, "Führung");
    await openView(ben, "Disposition");
    await ben.locator("#give-task-button").click();
    await ben.locator(".new-task__form").getByLabel("Sonstiger Auftrag").check();
    await ben.locator("#task-unit").selectOption({ label: "Sanitätseinsatzleitung SanEL" });
    await ben.locator("#task-text").fill("Lage der UHS melden");
    await ben.getByRole("button", { name: "Auftrag anlegen" }).click();
    await expect(ben.locator(".task-card"), "a task to a position").toContainText("Sanitätseinsatzleitung SanEL – Lage der UHS melden");
    await ben.locator(".task-card").getByRole("button", { name: /^Erledigt/ }).click();
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Führung & Raum");
    await expect(sanel.locator(":scope > .position-tree > .position--member[data-member='unit']"), "units under a position, chosen in two taps, as the next level").toHaveText("RTW 83/1Einsatzmittel");
  });

  // [itest->dsn~situation-map~5]
  await test.step("situation map: read in Führung, edited in Vorbereitung, kept offline by itself", async () => {
    await openWorkspace(ben, "Führung");
    await openView(ben, "Lagekarte");
    const state = ben.locator(".situation__state");
    await expect(state, "the map loads").not.toHaveText("Karte wird geladen …", { timeout: 30_000 });
    const map = await ben.locator(".situation__map").getAttribute("data-map");
    console.log(`situation map in ${test.info().project.name}: ${map}`);
    // Only a missing WebGL may keep the map away, never ORDO's own glue: it loads everywhere.
    await ben.evaluate(() => import("/map/ordo-map.js"));
    if (map !== "ready") {
      // Without WebGL (some headless browsers) the list stands in for the canvas (A11Y-5).
      test.info().annotations.push({ type: "situation map", description: `not shown: ${await state.innerText()}` });
      await expect(state).toHaveText(/Die Karte kann auf diesem Gerät nicht angezeigt werden/);
      return;
    }
    // MAP-2 (ADR-0065): the region was kept on the device by itself when the map first
    // opened, in the step before; so it now comes from the device.
    await expect(state, "kept on the device without being asked, so the map reads it from there").toHaveText("Kartengrundlage: auf diesem Gerät gespeichert (auch ohne Netz).", { timeout: 30_000 });
    await expect(ben.locator(".offline-region button"), "nothing to press for it").toHaveCount(0);
    await expect(ben.locator(".situation__map canvas")).toHaveCount(1);
    await expect(ben.locator(".maplibregl-ctrl-attrib"), "OpenStreetMap is credited").toContainText("OpenStreetMap");
    await expect(ben.getByRole("link", { name: "Lagekarte bearbeiten" }), "Führung reads the map, it does not edit it").toHaveCount(0);
    // [itest->dsn~map-drawing~2]
    // The editor (ADR-0064) on Vorbereitung – Führung & Raum, with no further click (ADR-0095):
    // a toolbar, taps, and the keyboard at the crosshair.
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Führung & Raum");
    await expect(ben.locator("#map-editor-title")).toHaveText("Lagekarte bearbeiten");
    await expect(ben.locator(".situation__map"), "the editor's map loads").toHaveAttribute("data-map", "ready", { timeout: 30_000 });
    const canvas = ben.locator(".situation__map canvas");
    const tool = (name: string) => ben.locator(".map-tools").getByRole("button", { name });
    const panelStatus = ben.locator(".map-panel__status");
    // [itest->dsn~command-and-space~2]
    // CMD-5: a position is put on the map from the tree beside it, and found there again.
    await ben.getByRole("button", { name: "Auf der Karte setzen: Sanitätseinsatzleitung SanEL" }).click();
    await expect(ben.locator("#map-panel-title"), "focus moves to the editor's panel").toBeFocused();
    await expect(tool("Symbol setzen")).toHaveAttribute("aria-pressed", "true");
    await expect(ben.locator("#place-kind")).toHaveValue("position");
    await expect(ben.locator("#place-reference option:checked")).toHaveText("Sanitätseinsatzleitung SanEL");
    await expect(panelStatus).toHaveText("Sanitätseinsatzleitung SanEL: auf die Stelle tippen oder in der Kartenmitte platzieren.");
    await ben.getByRole("button", { name: "In der Kartenmitte platzieren" }).click();
    await expect(panelStatus).toHaveText("Sanitätseinsatzleitung SanEL platziert.");
    await ben.getByRole("button", { name: "Auf der Karte zeigen: Sanitätseinsatzleitung SanEL" }).click();
    await expect(tool("Auswählen"), "showing chooses it on the map").toHaveAttribute("aria-pressed", "true");
    await expect(ben.locator(".map-panel__chosen")).toContainText("Sanitätseinsatzleitung SanEL");
    await audit(ben, "Führung & Raum");
    // Wide enough for both (ADR-0097): no tabs, the structure beside the map.
    const narrow = ben.viewportSize()!;
    await ben.setViewportSize({ width: 1800, height: 1000 });
    await expect(ben.locator(".command-space > .tabs"), "no tabs where both fit").toBeHidden();
    const [structureBox, editorBox] = [await ben.locator("#structure-title").boundingBox(), await ben.locator("#map-editor-title").boundingBox()];
    expect(Math.abs(structureBox!.y - editorBox!.y), "side by side").toBeLessThan(8);
    expect(editorBox!.x, "the map to the right").toBeGreaterThan(structureBox!.x + 300);
    await ben.setViewportSize(narrow);
    const wide = ben.viewportSize()!;
    await ben.setViewportSize({ width: 390, height: 844 });
    // The bar shows "Struktur", so every label takes one line (product owner, 2026-11-22).
    const here = ben.locator("nav.views .views__item[aria-current=page]");
    await expect(here.locator(".views__label--short"), "the short label in the bar").toHaveText("Struktur");
    await expect(here.locator(".views__label--long")).toBeHidden();
    const labelHeights = await ben.evaluate(() =>
      [...document.querySelectorAll("nav.views .views__label")].filter((l) => (l as HTMLElement).offsetParent).map((l) => Math.round(l.getBoundingClientRect().height)));
    expect(new Set(labelHeights).size, `one line each: ${labelHeights}`).toBe(1);
    await expect(ben.locator(".command-space .tabs__short").filter({ visible: true }), "short tab labels on a phone").toHaveText(["Struktur", "Karte", "Grundlage"]);
    await shot(ben, "command-space-phone", false);
    await shot(ben, "command-space-phone-full");
    await ben.setViewportSize(wide);
    await ben.getByRole("button", { name: "Auswahl aufheben" }).click();
    await tool("Symbol setzen").click();
    await expect(tool("Symbol setzen"), "the chosen tool is pressed").toHaveAttribute("aria-pressed", "true");
    await expect(tool("Auswählen")).toHaveAttribute("aria-pressed", "false");
    await ben.locator("#place-kind").selectOption({ label: "Einsatzort" });
    await ben.locator("#place-label").fill("Stadtfest");
    await ben.getByRole("button", { name: "In der Kartenmitte platzieren" }).click();
    const objects = ben.locator(".map-object");
    await expect(objects.first(), "the map opened on the map region; the incident is on the map and in the list").toContainText(/Stadtfest · Einsatzort · 48\.[34]\d+, 10\.[89]\d+/);
    await expect(panelStatus).toHaveText("Stadtfest platziert.");
    // The device's own position, when asked.
    await ben.context().grantPermissions(["geolocation"]);
    await ben.context().setGeolocation({ latitude: 48.36, longitude: 10.89 });
    await ben.getByRole("button", { name: "Zu meinem Standort" }).click();
    await expect(ben.locator(".map-locate__state")).toHaveText("Die Karte zeigt Ihren Standort.");
    // The keyboard moves the map; the unit goes where the centre is now.
    await canvas.focus();
    for (let i = 0; i < 3; i++) await ben.keyboard.press("ArrowRight");
    await expect.poll(async () => ben.evaluate(() => document.activeElement?.classList.contains("maplibregl-canvas"))).toBe(true);
    await ben.locator("#place-kind").selectOption({ label: "Einsatzmittel" });
    await ben.locator("#place-reference").selectOption({ label: "RTW 83/1" });
    await ben.getByRole("button", { name: "In der Kartenmitte platzieren" }).click();
    const unit = ben.locator(".map-object[data-object='unit:RTW 83/1']");
    await expect(unit).toContainText("RTW 83/1 · Einsatzmittel");
    // [itest->dsn~tactical-symbols~2]
    // MAP-3: objects carry the profile's tactical symbols, on the map and in the list.
    await expect(unit.getByRole("img", { name: "Rettungswagen" }), "a unit of type RTW gets its symbol").toHaveAttribute("data-symbol", "rtw");
    await expect(objects.first().locator(".map-object__symbol"), "the incident location gets its symbol").toHaveAttribute("data-symbol", "incident");
    expect(Number(await ben.locator(".situation__map").getAttribute("data-symbols")), "the map has the symbols as images").toBeGreaterThan(20);
    // [itest->dsn~tactical-symbols~2]
    // MAP-10: every symbol of the profile, by topic, placed as often as needed.
    await ben.locator("#place-kind").selectOption({ label: "Taktisches Zeichen" });
    await expect(ben.locator("#place-symbol optgroup"), "grouped by topic").toHaveCount(6);
    await expect(ben.locator("#place-symbol optgroup[label='Gefahren und Schäden'] option", { hasText: "Gefahr akut" })).toHaveCount(1);
    await ben.locator("#place-symbol").selectOption({ label: "Gefahr" });
    await ben.locator("#place-symbol-label").fill("Glasscherben");
    await ben.getByRole("button", { name: "In der Kartenmitte platzieren" }).click();
    await expect(panelStatus).toHaveText("Glasscherben platziert.");
    await ben.locator("#place-symbol-label").fill("");
    await ben.getByRole("button", { name: "In der Kartenmitte platzieren" }).click();
    const hazards = objects.filter({ has: ben.locator(".map-object__symbol[data-symbol='hazard']") });
    await expect(hazards, "the same symbol twice").toHaveCount(2);
    await expect(hazards.filter({ hasText: "Glasscherben" }), "named by its label and the symbol").toContainText("Glasscherben · Gefahr");
    expect(await unit.locator(".map-object__at").innerText(), "placed where the keyboard moved the map from the device's position").toMatch(/^48\.36000, 10\.(?!89000)\d+$/);
    // MAP-4 by keyboard: the incident area, corner by corner at the crosshair.
    await tool("Fläche").click();
    await ben.locator("#draw-category").selectOption({ label: "Einsatzraum" });
    await ben.locator("#draw-label").fill("Festgelände");
    const corner = async (key: string) => {
      await canvas.focus();
      for (let i = 0; i < 3; i++) await ben.keyboard.press(key);
      await ben.getByRole("button", { name: "Ecke in der Kartenmitte setzen" }).click();
    };
    await corner("ArrowUp");
    await corner("ArrowLeft");
    await ben.getByRole("button", { name: "Fertig" }).click();
    await expect(ben.locator(".map-panel [role=alert]"), "an area needs three corners").toHaveText("Eine Fläche braucht mindestens drei Ecken.");
    await corner("ArrowDown");
    await corner("ArrowDown");
    await ben.getByRole("button", { name: "Letzte Ecke entfernen" }).click();
    await expect(ben.locator(".draw-form__count")).toHaveText("3 Ecken gesetzt.");
    await ben.getByRole("button", { name: "Fertig" }).click();
    await expect(objects.filter({ hasText: "Festgelände" }), "the area is on the map and in the list, by its kind").toContainText("Festgelände · Einsatzraum");
    await expect(ben.locator(".draw-form__count"), "ready for the next").toHaveText("Noch keine Ecke gesetzt.");
    // MAP-4 by pointer: a cordon tapped corner by corner.
    await tool("Linie").click();
    await ben.locator("#draw-category").selectOption({ label: "Absperrung" });
    const box = await canvas.boundingBox();
    if (!box) throw new Error("no map canvas");
    const at = (x: number, y: number) => ({ x: Math.round(box.width * x), y: Math.round(box.height * y) });
    await canvas.click({ position: at(0.2, 0.75) });
    await canvas.click({ position: at(0.45, 0.8) });
    await expect(ben.locator(".draw-form__count"), "each tap sets a corner").toHaveText("2 Ecken gesetzt.");
    await ben.getByRole("button", { name: "Fertig" }).click();
    const cordon = objects.filter({ hasText: "Absperrung · Absperrung" });
    await expect(cordon, "the line is saved under its category's name").toHaveCount(1);
    // Reshaping by dragging a corner (Terra Draw), saved as one change.
    const before = await cordon.locator(".map-object__at").innerText();
    await cordon.getByRole("button", { name: "Absperrung bearbeiten" }).click();
    await expect(tool("Auswählen")).toHaveAttribute("aria-pressed", "true");
    const apply = ben.getByRole("button", { name: "Übernehmen" });
    await expect(apply, "Terra Draw has the line").toBeEnabled();
    // The list's button scrolled the page: measure the map again where it is now.
    await canvas.scrollIntoViewIfNeeded();
    const now = await canvas.boundingBox();
    if (!now) throw new Error("no map canvas");
    const first = at(0.2, 0.75);
    await ben.mouse.move(now.x + first.x, now.y + first.y);
    await ben.mouse.down();
    await ben.mouse.move(now.x + first.x + 30, now.y + first.y - 40, { steps: 8 });
    await ben.mouse.up();
    await apply.click();
    await expect(panelStatus).toHaveText("Absperrung geändert.");
    await expect(cordon.locator(".map-object__at"), "its first corner moved").not.toHaveText(before);
    // Moving a symbol: choose it, then tap the new place.
    const unitAt = await unit.locator(".map-object__at").innerText();
    await unit.getByRole("button", { name: "RTW 83/1 bearbeiten" }).click();
    await expect(panelStatus).toHaveText("RTW 83/1 ausgewählt.");
    await canvas.click({ position: at(0.15, 0.2) });
    await expect(panelStatus).toHaveText("RTW 83/1 verschoben.");
    await expect(unit.locator(".map-object__at"), "the unit is where the tap was").not.toHaveText(unitAt);
    await audit(ben, "Map editor");
    await shot(ben, "map-editor");
    await ben.getByRole("button", { name: "RTW 83/1 von der Karte entfernen" }).click();
    await expect(unit).toHaveCount(0);
    // [itest->dsn~icon-rules~1]
    // Removed by an icon, at once: undo instead of a question; undoing places it again.
    await expect(ben.locator(".undo-bar"), "the removal can be undone").toContainText("RTW 83/1 entfernt.");
    await ben.locator(".undo-bar").getByRole("button", { name: "Rückgängig" }).click();
    await expect(unit, "back on the map").toHaveCount(1);
    await expect(unit.getByRole("button", { name: "RTW 83/1 von der Karte entfernen" }), "a row action is an icon with its name").toHaveClass(/icon-button/);
    // [itest->dsn~uhs-details~4]
    // A UHS's place is its symbol on the map, set from its details panel.
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Einsatzmittel");
    await ben.getByRole("button", { name: "Ort und Personal von UHS Eingang" }).click();
    await expect(ben.locator("#uhs-details-dialog .uhs-on-map")).toHaveText("Auf der Lagekarte: noch nicht gesetzt");
    await ben.locator("#uhs-details-dialog").getByRole("link", { name: "Auf der Karte setzen: UHS Eingang" }).click();
    await expect(ben.locator("#map-editor-title"), "the editor opens, ready to place it").toHaveText("Lagekarte bearbeiten");
    await expect(tool("Symbol setzen")).toHaveAttribute("aria-pressed", "true");
    await expect(ben.locator("#place-reference")).toHaveValue(/.+/);
    await expect(ben.locator(".situation__map")).toHaveAttribute("data-map", "ready", { timeout: 30_000 });
    await ben.getByRole("button", { name: "In der Kartenmitte platzieren" }).click();
    await expect(panelStatus).toHaveText("UHS Eingang platziert.");
    await expect(objects.filter({ hasText: "UHS Eingang · UHS" }), "the UHS is on the map").toHaveCount(1);
    await ben.goBack();
    await ben.getByRole("button", { name: "Ort und Personal von UHS Eingang" }).click();
    await expect(ben.locator("#uhs-details-dialog .uhs-on-map"), "the panel shows where it is").toHaveText(/^Auf der Lagekarte: 48\.\d{5}, 10\.\d{5}$/);
    await ben.goForward();
    await expect(ben.locator("#map-editor-title")).toHaveText("Lagekarte bearbeiten");
    await openWorkspace(ben, "Führung");
    await openView(ben, "Lagekarte");
    await expect(ben.locator("#situation-title")).toHaveText("Lagekarte");
    await expect(ben.locator(".situation__map"), "the situation map loads again").toHaveAttribute("data-map", "ready", { timeout: 30_000 });
    await expect(objects.filter({ hasText: "Festgelände" }), "what was drawn is on the situation map").toHaveCount(1);
    // [itest->dsn~map-object-cards~4]
    // MAP-11: the list opens each object's card, as a tap on the map does.
    const rtw = objects.filter({ hasText: "RTW 83/1" });
    await rtw.getByRole("button", { name: "Details: RTW 83/1" }).click();
    const rtwCard = rtw.locator(".object-card");
    await expect(rtwCard, "the unit's status").toContainText(/Status: \d – /);
    await expect(rtwCard.locator(".object-card__chips abbr").first(), "the crew's qualifications in short, the full term as tooltip").toHaveAttribute("title", /.+/);
    await expect(rtwCard.getByLabel("Status von RTW 83/1 setzen"), "the status set from the card").toBeVisible();
    await audit(ben, "Map object card");
    await rtwCard.getByRole("link", { name: "Bearbeiten: RTW 83/1" }).press("Escape");
    await expect(rtwCard, "Escape closes it").toHaveCount(0);
    const gateRow = objects.filter({ hasText: "UHS Eingang" });
    await gateRow.getByRole("button", { name: "Details: UHS Eingang" }).click();
    await expect(gateRow.locator(".object-card .triage"), "a UHS: its patients per triage category").toHaveCount(6);
    await gateRow.getByRole("button", { name: "Schließen: UHS Eingang" }).click();
    await rtw.getByRole("button", { name: "Details: RTW 83/1" }).click();
    await rtwCard.getByRole("link", { name: "Bearbeiten: RTW 83/1" }).click();
    await expect.poll(() => title(ben), { message: "the pencil leads to the unit in Vorbereitung" }).toContain("Vorbereitung");
    await expect(ben.locator(".unit-card[data-unit='RTW 83/1']").first(), "already in view and focused").toBeFocused();
    await openWorkspace(ben, "Führung");
    await openView(ben, "Lagekarte");
    await audit(ben, "Situation map");
    await shot(ben, "situation-map");
  });

  await test.step("transport: the UHS requests, the command assigns a unit and follows it", async () => {
    // [itest->dsn~transport-request~1]
    await openView(ben, "Transporte");
    const request = ben.locator("[aria-labelledby=transports-title] .transport-card");
    await expect(request, "the command sees the open request").toHaveCount(1);
    // [itest->dsn~planned-transports~1]
    const planned = ben.locator("[aria-labelledby=planned-title] .transport-card");
    await expect(planned, "planned transports are listed apart, with their time").toContainText("Geplant für 01.05.2030, 18:30");
    await expect(planned).toContainText("UHS Bühne Nord · KTW · Normal");
    await expect(request).toContainText("UHS Eingang · RTW · Sofort · Infektiös");
    await expect(request, "without patient data").not.toContainText("DE-0044");

    // [itest->dsn~transport-board~1]
    const free = ben.locator("[aria-labelledby=free-units-title]");
    await expect(free, "RTW 83/1 is on scene, not free").toContainText("Keine freien Transportmittel.");
    await openView(ben, "Disposition");
    await ben.getByLabel("Status von RTW 83/1 setzen").selectOption({ label: "2 – Einsatzbereit auf Wache" });
    await openView(ben, "Transporte");
    await expect(free.locator("li"), "a transport unit in an available status is free").toHaveText("RTW 83/1 · RTW · 2 – Einsatzbereit auf Wache");

    // [itest->dsn~hospital-list~2]
    // The list is kept in Vorbereitung, with the units (feedback of 2026-11-22); the board
    // in Führung says so while it is empty, and offers no adding or editing.
    const board = ben.locator("[aria-labelledby=hospitals-title]");
    await expect(board).toContainText("Noch keine Kliniken erfasst. Kliniken werden unter Vorbereitung – Einsatzmittel, Reiter „Kliniken“, erfasst.");
    await expect(board.getByRole("button", { name: "Klinik hinzufügen" }), "no adding in Führung").toHaveCount(0);
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Einsatzmittel");
    // [itest->dsn~page-tabs~3]
    await expect(ben.getByRole("tab"), "the units and the hospitals in tabs").toHaveText(["Einsatzmittel", "Kliniken"]);
    await pageTab(ben, "Kliniken");
    const hospitals = ben.locator("[aria-labelledby=hospitals-setup-title]");
    await expect(hospitals).toContainText("Noch keine Kliniken erfasst.");
    await hospitals.getByRole("button", { name: "Klinik hinzufügen" }).click();
    await expect(hospitals.locator("[role=alert]").last(), "a hospital needs a name").toHaveText("Bitte den Namen der Klinik angeben.");
    await ben.locator("#hospital-name-new").fill("Klinikum Mitte");
    await hospitals.getByLabel("Unfallchirurgie").check();
    await hospitals.getByLabel(/^Schockraum/).check();
    await hospitals.getByRole("button", { name: "Klinik hinzufügen" }).click();
    const entry = hospitals.locator(".hospital-card");
    await expect(entry.locator("h3")).toHaveText("Klinikum Mitte");
    await expect(entry).toContainText("Unfallchirurgie · Schockraum");
    // [itest->dsn~edit-dialog~3]
    // The entry is changed in the list's edit dialog, as every item of a card list.
    await entry.getByRole("button", { name: "Bearbeiten: Klinikum Mitte" }).click();
    const hospitalDialog = ben.getByRole("dialog", { name: "Klinikum Mitte bearbeiten" });
    await hospitalDialog.getByLabel("Stroke Unit").check();
    await hospitalDialog.getByRole("button", { name: "Klinik speichern" }).click();
    await expect(ben.getByRole("dialog"), "saving closes the dialog").toHaveCount(0);
    await expect(entry.getByRole("button", { name: "Bearbeiten: Klinikum Mitte" }), "the focus returns").toBeFocused();
    await expect(entry, "the entry is changed").toContainText("Unfallchirurgie · Schockraum · Stroke Unit");
    await openWorkspace(ben, "Führung");
    await openView(ben, "Transporte");
    const hospital = board.locator(".hospital-card");
    await expect(hospital.locator("h3")).toHaveText("Klinikum Mitte");
    await expect(hospital).toContainText("Unfallchirurgie · Schockraum · Stroke Unit");
    await expect(hospital.getByRole("button", { name: /^Bearbeiten/ }), "no editing in Führung").toHaveCount(0);
    await expect(hospital.locator(".hospital-card__capacity")).toHaveText("Aufnahme: nicht gemeldet");
    await hospital.getByLabel("SK I", { exact: true }).fill("2");
    await hospital.getByLabel("SK II", { exact: true }).fill("3");
    await hospital.getByRole("button", { name: "Aufnahme speichern: Klinikum Mitte" }).click();
    await expect(hospital.locator(".hospital-card__capacity"), "capacity per category, with who and when").toHaveText(/^Aufnahme: SK I 2 · SK II 3 · SK III – \(Stand .+, Ben Weber\)$/);
    await expect(hospital.locator(".hospital-card__sent")).toHaveText("Transportiert: 0");

    await request.getByLabel("Einsatzmittel", { exact: true }).selectOption("RTW 83/1");
    await request.getByLabel("Ziel", { exact: true }).selectOption({ label: "Klinikum Mitte" });
    await request.getByRole("button", { name: /^Zuweisen: / }).click();
    const transportStatus = request.locator(".transport-card__status");
    await expect(transportStatus, "a unit and a destination are assigned").toHaveText("Zugewiesen · RTW 83/1 → Klinikum Mitte");
    await request.getByRole("button", { name: /^Abgefahren: / }).click();
    await expect(transportStatus, "the status moves on, also skipping a step").toHaveText("Abgefahren · RTW 83/1 → Klinikum Mitte");
    await expect(free, "a unit on a transport is not free").toContainText("Keine freien Transportmittel.");
    await expect(hospital.locator(".hospital-card__sent"), "the board counts the patient sent").toHaveText("Transportiert: 1");
    await expect(request.getByRole("button", { name: /^Stornieren: / }), "no cancelling on the road").toHaveCount(0);
    await request.locator("summary").click();
    await expect(request.locator(".history-list li"), "every change with time and person").toHaveCount(3);
    await expect.poll(() => status(ben), { message: "the transport is sent" }).toContain("Alles gesendet");
    expect(Buffer.concat(benFrames.received).includes("DE-0044"), "the command device never receives the triage tag number").toBe(false);
    await audit(ben, "Transports");
    await shot(ben, "transports");
  });

  // [itest->dsn~incident-log~1]
  await test.step("the incident log: every change, corrections linked, only what each may read", async () => {
    await openWorkspace(ben, "Einsatztagebuch");
    await expect.poll(() => title(ben)).toContain("Einsatzleitung – Einsatztagebuch");
    const added = ben.locator(".log-entry", { hasText: "Einsatzmittel RTW 83/1 hinzugefügt" }).last();
    await expect(added, "LOG-1: each change with time, author and device").toContainText(/^\d\d:\d\d:\d\d/);
    await expect(added.locator(".log-entry__by")).toHaveText(/^Ben Weber · Gerät \w{6}$/);
    await expect(ben.locator(".log-entry[data-type='units.statusChanged']").first(), "status changes are logged").toContainText("RTW 83/1: Status");
    await expect(ben.locator(".log-entry[data-type='transport.updated']").first(), "and transports").toBeVisible();
    await expect(ben.locator(".log-entry[data-type^='patients.'], .log-entry[data-type^='epcr.'], .log-entry[data-type^='orders.']"), "LOG-6: no clinical entries in the command's log").toHaveCount(0);
    await audit(ben, "Incident log");
    await shot(ben, "incident-log");

    const medic = await newDevice(browser);
    await loginPair(medic, server, "anna@example.org");
    await openIncident(medic, INCIDENT);
    // The medic opens it as well: everyone in the incident does (LOG-6).
    await openWorkspace(medic, "Einsatztagebuch");
    await expect(medic.locator(".log-entry", { hasText: "Patient DE-0042 registriert" }), "the medic's log has the clinical entries").toHaveCount(1);
    const correction = medic.locator(".log-entry__corrects").first();
    await expect(correction, "LOG-3: a correction is a new entry that links to the old one").toHaveText(/^\(korrigiert den Eintrag von \d\d:\d\d:\d\d\)$/);
    const corrected = medic.locator(`[id='${(await correction.getAttribute("href"))!.slice(1)}']`);
    await expect(corrected, "which stays in the log").toContainText(/Vitalwerte für Patient DE-\d{4}/);
    await medic.context().close();

    await openWorkspace(ben, "Führung");
    await expect.poll(() => title(ben)).toContain("Einsatzleitung – Führung");
  });

  // [itest->dsn~uhs-discharge~1]
  await test.step("the UHS discharges the transported patient; the load drops", async () => {
    const medic = await newDevice(browser);
    await loginPair(medic, server, "anna@example.org");
    await openIncident(medic, INCIDENT);
    await openView(medic, "Patienten");
    await openRecord(medic, "DE-0044");
    const stay = medic.locator("[aria-labelledby=section-stay]");
    await openSection(medic, "section-stay");
    await expect(stay.locator(".history li"), "registration is the admission").toHaveCount(1);
    await expect(stay.getByLabel("Transportiert", { exact: true }), "the transport has left: transported is chosen").toBeChecked();
    await expect(stay.locator(".hint")).toHaveText("Mit dem Transport: RTW · RTW 83/1 → Klinikum Mitte");
    await audit(medic, "Discharge");
    await stay.getByRole("button", { name: "Entlassen", exact: true }).click();
    await expect(medic.locator(".record__discharged"), "the record says so at the top").toHaveText("Entlassen: Transportiert");
    await expect(stay.locator(".history li").nth(1)).toContainText("Entlassen: Transportiert");
    await expect(stay.getByRole("button", { name: "DE-0044 wieder aufnehmen" }), "a mistake is undone by readmitting").toBeVisible();
    await expect.poll(() => status(medic), { message: "the discharge is sent" }).toContain("Alles gesendet");
    await medic.goBack();
    const discharged = medic.locator(".discharged-list");
    await expect(discharged.locator("summary"), "discharged patients are listed apart").toHaveText("Entlassene Patienten (1)");
    await expect(medic.locator(".patient-card[data-tag='DE-0044']"), "collapsed").toBeHidden();
    await discharged.locator("summary").click();
    await expect(discharged.locator(".patient-card[data-tag='DE-0044']"), "with the outcome").toContainText("Transportiert");
    await audit(medic, "Patients with discharged");
    await medic.context().close();

    await openView(ben, "Lagekarte");
    await expect
      .poll(() => ben.locator(".count-card", { hasText: "UHS Eingang" }).locator("p").textContent(), { message: "command sees the load without the discharged patient, against its whole capacity" })
      .toBe("Gesamt: 0 · Belegung: 0 / 5 · Entlassen: 1");
    expect(Buffer.concat(benFrames.received).includes("DE-0044"), "but not who left").toBe(false);
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Einsatzmittel"); // the next step checks that units disappear when locked
  });

  // [itest->dsn~reports~3]
  await test.step("reports: a patient case is assigned, placed on the map and taken over by the crew", async () => {
    // A medical team with Petra, a physician, on its crew: she moves there from the UHS.
    // Physicians work at every UHS (ADR-0056), so she registers the patient she takes over;
    // a crew member on no staff list could not (UHS-10, ADR-0086).
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Einsatzmittel");
    await ben.locator("#unit-call-sign").fill("SanTrp 1");
    await ben.locator("#unit-type").fill("SanTrp");
    await ben.locator("#unit-organisation").fill("DRK Musterstadt");
    await ben.getByRole("button", { name: "Einsatzmittel hinzufügen" }).click();
    const team = ben.locator(".unit-card[data-unit='SanTrp 1']");
    await team.getByRole("button", { name: /^Besatzung von SanTrp 1/ }).click();
    await ben.locator("#crew-santrp-1").fill("petra@localhost");
    await ben.locator("#crew-dialog").getByRole("button", { name: "Zur Besatzung hinzufügen" }).click();
    const enter = ben.locator("#crew-dialog").getByRole("button", { name: "Erfassen und hinzufügen: petra@localhost" });
    const move = ben.locator("#crew-dialog").getByRole("button", { name: "Hierher umsetzen: petra@localhost zu SanTrp 1" });
    await expect(enter.or(move)).toBeVisible();
    if (await enter.isVisible()) await enter.click();
    await move.click();
    await expect(ben.locator("#crew-dialog .crew-list"), "by her name in the register; the crew keeps her account").toContainText("Dr. Petra Lang");
    await ben.keyboard.press("Escape");

    // REP-1, ADR-0081: the command records the report as an Auftrag, with its pin on the
    // same form, and assigns it to the team.
    await openWorkspace(ben, "Führung");
    await openView(ben, "Disposition");
    await expect(ben.locator("#tasks-title")).toHaveText("Offene Aufträge (0)");
    await ben.locator("#give-task-button").click();
    const form = ben.locator(".new-task");
    await form.getByRole("button", { name: "Auftrag anlegen" }).click();
    await expect(form.locator("[role=alert]"), "the kind is chosen, never assumed").toHaveText("Bitte die Art wählen.");
    await form.getByLabel("Patient", { exact: true }).check();
    await ben.locator("#task-place").fill("Bühne Nord, links");
    await ben.locator("#task-text").fill("Person liegt am Boden");
    await expect(ben.locator("#task-text-hint"), "the form asks for no medical details").toContainText("ohne medizinische Angaben");
    await expect(form.locator("#task-unit"), "the unit may come later").toHaveValue("");
    const pinMap = form.locator(".pin-picker .situation__map");
    await expect(pinMap, "the form's map loads").not.toHaveAttribute("data-map", "loading", { timeout: 30_000 });
    // Without WebGL (some headless browsers) there is no map to set a pin on.
    const placed = (await pinMap.getAttribute("data-map")) === "ready";
    if (placed) {
      await form.getByRole("button", { name: "Pin in Kartenmitte setzen" }).click();
      await expect(form.locator(".pin-picker__state"), "set from the keyboard, without leaving the page").toHaveText(/^Pin: 48\.\d{5}, 10\.\d{5}$/);
    }
    await audit(ben, "New Auftrag");
    await form.getByRole("button", { name: "Auftrag anlegen" }).click();
    const report = ben.locator(".task-card", { hasText: "Bühne Nord, links" });
    await expect(report, "with kind and time it came in").toContainText(/Patient · Eingang \d\d:\d\d/);
    await expect(report.locator(".task-card__status"), "received, nobody on it yet").toContainText("Eingegangen");
    // [itest->dsn~tasks~7]
    // An Auftrag no unit works on rings the bell until it is assigned.
    await expect(ben.locator("#attention-button")).toHaveAccessibleName("Hinweise: 1");
    await ben.locator("#attention-button").click();
    await expect(ben.locator("#attention-panel")).toContainText("Auftrag ohne Einsatzmittel: Patient: Bühne Nord, links");
    await ben.keyboard.press("Escape");
    await report.getByLabel("Zuweisen an").selectOption("SanTrp 1");
    await report.getByRole("button", { name: "Zuweisen", exact: true }).click();
    await expect(ben.locator("#attention-button"), "assigned: nothing left").toHaveAccessibleName("Hinweise: keine");
    await expect(report.locator(".task-card__status"), "one card, one state, its one unit").toContainText("Zugewiesen · SanTrp 1");
    await expect(ben.locator(".task-card"), "one card for the report and its unit").toHaveCount(1);
    // ADR-0085: a further unit joins the same Auftrag; a unit on it is not offered again.
    await expect(report.getByLabel("Zusätzlich schicken"), "only after asking for it").toBeHidden();
    await report.getByRole("button", { name: "Weiteres Einsatzmittel schicken" }).click();
    const further = report.getByLabel("Zusätzlich schicken");
    await expect(further.locator("option", { hasText: "SanTrp 1" }), "a unit on it is not offered again").toHaveCount(0);
    const second = (await further.locator("option:not([disabled])").first().textContent())!.trim();
    await further.selectOption(second);
    await report.getByRole("button", { name: "Schicken", exact: true }).click();
    await expect(ben.locator(".task-card"), "still one card").toHaveCount(1);
    await expect(report.locator(".task-card__unit > span"), "each unit with its state").toHaveText([
      "SanTrp 1 · Zugewiesen",
      `${second} · Zugewiesen`,
    ]);
    if (placed) {
      await expect(report, "REP-4: its pin is on the situation map").toContainText("Pin auf der Lagekarte");
      await expect(report.getByRole("button", { name: "Pin verschieben: Patient: Bühne Nord, links" }), "and moves in place").toHaveCount(1);
    }
    await audit(ben, "Aufträge");
    if (placed) {
      await openView(ben, "Lagekarte");
      const pin = ben.locator(".map-object", { hasText: "Patient: Bühne Nord, links · Auftrag" });
      await expect(pin.locator(".map-object__symbol"), "with the sign for an injured person").toHaveAttribute("data-symbol", "injured-person");
      await openView(ben, "Disposition");
    }

    // REP-3, REP-4: Petra, on the team's crew, finds the Auftrag with its place, and takes it over.
    const medic = await newDevice(browser);
    await loginPair(medic, server, "petra@example.org");
    await openIncident(medic, INCIDENT);
    // [itest->dsn~crew-workspace~3]
    // UNIT-9 (ADR-0079): the crew's own workspace: status, Aufträge and transports.
    await openWorkspace(medic, "Besatzung");
    await expect.poll(() => title(medic)).toContain("Sanität – Besatzung");
    const crewUnit = medic.locator(".crew-unit[data-unit='SanTrp 1']");
    await expect(crewUnit.locator("h2"), "only the units whose crew names her").toHaveText("SanTrp 1");
    await expect(medic.locator(".crew-unit"), "her one unit").toHaveCount(1);
    await crewUnit.getByLabel("Status von SanTrp 1 setzen").selectOption({ label: "3 – Einsatz übernommen" });
    await expect(crewUnit.locator(".unit-status__current"), "the crew sets its status").toContainText("3 – Einsatz übernommen");
    await expect(crewUnit.getByLabel("Status von SanTrp 1 setzen").locator("option[value='']"), "the prompt is no status").toHaveCount(0);
    const crewTask = crewUnit.locator(".task-card", { hasText: "Bühne Nord, links" });
    await expect(crewTask.locator("button", { hasText: /Abbrechen|nachfragen/ }), "cancelling and asking again are the command's").toHaveCount(0);
    // [itest->dsn~tasks~7]
    // TASK-3: the crew's bell counts the Auftrag not started yet, and leads to it.
    await expect(medic.locator("#attention-button"), "a bell for every role").toHaveAccessibleName(/^Hinweise: [1-9]/);
    await medic.locator("#attention-button").click();
    await expect(medic.locator(".attention__list")).toContainText("Neuer Auftrag: ");
    await medic.keyboard.press("Escape");
    await crewTask.getByRole("button", { name: /^In Arbeit/ }).click();
    await expect(crewTask.locator(".task-card__status"), "and moves its Auftrag on").toContainText("In Arbeit");
    await openView(medic, "Transporte");
    await expect(medic.locator("#crew-transports-title")).toHaveText("Transporte für Ihr Einsatzmittel");
    await expect(medic.locator(".crew-transports"), "none assigned to SanTrp 1").toContainText("Keine offenen Transporte.");
    await audit(medic, "Crew transports");
    await openView(medic, "Aufträge");
    await expect(crewTask, "one card: what, where, when").toContainText(/Patient · Eingang \d\d:\d\d/);
    if (placed) {
      await expect(crewTask.getByRole("link", { name: "In der Karten-App öffnen" }), "the place for the device's maps app").toHaveAttribute("href", /^(geo:48\.\d{6},10\.\d{6}|https:\/\/maps\.apple\.com\/\?ll=48\.\d{6},10\.\d{6}&q=.+)$/);
      await expect(crewTask.locator(".situation__map"), "and on a small map").toHaveCount(1);
      // REP-4: where the crew is, kept on the device; this browser has no position to give.
      await expect(crewTask.getByRole("button", { name: "Meinen Standort zeigen" })).toBeVisible();
    }
    await audit(medic, "Crew workspace");
    await crewTask.getByRole("button", { name: "Patient aufnehmen" }).click();
    // [itest->dsn~crew-patients~1]
    // ADR-0093: the crew registers in its own workspace, not the UHS's.
    await expect.poll(() => title(medic), { message: "the registration opens in the crew's workspace" }).toContain("Sanität – Besatzung");
    await expect(medic.locator(".uhs-chooser, #working-uhs"), "no UHS to choose").toHaveCount(0);
    await expect(medic.locator(".register-case"), "the registration names the Auftrag").toContainText("Aus dem Auftrag „Bühne Nord, links“ für SanTrp 1");
    await expect(medic.locator("#tag")).toBeFocused();
    // [itest->dsn~patient-holder~1]
    // EPCR-16 (ADR-0087): the crew registers the patient with its own unit.
    await expect(medic.locator("#uhs"), "the crew's unit is chosen").toHaveValue("unit:SanTrp 1");
    await expect(medic.locator("#uhs option[value^='unit:']"), "only its own units are offered").toHaveCount(await medic.locator("#uhs option").count() - 1);
    await register(medic, "DE-0077", 1);
    await expect(medic.locator(".patient-card[data-tag='DE-0077']"), "held by the unit").toContainText("SanTrp 1");
    await expect(medic.locator(".patient-card"), "least knowledge: only what the unit holds").toHaveCount(1);
    // [itest->dsn~crew-places~1]
    // UHS-19: where a patient can go, without a radio call; numbers only.
    const crewPlaces = medic.locator(".crew-places");
    await expect(crewPlaces.locator(".places__unit", { hasText: "UHS Eingang" }), "every UHS with its free places").toContainText(/\d+ frei von \d+/);
    await expect(crewPlaces, "no orders or transports of the UHS").not.toContainText("Anordnungen");
    // [itest->dsn~quick-handover~1]
    // EPCR-18: a quick handover by QR code; Anna, at UHS Eingang, opens its address and
    // takes the patient there.
    await openRecord(medic, "DE-0077", "Übergabe");
    await medic.getByRole("button", { name: "QR-Code zeigen" }).click();
    const quick = medic.locator("#handover-open");
    await expect(quick).toContainText("Schnellübergabe offen");
    await expect(quick.locator(".handover-code__image"), "the code, drawn on the device").toBeVisible();
    const link = await quick.locator(".handover-code__link").getAttribute("href");
    // [itest->dsn~patient-address~1]
    expect(link, "the patient's own address; the code names no clinical data").toMatch(/\/patient\/DE-0077\?handover=[0-9A-Z]{26}$/);
    await expect.poll(() => medic.evaluate(() => location.pathname), { message: "the record at its own address" }).toBe("/patient/DE-0077");
    await expect.poll(() => title(medic), { message: "in the crew's workspace" }).toContain("Sanität – Besatzung");
    await audit(medic, "Quick handover");
    await shot(medic, "quick-handover");
    await expect.poll(() => status(medic), { message: "the handover is sent" }).toContain("Alles gesendet");
    const receiver = await newDevice(browser);
    await loginPair(receiver, server, "anna@example.org");
    await openIncident(receiver, INCIDENT);
    await openView(receiver, "Patienten");
    await receiver.goto(link!);
    const take = receiver.locator("#take-dialog");
    await expect(take.locator("#take-title")).toHaveText("Patient DE-0077 übernehmen");
    await expect(take, "from where").toContainText("Bisher bei SanTrp 1");
    await expect(take.locator("#take-to"), "she writes for one UHS only: it is not asked").toHaveCount(0);
    // [itest->dsn~take-with-place~1]
    // UHS-18: the place is chosen with the taking.
    const place = take.locator("#take-place");
    await expect(place, "a free place is proposed").not.toHaveValue("");
    const placeLabel = (await place.locator("option:checked").textContent())!.trim();
    await take.getByRole("button", { name: "Übernehmen für UHS Eingang: Patient DE-0077" }).click();
    await expect(receiver.locator("#record-title"), "the record opens").toHaveText("Patient DE-0077");
    await expect(receiver.locator(".record__where"), "now at the UHS, on the place").toContainText(`UHS Eingang · ${placeLabel}`);
    // [itest->dsn~patient-flow~1]
    await expect(receiver.locator("[aria-labelledby=tile-way] .flow__step"), "the way: the crew, then the UHS").toHaveText([/SanTrp 1/, /Schnellübergabe per QR-Code[\s\S]*UHS Eingang[\s\S]*Jetzt/]);
    await expect.poll(() => status(receiver), { message: "the taking is sent" }).toContain("Alles gesendet");
    await receiver.context().close();
    await expect(quick, "taken: the code is gone on the crew's device").toHaveCount(0);
    await expect(medic.locator(".detail .notice"), "and the record with it: the patient is no longer the crew's").toContainText("nicht (mehr) bei Ihrem Einsatzmittel");
    await expect(medic.locator(".patient-card[data-tag='DE-0077']"), "gone from the crew's list").toHaveCount(0);
    await openWorkspace(medic, "Besatzung");
    await expect(crewTask.getByRole("button", { name: "Patient aufnehmen" }), "taken over: offered no more").toHaveCount(0);
    await expect(crewTask.locator(".task-card__taken")).toHaveText(/^Patient aufgenommen von SanTrp 1 \(\d\d:\d\d\)$/);

    // The command sees the crew's status and progress without a radio call, and that the
    // Auftrag has a patient, never who.
    await expect(report.locator(".task-card__status")).toContainText("In Arbeit · SanTrp 1");
    await expect(report.locator(".task-card__taken")).toHaveText(/^Patient aufgenommen von SanTrp 1 \(\d\d:\d\d\)$/);
    expect(Buffer.concat(benFrames.received).includes("DE-0077"), "the command device never receives the tag number").toBe(false);
    // REP-5: the crew's "Erledigt" ends it once; its pin leaves the map; the log keeps it.
    await crewTask.getByRole("button", { name: /^Erledigt/ }).click();
    await expect(crewUnit.locator(".task-card"), "done: off the crew's list").toHaveCount(0);
    await expect.poll(() => status(medic), { message: "the patient and the step are sent" }).toContain("Alles gesendet");
    await medic.context().close();
    // The other unit still works on it; when it is done too, by radio, the Auftrag ends.
    await expect(report.locator(".task-card__unit > span")).toHaveText(["SanTrp 1 · Erledigt", `${second} · Zugewiesen`]);
    await expect(report.locator(".task-card__status"), "open while one unit works").toContainText("In Arbeit");
    if (placed) await expect(report, "its pin stays").toContainText("Pin auf der Lagekarte");
    await expect(report.getByRole("button", { name: /^Erledigt/ }), "its steps once, in its row").toHaveCount(1);
    await report.getByRole("button", { name: `Erledigt: ${second} – Patient: Bühne Nord, links` }).click();
    await expect(ben.locator("#tasks-title")).toHaveText("Offene Aufträge (0)");
    if (placed) {
      await openView(ben, "Lagekarte");
      await expect(ben.locator(".map-object", { hasText: "Patient: Bühne Nord, links" }), "the pin of a finished Auftrag leaves the map").toHaveCount(0);
    }
    await openWorkspace(ben, "Einsatztagebuch");
    await expect(ben.locator(".log-entry[data-type='tasks.given']", { hasText: "Bühne Nord" })).toContainText("Auftrag eingegangen: Patient, Eingang");
    await expect(ben.locator(".log-entry[data-type='tasks.assigned']", { hasText: "SanTrp 1" })).toContainText("Auftrag „Patient: Bühne Nord, links“ an SanTrp 1 zugewiesen");
    await expect(ben.locator(".log-entry[data-type='tasks.updated']", { hasText: `Bühne Nord, links“: ${second} Erledigt` }), "each unit's step in the log").toHaveCount(1);
    await expect(ben.locator(".log-entry[data-type='tasks.patientRegistered']")).toContainText("Patient aus dem Auftrag Patient: Bühne Nord, links aufgenommen von SanTrp 1");
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Einsatzmittel"); // the next step checks that units disappear when locked
  });

  // [itest->dsn~uhs-staff-request~2]
  await test.step("a UHS asks for a physician; the command sends one onto its staff list", async () => {
    // UHS-15: only the UHS's staff ask. Petra, now on a unit's crew, is on no staff list.
    const medic = await newDevice(browser);
    await loginPair(medic, server, "petra@example.org");
    await openIncident(medic, INCIDENT);
    await openWorkspace(medic, "UHS");
    await openView(medic, "Patienten");
    const offPanel = medic.locator("[aria-labelledby=staff-requests-title]");
    await medic.locator("#working-uhs").selectOption({ label: "Alle UHS" });
    const off = offPanel.getByRole("button", { name: "Arzt/Ärztin anfordern" });
    await expect(off, "shown, but off without a UHS").toBeDisabled();
    await expect(off, "and greyed out, not by colour alone").toHaveCSS("border-top-style", "dashed");
    await expect(offPanel.locator(".hint"), "and says what to choose").toContainText("Meine UHS");
    await medic.locator("#working-uhs").selectOption({ label: "UHS Bühne Nord" });
    await expect(off, "off for someone not on its staff list, also where she may document").toBeDisabled();
    await expect(offPanel.locator(".hint"), "and says why").toContainText("Personalliste von UHS Bühne Nord");
    // Anna, on the staff list of UHS Eingang, asks for a paramedic …
    const asker = await newDevice(browser);
    await loginPair(asker, server, "anna@example.org");
    await openIncident(asker, INCIDENT);
    await openWorkspace(asker, "UHS");
    await openView(asker, "Patienten");
    await asker.locator("#working-uhs").selectOption({ label: "UHS Eingang" });
    const panel = asker.locator("[aria-labelledby=staff-requests-title]");
    await panel.getByRole("button", { name: "Notfallsanitäter/in anfordern" }).click();
    await expect(panel.locator(".staff-requests__list li").first(), "the request, open").toContainText(/^Notfallsanitäter\/in · \d\d:\d\d · offen$/);
    await audit(asker, "Staff request");
    // … the command sees it in Disposition and in the bell, and sends Petra …
    await openWorkspace(ben, "Führung");
    await openView(ben, "Disposition");
    await expect(ben.locator("#attention-button"), "the bell rings").toHaveAccessibleName(/^Hinweise: [1-9]/);
    const board = ben.locator("[aria-labelledby=staff-board-title]");
    const request = board.locator(".report-card", { hasText: "UHS Eingang fordert Notfallsanitäter/in an" });
    await request.getByLabel("Wer kommt?").fill("petra@localhost");
    await request.getByRole("button", { name: "Zuweisen" }).click();
    await expect(board, "met: no open requests left").toHaveCount(0);
    const sent = ben.locator(".task-card", { hasText: "Als Notfallsanitäter/in zu UHS Eingang" });
    await expect(sent.locator(".task-card__status"), "an Auftrag for the person sent").toContainText("Zugewiesen · Dr. Petra Lang");
    // … the UHS learns who comes, and Petra sees her Auftrag.
    await expect(panel.locator(".staff-requests__list li").first()).toContainText(/(Dr\. Petra Lang|petra@localhost) kommt$/);
    await asker.context().close();
    await openWorkspace(medic, "Besatzung");
    const mine = medic.locator("[aria-labelledby=crew-mine-title] .task-card", { hasText: "Als Notfallsanitäter/in zu UHS Eingang" });
    await expect(mine.locator("strong").first(), "where to go").toHaveText("UHS Eingang");
    await mine.getByRole("button", { name: /^Erledigt/ }).click();
    await expect(mine, "done when she is there").toHaveCount(0);
    await expect.poll(() => status(medic), { message: "sent" }).toContain("Alles gesendet");
    await medic.context().close();
    await expect(sent, "the command sees it done").toHaveCount(0);
    await openWorkspace(ben, "Vorbereitung");
    await openView(ben, "Einsatzmittel"); // the next step checks that units disappear when locked
    const uhs = ben.locator(".uhs-card[data-uhs]", { hasText: "UHS Eingang" });
    await expect(uhs.locator(".uhs-card__staff"), "on the UHS's staff list").toHaveText("Personal: 2 Personen");
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
    // [itest->dsn~local-persistence~2]
    expect(await ben.evaluate(() => location.pathname), "the reload kept the page: the device knows its permissions").toBe("/command.prepare/units");
  });

  // [itest->dsn~organisation-roles~1]
  await test.step("an administrator grants an organisation role until a date and withdraws it", async () => {
    // Planners plan incidents: the incident list offers it only with the role.
    await ben.locator("#profile-button").click();
    await ben.getByRole("button", { name: "Einsatz wechseln" }).click();
    await expect(ben.getByRole("button", { name: `Öffnen: ${INCIDENT}` })).toBeVisible();
    const benMayPlan = async () => (await ben.locator("#plan-name").count()) === 1;
    expect(await benMayPlan(), "Ben does not plan yet").toBe(false);

    await openWorkspace(dora, "Zugriff");
    await openView(dora, "Berechtigungen");
    await expect.poll(() => title(dora)).toContain("Administration – Zugriff");
    const doraAdmin = dora.locator(".role-list [data-user='dora@localhost'] [data-role='admin']");
    await expect(doraAdmin, "the administrator from the configuration").toContainText("aus der Serverkonfiguration");
    await expect(doraAdmin.getByRole("button"), "is not changed in ORDO").toHaveCount(0);

    // A planner for the season: until the end of tomorrow.
    const tomorrow = new Date(Date.now() + 24 * 3600_000);
    const day = (n: number) => String(n).padStart(2, "0");
    await dora.getByLabel("Person (Benutzerkennung)").fill("ben@localhost");
    await dora.getByLabel("Rolle", { exact: true }).selectOption({ label: "Einsatzplanung (alle Einsätze)" });
    await dora.getByLabel("Gültig bis (optional)").fill(`${tomorrow.getFullYear()}-${day(tomorrow.getMonth() + 1)}-${day(tomorrow.getDate())}`);
    await dora.getByRole("button", { name: "Vergeben" }).click();
    const benPlanner = dora.locator(".role-list [data-user='ben@localhost'] [data-role='planner']");
    await expect(benPlanner.locator("span").first(), "granted, with its end, shown by name").toHaveText(`Einsatzplanung (alle Einsätze) · bis ${day(tomorrow.getDate())}.${day(tomorrow.getMonth() + 1)}.${tomorrow.getFullYear()}`);
    await expect(dora.locator(".role-list [data-user='ben@localhost'] .roster__person")).toHaveText("Ben Weber");
    await audit(dora, "Organisation roles");
    await expect.poll(benMayPlan, { message: "Ben's devices may plan at once" }).toBe(true);

    await benPlanner.getByRole("button", { name: "Einsatzplanung (alle Einsätze) entziehen: ben@localhost" }).click();
    await expect(benPlanner, "withdrawn").toHaveCount(0);
    await expect.poll(benMayPlan, { message: "and no longer" }).toBe(false);
    // [itest->dsn~icon-rules~1]
    await dora.locator(".undo-bar").getByRole("button", { name: "Rückgängig" }).click();
    await expect(benPlanner, "undone: given back").toHaveCount(1);
    await expect.poll(benMayPlan, { message: "and may plan again" }).toBe(true);
    await benPlanner.getByRole("button", { name: "Einsatzplanung (alle Einsätze) entziehen: ben@localhost" }).click();
    await expect(benPlanner).toHaveCount(0);
    await openIncident(ben, INCIDENT);
  });

  // [itest->dsn~device-revocation~1]
  await test.step("a lost device is revoked and deletes its data", async () => {
    // Ben sees only his own device, from his person's menu (SEC-4).
    await ben.locator("#profile-button").click();
    await ben.getByRole("link", { name: "Meine Geräte" }).click();
    await expect(ben.locator("#devices-title")).toHaveText("Meine Geräte");
    await expect(ben.locator(".device-card"), "a person sees their own devices").toHaveCount(1);
    await expect(ben.locator(".device-card h3")).toContainText("dieses Gerät");
    // [itest->dsn~device-names~1]
    await expect(ben.locator(".device-card h3"), "named after its browser and system").toHaveText(/^\w+ auf \w+ · …\w{6} \(dieses Gerät\)$/);
    await expect(ben.locator(".device-card dt", { hasText: "Person" }), "no person shown for own devices").toHaveCount(0);

    // Dora, an administrator, sees all devices.
    await openWorkspace(dora, "Zugriff");
    await openView(dora, "Geräte");
    await expect(dora.locator("#devices-title")).toHaveText("Geräte");
    await expect(dora.locator(".device-card"), "an admin sees all devices, the guest's phone and tablet too").toHaveCount(14);
    await expect(dora.locator(".device-card h3", { hasText: "Laptop Einsatzplanung" }), "with the name chosen at pairing").toContainText("dieses Gerät");
    await audit(dora, "Devices");
    const tablet = dora.locator(".device-card", { hasText: "Ben Weber" });
    await tablet.getByRole("button", { name: /^Sperren/ }).click();
    await expect(dora.getByRole("button", { name: "Ja, sperren" }), "focus moves to the confirmation").toBeFocused();
    await audit(dora, "Revoke confirmation", [1200]);
    await shot(dora, "devices-revoke");
    await dora.getByRole("button", { name: "Ja, sperren" }).click();
    await expect(dora.locator(".device-card"), "the revoked tablet is gone from the list").toHaveCount(13);
    await expect.poll(() => announced(dora)).toContain("gesperrt");

    // Ben's tablet is cut off at once and deletes its key and data.
    await expect(ben.getByRole("heading", { name: "Gerät gesperrt" }), "the tablet learns it was revoked").toBeVisible();
    expect(await ben.evaluate(() => [localStorage.getItem("ordo.pairing"), localStorage.getItem("ordo.replica")]), "remote wipe").toEqual([null, null]);
    await audit(ben, "Device revoked", [1200]);
    await shot(ben, "device-revoked");
    await ben.getByRole("button", { name: "Neu koppeln" }).click();
    await expect(ben.getByRole("button", { name: "Anmelden" }), "the tablet can be paired again").toBeVisible();
  });

  // [itest->dsn~local-accounts~1]
  await test.step("an administrator manages local login accounts in Dex", async () => {
    await openWorkspace(dora, "Zugriff");
    await openView(dora, "Konten");
    const accounts = dora.locator(".account-list");
    await expect(accounts.locator("[data-account='anna@example.org'] .roster__person"), "Dex's accounts, by name").toHaveText("Anna Schmidt");

    // A helper joins on scene: an account with a first password.
    await dora.getByLabel("E-Mail-Adresse (Anmeldename)").fill("eva@example.org");
    await dora.getByLabel("Name", { exact: true }).fill("Eva Becker");
    await dora.getByLabel("Erstes Passwort").fill("kurz");
    await dora.getByRole("button", { name: "Konto anlegen" }).click();
    await expect(dora.locator("form [role=alert]").last(), "too short a password is not sent").toHaveText(/mindestens 12 Zeichen/);
    await dora.getByLabel("Erstes Passwort").fill("Helferin-am-Stadtfest");
    await dora.getByRole("button", { name: "Konto anlegen" }).click();
    const eva = accounts.locator("[data-account='eva@example.org']");
    await expect(eva.locator(".roster__person"), "created in Dex").toHaveText("Eva Becker");
    await expect(dora.getByLabel("E-Mail-Adresse (Anmeldename)"), "the form is emptied").toHaveValue("");
    await audit(dora, "Accounts");

    // She logs in with it; ORDO names her from the e-mail address.
    const evaDevice = await newDevice(browser);
    await loginPair(evaDevice, server, "eva@example.org", undefined, "Helferin-am-Stadtfest");

    // A new password, set by the administrator.
    await eva.getByRole("button", { name: "Passwort neu setzen: eva@example.org" }).click();
    await expect(dora.getByLabel("Neues Passwort"), "focus moves to the new password").toBeFocused();
    await dora.getByLabel("Neues Passwort").fill("Neues-Passwort-fuer-Eva");
    await dora.getByRole("button", { name: "Passwort speichern" }).click();
    await expect(dora.locator(".card > [role=status]"), "saved").toHaveText("Gespeichert.");

    // Accounts from Dex's configuration can only be changed there.
    await accounts.locator("[data-account='anna@example.org']").getByRole("button", { name: "Konto löschen: anna@example.org" }).click();
    await dora.getByRole("button", { name: "Ja, Konto löschen" }).click();
    await expect(dora.locator(".card > [role=alert]"), "Dex refuses").toHaveText(/abgelehnt/);

    // Deleting Eva's account ends her login on her device at once.
    await eva.getByRole("button", { name: "Konto löschen: eva@example.org" }).click();
    await expect(dora.getByRole("button", { name: "Ja, Konto löschen" }), "asked in place").toBeFocused();
    await dora.getByRole("button", { name: "Ja, Konto löschen" }).click();
    await expect(eva, "deleted in Dex").toHaveCount(0);
    await expect(evaDevice.getByRole("heading", { name: /Anmeldung abgelaufen/ }), "her device locks").toBeVisible({ timeout: 30_000 });
    await evaDevice.context().close();

    // Logging out from the incident list unpairs the device: another person can log in.
    const spare = await newDevice(browser);
    await loginPair(spare, server, "ben@example.org", "Ersatztablet");
    await spare.getByRole("button", { name: "Abmelden" }).click();
    await expect(spare.getByRole("button", { name: "Anmelden" }), "back at the login").toBeVisible();
    await spare.context().close();
  });

  // [itest->dsn~login-providers~1]
  await test.step("an administrator connects the organisation's login provider after a test login", async () => {
    await openWorkspace(dora, "Zugriff");
    await openView(dora, "Anmeldung");
    const providers = dora.locator(".provider-list");
    await expect(providers.locator("[data-provider='local']"), "Dex's local accounts are listed, not changeable").toContainText("Lokale Konten dieses Servers");
    await expect(dora.locator("#provider-redirect"), "the address to register at the provider").toHaveText(`${DEX_URL}/callback`);
    await dora.getByLabel("Name auf der Anmeldeseite").fill("Organisation");
    await dora.getByLabel("Aussteller-URL (Issuer)").fill(UPSTREAM.issuer);
    await dora.getByLabel("Client-ID").fill(UPSTREAM.clientId);
    await dora.getByRole("button", { name: "Testanmeldung vorbereiten" }).click();
    await expect(dora.locator("form [role=alert]").last(), "the secret is needed").toHaveText(/Client-Secret/);
    await dora.getByLabel("Client-Secret").fill(UPSTREAM.secret);
    await dora.getByRole("button", { name: "Testanmeldung vorbereiten" }).click();
    await audit(dora, "Login providers");

    // The test login, in a new tab: through the temporary connector to the provider.
    const [tab] = await Promise.all([dora.context().waitForEvent("page"), dora.locator("#provider-test-link").click()]);
    await tab.locator("#login").fill(UPSTREAM.email);
    await tab.locator("#password").fill(UPSTREAM.password);
    await tab.locator("#submit-login").click();
    await expect(tab.getByRole("heading", { level: 1 }), "the tab says how it went").toHaveText("Testanmeldung erfolgreich");
    await tab.close();
    const result = dora.locator("#provider-test-result");
    await expect(result, "the identity ORDO would take").toContainText("olga@localhost", { timeout: 15_000 });
    await expect(result).toContainText("Olga Ostermann");
    const save = dora.getByRole("button", { name: "Anbieter speichern" });
    await expect(save, "saving needs a confirmation").toBeDisabled();
    await dora.getByLabel(/Ich bestätige: Anmeldungen über „Organisation“/).check();
    await save.click();
    await expect(dora.locator(".card > [role=status]")).toHaveText("Anbieter gespeichert.");
    const organisation = providers.locator("[data-provider='organisation']");
    await expect(organisation, "saved in Dex, without its secret").toContainText(UPSTREAM.issuer);

    // A person of the organisation logs in through it on a new device.
    const olga = await newDevice(browser);
    await olga.goto(server.url);
    await olga.getByRole("button", { name: "Anmelden" }).click();
    // Dex's theme nests the button in a link: found by its text.
    await olga.getByText("Log in with Organisation").click();
    await olga.locator("#login").fill(UPSTREAM.email);
    await olga.locator("#password").fill(UPSTREAM.password);
    await olga.locator("#submit-login").click();
    // A real login asks the provider for a renewal token, so it asks the person once.
    await olga.locator("button", { hasText: "Grant Access" }).click();
    await expect(olga.getByRole("heading", { name: "ORDO – Einsätze" }), "paired through the organisation's provider").toBeVisible();
    await olga.context().close();

    // Removed again (asked in place); the login page then offers the local accounts only.
    await organisation.getByRole("button", { name: "Entfernen: Organisation" }).click();
    await dora.getByRole("button", { name: "Ja, entfernen" }).click();
    await expect(dora.locator(".card > [role=status]")).toHaveText("Anbieter entfernt.");
    await expect(organisation).toHaveCount(0);
  });

  // [itest->dsn~content-security-policy~1]
  await test.step("the whole scenario ran under the strict Content Security Policy", async () => {
    const policy = (await dora.request.get(server.url)).headers()["content-security-policy"];
    expect(policy, "scripts and styles only from ORDO").toMatch(/script-src 'self' 'wasm-unsafe-eval' 'sha256-[^']+'; style-src 'self';/);
    // Chromium and Firefox report violations promptly, so axe-core's own <style> elements
    // can be told apart (see `runAxe`). WebKit reports them seconds later, without a
    // source, also after reloads; there the same app code is only checked by the header.
    if (dora.context().browser()?.browserType().name() === "webkit") return;
    expect(cspViolations, "no page broke the policy").toEqual([]);
    // And the check sees a violation: an inline style the policy blocks.
    await dora.evaluate(() => document.head.append(Object.assign(document.createElement("style"), { textContent: "p{}" })));
    await expect.poll(() => cspViolations.length, { message: "a blocked inline style is reported" }).toBe(1);
  });
});
