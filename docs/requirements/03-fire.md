# ORDO Requirements — Functional requirements: Fire package

> Part of the [ORDO requirements](README.md). Format: OpenFastTrace items, see the README.

### 4.8 Fire package

#### FIRE-1 — Fire command workspace
`req~fire-1~1`

Fire command workspace: units, sectors, tasks, map, log with fire wording and symbols

Tags: must, fire

Needs: dsn

#### FIRE-2 — Breathing-apparatus monitoring
`req~fire-2~1`

**Breathing-apparatus monitoring** (*Atemschutzüberwachung*): register teams, start time, pressure readings, alarms at configurable thresholds, retreat reminders. Safety-critical: own detailed requirements and tests before release

Tags: should, fire

Needs: dsn

#### FIRE-3 — Water supply
`req~fire-3~1`

Water supply: hydrants and water sources on the map, hose lines

Tags: should, fire

Needs: dsn

#### FIRE-4 — Injured or missing firefighter
`req~fire-4~1`

Injured or missing firefighter: one action raises a request to Med and marks the event in the log (via connector)

Tags: should, fire

Needs: dsn

#### FIRE-5 — Hazardous materials
`req~fire-5~1`

Hazardous materials support (substance lookup, danger zones)

Tags: could, fire

Needs: dsn
