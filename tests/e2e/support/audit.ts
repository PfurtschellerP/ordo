// Audit of the rendered page in every theme and at phone and laptop width.
//
// - axe-core (Deque, the de-facto standard engine) checks WCAG 2.2 A/AA rules:
//   contrast, names, roles, ARIA, target size, ...
// - Our own checks cover what axe does not:
//   - touch targets of at least 44 CSS px (UX-2, stricter than WCAG's 24 px);
//   - reflow: no horizontal page scroll, no content wider than its card (WCAG 1.4.10);
//   - table rows whose cells end on one line (broken table layouts);
//   - fixed bars (bottom navigation) covering content when scrolled to the end.
// axe runs in Chromium and WebKit (see below); Firefox runs our own checks.
import AxeBuilder from "@axe-core/playwright";
import { expect, Page } from "@playwright/test";

export const THEMES = ["day", "night", "high-contrast"] as const;

type Tooling = { ordoE2eTooling?: number };

/** Runs axe-core, marked in the page: it reads stylesheets through <style> elements of its
 *  own, which the app's Content Security Policy rightly blocks; the scenario ignores
 *  violations while the mark is set. */
async function runAxe(page: Page) {
  await page.evaluate(() => {
    const w = window as unknown as Tooling;
    w.ordoE2eTooling = (w.ordoE2eTooling ?? 0) + 1;
  });
  try {
    return await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"]).analyze();
  } finally {
    // Unmarked as a later task, after the violation events axe-core caused.
    await page.evaluate(
      () =>
        new Promise<void>((done) =>
          setTimeout(() => {
            const w = window as unknown as Tooling;
            w.ordoE2eTooling = (w.ordoE2eTooling ?? 1) - 1;
            done();
          }, 50),
        ),
    );
  }
}

const LAYOUT_CHECKS = () => {
  const visible = (el: Element) => {
    if (el.closest(".visually-hidden")) return false;
    const r = el.getBoundingClientRect();
    const cs = getComputedStyle(el);
    return r.width > 0 && r.height > 0 && cs.visibility !== "hidden" && cs.display !== "none";
  };
  const describe = (el: Element, text: string) =>
    `${el.tagName.toLowerCase()}${el.className ? "." + String(el.className).split(" ").join(".") : ""} "${text.slice(0, 30)}"`;
  const problems: string[] = [];

  for (const row of document.querySelectorAll("tr")) {
    if (!visible(row)) continue;
    const bottoms = [...(row as HTMLTableRowElement).cells].map((c) => Math.round(c.getBoundingClientRect().bottom));
    if (Math.max(...bottoms) - Math.min(...bottoms) > 1) problems.push(`table row cells end at different lines: ${bottoms.join(", ")}`);
  }
  for (const cell of document.querySelectorAll("td, th")) {
    if (!visible(cell)) continue;
    const display = getComputedStyle(cell).display;
    if (display !== "table-cell") problems.push(`table cell with display:${display}: ${describe(cell, (cell as HTMLElement).innerText)}`);
  }

  const targets = "button, select, textarea, input[type=text], input[type=password], input[type=date], input[type=time], a.views__item, a.patient-card, a.back-link, label.button, label.chip, summary, [role=option]";
  for (const el of document.querySelectorAll(targets)) {
    if (!visible(el)) continue;
    const h = el.getBoundingClientRect().height;
    if (h < 44) problems.push(`touch target ${h.toFixed(0)} px < 44 px: ${describe(el, (el as HTMLElement).innerText || el.id)}`);
  }

  const doc = document.documentElement;
  if (doc.scrollWidth > doc.clientWidth + 1) problems.push(`page scrolls horizontally: ${doc.scrollWidth} > ${doc.clientWidth}`);
  for (const box of document.querySelectorAll(".card, .record__section, .entry, .editor")) {
    if (visible(box) && box.scrollWidth > box.clientWidth + 1) {
      // Name the innermost element that sticks out, to make the fix obvious.
      const right = box.getBoundingClientRect().right - parseFloat(getComputedStyle(box).paddingRight);
      const culprit = [...box.querySelectorAll("*")].reverse().find((e) => visible(e) && e.getBoundingClientRect().right > right + 1);
      const what = culprit ? `; sticks out: ${describe(culprit, (culprit as HTMLElement).innerText || culprit.id || "")}` : "";
      problems.push(`content wider than its box: ${describe(box, (box as HTMLElement).innerText || "")} (${box.scrollWidth} > ${box.clientWidth}${what})`);
    }
  }

  const bars = [...document.querySelectorAll("body *")].filter(
    (e) => getComputedStyle(e).position === "fixed" && visible(e) && !e.closest("dialog, .launcher__panel"),
  );
  if (bars.length) {
    const before = window.scrollY;
    window.scrollTo(0, doc.scrollHeight);
    const content = [...document.querySelectorAll("#main h2, #main h3, #main p, #main li, #main button, #main a, #main input, #main select, #main textarea, #main label, #main td, #main th")]
      .filter((e) => visible(e) && !bars.some((b) => b.contains(e)));
    for (const bar of bars) {
      const b = bar.getBoundingClientRect();
      const hit = content.find((e) => {
        const r = e.getBoundingClientRect();
        return r.bottom > b.top + 1 && r.top < b.bottom - 1 && r.right > b.left && r.left < b.right;
      });
      if (hit) problems.push(`covered by a fixed bar when scrolled to the end: ${describe(hit, (hit as HTMLElement).innerText || hit.id || "")}`);
    }
    window.scrollTo(0, before);
  }
  return problems;
};

/** Waits until no CSS transition or animation runs, so checks and screenshots see the final state. */
export async function settle(page: Page): Promise<void> {
  await expect.poll(() => page.evaluate(() => document.getAnimations().length), { timeout: 5_000 }).toBe(0);
}

/** Audits the page in every theme and at every width; failures are soft (the scenario goes on). */
export async function audit(page: Page, label: string, widths = [1200, 320]): Promise<void> {
  const size = page.viewportSize() ?? { width: 1200, height: 1000 };
  for (const width of widths) {
    await page.setViewportSize({ width, height: 1000 });
    for (const theme of THEMES) {
      await page.evaluate((t) => (document.documentElement.dataset.theme = t), theme);
      await settle(page);
      // axe runs in Chromium and WebKit. Playwright's patched Firefox build crashes inside
      // axe on some pages (November 2026); stock Firefox 159 runs axe on the same pages
      // without findings. Firefox still runs the whole scenario and our own checks.
      const firefox = page.context().browser()?.browserType().name() === "firefox";
      const axe = firefox
        ? { violations: [] as { id: string; help: string; nodes: { target: unknown[] }[] }[] }
        : await runAxe(page);
      const problems = [
        ...axe.violations.map((v) => `axe ${v.id}: ${v.help} — ${v.nodes.slice(0, 3).map((n) => n.target.join(" ")).join("; ")}`),
        ...(await page.evaluate(LAYOUT_CHECKS)),
      ];
      expect.soft(problems, `${label}, theme ${theme}, ${width} px`).toEqual([]);
    }
  }
  await page.evaluate(() => delete document.documentElement.dataset.theme);
  await page.setViewportSize(size);
}
