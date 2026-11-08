# 3. Context and scope

> Part of the [ORDO architecture](README.md) (arc42).

## 3.1 Business context

| Partner | Exchange with ORDO | When |
|---|---|---|
| Responders' devices (tablet, laptop, phone) | Events, workspaces, maps | MVP |
| Other ORDO installations (other organisations) | Shared incidents via federation | Phase 4 |
| Alerting systems, control centres | Incoming alerts; exchange via EMSI / CAP | Later |
| Hospitals | Capacity, pre-notification (IVENA eHealth), handover (HL7 FHIR) | Later |
| Identity providers | Login via OpenID Connect | Later |
| Messaging (TI-Messenger, BwMessenger, Element) | Matrix adapter | Later |
| Map data | OpenStreetMap-based PMTiles regions | MVP |

## 3.2 Technical context: devices and servers

```mermaid
flowchart LR
  subgraph Scene["At the scene"]
    T1["Fire command tablet"]
    T2["UHS tablet (Med)"]
    P1["Medic phone (Med)"]
    E["Edge server (optional)<br/>mini PC in command vehicle"]
  end

  subgraph Org["Organisation (self-hosted or hosted)"]
    S["ORDO server"]
    DB[("Database<br/>SQLite / PostgreSQL")]
    TILES[("Map tiles<br/>PMTiles")]
  end

  OTHER["Other ORDO server<br/>(other organisation)"]
  EXT["External systems<br/>(hospitals, control centre, OIDC)"]

  T1 <-- "sync: shared compartment" --> S
  T2 <-- "sync: shared + clinical" --> S
  P1 <-- "sync: shared + clinical" --> S
  T1 <-. "local Wi-Fi" .-> E
  T2 <-. "local Wi-Fi" .-> E
  E <-. "sync when online" .-> S
  S --- DB
  S --- TILES
  S <-. "federation (later)" .-> OTHER
  S <-. adapters .-> EXT
```

- The **ORDO app** runs on every device and keeps a local copy of the incidents it
  takes part in — but only the **compartments** its user may read.
- The **ORDO server** receives changes, puts them in order, stores them and sends them
  to the devices entitled to them. It also serves the app itself, so one binary is a
  complete installation.
- The **edge server** (later) is the same binary on a small computer in the command
  vehicle, for scenes without internet.
