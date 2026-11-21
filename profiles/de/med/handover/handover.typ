// Handover document of the DE profile (ADR-0089, EPCR-20): what a hospital, a control
// centre or another service gets when ORDO hands a patient over outside ORDO. The server
// compiles it with Typst to PDF/A-3a and PDF/UA-1 and attaches the same data as an HL7
// FHIR bundle. An organisation may replace this file (with its fonts) in its own profile.
//
// The data: json("/data.json"), see packages/med/handover/src/data.rs. It holds IDs, the
// profile's labels and what people wrote; the words below map the IDs. Only layout here:
// nothing rates, checks or proposes a value (EPCR-9).

#let d = json("/data.json")

#let category = (
  immediate: "SK I – rot",
  urgent: "SK II – gelb",
  minor: "SK III – grün",
  expectant: "SK IV – blau",
  deceased: "Verstorben",
  uninjured: "Betroffene Person, unverletzt",
)
#let step = (
  registered: "Erfasst",
  readmitted: "Wieder aufgenommen",
  moved: "Verlegt",
  handedOver: "Übergabe",
  taken: "Übernommen",
  takenBack: "Übergabe zurückgenommen",
  discharged: "Entlassen",
  handedOutside: "Übergeben außerhalb von ORDO",
)
#let outcome = (
  releasedOnSite: "vor Ort entlassen",
  transported: "transportiert",
  refused: "Behandlung oder Transport verweigert",
  deceased: "verstorben",
)
#let status = (
  given: "angeordnet",
  acknowledged: "übernommen",
  done: "durchgeführt",
  withdrawn: "zurückgezogen",
)

#let date(m) = m.date.split("-").rev().join(".")
#let at(m) = [#date(m), #m.time Uhr]
#let label(map, id) = map.at(id, default: id)

#set document(
  title: "Patientenübergabe " + d.tag,
  author: if d.handover != none { d.handover.by } else { "ORDO" },
  keywords: ("Patientenübergabe", d.tag),
)
#set text(font: "Atkinson Hyperlegible Next", lang: "de", region: "DE", size: 10pt)
#set par(justify: false)
#set page(
  paper: "a4",
  margin: (x: 18mm, top: 18mm, bottom: 22mm),
  footer: context [
    #set text(size: 8pt)
    Patient #d.tag · erstellt #at(d.created) mit ORDO · Die Daten liegen dieser Datei als
    HL7-FHIR-Bundle bei. #h(1fr) Seite #counter(page).display() von
    #counter(page).final().first()
  ],
)
#set table(stroke: 0.5pt + luma(60%), inset: 5pt)
#show table.cell.where(y: 0): set text(weight: "bold")
#show heading.where(level: 2): set block(above: 1.4em, below: 0.6em)

= Patientenübergabe #d.tag

#grid(
  columns: if d.photo != none { (1fr, 32mm) } else { (1fr,) },
  gutter: 6mm,
  [
    #text(size: 14pt, weight: "bold")[Sichtungskategorie: #label(category, d.category)]

    #set terms(separator: [: ], spacing: 0.6em)
    #for f in d.facts [/ #f.label: #f.value
    ]
  ],
  ..if d.photo != none {
    (image(d.photo, width: 32mm, alt: "Foto des Patienten zur Identifizierung"),)
  },
)

#if d.handover != none {
  let h = d.handover
  [== Übergabe]
  table(
    columns: (auto, 1fr),
    table.header[Angabe][Wert],
    [Übergeben an], [#h.to],
    [Übergeben von], [#h.from],
    [Zeit], at(h.at),
    [Übergeben durch], [#h.by],
    ..if h.note != none { ([Notiz], [#h.note]) },
  )
}

== Sichtung
#table(
  columns: (auto, auto, 1fr, auto),
  table.header[Zeit][Kategorie][Durch][Algorithmus],
  ..d.triage.map(t => (at(t.at), label(category, t.category), t.by, if t.algorithm != none { t.algorithm } else { "–" })).flatten(),
)

== Weg des Patienten
#table(
  columns: (auto, auto, 1fr, auto),
  table.header[Zeit][Schritt][Stelle][Durch],
  ..d.way.map(s => (
    at(s.at),
    label(step, s.kind),
    if s.outcome != none { label(outcome, s.outcome) } else if s.holder != none { s.holder } else { "–" },
    s.by,
  )).flatten(),
)

#if d.orders.len() > 0 {
  [== Anordnungen]
  table(
    columns: (auto, 1fr, auto, auto),
    table.header[Angeordnet][Was][Durch][Stand],
    ..d.orders.map(o => (
      at(o.at),
      o.what,
      o.by,
      label(status, o.status) + if o.done != none { " " + o.done.time + " Uhr" } else { "" }
        + if o.performed_by != none { " (" + o.performed_by + ")" } else { "" },
    )).flatten(),
  )
}

#for form in d.forms {
  [== #form.label]
  if form.kind == "snapshot" {
    let e = form.entries.first()
    table(
      columns: (auto, 1fr),
      table.header[Angabe][Wert],
      ..e.values.map(v => (v.label, v.value)).flatten(),
    )
    text(size: 8pt)[Zuletzt geändert #at(e.at) von #e.by]
  } else {
    table(
      columns: (auto, 1fr),
      table.header[Zeit, durch][Werte],
      ..form.entries.map(e => (
        [#at(e.at) \ #e.by],
        e.values.map(v => [*#v.label:* #v.value]).join([ · ]),
      )).flatten(),
    )
  }
}
