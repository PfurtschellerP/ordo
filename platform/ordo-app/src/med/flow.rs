//! The patient's way as a flow chart (PAT-5, ADR-0087): the stations the patient was at
//! (the crew of a unit, a UHS, outside ORDO, the discharge) as boxes, and between them
//! arrows that say how the patient got there and when. Shown on the record's overview and
//! on its *Übergabe* tab.

use leptos::prelude::*;
use ordo_med_patients::transport::TransportStatus;
use ordo_med_patients::{Holder, PatientRegister, Stay};

use crate::core::{self, Handle};
use crate::med::text;
use crate::shell::ui::Icon;

/// What a station is, for its icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Uhs,
    Unit,
    /// Left the incident: discharged, or handed over outside ORDO.
    Left,
}

impl Kind {
    fn of(holder: &Holder) -> Self {
        match holder {
            Holder::Uhs(_) => Self::Uhs,
            Holder::Unit(_) => Self::Unit,
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Uhs => "tent",
            Self::Unit => "truck",
            Self::Left => "log-out",
        }
    }
}

/// How a station stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum State {
    Past,
    /// Where the patient is now.
    Now,
    /// Where an open handover goes: not taken yet.
    Open,
}

/// The arrow into a station: how the patient got there, and when.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Edge {
    pub what: String,
    pub time: String,
}

/// One box of the chart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Station {
    pub kind: Kind,
    pub title: String,
    /// What happened there, e.g. "Übernommen 14:12", a transport.
    pub lines: Vec<String>,
    pub edge: Option<Edge>,
    pub state: State,
}

/// How the chart names things: holders and times.
pub(crate) struct Names<'a> {
    pub holder: &'a dyn Fn(&Holder) -> String,
    pub clock: &'a dyn Fn(u64) -> String,
}

// [impl->dsn~patient-flow~1]
/// The stations of a patient's stay (`(time, change)`, oldest first), starting with the
/// holder they were registered with; `transports` (time, words) are lines of the station
/// the patient was at when the transport was requested.
pub(crate) fn stations(
    stay: &[(u64, &Stay)],
    first: &Holder,
    transports: &[(u64, String)],
    names: &Names,
) -> Vec<Station> {
    let mut out: Vec<Station> = Vec::new();
    let mut holder = first.clone();
    // The handover not taken yet: when, and to whom (none: quick).
    let mut pending: Option<(u64, Option<Holder>)> = None;
    let station = |holder: &Holder, edge: Option<Edge>, lines: Vec<String>| Station {
        kind: Kind::of(holder),
        title: (names.holder)(holder),
        lines,
        edge,
        state: State::Past,
    };
    let edge = |what: String, at: u64| {
        Some(Edge {
            what,
            time: (names.clock)(at),
        })
    };
    let mut transports = transports.iter().peekable();
    for (i, (at, change)) in stay.iter().enumerate() {
        // A transport requested before this change belongs to the station before it.
        while let (Some(last), Some((t, _))) = (out.last_mut(), transports.peek())
            && t <= at
        {
            last.lines.push(
                transports
                    .next()
                    .map(|(_, w)| w.clone())
                    .unwrap_or_default(),
            );
        }
        let time = (names.clock)(*at);
        match change {
            Stay::Admitted if i == 0 => {
                out.push(station(&holder, None, vec![text::flow_registered(&time)]));
            }
            Stay::Admitted => {
                out.push(station(
                    &holder,
                    edge(text::READMITTED_STEP.to_owned(), *at),
                    Vec::new(),
                ));
            }
            Stay::Moved { to, .. } => {
                holder = Holder::Uhs(to.clone());
                out.push(station(
                    &holder,
                    edge(text::FLOW_MOVED.to_owned(), *at),
                    Vec::new(),
                ));
            }
            Stay::HandedOver { to, .. } => pending = Some((*at, to.clone())),
            Stay::Taken { from, to } if from == to => {
                if let Some(last) = out.last_mut() {
                    last.lines.push(text::flow_taken_back(&time));
                }
                pending = None;
            }
            Stay::Taken { to, .. } => {
                let (since, quick) = match pending.take() {
                    Some((since, named)) => (since, named.is_none()),
                    None => (*at, false),
                };
                holder = to.clone();
                let what = if quick {
                    text::HANDOVER_QUICK_STEP
                } else {
                    text::FLOW_HANDOVER
                };
                out.push(station(
                    &holder,
                    edge(what.to_owned(), since),
                    vec![text::flow_taken(&time)],
                ));
            }
            Stay::Discharged { outcome, .. } => {
                pending = None;
                out.push(Station {
                    kind: Kind::Left,
                    title: text::outcome(*outcome).to_owned(),
                    lines: Vec::new(),
                    edge: edge(text::discharge().to_owned(), *at),
                    state: State::Past,
                });
            }
            Stay::HandedOutside { receiver, .. } => {
                pending = None;
                out.push(Station {
                    kind: Kind::Left,
                    title: receiver.clone(),
                    lines: vec![text::FLOW_OUTSIDE.to_owned()],
                    edge: edge(text::FLOW_HANDOVER.to_owned(), *at),
                    state: State::Past,
                });
            }
        }
    }
    if let Some(last) = out.last_mut() {
        last.lines.extend(transports.map(|(_, w)| w.clone()));
        if last.kind != Kind::Left {
            last.state = State::Now;
        }
    }
    if let Some((since, to)) = pending {
        out.push(Station {
            kind: to.as_ref().map_or(Kind::Uhs, Kind::of),
            title: to.as_ref().map_or_else(
                || text::FLOW_QUICK_RECEIVER.to_owned(),
                |h| (names.holder)(h),
            ),
            lines: vec![text::FLOW_NOT_TAKEN.to_owned()],
            edge: edge(text::FLOW_HANDOVER_OPEN.to_owned(), since),
            state: State::Open,
        });
    }
    out
}

/// The stations of the patient `tag`, named by this device.
pub(crate) fn patient_stations(
    core: Handle,
    tag: &str,
    register: &PatientRegister,
) -> Vec<Station> {
    let Some(patient) = register.patient(tag) else {
        return Vec::new();
    };
    let stay: Vec<(u64, &Stay)> = patient
        .stay
        .iter()
        .map(|e| (e.at.wall_ms, &e.change))
        .collect();
    let transports: Vec<(u64, String)> = crate::med::transports::patient_transports(core, tag)
        .into_iter()
        .filter(|t| t.status != TransportStatus::Cancelled)
        .map(|t| {
            (
                t.requested_at().wall_ms,
                text::flow_transport(
                    &crate::med::transports::transport_line(&t),
                    &crate::med::transports::state_label(&t),
                ),
            )
        })
        .collect();
    let holder = |h: &Holder| core::holder_name(core, h);
    let names = Names {
        holder: &holder,
        clock: &core::clock,
    };
    stations(
        &stay,
        &crate::med::record::first_holder(patient),
        &transports,
        &names,
    )
}

// [impl->dsn~patient-flow~1]
/// The chart: an ordered list, so a screen reader reads the way in order; each item says
/// how the patient got there, then where. The arrows are drawn, not read. `next` adds the
/// dashed "?" box for the handover still to choose.
#[component]
pub(crate) fn PatientFlow(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    register: Memo<PatientRegister>,
    #[prop(into, optional)] next: Signal<bool>,
) -> impl IntoView {
    let list = Memo::new(move |_| {
        revision.track();
        register.with(|r| patient_stations(core, &tag, r))
    });
    view! { <FlowChart list next /> }
}

// [impl->dsn~patient-flow~1]
/// The chart drawn from its stations; also shown in the component gallery.
#[component]
pub(crate) fn FlowChart(
    #[prop(into)] list: Signal<Vec<Station>>,
    #[prop(into)] next: Signal<bool>,
) -> impl IntoView {
    view! {
        <ol class="flow">
            {move || list.get().into_iter().map(|s| {
                let class = match s.state {
                    State::Past => "flow__step",
                    State::Now => "flow__step flow__step--now",
                    State::Open => "flow__step flow__step--open",
                };
                view! {
                    <li class=class>
                        {s.edge.map(|e| view! {
                            <span class="flow__edge">
                                <span class="flow__edge-what">{e.what}</span>
                                <span class="flow__edge-time">{e.time}</span>
                            </span>
                        })}
                        <span class="flow__box" class:flow__box--left=s.kind == Kind::Left>
                            <span class="flow__title"><Icon name=s.kind.icon() />{s.title}</span>
                            {s.lines.into_iter().map(|l| view! { <span class="flow__line">{l}</span> }).collect_view()}
                            {(s.state == State::Now).then(|| view! { <strong class="flow__now">{text::NOW}</strong> })}
                        </span>
                    </li>
                }
            }).collect_view()}
            <Show when=move || next.get()>
                <li class="flow__step flow__step--next">
                    <span class="flow__edge"><span class="flow__edge-what">{text::FLOW_NEXT_EDGE}</span></span>
                    <span class="flow__box"><span class="flow__title">"?"</span><span class="flow__line">{text::HANDOVER_NEXT}</span></span>
                </li>
            </Show>
        </ol>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> (impl Fn(&Holder) -> String, impl Fn(u64) -> String) {
        (
            |h: &Holder| match h {
                Holder::Uhs(id) => format!("UHS {id}"),
                Holder::Unit(u) => u.clone(),
            },
            |t: u64| format!("{t}:00"),
        )
    }

    // [utest->dsn~patient-flow~1]
    #[test]
    fn the_way_shows_each_holder_once_with_the_handover_on_its_arrow() {
        let (holder, clock) = names();
        let names = Names {
            holder: &holder,
            clock: &clock,
        };
        let crew = Holder::Unit("SanTrp 1".into());
        let nord = Holder::Uhs("nord".into());
        let changes = [
            Stay::Admitted,
            Stay::HandedOver {
                to: Some(nord.clone()),
                note: None,
            },
            Stay::Taken {
                from: crew.clone(),
                to: nord.clone(),
            },
            Stay::HandedOver {
                to: Some(Holder::Uhs("süd".into())),
                note: None,
            },
            Stay::Taken {
                from: nord.clone(),
                to: nord.clone(),
            },
            Stay::HandedOver {
                to: None,
                note: None,
            },
        ];
        let stay: Vec<(u64, &Stay)> = changes
            .iter()
            .enumerate()
            .map(|(i, c)| (10 + i as u64, c))
            .collect();
        let transports = [(13, "RTW 2 → Klinikum".to_owned())];
        let s = stations(&stay, &crew, &transports, &names);
        let titles: Vec<&str> = s.iter().map(|s| s.title.as_str()).collect();
        assert_eq!(titles, ["SanTrp 1", "UHS nord", text::FLOW_QUICK_RECEIVER]);
        assert_eq!(s[0].lines, [text::flow_registered("10:00")]);
        assert_eq!(s[0].state, State::Past);
        // The arrow carries the handover's time, the box the taking.
        assert_eq!(
            s[1].edge,
            Some(Edge {
                what: text::FLOW_HANDOVER.to_owned(),
                time: "11:00".into()
            })
        );
        assert_eq!(
            s[1].lines,
            [
                text::flow_taken("12:00"),
                "RTW 2 → Klinikum".to_owned(),
                text::flow_taken_back("14:00"),
            ]
        );
        assert_eq!(s[1].state, State::Now);
        assert_eq!(
            s[2].state,
            State::Open,
            "the quick handover is not taken yet"
        );
    }

    // [utest->dsn~patient-flow~1]
    #[test]
    fn a_patient_who_left_ends_in_a_box_that_is_not_now() {
        let (holder, clock) = names();
        let names = Names {
            holder: &holder,
            clock: &clock,
        };
        let changes = [
            Stay::Admitted,
            Stay::HandedOutside {
                receiver: "Klinikum Mitte".into(),
                note: None,
            },
        ];
        let stay: Vec<(u64, &Stay)> = changes
            .iter()
            .enumerate()
            .map(|(i, c)| (10 + i as u64, c))
            .collect();
        let s = stations(&stay, &Holder::Uhs("nord".into()), &[], &names);
        assert_eq!(s.len(), 2);
        assert_eq!(s[1].kind, Kind::Left);
        assert_eq!(s[1].title, "Klinikum Mitte");
        assert!(s.iter().all(|s| s.state == State::Past));
    }
}
