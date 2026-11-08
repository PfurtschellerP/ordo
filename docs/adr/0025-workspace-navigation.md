---
status: accepted
date: 2026-11-08
---

# Navigation: workspace launcher plus view rail or bottom bar

## Context and Problem Statement

ORDO will grow to many service packages (Med, Fire, Police, …) and workspaces. Flat tabs
at the top of the page do not scale. People may hold several roles; the user experience
must stay the same whatever role someone has.

## Considered Options

* Tabs at the top for everything
* A start menu (launcher) only, holding workspaces and views
* Two levels: launcher for workspaces (switched rarely) and always-visible views (switched often): a rail on wide screens, a bottom bar on phones

## Decision Outcome

Chosen option: "Two levels", because switching workspaces is rare while views are used
all the time; views must stay one tap away (UX-1), and the structure scales to any number
of packages.

Rules:

* The launcher shows every workspace of the packages the organisation uses, grouped by
  package. A workspace the user may not use in this incident stays in its place, marked
  as unavailable (dashed border, `aria-disabled`) with the reason; packages the
  organisation does not use are not shown.
* A person can hold several roles and switches freely between the workspaces they may
  use; the device opens the last used workspace, or the first one available.
* Incident and connection state are always visible in the header.
* The launcher is a disclosure button with a visible label; Escape closes it and returns
  focus; the current workspace and view are marked with `aria-current`.

### Consequences

* Good, because the layout is identical for every role and scales to new packages.
* Good, because workspaces are data (ADR-0011) and can later come from package and
  profile definitions.
* Neutral: notifications that concern other workspaces (e.g. "responder injured") will
  need a global place in the header.

## More Information

Details: [5.5.3 Web app](../architecture/05-building-block-view.md#553-web-app-ordo-app),
end-to-end test `tests/e2e/scenario.py`.
