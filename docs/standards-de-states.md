# German state sources

Fire service and disaster protection are state law in Germany. Doctrine is drafted
jointly — the Feuerwehr-Dienstvorschriften (FwDV) by the AFKzV of the interior ministers'
conference — and then **introduced by each state**, by decree, sometimes as an extended
*DV* for the whole fire and disaster protection service, sometimes with amendments of its
own. Status codes, alerting rules and mass-casualty concepts are set by each state, too.

ORDO keeps what all states share in the profile `de`, and what a state introduces,
changes or adds in a state profile `de-<state>` that extends it
([architecture 8.8.3](architecture/08-concepts.md#883-layering)). Each profile lists its
sources in `profile.toml`; this page is the overview. Catalogues (command structure,
status codes, …) are added to the profiles when the module that uses them is built,
starting with the medical branch of the command structure
([ADR-0031](adr/0031-command-structure-from-doctrine.md)).

**Tactical symbols** are national: the BBK's *Taktische Zeichen im Bevölkerungsschutz*
(May 2024), recommended to the states for introduction as FwDV 102 / DV 102. No state
decree introducing it was found yet; see [standards](standards.md#command-and-control).

## Command doctrine and other sources per state

| State | Profile | Command doctrine as introduced | Other state sources |
|---|---|---|---|
| Baden-Württemberg | [`de-bw`](../profiles/de-bw/profile.toml) | [FwDV 100 Führung und Leitung im Einsatz](https://www.lfs-bw.de/fileadmin/LFS-BW/themen/gesetze_vorschriften/fwdv/dokumente/FwDV_100.pdf), AFKzV, 10 March 1999 — copy of the Landesfeuerwehrschule | [Taktische Statusmeldungen](https://lfs-bw.de/fileadmin/LFS-BW/themen/funk/digitalfunk/regelungenbetriebshandbuch/dokumente/Taktische_Statusmeldungen_022025.pdf), 02/2025 |
| Bayern | [`de-by`](../profiles/de-by/profile.toml) | [FwDV 100, bekanntgemacht als bayerische Verwaltungsvorschrift](https://www.gesetze-bayern.de/Content/Document/BayVwV96946/true), BayVwV | [Richtlinie FMS (Funkmeldesystem)](https://www.gesetze-bayern.de/Content/Document/BayVwV156097), BayVwV |
| Berlin | [`de-be`](../profiles/de-be/profile.toml) | *not found yet* | — |
| Brandenburg | [`de-bb`](../profiles/de-bb/profile.toml) | [Erlass zur Einführung der FwDV 100 als Feuerwehr- und Katastrophenschutz-Dienstvorschrift 100](https://bravors.brandenburg.de/verwaltungsvorschriften/fwdv100), 27 October 1999 — applies to fire service and disaster protection; replaced a separate KatS-DV 100 | — |
| Bremen | [`de-hb`](../profiles/de-hb/profile.toml) | [FwDV 100, Ausbildungsunterlage der Feuerwehr Bremen](https://www.feuerwehr.bremen.de/sixcms/media.php/13/FwDV_100.pdf) — training material (Lehrmeinung), not the introducing decree | — |
| Hamburg | [`de-hh`](../profiles/de-hh/profile.toml) | *not found yet* | — |
| Hessen | [`de-he`](../profiles/de-he/profile.toml) | [FwDV 100 Führung und Leitung im Einsatz](https://hlfs.hessen.de/sites/hlfs.hessen.de/files/2022-09/FwDV%20100%20-%20Homepage.pdf), Stand September 1999 — copy of the Hessische Landesfeuerwehrschule | — |
| Mecklenburg-Vorpommern | [`de-mv`](../profiles/de-mv/profile.toml) | [FwDV 100](https://www.brand-kats-mv.de/static/BKS/Dateien/PDF/Aus-%20und%20Fortbildung/FwDV%20100.pdf) — copy of the Landesschule für Brand- und Katastrophenschutz | — |
| Niedersachsen | [`de-ni`](../profiles/de-ni/profile.toml) | [FwDV 100 (10 March 1999), introduced by decree, published in the download area of the Niedersächsische Feuerwehrschulen](https://www.feuerwehrschulen.niedersachsen.de/servlets/download?C=52170702&L=20), 10 March 1999 | — |
| Nordrhein-Westfalen | [`de-nw`](../profiles/de-nw/profile.toml) | [FwDV 100, introduced by Runderlass of the Ministry of the Interior of 23 December 1999](https://recht.nrw.de/lrmb/verwaltungsvorschrift/feuerwehr-dienstvorschrift-100-fwdv-100-fuehrung-und-leitung-im-einsatz/), Stand 10 March 1999 — copy also at the Institut der Feuerwehr | — |
| Rheinland-Pfalz | [`de-rp`](../profiles/de-rp/profile.toml) | [DV 100 Führung und Leitung im Einsatz (RP)](https://lfbk.rlp.de/fileadmin/lfbk/Inhaltsdateien/DV100.pdf), Stand 2000, introduced by letter of the Ministry of the Interior of 15 November 2000 — an extended FwDV 100 for the whole fire and disaster protection service | [Stabsarbeit: 4fach-Vordruck (message form) and Infofluss-Tischvorlage](https://lfbk.rlp.de/downloads/sonstiges), 2025-11-17; [Gefahrstoffkonzept RLP](https://lfbk.rlp.de/fileadmin/lfbk/Inhaltsdateien/Gefahrstoffkonzept_April05a.pdf), April 2005; [Downloads of the Landesfeuerwehr- und Katastrophenschutzakademie (LFKA/LFBK)](https://lfbk.rlp.de/downloads) |
| Saarland | [`de-sl`](../profiles/de-sl/profile.toml) | [FwDV 100](https://www.saarland.de/mibs/DE/institution/landesfeuerwehrschule/service/downloads/FW_Dienstvorschrift_100), Stand March 1999 with amendments 2005 | — |
| Sachsen | [`de-sn`](../profiles/de-sn/profile.toml) | [FwDV 100](https://www.lfs.sachsen.de/download/fwdv100.pdf), Stand March 1999 — copy of the Landesfeuerwehr- und Katastrophenschutzschule | — |
| Sachsen-Anhalt | [`de-st`](../profiles/de-st/profile.toml) | [DV 100 Führung und Leitung im Einsatz – Führungsdienstvorschrift für den Feuerwehrdienst sowie für die Bewältigung von Großschadenslagen und Katastrophen](https://umwelt-online.de/recht/anlasi/sicher/lsa/erlfwdv23.htm), Stand 2011; introduced by ERL DV of 3 January 2023 (MBl. LSA 2023 p. 27), valid until 31 December 2027 — the decree also introduces FwDV/DV 800 (2017) and 810 (2018); secondary copy of the decree | — |
| Schleswig-Holstein | [`de-sh`](../profiles/de-sh/profile.toml) | [FwDV 100; Einführungserlass for the fire service regulations IV 333 – 260-296/2018](https://lfs-sh.de/Inhalte/Vorschriften/Dienstvorschriften.php), FwDV 100 March 1999; decree March 2021 | — |
| Thüringen | [`de-th`](../profiles/de-th/profile.toml) | *not found yet* | — |

Most links are copies hosted by the state's fire service school; where only a secondary
copy was found, the note says so. Editions differ: most states use FwDV 100 of March 1999
(some with the 2005 amendments); Rhineland-Palatinate introduced an extended DV 100 (2000),
Saxony-Anhalt a DV 100 of 2011.

## Open

- Berlin, Hamburg and Thuringia: introducing decree or official copy not found yet.
- Bremen: only a training document of the Feuerwehr Bremen was found.
- Which states have introduced FwDV 102 / DV 102 (tactical symbols), and since when.
- Per state: status codes (where they differ from 1–9), mass-casualty (MANV) concepts and
  the medical command structure (Sanitätseinsatzleitung), which ORDO needs first.

Corrections and missing documents are welcome; please give the official source.
