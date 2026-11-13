//! Views. Semantic HTML first (A11Y-4): real buttons, labels, fieldsets, tables, a skip
//! link and a live region; all colours and sizes come from design tokens.

use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::components::Router;
use leptos_router::hooks::{use_location, use_navigate};
use ordo_contracts::TriageCategory::{
    self, Deceased, Expectant, Immediate, Minor, Uninjured, Urgent,
};
use ordo_med_patients::{self as med, PatientRegister, UhsCreated};

use crate::core::{self, Connection, Core, Handle, Pairing};
use crate::ui::{Disclosure, DisclosureButton, Icon, SettingSelect};
use crate::{devices, incidents, nav, palette, patients, text};

/// Triage categories in display order.
pub(crate) const CATEGORIES: [TriageCategory; 6] =
    [Immediate, Urgent, Minor, Expectant, Deceased, Uninjured];

fn css_class(category: TriageCategory) -> &'static str {
    match category {
        Immediate => "triage triage--immediate",
        Urgent => "triage triage--urgent",
        Minor => "triage triage--minor",
        Expectant => "triage triage--expectant",
        Deceased => "triage triage--deceased",
        Uninjured => "triage triage--uninjured",
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
        let events = c.events(med::UHS_COMPARTMENT);
        med::uhs_list(&events)
    })
}

/// Pairing or workspace.
/// The app: opens the encrypted device storage first (ADR-0041), then starts.
#[component]
pub fn App() -> impl IntoView {
    apply_theme(&core::preference(THEME_KEY).unwrap_or_default());
    apply_hand(&core::preference(HAND_KEY).unwrap_or_default());
    let vault = RwSignal::new(None::<Result<(), String>>);
    leptos::task::spawn_local(async move {
        let opened = crate::vault::open().await;
        // Which storage the device uses, for support and tests (`data-storage` on <html>).
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.document_element());
        if opened.is_ok() {
            match crate::storage::open().await {
                Ok(()) => {
                    crate::attachments::load().await;
                    if let Some(root) = &root {
                        let _ = root.set_attribute("data-storage", "opfs");
                    }
                }
                Err(e) => {
                    // Without OPFS in workers the device keeps its data in `localStorage`.
                    web_sys::console::warn_1(&format!("ORDO device storage: {e}").into());
                    if let Some(root) = &root {
                        let _ = root.set_attribute("data-storage", "local");
                        let _ = root.set_attribute("data-storage-error", &e);
                    }
                }
            }
        }
        vault.set(Some(opened));
    });
    move || match vault.get() {
        None => view! { <main id="main" class="page page--narrow"><p>{text::STARTING}</p></main> }
            .into_any(),
        Some(Err(error)) => view! {
            <main id="main" class="page page--narrow">
                <h1>{text::APP_NAME}</h1>
                <p role="alert" class="error">{error}</p>
                <button type="button" class="button" on:click=|_| {
                    if let Some(w) = web_sys::window() {
                        let _ = w.location().reload();
                    }
                }>{text::RETRY}</button>
            </main>
        }
        .into_any(),
        Some(Ok(())) => view! { <Start /> }.into_any(),
    }
}

/// Pairing or the paired device.
#[component]
fn Start() -> impl IntoView {
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
            <Screens pairing login_error />
        </Router>
    }
}

// [impl->dsn~licence-notices~2]
/// The screen for the URL: the licence notices, pairing or the paired device. Pairing links
/// to the notices; a paired device has them under *Administration – Info* (LIC-1).
#[component]
fn Screens(pairing: RwSignal<Option<Pairing>>, login_error: Option<String>) -> impl IntoView {
    let location = use_location();
    let pathname = location.pathname;
    // A memo, so moving between views does not build the device's screen anew.
    let on_notices = Memo::new(move |_| pathname.get() == crate::notices::PATH);
    // The screen before the notices, to return to.
    let previous = StoredValue::new("/".to_owned());
    Effect::new(move || {
        let path = pathname.get();
        if path != crate::notices::PATH {
            previous.set_value(format!("{path}{}", location.search.get()));
        }
    });
    view! {
        {move || {
            if on_notices.get() {
                return view! { <crate::notices::NoticesPage back=previous.get_value() /> }.into_any();
            }
            match pairing.get() {
                None => view! { <PairingForm pairing login_error=login_error.clone() /> }.into_any(),
                Some(p) => view! { <Device pairing=p paired=pairing on_unpair=move || pairing.set(None) /> }.into_any(),
            }
        }}
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
    let login_enabled = RwSignal::new(None::<bool>);
    let error = RwSignal::new(login_error.map(|e| format!("{}: {e}", text::LOGIN_FAILED)));
    // (pairing ID, user code, address to open)
    let qr = RwSignal::new(None::<(String, String, String)>);
    leptos::task::spawn_local(async move {
        match core::fetch_json("GET", "/auth/config", None).await {
            Ok(config) => {
                login_enabled.set(Some(config["login"].as_bool().unwrap_or(false)));
            }
            Err(e) => {
                login_enabled.set(Some(false));
                error.set(Some(e));
            }
        }
    });
    // [impl->dsn~device-names~1]
    // The device's name (ORG-8), suggested from the browser; logging in again keeps the
    // stored name, so none is sent then.
    let device_name = RwSignal::new(core::suggested_device_name());
    let name_param = move || {
        (!is_relogin)
            .then(|| device_name.get_untracked().trim().to_owned())
            .filter(|n| !n.is_empty())
    };
    // The device to log in with: the existing one (log in again) or a new key pair.
    let device_for_login = move || match relogin.get_value() {
        Some(p) => Some(core::PendingDevice {
            device: p.device,
            private_key: p.private_key,
            incident: p.incident,
        }),
        // The incident is chosen after logging in (ADR-0046); a shared link may name one.
        None => core::PendingDevice::create(&incident.get_untracked()),
    };
    let login = move |_| {
        let Some(device) = device_for_login() else {
            return;
        };
        let Some(key) = device.public_key() else {
            return;
        };
        device.save();
        let name = name_param().map_or_else(String::new, |n| {
            format!(
                "&name={}",
                percent_encoding::utf8_percent_encode(&n, percent_encoding::NON_ALPHANUMERIC)
            )
        });
        if let Some(window) = web_sys::window() {
            let _ = window.location().assign(&format!(
                "/auth/login?device={}&key={key}{name}",
                device.device
            ));
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
            let body =
                serde_json::json!({ "device": device.device, "key": key, "name": name_param() });
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
            <p role="alert" class="error">{move || error.get()}</p>
            {move || match login_enabled.get() {
                None => ().into_any(),
                Some(false) => view! { <p class="notice">{text::PAIR_NO_LOGIN}</p> }.into_any(),
                Some(true) => view! {
                    {(!is_relogin).then(|| view! {
                        <div class="field">
                            <label for="device-name">{text::DEVICE_NAME_LABEL}</label>
                            <input id="device-name" type="text" autocomplete="off" maxlength="60" bind:value=device_name />
                            <p class="hint">{text::DEVICE_NAME_HINT}</p>
                        </div>
                    })}
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
            <crate::notices::NoticesLink />
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
            plan: None,
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
            <label for="incident">{text::PAIR_INCIDENT}</label>
            <input id="incident" type="text" spellcheck="false" bind:value=incident />
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
        let readable = workspace.requires.iter().all(|comp| {
            c.incident
                .as_ref()
                .is_some_and(|i| c.replica.is_granted(i, comp))
        });
        // [impl->dsn~views-by-role~1]
        // Views follow the roles in this incident (decided by the server's policies).
        let by_incident = match workspace.permission {
            Some(p) => c.incident.as_ref().is_some_and(|i| c.replica.may(i, p)),
            // Administration: only organisation roles open it (ADR-0061).
            None => workspace.or_organisation.is_empty(),
        };
        let permitted = by_incident
            || workspace
                .or_organisation
                .iter()
                .any(|p| c.replica.has_permission(p));
        if readable && permitted {
            Ok(())
        } else {
            Err(text::REASON_NO_ACCESS)
        }
    })
}

/// Whether the user may open view `v` of a workspace they have access to: a view's own
/// permission comes from the roles in this incident or an organisation role (ADR-0061).
pub(crate) fn view_allowed(core: Handle, v: &nav::View) -> bool {
    v.permission.is_none_or(|p| {
        core.with_value(|c| {
            c.replica.has_permission(p) || c.incident.as_ref().is_some_and(|i| c.replica.may(i, p))
        })
    })
}

/// The views of `w` the user may open, in order.
pub(crate) fn allowed_views(core: Handle, w: &'static nav::Workspace) -> Vec<&'static nav::View> {
    w.views.iter().filter(|v| view_allowed(core, v)).collect()
}

const NAV_KEY: &str = "ordo.nav";
const THEME_KEY: &str = "ordo.theme";
const HAND_KEY: &str = "ordo.hand";

/// Sets (or, when empty, removes) the attribute `attribute` on `<html>` and remembers the
/// value on this device under `key`: device settings applied by style rules only.
fn apply_setting(attribute: &str, key: &str, value: &str) {
    if let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ = if value.is_empty() {
            root.remove_attribute(attribute)
        } else {
            root.set_attribute(attribute, value)
        };
    }
    core::set_preference(key, (!value.is_empty()).then_some(value));
}

/// Applies a colour theme (`""` = follow the system) and remembers it on this device.
pub(crate) fn apply_theme(theme: &str) {
    apply_setting("data-theme", THEME_KEY, theme);
}

// [impl->dsn~header-layout~2]
/// Applies the handedness (`""` = right, `"left"`) and remembers it on this device: the
/// bottom bar puts the menu at the end the thumb reaches.
pub(crate) fn apply_hand(hand: &str) {
    apply_setting("data-hand", HAND_KEY, hand);
}

/// The content of both menu buttons (header and bottom bar).
#[component]
fn MenuLabel() -> impl IntoView {
    view! {
        <Icon name="layout-grid" />
        <span class="launcher__text">{text::MENU}</span>
        <span class="visually-hidden">" – " {text::LAUNCHER}</span>
    }
}

/// Focuses the menu button of the current layout (header on wide screens, bottom bar on
/// phones); the other one is hidden.
fn focus_menu_button() {
    let visible = |id: &str| {
        web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(id))
            .and_then(|e| wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(e).ok())
            .is_some_and(|e| e.offset_parent().is_some())
    };
    focus(if visible("launcher-button") {
        "launcher-button"
    } else {
        "launcher-bar-button"
    });
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

/// Follows an internal link through [`Nav`]: `leptos_router` writes the address of a plain
/// link only once its `<Routes>` have rendered, which ORDO does not use, so the address bar
/// (and with it reload and the back button) would stay behind. Clicks with a modifier
/// (new tab or window) are left to the browser.
pub(crate) fn follow(
    nav: Nav,
    route: nav::Route,
) -> impl Fn(leptos::ev::MouseEvent) + Clone + 'static {
    move |ev: leptos::ev::MouseEvent| {
        if ev.button() == 0 && !(ev.ctrl_key() || ev.meta_key() || ev.shift_key() || ev.alt_key()) {
            ev.prevent_default();
            nav.go(route.clone());
        }
    }
}

/// [`follow`] with the workspace's [`Nav`] from the context (views inside the workspace).
pub(crate) fn follow_route(route: nav::Route) -> impl Fn(leptos::ev::MouseEvent) + Clone + 'static {
    let nav = use_context::<Nav>();
    move |ev: leptos::ev::MouseEvent| {
        if let Some(nav) = nav {
            follow(nav, route.clone())(ev);
        }
    }
}

/// Runs `f` once the view has updated after a navigation.
pub(crate) fn after_render(f: impl FnOnce() + 'static) {
    set_timeout(f, std::time::Duration::from_millis(60));
}

// [impl->dsn~workspace-navigation~3]
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
    provide_context(crate::profile::Revision(revision));
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
    // Back to the incident list; the data of this incident stays on the device.
    let change_incident = move || {
        if let Some(mut p) = paired.get_untracked() {
            p.incident.clear();
            p.plan = None;
            core::save_pairing(&p);
            paired.set(Some(p));
        }
    };
    let has_incident = core.with_value(|c| c.incident.is_some());
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
                {if has_incident {
                    view! { <Workspace core revision on_unpair=unpair on_change_incident=change_incident /> }.into_any()
                } else {
                    view! { <incidents::IncidentPicker core revision paired on_unpair=unpair /> }.into_any()
                }}
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
    on_change_incident: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let nav = Nav::new();
    provide_context(nav);
    provide_context(patients::Drafts::default());
    provide_context(crate::orders::Performers::default());
    let open = RwSignal::new(false);
    let palette = RwSignal::new(false);
    let pathname = use_location().pathname;
    let route = Memo::new(move |_| nav::Route::from_path(&pathname.get()));
    let saved = StoredValue::new(core::preference(NAV_KEY).and_then(|s| {
        let (w, v) = s.split_once('/')?;
        Some(nav::Route::view(w, v))
    }));

    // The shown workspace: the one in the URL if accessible, else the last used one, else
    // the first accessible one.
    let current = Memo::new(move |_| {
        revision.track();
        let usable = |id: &str| nav::workspace(id).filter(|w| access(core, w).is_ok());
        route
            .with(|r| r.as_ref().and_then(|r| usable(&r.workspace)))
            .or_else(|| saved.with_value(|s| s.as_ref().and_then(|r| usable(&r.workspace))))
            // Automatically only a workspace of the incident, never the device list: it is
            // reachable before the incident is, and would stay chosen.
            .or_else(|| {
                nav::WORKSPACES
                    .iter()
                    .find(|w| !w.requires.is_empty() && access(core, w).is_ok())
            })
            .map(|w| w.id)
    });
    let current_view = Memo::new(move |_| {
        revision.track();
        let workspace = nav::workspace(current.get()?)?;
        // The view in the URL, else the last used one of this workspace (e.g. after the
        // device was locked and the URL is the login's).
        let id = route
            .with(|r| {
                r.as_ref()
                    .filter(|r| r.workspace == workspace.id)
                    .map(|r| r.view.clone())
            })
            .or_else(|| {
                saved.with_value(|s| {
                    s.as_ref()
                        .filter(|r| r.workspace == workspace.id)
                        .map(|r| r.view.clone())
                })
            });
        let views = allowed_views(core, workspace);
        views
            .iter()
            .find(|v| Some(v.id) == id.as_deref())
            .or_else(|| views.first())
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
        let first = allowed_views(core, w)
            .first()
            .map_or(w.views[0].id, |v| v.id);
        nav.go(nav::Route::view(w.id, first));
        focus("main");
    };
    let incident = core.with_value(|c| {
        c.incident
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default()
    });
    let incident_name = move || {
        revision.track();
        core.with_value(|c| c.incident_info().map(|i| i.name))
            .unwrap_or_else(|| format!("{}…", incident.chars().take(10).collect::<String>()))
    };
    let incident = core.with_value(|c| {
        c.incident
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default()
    });
    let user = core.with_value(|c| c.replica.user().to_string());
    let me = core.with_value(|c| c.replica.user().clone());
    // Unpairing deletes the key on this device; if the server can be reached, it forgets
    // the device as well (SEC-4), so it no longer appears in the device list.
    // Kept locally (the callbacks are not `Send`), so the profile menu can hold them.
    let on_unpair = StoredValue::new_local(on_unpair);
    let on_change_incident = StoredValue::new_local(on_change_incident);
    let unpair = move |_| {
        let device = core.with_value(|c| c.replica.device());
        core::revoke_device(core, device);
        core::forget_device();
        on_unpair.with_value(|f| f());
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
    // Ctrl+K / ⌘K opens the command palette from anywhere (UX-10).
    let profile_open = RwSignal::new(false);
    // While the palette is open, Ctrl+K moves its selection up and Esc closes it (ADR-0051).
    let keys = window_event_listener(leptos::ev::keydown, move |ev| {
        if (ev.ctrl_key() || ev.meta_key()) && !ev.alt_key() && ev.key().eq_ignore_ascii_case("k") {
            ev.prevent_default();
            if !palette.get_untracked() {
                open.set(false);
                profile_open.set(false);
                palette.set(true);
            }
        } else if crate::ui::move_in_list(&ev) {
            // j/k and the arrow keys in lists (ADR-0051).
        } else if ev.key() == "Escape" {
            // Escape closes an open menu wherever the focus is, and returns it to the
            // menu's button (ADR-0025, ADR-0049).
            if open.get_untracked() {
                open.set(false);
                focus_menu_button();
            }
        }
    });
    on_cleanup(move || keys.remove());
    // [impl->dsn~header-layout~2]
    crate::ui::close_on_click_outside(
        open,
        "#launcher-button, #launcher-bar-button, #launcher-panel".to_owned(),
    );
    let theme = RwSignal::new(core::preference(THEME_KEY).unwrap_or_default());
    let hand = RwSignal::new(core::preference(HAND_KEY).unwrap_or_default());
    let close_profile = Callback::new(move |()| profile_open.set(false));
    // The person's roles in this incident, shown in the profile menu.
    let my_roles = {
        let me = me.clone();
        move || {
            revision.track();
            core.with_value(|c| c.incident_info().and_then(|i| i.roster.get(&me).cloned()))
                .filter(|roles| !roles.is_empty())
                .map_or_else(
                    || text::NO_ROLE.to_owned(),
                    |roles| {
                        roles
                            .iter()
                            .map(|r| text::role_label(r))
                            .collect::<Vec<_>>()
                            .join(", ")
                    },
                )
        }
    };

    view! {
        <a class="skip-link" href="#main" on:click=move |ev| {
            ev.prevent_default();
            focus("main");
        }>{text::SKIP_LINK}</a>
        <header class="app-header">
            <div class="app-header__start">
                <DisclosureButton id="launcher-button" controls="launcher-panel" open
                    class="launcher__button launcher__button--header" on_open=Some(close_profile)>
                    <MenuLabel />
                </DisclosureButton>
                <div class="app-header__heading">
                    <h1 class="app-header__title">{title}</h1>
                    <p class="app-header__incident">
                        <span>{text::INCIDENT} ": "</span>
                        <span id="incident-name" title=incident.clone()>{incident_name}</span>
                    </p>
                </div>
            </div>
            <button
                id="palette-button"
                type="button"
                class="search-button"
                aria-keyshortcuts="Control+K Meta+K"
                on:click=move |_| palette.set(true)
            >
                <Icon name="search" />
                <span>{text::SEARCH}</span>
                <kbd class="search-button__keys" aria-hidden="true">{shortcut}</kbd>
            </button>
            <div class="app-header__end">
                <ConnectionStatus core revision />
                <Disclosure id="profile" open=profile_open class="profile-menu"
                    button_class="profile-menu__button" panel_class="profile-menu__panel"
                    on_open=Some(Callback::new(move |()| open.set(false)))
                    button=Box::new({
                        let me = me.clone();
                        move || view! {
                            <crate::profile::Avatar core revision user=me.clone() />
                            <span class="visually-hidden">{text::SIGNED_IN_AS} " "</span>
                            <span class="profile-menu__name">{move || {
                                revision.track();
                                crate::profile::display_name(core, &me)
                            }}</span>
                        }.into_any()
                    })>
                    <h2 class="launcher__title">{text::PROFILE}</h2>
                    <p class="launcher__user">{text::SIGNED_IN_AS} " " <strong>{{
                        let user = user.clone();
                        move || {
                            revision.track();
                            user.parse::<ordo_core::UserId>().map_or_else(|_| user.clone(), |id| crate::profile::display_name(core, &id))
                        }
                    }}</strong> " (" {user.clone()} ")"</p>
                    <p id="my-roles">{text::MY_ROLES} ": " {my_roles}</p>
                    <crate::profile::ProfilePicture core revision />
                    <crate::profile::DisplayName core revision />
                    <SettingSelect id="theme" label=text::APPEARANCE options=&text::THEMES value=theme apply=apply_theme />
                    <SettingSelect id="hand" label=text::HAND options=&text::HANDS value=hand apply=apply_hand />
                    <a id="my-devices" class="button" href=nav::Route::view("person.devices", "devices").to_path()
                        on:click=follow(nav, nav::Route::view("person.devices", "devices"))>{text::MY_DEVICES}</a>
                    <button id="change-incident" type="button" class="button"
                        on:click=move |_| on_change_incident.with_value(|f| f())>{text::CHANGE_INCIDENT}</button>
                    <button type="button" class="button" on:click=unpair>{text::UNPAIR}</button>
                </Disclosure>
            </div>
        </header>
        <div class="layout">
            <div class="navigation">
              <div class="navigation__inner">
                <div class="launcher">
                    <DisclosureButton id="launcher-bar-button" controls="launcher-panel" open
                        class="launcher__button launcher__button--bar" on_open=Some(close_profile)>
                        <MenuLabel />
                    </DisclosureButton>
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
                    </div>
                </div>
                <nav class="views" aria-label=text::VIEWS>
                    {move || {
                        // Views follow permissions, which change with the roles.
                        revision.track();
                        current
                            .get()
                            .and_then(nav::workspace)
                            .map(|w| {
                                allowed_views(core, w)
                                    .into_iter()
                                    .map(|v| {
                                        let selected = move || current_view.get() == Some(v.id);
                                        view! {
                                            <a
                                                class="views__item"
                                                href=nav::Route::view(w.id, v.id).to_path()
                                                on:click=follow(nav, nav::Route::view(w.id, v.id))
                                                aria-current=move || selected().then_some("page")
                                            >
                                                <Icon name=v.icon />
                                                <span class="views__label">{v.label}</span>
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
                // [impl->dsn~tasks~2]
                // The open tasks, in short, at the top of every view of the incident command
                // (TASK-3); the whole list is the page *Aufträge*.
                <Show when=move || {
                    current.get().is_some_and(|w| w.starts_with("command."))
                        && current_view.get() != Some("dispatch")
                }>
                    <crate::tasks::TasksStrip core revision />
                </Show>
                {move || match (current.get(), current_view.get()) {
                    (Some("med.uhs"), Some("uhs")) => view! { <UhsSection core revision /> }.into_any(),
                    (Some("med.uhs"), Some("orders")) => view! { <crate::orders::OrdersView core revision nav /> }.into_any(),
                    (Some("med.uhs"), Some("overview")) => view! { <crate::overview::OverviewView core revision /> }.into_any(),
                    (Some("med.uhs"), _) => {
                        view! { <patients::PatientsView core revision nav record=Signal::from(record) /> }.into_any()
                    }
                    // [impl->dsn~workspaces-by-phase~3]
                    // [impl->dsn~map-drawing~2]
                    (Some("command.lead"), Some("map")) => view! {
                        <crate::situation::SituationView core revision />
                        <CommandWorkspace core revision />
                    }.into_any(),
                    (Some("command.lead"), Some("dispatch")) => view! { <crate::tasks::OpenTasks core revision /> }.into_any(),
                    (Some("command.lead"), Some("transports")) => view! { <crate::transports::TransportsView core revision /> }.into_any(),
                    (Some("command.lead"), _) => view! { <crate::summary::CommandOverview core revision /> }.into_any(),
                    (Some("command.prepare"), Some("units")) => view! { <crate::units::ResourcesView core revision /> }.into_any(),
                    // The command structure is prepared with the people (ADR-0067); old
                    // links to its own view land here.
                    (Some("command.prepare"), Some("people" | "structure")) => view! {
                        <crate::incidents::RosterView core revision />
                        <crate::devices::IncidentDevicesView core revision />
                        <crate::command::LeadershipView core revision />
                    }.into_any(),
                    // The situation map prepared on one page: its region, then the editor,
                    // also opened to place one object (`place:<kind>:<ID>`).
                    (Some("command.prepare"), Some("map")) => view! {
                        <crate::situation::MapPreparation core revision preset=record.get_untracked().and_then(|r| crate::situation::preset(&r)) />
                    }.into_any(),
                    (Some("command.prepare"), _) => view! { <crate::incidents::IncidentPage core revision /> }.into_any(),
                    (Some("incident.log"), _) => view! { <crate::log::LogView core revision /> }.into_any(),
                    (Some("admin.access"), Some("permissions")) => view! { <crate::roles::RolesView core revision /> }.into_any(),
                    (Some("admin.access"), Some("devices")) => view! { <devices::DevicesView core revision /> }.into_any(),
                    (Some("admin.access"), Some("login")) => view! { <crate::login_providers::LoginProvidersView core revision /> }.into_any(),
                    (Some("admin.access"), _) => view! { <crate::accounts::AccountsView core revision /> }.into_any(),
                    (Some("admin.info"), _) => view! { <crate::notices::InfoView /> }.into_any(),
                    (Some("person.devices"), _) => view! { <devices::DevicesView core revision mine_only=true /> }.into_any(),
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
// [impl->dsn~compact-navigation~2]
#[component]
fn ConnectionStatus(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let state = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            (
                c.connection,
                c.replica.outbox_len() + crate::attachments::queued(),
                c.error.clone(),
                c.storage_failed,
            )
        })
    });
    let words = move || match state.get().0 {
        Connection::Online => text::ONLINE,
        Connection::Connecting => text::CONNECTING,
        Connection::Offline => text::OFFLINE,
    };
    // One design on every screen (ADR-0049): an icon whose shape tells the state and whose
    // colour repeats it (never colour alone), the number of unsent inputs, and the words
    // behind the button. Offline is a warning, not an error: inputs are kept and sent later.
    let kind = move || match state.get() {
        (Connection::Online, 0, ..) => ("synced", "cloud-check"),
        (Connection::Online, ..) => ("sending", "cloud-upload"),
        (Connection::Connecting, ..) => ("connecting", "refresh-cw"),
        (Connection::Offline, ..) => ("offline", "cloud-off"),
    };
    let label =
        Signal::derive(move || text::connection_label(words(), &text::pending(state.get().1)));
    let open = RwSignal::new(false);
    view! {
        <Disclosure id="connection" open
            class=Signal::derive(move || format!("connection connection--{}", kind().0))
            button_class="connection__button" panel_class="connection__panel" label=Some(label)
            button=Box::new(move || view! {
                {move || view! { <Icon name=kind().1 /> }}
                <span class="connection__count">{move || {
                    let n = state.get().1;
                    (n > 0).then(|| n.to_string())
                }}</span>
            }.into_any())>
            <p class="status">
                <strong>{words}</strong>
                <span class="status__pending">{move || text::pending(state.get().1)}</span>
            </p>
        </Disclosure>
        {move || state.get().2.map(|e| view! { <p role="alert" class="error connection__error">{e}</p> })}
        {move || state.get().3.then(|| view! { <p role="alert" class="error connection__error">{text::STORAGE_FAILED}</p> })}
    }
}

// [impl->dsn~uhs-registry~1]
/// On-site care units with their load, and a form to set one up (UHS-1, UHS-4).
/// What someone types into a unit's details panel, and whether it is open: kept by the
/// view under its owner, so a list that updates while they type keeps it.
#[derive(Clone, Copy)]
struct UhsDraft {
    open: RwSignal<bool>,
    name: RwSignal<String>,
    capacity: RwSignal<String>,
    person: RwSignal<String>,
    organisation: RwSignal<String>,
    service: RwSignal<String>,
    /// Its strength as a unit.
    strength: crate::units::StrengthDraft,
    /// Capacity per category, in the order of [`CAPACITY_CATEGORIES`].
    capacities: [RwSignal<String>; 5],
    /// Where the person being added to the staff is assigned already (UNIT-8).
    elsewhere: RwSignal<Vec<ordo_units::Post>>,
}

/// The triage categories a UHS can keep capacity and places for (UHS-13): all but the
/// deceased, who take no treatment place.
const CAPACITY_CATEGORIES: [TriageCategory; 5] = [Immediate, Urgent, Minor, Expectant, Uninjured];

impl UhsDraft {
    /// The draft of the unit `id`, created under `owner` the first time, filled with its
    /// current values.
    fn kept(
        drafts: StoredValue<std::collections::HashMap<String, UhsDraft>>,
        owner: &Owner,
        id: &str,
        name: &str,
        capacity: u32,
        details: &med::UhsDetails,
    ) -> Self {
        let mut out = None;
        drafts.update_value(|m| {
            let d = *m.entry(id.to_owned()).or_insert_with(|| {
                owner.with(|| UhsDraft {
                    open: RwSignal::new(false),
                    name: RwSignal::new(name.to_owned()),
                    capacity: RwSignal::new(capacity.to_string()),
                    person: RwSignal::new(String::new()),
                    organisation: RwSignal::new(details.organisation.clone().unwrap_or_default()),
                    service: RwSignal::new(details.service.clone().unwrap_or_default()),
                    strength: crate::units::StrengthDraft::new(details.strength, &details.special),
                    capacities: CAPACITY_CATEGORIES.map(|c| {
                        RwSignal::new(
                            details
                                .capacities
                                .get(&c)
                                .map(ToString::to_string)
                                .unwrap_or_default(),
                        )
                    }),
                    elsewhere: RwSignal::new(Vec::new()),
                })
            });
            out = Some(d);
        });
        out.expect("inserted")
    }
}

// [impl->dsn~uhs-details~3]
/// Name, capacity, place on the situation map and assigned staff of one unit (UHS-1),
/// behind a disclosure.
#[component]
fn UhsDetailsPanel(
    core: Handle,
    revision: RwSignal<u64>,
    id: String,
    name: String,
    details: med::UhsDetails,
    draft: UhsDraft,
) -> impl IntoView {
    let UhsDraft {
        open,
        name: new_name,
        capacity: cap,
        person,
        organisation,
        service,
        strength,
        capacities,
        elsewhere,
    } = draft;
    let slug = core::slug(&id);
    let staff = details.staff.clone();
    let people = Memo::new(move |_| {
        revision.track();
        core::incident_people(core)
    });
    let error = RwSignal::new(None::<&'static str>);
    let save = {
        let (id, current_name) = (id.clone(), name.clone());
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            // [impl->dsn~uhs-capacity-by-category~1]
            // A capacity per category, if given, makes the total.
            let mut by_category = std::collections::BTreeMap::new();
            for (category, value) in CAPACITY_CATEGORIES.into_iter().zip(capacities) {
                let value = value.get_untracked();
                if value.trim().is_empty() {
                    continue;
                }
                match value.trim().parse::<u32>() {
                    Ok(0) => {}
                    Ok(n) => {
                        by_category.insert(category, n);
                    }
                    Err(_) => {
                        error.set(Some(text::UHS_CAPACITY_NUMBER));
                        return;
                    }
                }
            }
            let capacity = if by_category.is_empty() {
                let Ok(capacity) = cap.get_untracked().trim().parse::<u32>() else {
                    error.set(Some(text::UHS_CAPACITY_NUMBER));
                    return;
                };
                capacity
            } else {
                let sum = by_category.values().sum::<u32>();
                cap.set(sum.to_string());
                sum
            };
            let chosen = new_name.get_untracked().trim().to_owned();
            if chosen.is_empty() {
                error.set(Some(text::UHS_NAME_NEEDED));
                return;
            }
            // [impl->dsn~uhs-as-unit~1]
            // Its strength as a unit, or none, and its special functions.
            let (reported, special) = match strength.read() {
                Ok(read) => read,
                Err(e) => {
                    error.set(Some(e));
                    return;
                }
            };
            let was = (
                details.organisation.clone().unwrap_or_default(),
                details.service.clone().unwrap_or_default(),
                details.strength,
            );
            let now = (
                organisation.get_untracked().trim().to_owned(),
                service.get_untracked(),
                reported,
            );
            let update = med::UhsUpdated {
                id: id.clone(),
                // Only a changed name is sent: the log then shows the renaming.
                name: (chosen != current_name).then_some(chosen),
                // The place is the UHS on the situation map (ADR-0064); a location text
                // written before stays as it is.
                location: None,
                capacity: Some(capacity),
                staff: None,
                // Only what changed is sent, as for the name.
                organisation: (now.0 != was.0).then(|| now.0.clone()),
                service: (now.1 != was.1).then(|| now.1.clone()),
                strength: (now.2 != was.2)
                    .then_some(now.2.map(|s| [s.leaders, s.sub_leaders, s.crew])),
                // [impl->dsn~detailed-strength~1]
                unit_leaders: (now.2.map(|s| s.unit_leaders) != was.2.map(|s| s.unit_leaders))
                    .then(|| now.2.map_or(0, |s| s.unit_leaders)),
                special: (special != details.special).then(|| special.clone()),
                capacities: (by_category != details.capacities).then(|| by_category.clone()),
            };
            if core::update_uhs(core, revision, &update).is_ok() {
                error.set(None);
                if update.capacities.is_some() && !by_category.is_empty() {
                    let _ = core::set_category_places(core, revision, &id, &by_category);
                }
            }
        }
    };
    let commit = {
        let (id, staff) = (id.clone(), staff.clone());
        Callback::new(move |who: String| {
            let mut list = staff.clone();
            // Moved here while on the list already: kept once.
            list.retain(|s| !ordo_units::same_person(s, &who));
            list.push(who);
            let update = med::UhsUpdated {
                id: id.clone(),
                staff: Some(list),
                ..Default::default()
            };
            if core::update_uhs(core, revision, &update).is_ok() {
                person.set(String::new());
                elsewhere.set(Vec::new());
            }
        })
    };
    let here = med::uhs_post(&id, &name);
    // [impl->dsn~double-booking~1]
    // Someone on a crew or another UHS is moved here, not counted twice (UNIT-8).
    let add = {
        let here = here.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let who = person.get_untracked().trim().to_owned();
            if who.is_empty() {
                return;
            }
            let away = crate::units::assignments(core).elsewhere(&who, &here);
            if away.is_empty() {
                commit.run(who);
            } else {
                elsewhere.set(away);
            }
        }
    };
    let on_move = Callback::new(move |()| {
        let who = person.get_untracked().trim().to_owned();
        crate::units::move_away(core, revision, &who, &elsewhere.get_untracked());
        commit.run(who);
    });
    let members = staff
        .iter()
        .enumerate()
        .map(|(i, member)| {
            let (id, staff) = (id.clone(), staff.clone());
            let actions = vec![crate::ui::Action {
                text: text::CREW_REMOVE,
                label: text::staff_remove_label(member),
                quiet: true,
                run: Callback::new(move |()| {
                    let mut list = staff.clone();
                    list.remove(i);
                    let update = med::UhsUpdated {
                        id: id.clone(),
                        staff: Some(list),
                        ..Default::default()
                    };
                    let _ = core::update_uhs(core, revision, &update);
                }),
            }];
            let name = {
                let member = member.clone();
                crate::profile::naming(move || crate::profile::display_text(core, &member))
            };
            view! { <li><strong>{name}</strong> <crate::ui::Actions actions /></li> }
        })
        .collect_view();
    view! {
        <crate::ui::Disclosure
            id=format!("uhs-details-{slug}")
            open
            class="load-list__details"
            button_class="button button--small button--quiet"
            panel_class="load-list__details-panel"
            label=Some(Signal::derive({
                let name = name.clone();
                move || text::uhs_details_label(&name)
            }))
            button=Box::new(|| view! { {text::UHS_DETAILS} }.into_any())
        >
            <form class="form form--inline" novalidate on:submit=save>
                <div class="field">
                    <label for=format!("uhs-name-{slug}")>{text::UHS_NAME}</label>
                    <input id=format!("uhs-name-{slug}") type="text" autocomplete="off" bind:value=new_name />
                </div>
                <div class="field field--small">
                    <label for=format!("uhs-capacity-{slug}")>{text::UHS_CAPACITY}</label>
                    <input id=format!("uhs-capacity-{slug}") type="text" inputmode="numeric" autocomplete="off" bind:value=cap />
                </div>
                <div class="field">
                    <label for=format!("uhs-organisation-{slug}")>{text::UNIT_ORGANISATION}</label>
                    <input id=format!("uhs-organisation-{slug}") type="text" autocomplete="off" bind:value=organisation />
                </div>
                <div class="field">
                    <label for=format!("uhs-service-{slug}")>{text::UNIT_SERVICE}</label>
                    <select id=format!("uhs-service-{slug}") prop:value=move || service.get() on:change=move |ev| service.set(event_target_value(&ev))>
                        <option value="">{text::NO_SERVICE}</option>
                        {crate::units::CATALOGUE.services.iter().map(|s| view! { <option value=s.id.clone()>{s.label.clone()}</option> }).collect_view()}
                    </select>
                </div>
                <crate::units::StrengthFields draft=strength prefix=format!("uhs-{slug}") title=text::UHS_STRENGTH
                    detailed=Signal::derive(move || { revision.track(); crate::units::detailed(core) }) />
                <p id=format!("uhs-strength-hint-{slug}") class="hint">{text::UHS_STRENGTH_HINT}</p>
                <fieldset class="choices strength-input" aria-describedby=format!("uhs-capacities-hint-{slug}")>
                    <legend>{text::UHS_CAPACITIES_LEGEND}</legend>
                    {CAPACITY_CATEGORIES
                        .into_iter()
                        .zip(capacities)
                        .map(|(category, value)| {
                            let id = format!("uhs-capacity-{}-{slug}", core::slug(text::label(category)));
                            view! {
                                <div class="field field--small">
                                    <label for=id.clone()>{text::label(category)}</label>
                                    <input id=id type="text" inputmode="numeric" autocomplete="off" bind:value=value
                                        title=text::description(category) />
                                </div>
                            }
                        })
                        .collect_view()}
                </fieldset>
                <p id=format!("uhs-capacities-hint-{slug}") class="hint">{text::UHS_CAPACITIES_HINT}</p>
                <button type="submit" class="button button--small">{text::UHS_SAVE}</button>
            </form>
            <p role="alert" class="error">{move || error.get()}</p>
            <UhsOnMap core revision id=id.clone() name=name.clone() written=details.location.clone() />
            <h4>{text::UHS_STAFF}</h4>
            {if staff.is_empty() {
                view! { <p class="hint">{text::UHS_STAFF_NONE}</p> }.into_any()
            } else {
                view! { <ul class="crew-list">{members}</ul> }.into_any()
            }}
            <form class="form form--inline" novalidate on:submit=add>
                <div class="field">
                    <label for=format!("uhs-staff-{slug}")>{text::CREW_MEMBER}</label>
                    <input id=format!("uhs-staff-{slug}") type="text" autocomplete="off"
                        list=format!("uhs-staff-{slug}-people") bind:value=person />
                    <crate::ui::Suggestions id=format!("uhs-staff-{slug}-people") values=people core />
                </div>
                <button type="submit" class="button button--small">{text::UHS_STAFF_ADD}</button>
            </form>
            <crate::units::AssignedElsewhere person elsewhere here=here.label.clone() on_move />
        </crate::ui::Disclosure>
    }
}

// [impl->dsn~uhs-details~3]
/// Where a UHS is: its place on the situation map, and a location text written before
/// (kept, events are never changed). The incident command sets the place on the map.
#[component]
fn UhsOnMap(
    core: Handle,
    revision: RwSignal<u64>,
    id: String,
    name: String,
    written: Option<String>,
) -> impl IntoView {
    let object = ordo_map::ObjectKind::Uhs.object_id(&id);
    let at = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            ordo_map::SituationMap::from_events(&c.events(ordo_map::COMPARTMENT))
                .objects
                .get(&object)
                .and_then(|o| o.placed.geometry.anchor())
        })
    });
    let may_place = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            c.incident.as_ref().is_some_and(|i| {
                c.replica.may(i, "view_situation") && c.replica.may(i, "edit_units")
            })
        })
    });
    let place = crate::situation::editor_route(Some(&crate::situation::place_record(
        ordo_map::ObjectKind::Uhs,
        &id,
    )));
    let name = StoredValue::new(name);
    let visible = move || {
        if at.get().is_some() {
            text::UHS_MOVE_ON_MAP
        } else {
            text::UHS_PLACE_ON_MAP
        }
    };
    view! {
        <p class="uhs-on-map">
            {text::UHS_ON_MAP} ": "
            <span class="uhs-on-map__at">{move || at.get().map_or_else(
                || text::UHS_NOT_ON_MAP.to_owned(),
                |[lon, lat]| format!("{lat:.5}, {lon:.5}"),
            )}</span>
        </p>
        {written.filter(|w| !w.is_empty()).map(|w| view! {
            <p class="hint">{text::UHS_LOCATION} ": " {w}</p>
        })}
        <Show when=move || may_place.get()>
            <a class="button button--small" href=place.to_path() on:click=follow_route(place.clone())
                aria-label=move || name.with_value(|n| text::uhs_place_label(visible(), n))>
                {visible}
            </a>
        </Show>
    }
}

// [impl->dsn~uhs-resources~1]
/// The UHS among the resources (UHS-11): each with its staff, which the incident command
/// assigns there; location and capacity in the same panel.
#[component]
pub(crate) fn UhsResources(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let rows = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            let events = c.events(med::UHS_COMPARTMENT);
            let details = med::uhs_details(&events);
            med::uhs_list(&events)
                .into_iter()
                .map(|u| {
                    let d = details.get(&u.id).cloned().unwrap_or_default();
                    (u.id, u.name, u.capacity, d)
                })
                .collect::<Vec<_>>()
        })
    });
    // Drafts of the panels, created under this view's owner (see `UhsDraft`).
    let owner = Owner::current().expect("a view has an owner");
    let drafts = StoredValue::new(std::collections::HashMap::<String, UhsDraft>::new());
    let draft = move |id: &str, name: &str, capacity: u32, details: &med::UhsDetails| {
        UhsDraft::kept(drafts, &owner, id, name, capacity, details)
    };
    view! {
        <section aria-labelledby="uhs-resources-title" class="card">
            <h2 id="uhs-resources-title">{text::UHS_RESOURCES_TITLE}</h2>
            {move || {
                let rows = rows.get();
                if rows.is_empty() {
                    return view! { <p>{text::UHS_EMPTY}</p> }.into_any();
                }
                view! {
                    <ul class="unit-list">
                        {rows
                            .into_iter()
                            .map(|(id, name, capacity, details)| {
                                let staff = text::uhs_staff_count(details.staff.len());
                                view! {
                                    <li class="unit-card uhs-card" data-uhs=id.clone()>
                                        <h3>{name.clone()}</h3>
                                        <p class="uhs-card__staff">{staff}</p>
                                        <UhsDetailsPanel core revision id=id.clone() name=name.clone() details=details.clone()
                                            draft=draft(&id, &name, capacity, &details) />
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
        </section>
    }
}

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
        let (register, places) = core.with_value(|c| {
            (
                PatientRegister::from_events(&c.events(med::COMPARTMENT)),
                med::places(&c.events(med::UHS_COMPARTMENT)),
            )
        });
        let counts = register.counts();
        let details = core.with_value(|c| med::uhs_details(&c.events(med::UHS_COMPARTMENT)));
        uhs_list(core)
            .into_iter()
            .map(|u| {
                let n = counts
                    .get(&u.id)
                    .map_or(0, |c| CATEGORIES.iter().map(|k| c.get(*k)).sum());
                let total = places.get(&u.id).map_or(0, Vec::len);
                let taken = register
                    .patients()
                    .filter(|p| p.uhs == u.id && p.place.is_some())
                    .count();
                let d = details.get(&u.id).cloned().unwrap_or_default();
                (
                    u.id,
                    u.name,
                    n,
                    u.capacity,
                    total,
                    total.saturating_sub(taken),
                    d,
                )
            })
            .collect::<Vec<_>>()
    };
    // Drafts of the details panels, created under this view's owner (see `UhsDraft`).
    let owner = Owner::current().expect("a view has an owner");
    let drafts = StoredValue::new(std::collections::HashMap::<String, UhsDraft>::new());
    let draft = move |id: &str, name: &str, capacity: u32, details: &med::UhsDetails| {
        UhsDraft::kept(drafts, &owner, id, name, capacity, details)
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
                            .enumerate()
                            .map(|(i, (id, name, n, cap, total, free, details))| {
                                let editor = view! {
                                    <UhsDetailsPanel core revision id=id.clone() name=name.clone() details=details.clone()
                                        draft=draft(&id, &name, cap, &details) />
                                };
                                // [impl->dsn~treatment-places~1]
                                let count = RwSignal::new(total.to_string());
                                let field = format!("places-{i}");
                                let set = move |ev: leptos::ev::SubmitEvent| {
                                    ev.prevent_default();
                                    if let Ok(count) = count.get().trim().parse::<u32>() {
                                        let _ = core::set_places(core, revision, &id, count);
                                    }
                                };
                                view! {
                                    <li class="load-list__item">
                                        <span class="load-list__name">{name.clone()}</span>
                                        <span>{text::COL_LOAD} ": " {text::load(n, cap)}</span>
                                        <meter min="0" max=cap value=n aria-hidden="true"></meter>
                                        <span class="load-list__places">{text::PLACES} ": " {text::places_free(free, total)}</span>
                                        <form class="form form--inline" on:submit=set>
                                            <div class="field field--small">
                                                <label for=field.clone()>{text::places_of(&name)}</label>
                                                <input id=field type="text" inputmode="numeric" pattern="[0-9]+" required bind:value=count />
                                            </div>
                                            <button type="submit" class="button button--small">{text::PLACES_SET}</button>
                                        </form>
                                        {details.location.clone().map(|l| view! { <span class="load-list__location">{text::UHS_LOCATION} ": " {l}</span> })}
                                        {(!details.staff.is_empty()).then(|| view! {
                                            <span class="load-list__staff">{text::UHS_STAFF} ": " {crate::profile::naming({
                                                let staff = details.staff.clone();
                                                move || staff.iter().map(|s| crate::profile::display_text(core, s)).collect::<Vec<_>>().join(", ")
                                            })}</span>
                                        })}
                                        {editor}
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

/// One on-site care unit's patients per triage category, their total against its
/// capacity, and how many were discharged; `children` add more lines (UHS-8).
#[component]
pub fn CountCard(
    name: String,
    capacity: u32,
    counts: ordo_contracts::PatientCountsV1,
    /// Capacity per triage category, shown next to the count (UHS-13).
    #[prop(optional)]
    capacities: std::collections::BTreeMap<TriageCategory, u32>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let c = counts;
    let total: u32 = CATEGORIES.iter().map(|k| c.get(*k)).sum();
    view! {
        <li class="count-card">
            <h3>{name}</h3>
            <ul class="count-card__counts">
                {CATEGORIES
                    .into_iter()
                    .map(|k| {
                        // [impl->dsn~uhs-capacity-by-category~1]
                        let of = capacities.get(&k).map(|n| format!(" / {n}"));
                        view! { <li><TriageBadge category=k /> <span class="count-card__n">{c.get(k)}{of}</span></li> }
                    })
                    .collect_view()}
            </ul>
            <p>
                {text::COL_TOTAL} ": " <strong>{total}</strong> " · " {text::COL_LOAD} ": " {text::load(total, capacity)}
                " · " {text::COL_DISCHARGED} ": " {c.discharged}
            </p>
            {children.map(|c| c())}
        </li>
    }
}

#[component]
fn CommandWorkspace(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let rows = Memo::new(move |_| {
        revision.track();
        let list = uhs_list(core);
        let counts = core.with_value(|c| {
            let events = c.events("incident");
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
                    .map(|(name, capacity, c)| view! { <CountCard name=name.clone() capacity=*capacity counts=c.clone() /> })
                    .collect_view();
                view! {
                    <ul class="count-list only-narrow">{list}</ul>
                    <table class="table table--counts only-wide">
                        <thead>
                            <tr>
                                <th scope="col">{text::COL_UHS}</th>
                                {CATEGORIES.into_iter().map(|c| view! { <th scope="col" class="number"><TriageBadge category=c /></th> }).collect_view()}
                                <th scope="col" class="number">{text::COL_TOTAL}</th>
                                <th scope="col" class="number">{text::COL_LOAD}</th>
                                <th scope="col" class="number">{text::COL_DISCHARGED}</th>
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
                                            <td class="number">{c.discharged}</td>
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
