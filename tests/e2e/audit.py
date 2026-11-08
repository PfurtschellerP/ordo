"""Checks of the rendered page (not of tokens): what the user actually sees.

- Text contrast (WCAG 1.4.3) of every visible text, against the composited background.
- Table rows: all cells of a row end at the same line (no broken card/table layout).
- Touch targets (UX-2): buttons and inputs at least 44 CSS px high.
- Reflow (WCAG 1.4.10): the page does not scroll horizontally, and no content is wider
  than its card (a table that only scrolls inside its card is a reflow failure too).
- Fixed bars (bottom navigation): scrolled to the end, they cover no content.

Returns a list of human-readable problems; empty means everything passed.
"""

AUDIT_JS = r"""
const parse = (c) => {
  const m = c && c.match(/rgba?\(([^)]+)\)/);
  if (!m) return null;
  const p = m[1].split(/[\s,\/]+/).filter(Boolean).map(Number);
  return { r: p[0], g: p[1], b: p[2], a: p.length > 3 ? p[3] : 1 };
};
const blend = (fg, bg) => ({
  r: fg.r * fg.a + bg.r * (1 - fg.a), g: fg.g * fg.a + bg.g * (1 - fg.a),
  b: fg.b * fg.a + bg.b * (1 - fg.a), a: 1,
});
const lum = ({ r, g, b }) => {
  const f = (v) => { v /= 255; return v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); };
  return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
};
const ratio = (a, b) => { const x = lum(a), y = lum(b); return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05); };
const background = (el) => {
  const chain = [];
  for (let e = el; e; e = e.parentElement) chain.unshift(e);
  let bg = { r: 255, g: 255, b: 255, a: 1 };
  for (const e of chain) {
    const c = parse(getComputedStyle(e).backgroundColor);
    if (c && c.a > 0) bg = blend(c, bg);
  }
  return bg;
};
const visible = (el) => {
  if (el.closest('.visually-hidden')) return false;
  const r = el.getBoundingClientRect();
  const cs = getComputedStyle(el);
  return r.width > 0 && r.height > 0 && cs.visibility !== 'hidden' && cs.display !== 'none';
};
const describe = (el, text) => `${el.tagName.toLowerCase()}${el.className ? '.' + String(el.className).split(' ').join('.') : ''} "${text.slice(0, 30)}"`;
const problems = [];

// Text contrast
for (const el of document.querySelectorAll('body *')) {
  if (!visible(el)) continue;
  let text = [...el.childNodes].filter((n) => n.nodeType === 3).map((n) => n.textContent.trim()).join(' ').trim();
  if (el.matches('input[type=text], input[type=password], textarea')) text = el.value || '';
  if (el.matches('select')) text = el.options[el.selectedIndex]?.text || '';
  if (!text) continue;
  const cs = getComputedStyle(el);
  const bg = background(el);
  const fg = blend(parse(cs.color), bg);
  const size = parseFloat(cs.fontSize), bold = parseInt(cs.fontWeight, 10) >= 700;
  const min = size >= 24 || (bold && size >= 18.66) ? 3 : 4.5;
  const r = ratio(fg, bg);
  if (r < min) problems.push(`contrast ${r.toFixed(2)} < ${min}: ${describe(el, text)} (${cs.color} on rgb(${bg.r | 0}, ${bg.g | 0}, ${bg.b | 0}))`);
}

// Table rows: cells end on the same line
for (const row of document.querySelectorAll('tr')) {
  if (!visible(row)) continue;
  const bottoms = [...row.cells].map((c) => Math.round(c.getBoundingClientRect().bottom));
  if (Math.max(...bottoms) - Math.min(...bottoms) > 1) problems.push(`table row cells end at different lines: ${bottoms.join(', ')} (${row.innerText.slice(0, 30).replace(/\n/g, ' ')})`);
}

// Table cells must stay table cells (e.g. display:flex on a <td> breaks row lines)
for (const cell of document.querySelectorAll('td, th')) {
  if (!visible(cell)) continue;
  const display = getComputedStyle(cell).display;
  if (display !== 'table-cell') problems.push(`table cell with display:${display}: ${describe(cell, cell.innerText)}`);
}

// Touch targets
for (const el of document.querySelectorAll('button, select, textarea, input[type=text], input[type=password], input[type=date], input[type=time], a.skip-link, a.views__item, a.patient-card, a.back-link, label.button, label.chip, summary, [role=option]')) {
  if (!visible(el) || el.matches('a.skip-link')) continue;
  const h = el.getBoundingClientRect().height;
  if (h < 44) problems.push(`touch target ${h.toFixed(0)} px < 44 px: ${describe(el, el.innerText || el.id)}`);
}

// Reflow: no horizontal scrolling of the page, nothing wider than its card
const doc = document.documentElement;
if (doc.scrollWidth > doc.clientWidth + 1) problems.push(`page scrolls horizontally: ${doc.scrollWidth} > ${doc.clientWidth}`);
for (const box of document.querySelectorAll('.card, .record__section, .entry, .editor')) {
  if (visible(box) && box.scrollWidth > box.clientWidth + 1) problems.push(`content wider than its box: ${describe(box, box.innerText || '')} (${box.scrollWidth} > ${box.clientWidth})`);
}

// Fixed bars must not cover content: scrolled to the very end, nothing lies beneath them.
const bars = [...document.querySelectorAll('body *')].filter((e) => getComputedStyle(e).position === 'fixed' && visible(e) && !e.closest('dialog, .launcher__panel'));
if (bars.length) {
  const before = window.scrollY;
  window.scrollTo(0, doc.scrollHeight);
  const content = [...document.querySelectorAll('#main h2, #main h3, #main p, #main li, #main button, #main a, #main input, #main select, #main textarea, #main label, #main td, #main th')]
    .filter((e) => visible(e) && !bars.some((b) => b.contains(e)));
  for (const bar of bars) {
    const b = bar.getBoundingClientRect();
    const hit = content.find((e) => { const r = e.getBoundingClientRect(); return r.bottom > b.top + 1 && r.top < b.bottom - 1 && r.right > b.left && r.left < b.right; });
    if (hit) problems.push(`covered by a fixed bar when scrolled to the end: ${describe(hit, hit.innerText || hit.id || '')}`);
  }
  window.scrollTo(0, before);
}
return problems;
"""

THEMES = ("day", "night", "high-contrast")


def settle(ff, timeout=5.0):
    """Waits until no CSS transition or animation runs, so measurements and screenshots
    show the final state."""
    import time
    end = time.time() + timeout
    while time.time() < end and ff.js("return document.getAnimations().length") > 0:
        time.sleep(0.05)


def audit(ff, label, check, widths=(1200, 320)):
    """Runs the audit in every theme and at every window width; reports via `check`."""
    for width in widths:
        ff.cmd("WebDriver:SetWindowRect", {"width": width, "height": 1000})
        for theme in THEMES:
            ff.js("document.documentElement.dataset.theme = arguments[0]", theme)
            settle(ff)
            problems = ff.js(AUDIT_JS) or []
            check(not problems, f"{label}, theme {theme}, {width} px: rendered page passes"
                  + ("" if not problems else ":\n      " + "\n      ".join(problems[:12])))
    ff.js("delete document.documentElement.dataset.theme")
    ff.cmd("WebDriver:SetWindowRect", {"width": 1200, "height": 1000})
