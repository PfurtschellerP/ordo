# ORDO Glossary

One language for responders, documentation and code. **Code and documentation use the
English term**; the German term is what users see in the DE profile. Regional labels
come from [profiles](architecture/08-concepts.md#88-regional-profiles), never from code.

## Incident command (shared)

| English (code, docs) | German (DE profile) | Meaning |
|---|---|---|
| incident | Einsatz | An emergency or planned operation that ORDO coordinates |
| planned operation | geplanter Einsatz, z. B. Sanitätswachdienst | An incident set up in advance, e.g. first-aid cover at a concert |
| incident keyword | Einsatzstichwort | Classification of an incident, from the profile |
| incident commander | Einsatzleiter (EL) | Person in overall command on scene |
| incident command | Einsatzleitung | The incident commander and their staff |
| handover of command | Übergabe der Einsatzleitung | Transfer of a leadership role to another person; always logged |
| command assistant | Führungsassistent, Melder | Supports the commander: log, radio, unit status |
| sector | Einsatzabschnitt (EA) | Part of the incident with its own leader (FwDV 100) |
| sub-sector | Untereinsatzabschnitt | Part of a sector |
| sector leader | Abschnittsleiter | Leads one sector |
| unit | Einsatzmittel (Fahrzeug, Trupp, Gruppe) | A vehicle or team with its crew |
| unit leader | Gruppenführer, Truppführer | Leads a unit |
| call sign | Funkrufname | Radio name of a unit |
| crew strength | (taktische) Stärke | Written in tactical notation, e.g. 1/8/**9**: 1 leader, 8 crew, 9 in total |
| unit status | Status (FMS/TMS 1–9) | Operational state of a unit; codes from the profile, mapped to core meanings |
| task | Auftrag | Order given to a unit or sector, with status |
| incident log | Einsatztagebuch | Append-only record of everything that happened |
| radio message | Funkspruch | Logged with sender, receiver and direction |
| situation map | Lagekarte | Map of the incident with units, sectors and drawings |
| tactical symbols | Taktische Zeichen (DV 102) | Standard map symbols for units and objects |
| staging area | Bereitstellungsraum | Where arriving units wait for tasks |
| cordon | Absperrung | Closed-off area |
| control centre | Leitstelle | Dispatch centre that alerts units (out of scope for ORDO) |

## Fire

| English | German | Meaning |
|---|---|---|
| breathing-apparatus monitoring | Atemschutzüberwachung | Tracking of BA teams, time and pressure |
| water supply | Löschwasserversorgung | Hydrants, water sources, hose lines |
| fire-brigade plan | Feuerwehrplan | Building plan for fire services |
| vehicle types | HLF, LF, TLF, DLK, ELW | Rescue pumper, pumper, water tender, aerial ladder, command vehicle |

## Med

| English | German | Meaning |
|---|---|---|
| on-site care unit | Unfallhilfsstelle (UHS) | Place where patients are treated on scene, e.g. at an event |
| treatment area | Behandlungsplatz (BHP) | Larger treatment structure in a mass-casualty incident |
| patient collection point | Patientenablage | Where patients are gathered before treatment |
| triage | Sichtung | Sorting patients by urgency |
| triage category | Sichtungskategorie SK I–IV | SK I immediate (red), SK II urgent (yellow), SK III minor (green), SK IV expectant (blue); plus deceased |
| triage tag | Patientenanhängekarte (PAK) | Physical tag with a unique number, attached to the patient |
| re-triage | Nachsichtung | Assigning a new triage category; history is kept |
| mass-casualty incident (MCI) | Massenanfall von Verletzten (MANV) | More patients than regular resources can handle |
| medical incident lead | Leitender Notarzt (LNA), Organisatorischer Leiter Rettungsdienst (OrgL) | Physician and organisational lead of the medical response |
| physician, emergency physician | Arzt, Notarzt (NA) | — |
| medic | Sanitäter, Rettungssanitäter, Notfallsanitäter | Medical staff below physician level |
| transport officer | Transportorganisation | Plans patient transports |
| transport request | Transportanforderung | Request to move a patient, with priority and needs |
| ambulance types | RTW, KTW, NEF | Emergency ambulance, patient transport ambulance, physician response vehicle |
| electronic patient care record (ePCR) | elektronisches Einsatzprotokoll | Documentation of findings, measures and medication per patient |
| refusal of treatment / transport | Behandlungs- / Transportverweigerung | Documented with signature |
| emergency dataset | DIVI MIND | German minimum emergency dataset, basis of the ePCR forms |

## Police

| English | German | Meaning |
|---|---|---|
| police liaison | Verbindungsbeamter | Police officer working with the other services' command |

## ORDO concepts

| Term | Meaning |
|---|---|
| platform | Lowest layer: events, sync, compartments, identities, module system, profiles, UI shell |
| shared module | Function used by all services: incidents, units, command, tasks, map, log, org |
| service package | Bundle for one service (Fire, Med, Police): modules, workspaces, wording |
| workspace | What a role sees when it opens ORDO; defined as data |
| connector | Versioned contract for data crossing a package border, published as events |
| adapter | Integration with an external system |
| compartment | Separately protected part of an incident's data, e.g. `incident`, `med.clinical` |
| event | Immutable record of one change; the source of all state |
| projection | Current state computed from events |
| HLC | Hybrid logical clock: timestamp close to real time, consistently ordered |
| home server | The server of the leading organisation; orders an incident's events |
| hash chain | Chained hashes over an incident's ordered events; detects tampering |
| federation | Cooperation of ORDO installations of different organisations |
| profile | Regional or organisational data package: labels, codes, symbols, forms |
| core meaning | The meaning a profile entry maps to, e.g. status 3 → `en_route` |
| design token | Named design value (colour, size, …) in the W3C DTCG format |
| safety token | Design token under `color.safety.*`; cannot be changed by organisation themes |
| specification item | Requirement, design item or coverage in OpenFastTrace format |

## Accessibility and law

| English | German | Meaning |
|---|---|---|
| accessibility statement | Erklärung zur Barrierefreiheit | Statement public bodies must publish for their ICT |
| easy-to-read / plain language | Leichte Sprache / Einfache Sprache | Simplified language for people with reading difficulties |
| German Sign Language | Deutsche Gebärdensprache (DGS) | — |
