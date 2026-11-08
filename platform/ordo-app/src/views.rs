//! Views. Semantic HTML first (A11Y-4): real buttons, labels, fieldsets, tables, a skip
//! link and a live region; all colours and sizes come from design tokens.

use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::components::Router;
use leptos_router::hooks::{use_location, use_navigate};
use ordo_contracts::TriageCategory::{self, Deceased, Expectant, Immediate, Minor, Urgent};
use ordo_med_patients::{self as med, PatientRegister, UhsCreated};
use ordo_units::{self as units, UnitRegister};

use crate::core::{self, Connection, Core, Handle, Pairing};
use crate::{devices, nav, palette, patients, text};

/// Triage categories in display order.
pub(crate) const CATEGORIES: [TriageCategory; 5] = [Immediate, Urgent, Minor, Expectant, Deceased];

fn css_class(category: TriageCategory) -> &'static str {
    match category {
        Immediate => "triage triage--immediate",
        Urgent => "triage triage--urgent",
        Minor => "triage triage--minor",
        Expectant => "triage triage--expectant",
        Deceased => "triage triage--deceased",
    }
}

// [impl->dsn~triage-badge~1]
/// Colour plus label plus shape, never colour alone (A11Y-3).
#[component]
pub(crate) fn TriageBadge(category: TriageCategory) -> impl IntoView {
    view! {
        <span class=css_class(category) title=text::description(category)>
            <span class="triage__shape" aria-hidden="true"></span>
            <span aria-hidden="true">{text::label(category)}</span>
            <span class="visually-hidden">{text::description(category)}</span>
        </span>
    }
}

/// The on-site care units of the incident (from the shared compartment).
pub(crate) fn uhs_list(core: Handle) -> Vec<UhsCreated> {
    core.with_value(|c| {
        let events = c
            .replica
            .events(&c.incident, med::UHS_COMPARTMENT)
            .unwrap_or_default();
        med::uhs_list(&events)
    })
}

/// Pairing or workspace.
#[component]
pub fn App() -> impl IntoView {
    apply_theme(&core::preference(THEME_KEY).unwrap_or_default());
    // A login that returned to this device: finish pairing it (ADR-0038).
    let returned = core::url_param("paired")
        .and_then(|user| core::PendingDevice::take().map(|device| device.paired(&user)));
    let login_error = core::url_param("login_error");
    if returned.is_some() || login_error.is_some() {
        core::clear_url_params();
    }
    if let Some(pairing) = &returned {
        core::save_pairing(pairing);
    }
    let pairing = RwSignal::new(returned.or_else(core::load_pairing));
    view! {
        <Router>
            {move || match pairing.get() {
                None => view! { <PairingForm pairing login_error=login_error.clone() /> }.into_any(),
                Some(p) => view! { <Device pairing=p paired=pairing on_unpair=move || pairing.set(None) /> }.into_any(),
            }}
        </Router>
    }
}

// [impl->dsn~oidc-login~1]
// [impl->dsn~qr-pairing~1]
/// Pairing: log in with the organisation's account, or pair a shared tablet by QR code;
/// the device makes its own key, which never leaves it (ADR-0038). With `relogin`, the
/// device's login has expired: the person logs in again with the same device key, and the
/// data on the device stays (ADR-0040).
#[component]
fn PairingForm(
    pairing: RwSignal<Option<Pairing>>,
    login_error: Option<String>,
    #[prop(optional)] relogin: Option<Pairing>,
) -> impl IntoView {
    let relogin = StoredValue::new(relogin);
    let is_relogin = relogin.with_value(Option::is_some);
    let incident = RwSignal::new(
        relogin
            .with_value(|r| r.as_ref().map(|p| p.incident.clone()))
            .or_else(core::incident_from_url)
            .unwrap_or_default(),
    );
    let server = RwSignal::new(None::<String>);
    let login_enabled = RwSignal::new(None::<bool>);
    let error = RwSignal::new(login_error.map(|e| format!("{}: {e}", text::LOGIN_FAILED)));
    // (pairing ID, user code, address to open)
    let qr = RwSignal::new(None::<(String, String, String)>);
    leptos::task::spawn_local(async move {
        match core::fetch_json("GET", "/auth/config", None).await {
            Ok(config) => {
                server.set(config["server"].as_str().map(str::to_owned));
                login_enabled.set(Some(config["login"].as_bool().unwrap_or(false)));
            }
            Err(e) => {
                login_enabled.set(Some(false));
                error.set(Some(e));
            }
        }
    });
    // The incident to open: the one entered, or a new one on this server.
    let chosen_incident = move || {
        let entered = incident.get_untracked().trim().to_owned();
        if !entered.is_empty() {
            return Some(entered);
        }
        let id = core::new_incident_id(&server.get_untracked()?)?;
        incident.set(id.clone());
        Some(id)
    };
    let new_incident = move |_| {
        if let Some(id) = server.get().and_then(|s| core::new_incident_id(&s)) {
            incident.set(id);
        }
    };
    // The device to log in with: the existing one (log in again) or a new key pair.
    let device_for_login = move || match relogin.get_value() {
        Some(p) => Some(core::PendingDevice {
            device: p.device,
            private_key: p.private_key,
            incident: p.incident,
        }),
        None => chosen_incident().and_then(|i| core::PendingDevice::create(&i)),
    };
    let login = move |_| {
        let Some(device) = device_for_login() else {
            return;
        };
        let Some(key) = device.public_key() else {
            return;
        };
        device.save();
        if let Some(window) = web_sys::window() {
            let _ = window
                .location()
                .assign(&format!("/auth/login?device={}&key={key}", device.device));
        }
    };
    let start_qr = move |_| {
        let Some(device) = device_for_login() else {
            return;
        };
        let Some(key) = device.public_key() else {
            return;
        };
        error.set(None);
        leptos::task::spawn_local(async move {
            let body = serde_json::json!({ "device": device.device, "key": key });
            let started = match core::fetch_json("POST", "/auth/device", Some(&body)).await {
                Ok(started) => started,
                Err(e) => {
                    error.set(Some(e));
                    return;
                }
            };
            let field = |name: &str| started[name].as_str().unwrap_or_default().to_owned();
            let id = field("id");
            let uri = started["verification_uri_complete"]
                .as_str()
                .map_or_else(|| field("verification_uri"), str::to_owned);
            qr.set(Some((id.clone(), field("user_code"), uri)));
            loop {
                core::sleep_ms(2_000).await;
                if qr.get_untracked().is_none() {
                    return;
                }
                match core::fetch_json("GET", &format!("/auth/device/{id}"), None).await {
                    Ok(state) if state["status"] == "done" => {
                        let user = state["user"].as_str().unwrap_or_default();
                        let paired = device.clone().paired(user);
                        core::save_pairing(&paired);
                        pairing.set(Some(paired));
                        return;
                    }
                    Ok(state) if state["status"] == "failed" => {
                        error.set(state["error"].as_str().map(str::to_owned));
                        qr.set(None);
                        return;
                    }
                    _ => {}
                }
            }
        });
    };
    view! {
        <main id="main" class="page page--narrow">
            <h1>
                {text::APP_NAME} " – "
                {if is_relogin { text::RELOGIN_TITLE } else { text::PAIR_TITLE }}
            </h1>
            <p>{if is_relogin { text::RELOGIN_INTRO } else { text::PAIR_INTRO }}</p>
            {(!is_relogin).then(|| view! {
                <div class="form">
                    <label for="incident">{text::PAIR_INCIDENT}</label>
                    <div class="row">
                        <input id="incident" type="text" spellcheck="false" bind:value=incident />
                        <button type="button" class="button" on:click=new_incident>{text::PAIR_NEW_INCIDENT}</button>
                    </div>
                </div>
            })}
            <p role="alert" class="error">{move || error.get()}</p>
            {move || match login_enabled.get() {
                None => ().into_any(),
                Some(false) => view! { <p class="notice">{text::PAIR_NO_LOGIN}</p> }.into_any(),
                Some(true) => view! {
                    <div class="row">
                        <button id="login" type="button" class="button button--primary" on:click=login>{text::PAIR_LOGIN}</button>
                        <button id="pair-qr" type="button" class="button" on:click=start_qr>{text::PAIR_QR}</button>
                    </div>
                    <p class="hint">{text::PAIR_QR_HINT}</p>
                }.into_any(),
            }}
            {move || qr.get().map(|(id, code, uri)| view! {
                <section class="card pairing-qr" aria-labelledby="pair-qr-title">
                    <h2 id="pair-qr-title">{text::PAIR_QR}</h2>
                    <img class="pairing-qr__code" src=format!("/auth/device/{id}/qr.svg") alt=text::PAIR_QR_ALT width="256" height="256" />
                    <p class="pairing-qr__user-code code">{text::pair_code(&code)}</p>
                    <p>{text::PAIR_QR_LINK} " " <a id="pair-qr-link" href=uri.clone()>{uri.clone()}</a></p>
                    <p role="status">{text::PAIR_WAITING}</p>
                </section>
            })}
            {if is_relogin {
                view! {
                    <button type="button" class="button" on:click=move |_| {
                        core::forget_device();
                        pairing.set(None);
                    }>{text::UNPAIR}</button>
                }
                .into_any()
            } else {
                view! {
                    <details class="manual-pairing">
                        <summary>{text::PAIR_MANUAL}</summary>
                        <ManualPairing pairing incident />
                    </details>
                }
                .into_any()
            }}
        </main>
    }
}

/// Manual pairing with data from the seed file (development and tests without login).
#[component]
fn ManualPairing(pairing: RwSignal<Option<Pairing>>, incident: RwSignal<String>) -> impl IntoView {
    let user = RwSignal::new(String::new());
    let device = RwSignal::new(String::new());
    let key = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let candidate = Pairing {
            user: user.get(),
            device: device.get(),
            private_key: key.get(),
            incident: incident.get(),
        };
        match Core::new(&candidate) {
            Ok(_) => {
                core::save_pairing(&candidate);
                pairing.set(Some(candidate));
            }
            Err(e) => error.set(Some(e)),
        }
    };
    view! {
        <p>{text::PAIR_MANUAL_INTRO}</p>
        <form on:submit=submit class="form">
            <label for="user">{text::PAIR_USER}</label>
            <input id="user" type="text" autocomplete="username" required bind:value=user />
            <label for="device">{text::PAIR_DEVICE}</label>
            <input id="device" type="text" autocomplete="off" spellcheck="false" required bind:value=device />
            <label for="key">{text::PAIR_KEY}</label>
            <input id="key" type="password" autocomplete="off" spellcheck="false" required bind:value=key />
            <p role="alert" class="error">{move || error.get()}</p>
            <button type="submit" class="button">{text::PAIR_SUBMIT}</button>
        </form>
    }
}

/// Whether the user may open `workspace` on this device, and if not, why not.
pub(crate) fn access(core: Handle, workspace: &nav::Workspace) -> Result<(), &'static str> {
    core.with_value(|c| {
        if !c.replica.is_ready() {
            return Err(text::REASON_NOT_CONNECTED);
        }
        if workspace
            .requires
            .iter()
            .all(|comp| c.replica.is_granted(&c.incident, comp))
        {
            Ok(())
        } else {
            Err(text::REASON_NO_ACCESS)
        }
    })
}

const NAV_KEY: &str = "ordo.nav";
const THEME_KEY: &str = "ordo.theme";

/// Applies a colour theme (`""` = follow the system) and remembers it on this device.
pub(crate) fn apply_theme(theme: &str) {
    if let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ = if theme.is_empty() {
            root.remove_attribute("data-theme")
        } else {
            root.set_attribute("data-theme", theme)
        };
    }
    core::set_preference(THEME_KEY, (!theme.is_empty()).then_some(theme));
}

/// Programmatic navigation for all views; the route itself is the URL path, managed by
/// `leptos_router` (history, back button, link handling).
#[derive(Clone, Copy)]
pub(crate) struct Nav {
    navigate: StoredValue<Navigate, LocalStorage>,
}

/// The router's navigation function.
type Navigate = Box<dyn Fn(&str, NavigateOptions)>;

impl Nav {
    fn new() -> Self {
        let navigate = use_navigate();
        Self {
            navigate: StoredValue::new_local(Box::new(navigate)),
        }
    }

    /// Goes to `route` and adds it to the browser history.
    pub fn go(self, route: nav::Route) {
        self.navigate
            .with_value(|n| n(&route.to_path(), NavigateOptions::default()));
    }

    /// Goes to `route`, replacing the current history entry.
    fn replace(self, route: nav::Route) {
        self.navigate.with_value(|n| {
            n(
                &route.to_path(),
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            )
        });
    }
}

/// Runs `f` once the view has updated after a navigation.
pub(crate) fn after_render(f: impl FnOnce() + 'static) {
    set_timeout(f, std::time::Duration::from_millis(60));
}

// [impl->dsn~workspace-navigation~2]
/// Layout: header with title, search and connection; navigation as a rail (wide
/// screens) or a bottom bar (phones, tablets in portrait), holding the views and the menu
/// (launcher) for switching workspaces.
// [impl->dsn~access-timeout~1]
/// The paired device: its state and connection, and either the workspace or, when the
/// device's login has expired, the lock screen to log in again (ADR-0040). The device locks
/// itself also offline, from the end of the login the server announced.
#[component]
fn Device(
    pairing: Pairing,
    paired: RwSignal<Option<Pairing>>,
    on_unpair: impl Fn() + Clone + Send + Sync + 'static,
) -> impl IntoView {
    let core = match Core::new(&pairing) {
        Ok(core) => StoredValue::new_local(core),
        Err(error) => return view! { <p role="alert" class="error">{error}</p> }.into_any(),
    };
    let revision = RwSignal::new(0u64);
    core::connect(core, revision);
    on_cleanup(move || core::disconnect(core));
    // Check the end of the login every minute, also without connection.
    if let Ok(tick) = set_interval_with_handle(
        move || revision.update(|r| *r += 1),
        std::time::Duration::from_secs(60),
    ) {
        on_cleanup(move || tick.clear());
    }
    let locked = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.login_expired || c.replica.login_expired(core::now_ms()))
    });
    let revoked = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.revoked)
    });
    let on_unpair = StoredValue::new(on_unpair);
    let unpair = move || on_unpair.with_value(|f| f());
    let relogin = StoredValue::new(pairing);
    view! {
        <Show
            when=move || !revoked.get()
            fallback=move || view! { <Revoked on_pair_again=unpair /> }
        >
            <Show
                when=move || !locked.get()
                fallback=move || view! { <PairingForm pairing=paired login_error=None relogin=relogin.get_value() /> }
            >
                <Workspace core revision on_unpair=unpair />
            </Show>
        </Show>
    }
    .into_any()
}

// [impl->dsn~device-revocation~1]
/// Shown after the server reported this device as revoked: its key and data are gone.
#[component]
fn Revoked(on_pair_again: impl Fn() + 'static) -> impl IntoView {
    after_render(|| focus("revoked-title"));
    view! {
        <main id="main" class="page page--narrow" tabindex="-1">
            <section class="card" aria-labelledby="revoked-title">
                <h1 id="revoked-title" tabindex="-1">{text::REVOKED_TITLE}</h1>
                <p role="alert" class="notice notice--warning">{text::REVOKED_TEXT}</p>
                <button type="button" class="button button--primary" on:click=move |_| on_pair_again()>
                    {text::PAIR_AGAIN}
                </button>
            </section>
        </main>
    }
}

#[component]
fn Workspace(
    core: Handle,
    revision: RwSignal<u64>,
    on_unpair: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let nav = Nav::new();
    provide_context(patients::Drafts::default());
    let open = RwSignal::new(false);
    let palette = RwSignal::new(false);
    let pathname = use_location().pathname;
    let route = Memo::new(move |_| nav::Route::from_path(&pathname.get()));
    let saved = core::preference(NAV_KEY).and_then(|s| {
        let (w, v) = s.split_once('/')?;
        Some(nav::Route::view(w, v))
    });

    // The shown workspace: the one in the URL if accessible, else the last used one, else
    // the first accessible one.
    let current = Memo::new(move |_| {
        revision.track();
        let usable = |id: &str| nav::workspace(id).filter(|w| access(core, w).is_ok());
        route
            .with(|r| r.as_ref().and_then(|r| usable(&r.workspace)))
            .or_else(|| saved.as_ref().and_then(|r| usable(&r.workspace)))
            .or_else(|| nav::WORKSPACES.iter().find(|w| access(core, w).is_ok()))
            .map(|w| w.id)
    });
    let current_view = Memo::new(move |_| {
        let workspace = nav::workspace(current.get()?)?;
        let id = route.with(|r| r.as_ref().map(|r| r.view.clone()));
        workspace
            .views
            .iter()
            .find(|v| Some(v.id) == id.as_deref())
            .or_else(|| workspace.views.first())
            .map(|v| v.id)
    });
    let record = Memo::new(move |_| route.with(|r| r.as_ref().and_then(|r| r.record.clone())));
    // Keep the URL on what is shown (e.g. `/` or a workspace without access), and remember
    // the last workspace and view on this device.
    Effect::new(move |_| {
        let (Some(w), Some(v)) = (current.get(), current_view.get()) else {
            return;
        };
        let shown = route.with(|r| r.as_ref().is_some_and(|r| r.workspace == w && r.view == v));
        if shown {
            core::set_preference(NAV_KEY, Some(&format!("{w}/{v}")));
        } else {
            nav.replace(nav::Route::view(w, v));
        }
    });
    let choose = move |w: &'static nav::Workspace| {
        open.set(false);
        nav.go(nav::Route::view(w.id, w.views[0].id));
        focus("main");
    };
    let incident = core.with_value(|c| c.incident.to_string());
    // Unpairing deletes the key on this device; if the server can be reached, it forgets
    // the device as well (SEC-4), so it no longer appears in the device list.
    let unpair = move |_| {
        let device = core.with_value(|c| c.replica.device());
        core::revoke_device(core, device);
        core::forget_device();
        on_unpair();
    };
    let title = move || match current.get().and_then(nav::workspace) {
        Some(w) => format!("{} – {}", w.package, w.label),
        None => text::APP_NAME.to_owned(),
    };
    let mac = web_sys::window()
        .and_then(|w| w.navigator().platform().ok())
        .is_some_and(|p| p.starts_with("Mac") || p.starts_with("iP"));
    let shortcut = if mac {
        palette::SHORTCUT_MAC
    } else {
        palette::SHORTCUT
    };
    // Ctrl+K / ⌘K opens and closes the command palette from anywhere (UX-10).
    let keys = window_event_listener(leptos::ev::keydown, move |ev| {
        if (ev.ctrl_key() || ev.meta_key()) && !ev.alt_key() && ev.key().eq_ignore_ascii_case("k") {
            ev.prevent_default();
            open.set(false);
            palette.update(|p| *p = !*p);
        }
    });
    on_cleanup(move || keys.remove());
    let theme = RwSignal::new(core::preference(THEME_KEY).unwrap_or_default());

    view! {
        <a class="skip-link" href="#main" on:click=move |ev| {
            ev.prevent_default();
            focus("main");
        }>{text::SKIP_LINK}</a>
        <header class="app-header">
            <h1 class="app-header__title">{title}</h1>
            <button
                id="palette-button"
                type="button"
                class="search-button"
                aria-keyshortcuts="Control+K Meta+K"
                on:click=move |_| palette.set(true)
            >
                <span class="search-button__icon" aria-hidden="true"></span>
                <span>{text::SEARCH}</span>
                <kbd class="search-button__keys" aria-hidden="true">{shortcut}</kbd>
            </button>
            <div class="app-header__state">
                <p class="app-header__incident">
                    <span>{text::INCIDENT} ": "</span>
                    <span class="code" title=incident.clone()>{incident.chars().take(10).collect::<String>()} "…"</span>
                </p>
                <ConnectionStatus core revision />
            </div>
        </header>
        <div class="layout">
            <div class="navigation">
              <div class="navigation__inner">
                <div class="launcher" on:keydown=move |ev| {
                    if ev.key() == "Escape" && open.get() {
                        open.set(false);
                        focus("launcher-button");
                    }
                }>
                    <button
                        id="launcher-button"
                        type="button"
                        class="launcher__button"
                        aria-expanded=move || open.get().to_string()
                        aria-controls="launcher-panel"
                        on:click=move |_| open.update(|o| *o = !*o)
                    >
                        <span class="launcher__icon" aria-hidden="true"></span>
                        <span>{text::MENU}</span>
                        <span class="visually-hidden">" – " {text::LAUNCHER}</span>
                    </button>
                    <div id="launcher-panel" class="launcher__panel" hidden=move || !open.get()>
                        <h2 class="launcher__title">{text::LAUNCHER}</h2>
                        {nav::packages()
                            .into_iter()
                            .map(|(package, workspaces)| {
                                view! {
                                    <section class="launcher__package" aria-label=package>
                                        <h3>{package}</h3>
                                        <ul>
                                            {workspaces
                                                .into_iter()
                                                .map(|w| {
                                                    let reason_id = format!("reason-{}", w.id.replace('.', "-"));
                                                    let reason = move || {
                                                        revision.track();
                                                        access(core, w).err()
                                                    };
                                                    view! {
                                                        <li>
                                                            <button
                                                                type="button"
                                                                class="launcher__item"
                                                                aria-disabled=move || reason().is_some().to_string()
                                                                aria-current=move || (current.get() == Some(w.id)).then_some("true")
                                                                aria-describedby=reason_id.clone()
                                                                on:click=move |_| {
                                                                    if reason().is_none() {
                                                                        choose(w);
                                                                    }
                                                                }
                                                            >
                                                                {w.label}
                                                            </button>
                                                            <span id=reason_id class="launcher__reason">{reason}</span>
                                                        </li>
                                                    }
                                                })
                                                .collect_view()}
                                        </ul>
                                    </section>
                                }
                            })
                            .collect_view()}
                        <section class="launcher__package" aria-label=text::DEVICE>
                            <h3>{text::DEVICE}</h3>
                            <label for="theme">{text::APPEARANCE}</label>
                            <select id="theme" prop:value=move || theme.get() on:change=move |ev| {
                                let value = event_target_value(&ev);
                                apply_theme(&value);
                                theme.set(value);
                            }>
                                {text::THEMES.iter().map(|(id, label)| view! { <option value=*id>{*label}</option> }).collect_view()}
                            </select>
                            <button type="button" class="button" on:click=unpair.clone()>{text::UNPAIR}</button>
                        </section>
                    </div>
                </div>
                <nav class="views" aria-label=text::VIEWS>
                    {move || {
                        current
                            .get()
                            .and_then(nav::workspace)
                            .map(|w| {
                                w.views
                                    .iter()
                                    .map(|v| {
                                        let selected = move || current_view.get() == Some(v.id);
                                        view! {
                                            <a
                                                class="views__item"
                                                href=nav::Route::view(w.id, v.id).to_path()
                                                aria-current=move || selected().then_some("page")
                                            >
                                                {v.label}
                                            </a>
                                        }
                                    })
                                    .collect_view()
                            })
                    }}
                </nav>
              </div>
            </div>
            <main id="main" class="page" tabindex="-1">
                {move || match (current.get(), current_view.get()) {
                    (Some("med.uhs"), Some("uhs")) => view! { <UhsSection core revision /> }.into_any(),
                    (Some("med.uhs"), _) => {
                        view! { <patients::PatientsView core revision nav record=Signal::from(record) /> }.into_any()
                    }
                    (Some("command.situation"), Some("units")) => view! { <ResourcesWorkspace core revision /> }.into_any(),
                    (Some("command.situation"), _) => view! { <CommandWorkspace core revision /> }.into_any(),
                    (Some("org.devices"), _) => view! { <devices::DevicesView core revision /> }.into_any(),
                    _ => view! { <p class="notice">{text::NO_WORKSPACE}</p> }.into_any(),
                }}
            </main>
        </div>
        <palette::Palette core revision nav open=palette theme />
        <p class="visually-hidden" role="status" aria-live="polite">
            {move || {
                revision.track();
                core.with_value(|c| c.announcement.clone())
            }}
        </p>
    }
    .into_any()
}

/// Moves keyboard focus to the element with `id` (after navigation).
pub(crate) fn focus(id: &str) {
    use wasm_bindgen::JsCast;
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(id))
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = el.focus();
    }
}

/// Clicks the element with `id` (e.g. to open an editor after navigation).
pub(crate) fn click(id: &str) {
    use wasm_bindgen::JsCast;
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(id))
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        el.click();
    }
}

/// Connection and outbox state, as text (OFF-5).
#[component]
fn ConnectionStatus(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let state = move || {
        revision.track();
        core.with_value(|c| {
            (
                c.connection,
                c.replica.outbox_len(),
                c.error.clone(),
                c.storage_failed,
            )
        })
    };
    view! {
        <div class="status" data-state=move || match state().0 {
            Connection::Online => "online",
            Connection::Connecting => "connecting",
            Connection::Offline => "offline",
        }>
            <span class="status__dot" aria-hidden="true"></span>
            <span>
                {move || match state().0 {
                    Connection::Online => text::ONLINE,
                    Connection::Connecting => text::CONNECTING,
                    Connection::Offline => text::OFFLINE,
                }}
            </span>
            <span class="status__pending">{move || text::pending(state().1)}</span>
            {move || state().2.map(|e| view! { <span role="alert" class="error">{e}</span> })}
            {move || state().3.then(|| view! { <span role="alert" class="error">{text::STORAGE_FAILED}</span> })}
        </div>
    }
}

// [impl->dsn~uhs-registry~1]
/// On-site care units with their load, and a form to set one up (UHS-1, UHS-4).
#[component]
fn UhsSection(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let capacity = RwSignal::new("10".to_owned());
    let error = RwSignal::new(None::<String>);
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let (n, cap) = (
            name.get().trim().to_owned(),
            capacity.get().trim().parse::<u32>(),
        );
        match cap {
            Ok(cap) if !n.is_empty() && cap > 0 => {
                match core::create_uhs(core, revision, &n, cap) {
                    Ok(()) => {
                        name.set(String::new());
                        error.set(None);
                    }
                    Err(e) => error.set(Some(e)),
                }
            }
            _ => error.set(Some(format!("{}: > 0", text::UHS_CAPACITY))),
        }
    };
    let rows = move || {
        revision.track();
        let counts = core.with_value(|c| {
            let events = c
                .replica
                .events(&c.incident, med::COMPARTMENT)
                .unwrap_or_default();
            PatientRegister::from_events(&events).counts()
        });
        uhs_list(core)
            .into_iter()
            .map(|u| {
                let n = counts
                    .get(&u.id)
                    .map_or(0, |c| CATEGORIES.iter().map(|k| c.get(*k)).sum());
                (u.name, n, u.capacity)
            })
            .collect::<Vec<_>>()
    };
    view! {
        <section aria-labelledby="uhs-title" class="card">
            <h2 id="uhs-title">{text::UHS_TITLE}</h2>
            {move || {
                let rows = rows();
                if rows.is_empty() {
                    return view! { <p>{text::UHS_EMPTY}</p> }.into_any();
                }
                view! {
                    <ul class="load-list">
                        {rows
                            .into_iter()
                            .map(|(name, n, cap)| {
                                view! {
                                    <li class="load-list__item">
                                        <span class="load-list__name">{name}</span>
                                        <span>{text::COL_LOAD} ": " {text::load(n, cap)}</span>
                                        <meter min="0" max=cap value=n aria-hidden="true"></meter>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
            <form on:submit=submit class="form form--inline">
                <div class="field">
                    <label for="uhs-name">{text::UHS_NAME}</label>
                    <input id="uhs-name" type="text" required bind:value=name />
                </div>
                <div class="field field--small">
                    <label for="uhs-capacity">{text::UHS_CAPACITY}</label>
                    <input id="uhs-capacity" type="text" inputmode="numeric" pattern="[0-9]+" required bind:value=capacity />
                </div>
                <button type="submit" class="button">{text::UHS_SUBMIT}</button>
            </form>
            <p role="alert" class="error">{move || error.get()}</p>
        </section>
    }
}

#[component]
fn CommandWorkspace(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let rows = Memo::new(move |_| {
        revision.track();
        let list = uhs_list(core);
        let counts = core.with_value(|c| {
            let events = c
                .replica
                .events(&c.incident, "incident")
                .unwrap_or_default();
            med::latest_counts(&events)
        });
        list.into_iter()
            .map(|u| {
                (
                    u.name,
                    u.capacity,
                    counts.get(&u.id).cloned().unwrap_or_default(),
                )
            })
            .collect::<Vec<_>>()
    });
    view! {
        <section aria-labelledby="counts-title" class="card">
            <h2 id="counts-title">{text::COUNTS_TITLE}</h2>
            {move || {
                let rows = rows.get();
                if rows.is_empty() {
                    return view! { <p>{text::COUNTS_EMPTY}</p> }.into_any();
                }
                // Narrow screens: one block per UHS instead of a wide table (reflow).
                let list = rows
                    .iter()
                    .map(|(name, capacity, c)| {
                        let total: u32 = CATEGORIES.iter().map(|k| c.get(*k)).sum();
                        view! {
                            <li class="count-card">
                                <h3>{name.clone()}</h3>
                                <ul class="count-card__counts">
                                    {CATEGORIES
                                        .into_iter()
                                        .map(|k| view! { <li><TriageBadge category=k /> <span class="count-card__n">{c.get(k)}</span></li> })
                                        .collect_view()}
                                </ul>
                                <p>{text::COL_TOTAL} ": " <strong>{total}</strong> " · " {text::COL_LOAD} ": " {text::load(total, *capacity)}</p>
                            </li>
                        }
                    })
                    .collect_view();
                view! {
                    <ul class="count-list only-narrow">{list}</ul>
                    <table class="table only-wide">
                        <thead>
                            <tr>
                                <th scope="col">{text::COL_UHS}</th>
                                {CATEGORIES.into_iter().map(|c| view! { <th scope="col" class="number"><TriageBadge category=c /></th> }).collect_view()}
                                <th scope="col" class="number">{text::COL_TOTAL}</th>
                                <th scope="col" class="number">{text::COL_LOAD}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {rows
                                .into_iter()
                                .map(|(name, capacity, c)| {
                                    let total: u32 = CATEGORIES.iter().map(|k| c.get(*k)).sum();
                                    view! {
                                        <tr>
                                            <th scope="row">{name}</th>
                                            {CATEGORIES.into_iter().map(|k| view! { <td class="number">{c.get(k)}</td> }).collect_view()}
                                            <td class="number total">{total}</td>
                                            <td class="number">{text::load(total, capacity)}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                }
                .into_any()
            }}
        </section>
    }
}

/// Units (vehicles, teams, other resources) with status (UNIT-1…3).
#[component]
fn ResourcesWorkspace(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let call_sign = RwSignal::new(String::new());
    let unit_type = RwSignal::new(String::new());
    let organisation = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let (u, t, o) = (call_sign.get(), unit_type.get(), organisation.get());
        if u.trim().is_empty() || t.trim().is_empty() {
            return;
        }
        match core::add_unit(core, revision, u.trim(), t.trim(), o.trim()) {
            Ok(()) => {
                call_sign.set(String::new());
                error.set(None);
            }
            Err(e) => error.set(Some(e)),
        }
    };
    let register = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            let events = c
                .replica
                .events(&c.incident, units::COMPARTMENT)
                .unwrap_or_default();
            UnitRegister::from_events(&events)
        })
    });
    view! {
        <section aria-labelledby="units-title" class="card">
            <h2 id="units-title">{text::UNITS_TITLE}</h2>
            {move || {
                register
                    .get()
                    .lost_updates
                    .into_iter()
                    .map(|l| view! { <p class="notice notice--warning">{text::lost_update(&l.unit, &l.overwritten, &l.kept)}</p> })
                    .collect_view()
            }}
            {move || {
                let units = register.get().units;
                if units.is_empty() {
                    return view! { <p>{text::UNITS_EMPTY}</p> }.into_any();
                }
                view! {
                    <ul class="unit-list">
                        {units
                            .into_values()
                            .map(|u| {
                                let unit = u.unit.clone();
                                let current = u.status.as_ref().map(|s| s.status.clone()).unwrap_or_default();
                                let shown = if current.is_empty() { text::NO_STATUS.to_owned() } else { text::status_label(&current) };
                                let details = [u.unit_type.as_str(), u.organisation.as_str()]
                                    .into_iter()
                                    .filter(|t| !t.is_empty())
                                    .collect::<Vec<_>>()
                                    .join(" · ");
                                view! {
                                    <li class="unit-card" data-unit=u.unit.clone()>
                                        <h3 class="unit-card__name">{u.unit.clone()}</h3>
                                        <p class="unit-card__details">{details}</p>
                                        <p class="unit-card__status">{text::COL_STATUS} ": " <strong>{shown}</strong></p>
                                        <select
                                            aria-label=text::set_status_label(&unit)
                                            on:change=move |ev| {
                                                let value = event_target_value(&ev);
                                                if !value.is_empty() {
                                                    let _ = core::set_status(core, revision, &unit, &value);
                                                }
                                            }
                                        >
                                            <option value="" selected=current.is_empty()>{text::COL_SET_STATUS}</option>
                                            {text::STATUS_CODES
                                                .iter()
                                                .map(|(code, _)| view! { <option value=*code selected=*code == current>{text::status_label(code)}</option> })
                                                .collect_view()}
                                        </select>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
            <form on:submit=submit class="form form--inline">
                <div class="field">
                    <label for="unit-call-sign">{text::UNIT_CALL_SIGN}</label>
                    <input id="unit-call-sign" type="text" required bind:value=call_sign />
                </div>
                <div class="field">
                    <label for="unit-type">{text::UNIT_TYPE}</label>
                    <input id="unit-type" type="text" required bind:value=unit_type />
                </div>
                <div class="field">
                    <label for="unit-organisation">{text::UNIT_ORGANISATION}</label>
                    <input id="unit-organisation" type="text" bind:value=organisation />
                </div>
                <button type="submit" class="button">{text::UNIT_SUBMIT}</button>
            </form>
            <p role="alert" class="error">{move || error.get()}</p>
        </section>
    }
}
