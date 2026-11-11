# ORDO Design System

All visual decisions of ORDO — colours, spacing, type, motion — are **design tokens** in
the open [W3C Design Tokens format](https://www.designtokens.org/tr/2025.10/) (DTCG
2025.10). Because the format is a standard, designers can also edit the tokens with
their own tools (e.g. Tokens Studio for Figma, Style Dictionary, Terrazzo).

Architecture: [chapter 8.6](../docs/architecture/08-concepts.md#86-design-system-and-accessibility).
Requirements: [§5.8 Accessibility, §5.9 Design system](../docs/requirements/07-quality.md#58-accessibility).

## Files

| File | Content |
|---|---|
| [`design.toml`](design.toml) | Manifest: token files, themes, protected groups |
| [`tokens/primitive.tokens.json`](tokens/primitive.tokens.json) | Raw values: colour scales, spacing, sizes, type, motion |
| [`tokens/component.tokens.json`](tokens/component.tokens.json) | Component tokens (buttons, inputs, triage badges, …), referring to semantic tokens |
| [`themes/*.tokens.json`](themes/) | Semantic colour tokens per theme: `day`, `night`, `high-contrast` |
| [`contrast.toml`](contrast.toml) | Colour pairs that must meet WCAG contrast in every theme |

## Layers

```
primitive  →  semantic (per theme)  →  component
color.red.600  color.safety.triage.immediate.fill  component.triageBadge.immediate.background
```

- **Themes change semantic tokens only.** All themes must define the same tokens.
- **`color.safety.*` is protected:** triage categories and alarms keep their meaning;
  organisation themes cannot change them.
- Spacing and type use `rem`, so everything grows with the user's text size.

## Themes

| Theme | Selected when | Notes |
|---|---|---|
| `day` | default | Light surfaces for daylight |
| `night` | `prefers-color-scheme: dark`, or `data-theme="night"` | Dark surfaces that don't dazzle |
| `high-contrast` | `prefers-contrast: more`, or `data-theme="high-contrast"` | Black on white; all text ≥ 7:1 (WCAG AAA) |

Setting `data-theme` on the `<html>` element overrides the operating-system choice.
`prefers-reduced-motion: reduce` sets all durations to 0 ms.

## Tool: `ordo-tokens`

```sh
# Check token files and the contrast of every theme
cargo run -p ordo-tokens -- check

# Check, then write ordo-tokens.css and contrast-report.md
cargo run -p ordo-tokens -- build --out target/design

# Check an organisation theme (branding) on top of a base theme
cargo run -p ordo-tokens -- check-theme my-org.tokens.json --base day
```

The build fails if any contrast pair falls below its WCAG minimum. Ratios are always
rounded **down** (4.499:1 does not count as 4.5:1).

## Organisation themes

An organisation theme is a token file that changes some semantic tokens of one base
theme, for example:

```json
{
  "color": {
    "$type": "color",
    "action": {
      "primary": {
        "background": { "$value": { "colorSpace": "srgb", "components": [0.6, 0.0, 0.1], "hex": "#99001A" } }
      }
    }
  }
}
```

It is rejected if it changes a token that doesn't exist in the base theme, touches
`color.safety.*`, or breaks any contrast pair.

## Assets

- Typeface: **Atkinson Hyperlegible Next** and **Mono** (SIL OFL 1.1)
- Icons: **Lucide** (ISC)

Icons are stored under `assets/icons/` with their licence files: Lucide 1.70.0 (only the icons the app uses, copied from the `lucide-static` package after checking its npm integrity hash). The typefaces are served with the app from `platform/ordo-app/fonts/`: regular and bold, Latin and Latin Extended, WOFF2, from the Fontsource packages 5.3.0 (`vendor-fonts.sh`, with their licences); the service worker keeps them for offline use.
