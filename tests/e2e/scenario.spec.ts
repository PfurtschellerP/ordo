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
test("ORDO in the browser", async ({ browser }) => {
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
    await grantRole(dora, "petra@localhost", "Arzt/Ärztin");
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
    for (const item of ["ordo.pairing", "ordo.storage-key"]) {
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

    // [itest->dsn~patient-photo~2]
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

    // [itest->dsn~transport-request~1]
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

    // [itest->dsn~medical-orders~1]
    // [itest->dsn~order-permission~1]
    await expect(anna.locator(".order-form"), "a medic gives no orders").toHaveCount(0);
    const petra = await newDevice(browser);
    await loginPair(petra, server, "petra@example.org");
    await openIncident(petra, INCIDENT);
    await openView(petra, "Patienten");
    await openRecord(petra, "DE-0044");
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
    await expect(order).toContainText("petra@localhost");
    await order.getByRole("button", { name: "Übernehmen: Paracetamol 1 g i.v." }).click();
    await expect(order.locator(".order-card__status")).toContainText("Übernommen");
    await audit(anna, "Orders");
    await order.getByRole("button", { name: "Durchgeführt: Paracetamol 1 g i.v." }).click();
    await expect(anna.locator("#main"), "done orders leave the open list").toContainText("Keine offenen Anordnungen.");
    await openView(anna, "Patienten");
    await openRecord(anna, "DE-0044");
    await expect(anna.locator("[aria-labelledby=section-orders] .order-card__status"), "the record keeps it").toContainText("Durchgeführt");
    await anna.locator("[aria-labelledby=section-orders] summary").click();
    await expect(anna.locator("[aria-labelledby=section-orders] .history-list li"), "with every step").toHaveCount(3);

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
    await expect(anna.locator(".connection"), "offline has its own icon and colour").toHaveClass(/connection--offline/);
    await register(anna, "DE-0045", 2, "UHS Eingang");
    await expect.poll(() => status(anna), { message: "pending input is shown" }).toContain("1 Eingabe noch nicht gesendet");
    // [itest->dsn~local-persistence~1]
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
      const m = document.getElementById("launcher-bar-button")!.getBoundingClientRect();
      return [Math.round(n.bottom), innerHeight, m.top >= n.top && m.bottom <= n.bottom];
    });
    expect(barBottom, "the bar sits at the bottom").toBe(viewportHeight);
    expect(menuInBar, "the menu is in the bar").toBe(true);
    // [itest->dsn~header-layout~1]
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
    await expect(anna.locator(".navigation .views__item .icon svg"), "icons above the labels in the bar").toHaveCount(3);
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
    await openRecord(anna, "DE-0042");
    await shot(anna, "record-phone", false);
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
    await openView(ben, "Personal");
    await expect(ben.locator(".roster__entry", { hasText: "anna@localhost" }).locator("img.avatar"), "others see Anna's picture").toBeVisible();
    await expect(ben.locator(".roster__entry", { hasText: "dora@localhost" }).locator(".avatar--initials"), "without a picture: initials").toHaveText("D");
    await openView(ben, "Patientenzahlen");
    // [itest->dsn~workspace-navigation~3]
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
    // [itest->dsn~header-layout~1]
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
    await ben.locator("#launcher-button").click();
    await ben.locator("#profile-button").click();
    await expect(ben.locator("#profile-panel"), "the person's menu holds their settings").toContainText("Angemeldet als ben@localhost");
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
      "UHS Bühne Nord 1 0 1 0 0 0 Gesamt: 2 · Belegung: 2 / 10 · Entlassen: 0",
      "UHS Eingang 1 0 1 0 0 0 Gesamt: 2 · Belegung: 2 / 5 · Entlassen: 0",
    ]);
    await ben.setViewportSize({ width: 1700, height: 1000 });
    const table = () =>
      ben.locator("#main tbody tr").evaluateAll((rows) =>
        rows.filter((r) => (r as HTMLElement).offsetParent).map((r) => [...(r as HTMLTableRowElement).cells].map((c) => c.innerText.split("\n")[0]).join(" ")),
      );
    await expect.poll(table, { message: "where there is room, the counts are a table" }).toEqual([
      "UHS Bühne Nord 1 0 1 0 0 0 2 2 / 10 0",
      "UHS Eingang 1 0 1 0 0 0 2 2 / 5 0",
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

  await test.step("transport: the UHS requests, the command assigns a unit and follows it", async () => {
    // [itest->dsn~transport-request~1]
    await openView(ben, "Transporte");
    const request = ben.locator(".transport-card");
    await expect(request, "the command sees the open request").toHaveCount(1);
    await expect(request).toContainText("UHS Eingang · RTW · Sofort · Infektiös");
    await expect(request, "without patient data").not.toContainText("DE-0044");
    await request.getByLabel("Einsatzmittel", { exact: true }).selectOption("RTW 83/1");
    await request.getByLabel("Ziel (z. B. Klinik)").fill("Klinikum Mitte");
    await request.getByRole("button", { name: /^Zuweisen: / }).click();
    const transportStatus = request.locator(".transport-card__status");
    await expect(transportStatus, "a unit and a destination are assigned").toHaveText("Zugewiesen · RTW 83/1 → Klinikum Mitte");
    await request.getByRole("button", { name: /^Abgefahren: / }).click();
    await expect(transportStatus, "the status moves on, also skipping a step").toHaveText("Abgefahren · RTW 83/1 → Klinikum Mitte");
    await expect(request.getByRole("button", { name: /^Stornieren: / }), "no cancelling on the road").toHaveCount(0);
    await request.locator("summary").click();
    await expect(request.locator(".history-list li"), "every change with time and person").toHaveCount(3);
    await expect.poll(() => status(ben), { message: "the transport is sent" }).toContain("Alles gesendet");
    expect(Buffer.concat(benFrames.received).includes("DE-0044"), "the command device never receives the triage tag number").toBe(false);
    await audit(ben, "Transports");
    await shot(ben, "transports");
  });

  // [itest->dsn~uhs-discharge~1]
  await test.step("the UHS discharges the transported patient; the load drops", async () => {
    const medic = await newDevice(browser);
    await loginPair(medic, server, "anna@example.org");
    await openIncident(medic, INCIDENT);
    await openView(medic, "Patienten");
    await openRecord(medic, "DE-0044");
    const stay = medic.locator("[aria-labelledby=section-stay]");
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

    await openView(ben, "Patientenzahlen");
    await expect
      .poll(() => ben.locator(".count-card", { hasText: "UHS Eingang" }).locator("p").textContent(), { message: "command sees the load without the discharged patient" })
      .toBe("Gesamt: 1 · Belegung: 1 / 5 · Entlassen: 1");
    expect(Buffer.concat(benFrames.received).includes("DE-0044"), "but not who left").toBe(false);
    await openView(ben, "Einsatzmittel"); // the next step checks that units disappear when locked
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
    await expect(dora.locator(".device-card"), "an admin sees all devices").toHaveCount(6);
    await audit(dora, "Devices");
    const tablet = dora.locator(".device-card", { hasText: "ben@localhost" });
    await tablet.getByRole("button", { name: /^Sperren/ }).click();
    await expect(dora.getByRole("button", { name: "Ja, sperren" }), "focus moves to the confirmation").toBeFocused();
    await audit(dora, "Revoke confirmation", [1200]);
    await shot(dora, "devices-revoke");
    await dora.getByRole("button", { name: "Ja, sperren" }).click();
    await expect(dora.locator(".device-card"), "the revoked tablet is gone from the list").toHaveCount(5);
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
