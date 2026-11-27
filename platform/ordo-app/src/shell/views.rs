//! Views. Semantic HTML first (A11Y-4): real buttons, labels, fieldsets, tables, a skip
//! link and a live region; all colours and sizes come from design tokens.

use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::components::Router;
use leptos_router::hooks::{use_location, use_navigate};

use crate::core::{self, Connection, Core, Handle, Pairing};
use crate::shell::ui::{Disclosure, DisclosureButton, Icon, SettingSelect};
use crate::{shared::incidents, shell::nav, shell::palette, text};

/// Pairing or workspace.
/// The app: opens the encrypted device storage first (ADR-0041), then starts.
#[component]
pub fn App() -> impl IntoView {
    apply_theme(&core::preference(THEME_KEY).unwrap_or_default());
    apply_hand(&core::preference(HAND_KEY).unwrap_or_default());
    apply_keymap(&core::preference(crate::shell::keys::KEYMAP_KEY).unwrap_or_default());
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
        vault.set(Some(opened.and_then(|()| core::check_pairing())));
    });
    move || match vault.get() {
        None => {
            view! { <main id="main" class="page page--narrow"><p>{text::starting()}</p></main> }
                .into_any()
        }
        Some(Err(error)) => view! {
            <main id="main" class="page page--narrow">
                <h1>{text::app_name()}</h1>
                <p role="alert" class="error">{error}</p>
                <button type="button" class="button" on:click=|_| {
                    if let Some(w) = web_sys::window() {
                        let _ = w.location().reload();
                    }
                }>{text::retry()}</button>
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
    // A guest's code (ADR-0077, ADR-0084), from the QR code the incident command showed.
    let guest_code = core::url_param("guest");
    if returned.is_some() || login_error.is_some() || guest_code.is_some() {
        core::clear_url_params();
    }
    if let Some(pairing) = &returned {
        core::save_pairing(pairing);
    }
    let pairing = RwSignal::new(returned.or_else(core::load_pairing));
    view! {
        <Router>
            <Screens pairing login_error guest_code />
        </Router>
    }
}

// [impl->dsn~licence-notices~2]
/// The screen for the URL: the licence notices, pairing or the paired device. Pairing links
/// to the notices; a paired device has them under *Administration – Info* (LIC-1).
#[component]
fn Screens(
    pairing: RwSignal<Option<Pairing>>,
    login_error: Option<String>,
    guest_code: Option<String>,
) -> impl IntoView {
    let location = use_location();
    let pathname = location.pathname;
    // A memo, so moving between views does not build the device's screen anew.
    let on_notices = Memo::new(move |_| pathname.get() == crate::admin::notices::PATH);
    let on_gallery = Memo::new(move |_| pathname.get() == crate::shell::gallery::PATH);
    // The screen before the notices, to return to.
    let previous = StoredValue::new("/".to_owned());
    Effect::new(move || {
        let path = pathname.get();
        if path != crate::admin::notices::PATH && path != crate::shell::gallery::PATH {
            previous.set_value(format!("{path}{}", location.search.get()));
        }
    });
    view! {
        {move || {
            if on_notices.get() {
                return view! { <crate::admin::notices::NoticesPage back=previous.get_value() /> }.into_any();
            }
            // [impl->dsn~component-gallery~1]
            if on_gallery.get() {
                return view! { <crate::shell::gallery::GalleryPage back=previous.get_value() /> }.into_any();
            }
            match (pairing.get(), guest_code.clone()) {
                (None, Some(code)) => view! { <GuestPairing pairing code /> }.into_any(),
                (None, None) => view! { <PairingForm pairing login_error=login_error.clone() /> }.into_any(),
                (Some(p), _) => view! { <Device pairing=p paired=pairing on_unpair=move || pairing.set(None) /> }.into_any(),
            }
        }}
    }
}

// [impl->dsn~guest-access~4]
/// A helper without an account pairs this device with the code of the QR code the
/// incident command showed them (ADR-0077): the device makes its own key, as for a login,
/// and opens the incident with the role it was given.
#[component]
fn GuestPairing(pairing: RwSignal<Option<Pairing>>, code: String) -> impl IntoView {
    let error = RwSignal::new(None::<&'static str>);
    leptos::task::spawn_local(async move {
        let failed = move || error.set(Some(text::guest_code_invalid()));
        let Some(device) = core::PendingDevice::create("") else {
            return failed();
        };
        let Some(key) = device.public_key() else {
            return failed();
        };
        let body = serde_json::json!({
            "code": code,
            "device": device.device,
            "key": key,
            "name": core::suggested_device_name(),
        });
        match core::fetch_json("POST", "/auth/guest", Some(&body)).await {
            Ok(answer) => {
                let (Some(user), Some(incident)) =
                    (answer["user"].as_str(), answer["incident"].as_str())
                else {
                    return failed();
                };
                let mut paired = device.paired(user);
                paired.incident = incident.to_owned();
                core::save_pairing(&paired);
                pairing.set(Some(paired));
            }
            Err(_) => failed(),
        }
    });
    view! {
        <main id="main" class="page page--narrow">
            <h1>{text::app_name()} " – " {text::guest_pair_title()}</h1>
            {move || match error.get() {
                None => view! { <p role="status">{text::guest_pairing()}</p> }.into_any(),
                Some(e) => view! { <p role="alert" class="error">{e}</p> }.into_any(),
            }}
        </main>
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
    let error = RwSignal::new(login_error.map(|e| format!("{}: {e}", text::login_failed())));
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
                {text::app_name()} " – "
                {if is_relogin { text::relogin_title() } else { text::pair_title() }}
            </h1>
            <p>{if is_relogin { text::relogin_intro() } else { text::pair_intro() }}</p>
            <p role="alert" class="error">{move || error.get()}</p>
            {move || match login_enabled.get() {
                None => ().into_any(),
                Some(false) => view! { <p class="notice">{text::pair_no_login()}</p> }.into_any(),
                Some(true) => view! {
                    {(!is_relogin).then(|| view! {
                        <div class="field">
                            <label for="device-name">{text::device_name_label()}</label>
                            <input id="device-name" type="text" autocomplete="off" maxlength="60" bind:value=device_name />
                            <p class="hint">{text::device_name_hint()}</p>
                        </div>
                    })}
                    <div class="row">
                        <button id="login" type="button" class="button button--primary" on:click=login>{text::pair_login()}</button>
                        <button id="pair-qr" type="button" class="button" on:click=start_qr>{text::pair_qr()}</button>
                    </div>
                    <p class="hint">{text::pair_qr_hint()}</p>
                }.into_any(),
            }}
            {move || qr.get().map(|(id, code, uri)| view! {
                <section class="card pairing-qr" aria-labelledby="pair-qr-title">
                    <h2 id="pair-qr-title">{text::pair_qr()}</h2>
                    <img class="pairing-qr__code" src=format!("/auth/device/{id}/qr.svg") alt=text::pair_qr_alt() width="256" height="256" />
                    <p class="pairing-qr__user-code code">{text::pair_code(&code)}</p>
                    <p>{text::pair_qr_link()} " " <a id="pair-qr-link" href=uri.clone()>{uri.clone()}</a></p>
                    <p role="status">{text::pair_waiting()}</p>
                </section>
            })}
            {if is_relogin {
                view! {
                    <button type="button" class="button" on:click=move |_| {
                        core::forget_device();
                        pairing.set(None);
                    }>{text::unpair()}</button>
                }
                .into_any()
            } else {
                view! {
                    <details class="manual-pairing">
                        <summary>{text::pair_manual()}</summary>
                        <ManualPairing pairing incident />
                    </details>
                }
                .into_any()
            }}
            <crate::admin::notices::NoticesLink />
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
        <p>{text::pair_manual_intro()}</p>
        <form on:submit=submit class="form">
            <label for="user">{text::pair_user()}</label>
            <input id="user" type="text" autocomplete="username" required bind:value=user />
            <label for="device">{text::pair_device()}</label>
            <input id="device" type="text" autocomplete="off" spellcheck="false" required bind:value=device />
            <label for="key">{text::pair_key()}</label>
            <input id="key" type="password" autocomplete="off" spellcheck="false" required bind:value=key />
            <label for="incident">{text::pair_incident()}</label>
            <input id="incident" type="text" spellcheck="false" bind:value=incident />
            <p role="alert" class="error">{move || error.get()}</p>
            <button type="submit" class="button">{text::pair_submit()}</button>
        </form>
    }
}

/// Whether the user may open `workspace` on this device, and if not, why not.
pub(crate) fn access(core: Handle, workspace: &nav::Workspace) -> Result<(), &'static str> {
    core.with_value(|c| {
        if !c.replica.is_ready() {
            return Err(text::reason_not_connected());
        }
        let readable = workspace.requires.iter().all(|comp| {
            c.incident
                .as_ref()
                .is_some_and(|i| c.replica.is_granted(i, comp))
        });
        // [impl->dsn~views-by-role~1]
        // Views follow the roles in this incident (decided by the server's policies).
        let by_incident = match workspace.permission {
            Some(p) => c.may(p),
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
            Err(text::reason_no_access())
        }
    })
}

/// Whether the user may open view `v` of a workspace they have access to: a view's own
/// permission comes from the roles in this incident or an organisation role (ADR-0061).
pub(crate) fn view_allowed(core: Handle, v: &nav::View) -> bool {
    v.permission
        .is_none_or(|p| core.with_value(|c| c.replica.has_permission(p) || c.may(p)))
        && core.with_value(|c| {
            v.requires.iter().all(|comp| {
                c.incident
                    .as_ref()
                    .is_some_and(|i| c.replica.is_granted(i, comp))
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

/// Shows a colour theme without remembering it (the component gallery's switch).
pub(crate) fn show_theme(theme: &str) {
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
}

/// The colour theme this device remembers (`""` = follow the system).
pub(crate) fn stored_theme() -> String {
    core::preference(THEME_KEY).unwrap_or_default()
}

// [impl->dsn~header-layout~2]
/// Applies the handedness (`""` = right, `"left"`) and remembers it on this device: the
/// bottom bar puts the menu at the end the thumb reaches.
pub(crate) fn apply_hand(hand: &str) {
    apply_setting("data-hand", HAND_KEY, hand);
}

// [impl->dsn~vim-keys~3]
/// Turns vim keys on (`"vim"`) or off (`""`) and remembers it on this device (ADR-0090).
pub(crate) fn apply_keymap(keymap: &str) {
    apply_setting("data-keymap", crate::shell::keys::KEYMAP_KEY, keymap);
}

// [impl->dsn~page-tabs~4]
/// A page in tabs: the tab list, then a panel per tab with its sections.
fn page_with_tabs(
    view: &'static str,
    declared: &'static nav::View,
    page: Vec<crate::shell::registry::Section>,
    chosen: RwSignal<&'static str>,
) -> impl IntoView {
    use crate::shell::ui::{Part, Parts};
    let mut sections: Vec<(&'static str, Vec<AnyView>)> =
        declared.tabs.iter().map(|t| (t.key, Vec::new())).collect();
    for (place, section) in page {
        let key = crate::shell::registry::tab_of(declared.tabs, place);
        if let Some(part) = sections.iter_mut().find(|(k, _)| Some(*k) == key) {
            part.1.push(section);
        }
    }
    // A tab without content (a package not in use, no permission) is left out.
    let parts: Vec<Part> = declared
        .tabs
        .iter()
        .zip(sections)
        .filter(|(_, (_, content))| !content.is_empty())
        .map(|(tab, (key, content))| Part {
            key,
            label: (tab.label)().to_owned(),
            short: tab.short.map(|s| s().to_owned()),
            column: tab.column,
            content: view! { <div class="page__tab">{content}</div> }.into_any(),
        })
        .collect();
    if !parts.iter().any(|p| p.key == chosen.get_untracked())
        && let Some(first) = parts.first()
    {
        chosen.set(first.key);
    }
    view! {
        <Parts id=format!("page-{view}") label=text::page_tabs() parts chosen mode=declared.columns />
    }
}

/// The content of both menu buttons (header and bottom bar).
#[component]
fn MenuLabel() -> impl IntoView {
    view! {
        <Icon name="layout-grid" />
        <span class="launcher__text">{text::menu()}</span>
        <span class="visually-hidden">" – " {text::launcher()}</span>
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

    /// Navigation that goes nowhere: for the component gallery's sample links.
    pub fn inert() -> Self {
        Self {
            navigate: StoredValue::new_local(Box::new(|_: &str, _: NavigateOptions| {})),
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
    provide_context(crate::shell::profile::Revision(revision));
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
    // [impl->dsn~guest-access~4]
    // A guest cannot log in again: their access ended with their duty (ADR-0077).
    let guest = pairing
        .user
        .parse::<ordo_core::UserId>()
        .is_ok_and(|u| u.is_guest());
    let relogin = StoredValue::new(pairing);
    view! {
        <Show
            when=move || !revoked.get()
            fallback=move || view! { <Revoked on_pair_again=unpair /> }
        >
            <Show
                when=move || !locked.get()
                fallback=move || if guest {
                    view! {
                        <main id="main" class="page page--narrow">
                            <h1>{text::app_name()} " – " {text::guest_ended_title()}</h1>
                            <p>{text::guest_ended()}</p>
                        </main>
                    }.into_any()
                } else {
                    view! { <PairingForm pairing=paired login_error=None relogin=relogin.get_value() /> }.into_any()
                }
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
                <h1 id="revoked-title" tabindex="-1">{text::revoked_title()}</h1>
                <p role="alert" class="notice notice--warning">{text::revoked_text()}</p>
                <button type="button" class="button button--primary" on:click=move |_| on_pair_again()>
                    {text::pair_again()}
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
    let undo = crate::shell::ui::UndoSlot(RwSignal::new(None));
    crate::shell::registry::provide();
    let open = RwSignal::new(false);
    let palette = RwSignal::new(false);
    let pathname = use_location().pathname;
    let saved = StoredValue::new(core::preference(NAV_KEY).and_then(|s| {
        let (w, v) = s.split_once('/')?;
        Some(nav::Route::view(w, v))
    }));
    // The workspace shown last, for a patient's own address.
    let last = RwSignal::new(saved.with_value(|s| s.as_ref().map(|r| r.workspace.clone())));
    // [impl->dsn~patient-address~1]
    // `/patient/<tag>` opens the record in the workspace the person works in, if it shows
    // patients, else in the first one that does (ADR-0093); the address stays.
    let route = Memo::new(move |_| {
        let route = nav::Route::from_path(&pathname.get())?;
        if !route.is_patient() {
            return Some(route);
        }
        revision.track();
        let shows = |w: &&'static nav::Workspace| {
            access(core, w).is_ok() && allowed_views(core, w).iter().any(|v| v.id == "patients")
        };
        let workspace = last
            .get_untracked()
            .and_then(|id| nav::workspace(&id))
            .filter(shows)
            .or_else(|| nav::workspaces().find(shows))?;
        Some(nav::Route::record(
            workspace.id,
            "patients",
            route.record.as_deref()?,
        ))
    });

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
                nav::workspaces().find(|w| !w.requires.is_empty() && access(core, w).is_ok())
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
        last.set(Some(w.to_owned()));
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
        Some(w) => format!("{} – {}", (w.package)(), (w.label)()),
        None => text::app_name().to_owned(),
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
    let key_help = RwSignal::new(false);
    let vim = StoredValue::new(crate::shell::keys::Keys::default());
    let shell = crate::shell::keys::Shell {
        palette,
        menu: open,
        help: key_help,
    };
    let keys = window_event_listener(leptos::ev::keydown, move |ev| {
        if (ev.ctrl_key() || ev.meta_key()) && !ev.alt_key() && ev.key().eq_ignore_ascii_case("k") {
            ev.prevent_default();
            if !palette.get_untracked() {
                open.set(false);
                profile_open.set(false);
                palette.set(true);
            }
        } else if crate::shell::keys::handle(&ev, vim, shell) {
            // Vim keys, where turned on (ADR-0090, ADR-0091): before the list keys.
        } else if crate::shell::ui::move_in_list(&ev) {
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
    crate::shell::ui::close_on_click_outside(
        open,
        "#launcher-button, #launcher-bar-button, #launcher-panel".to_owned(),
    );
    let theme = RwSignal::new(core::preference(THEME_KEY).unwrap_or_default());
    let hand = RwSignal::new(core::preference(HAND_KEY).unwrap_or_default());
    let keymap =
        RwSignal::new(core::preference(crate::shell::keys::KEYMAP_KEY).unwrap_or_default());
    let close_profile = Callback::new(move |()| profile_open.set(false));
    // The person's roles in this incident, shown in the profile menu.
    let my_roles = {
        let me = me.clone();
        move || {
            revision.track();
            core.with_value(|c| c.incident_info().and_then(|i| i.roster.get(&me).cloned()))
                .filter(|roles| !roles.is_empty())
                .map_or_else(
                    || text::no_role().to_owned(),
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
        }>{text::skip_link()}</a>
        <header class="app-header">
            <div class="app-header__start">
                <DisclosureButton id="launcher-button" controls="launcher-panel" open
                    class="launcher__button launcher__button--header" on_open=Some(close_profile)>
                    <MenuLabel />
                </DisclosureButton>
                <div class="app-header__heading">
                    <h1 class="app-header__title">{title}</h1>
                    <p class="app-header__incident">
                        <span>{text::incident()} ": "</span>
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
                <span>{text::search()}</span>
                <kbd class="search-button__keys" aria-hidden="true">{shortcut}</kbd>
            </button>
            <div class="app-header__end">
                <crate::shell::attention::AttentionBell core revision on_open=Callback::new(move |()| { open.set(false); profile_open.set(false); }) />
                <ConnectionStatus core revision />
                <Disclosure id="profile" open=profile_open class="profile-menu"
                    button_class="profile-menu__button" panel_class="profile-menu__panel"
                    on_open=Some(Callback::new(move |()| open.set(false)))
                    button=Box::new({
                        let me = me.clone();
                        move || view! {
                            <crate::shell::profile::Avatar core revision user=me.clone() />
                            <span class="visually-hidden">{text::signed_in_as()} " "</span>
                            <span class="profile-menu__name">{move || {
                                revision.track();
                                crate::shell::profile::display_name(core, &me)
                            }}</span>
                        }.into_any()
                    })>
                    <h2 class="launcher__title">{text::profile()}</h2>
                    <p class="launcher__user">{text::signed_in_as()} " " <strong>{{
                        let user = user.clone();
                        move || {
                            revision.track();
                            user.parse::<ordo_core::UserId>().map_or_else(|_| user.clone(), |id| crate::shell::profile::display_name(core, &id))
                        }
                    }}</strong> " (" {user.clone()} ")"</p>
                    <p id="my-roles">{text::my_roles()} ": " {my_roles}</p>
                    <crate::shell::profile::ProfilePicture core revision />
                    <crate::shell::profile::DisplayName core revision />
                    <SettingSelect id="theme" label=text::appearance() options=&*text::THEMES value=theme apply=apply_theme />
                    <SettingSelect id="hand" label=text::hand() options=&*text::HANDS value=hand apply=apply_hand />
                    <SettingSelect id="keymap" label=text::keymap() options=&*text::KEYMAPS value=keymap apply=apply_keymap />
                    <LanguageSelect core />
                    <button id="keys-button" type="button" class="button button--with-icon" aria-haspopup="dialog"
                        on:click=move |_| { profile_open.set(false); key_help.set(true); }>
                        <Icon name="keyboard" />{text::keys_title()}
                    </button>
                    <a id="my-devices" class="button" href=nav::Route::view("person.devices", "devices").to_path()
                        on:click=follow(nav, nav::Route::view("person.devices", "devices"))>{text::my_devices()}</a>
                    <button id="change-incident" type="button" class="button"
                        on:click=move |_| on_change_incident.with_value(|f| f())>{text::change_incident()}</button>
                    <button type="button" class="button" on:click=unpair>{text::unpair()}</button>
                </Disclosure>
            </div>
        </header>
        <div class="layout">
            <div class="navigation">
              <div class="navigation__inner">
                // [impl->dsn~record-overview~3]
                // On phones a page may put its own actions here, above the views, within
                // thumb reach (the patient record's quick actions); empty elsewhere.
                <div class="launcher">
                    <DisclosureButton id="launcher-bar-button" controls="launcher-panel" open
                        class="launcher__button launcher__button--bar" on_open=Some(close_profile)>
                        <MenuLabel />
                    </DisclosureButton>
                    <div id="launcher-panel" class="launcher__panel" hidden=move || !open.get()>
                        <h2 class="launcher__title">{text::launcher()}</h2>
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
                <nav class="views" aria-label=text::views()>
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
                                                // [impl->dsn~workspaces-by-phase~5]
                                                // The bar shows the short label where there is one.
                                                <span class="views__label" class:views__label--long=v.short.is_some()>{(v.label)()}</span>
                                                {v.short.map(|short| view! { <span class="views__label views__label--short">{short()}</span> })}
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
                // [impl->dsn~package-registry~1]
                // Every package's sections of the page, in their places.
                {move || {
                    let (Some(workspace), Some(view)) = (current.get(), current_view.get()) else {
                        return view! { <p class="notice">{text::no_workspace()}</p> }.into_any();
                    };
                    let args = crate::shell::registry::PageArgs { core, revision, nav, record };
                    let declared = nav::workspace(workspace)
                        .and_then(|w| w.views.iter().find(|v| v.id == view));
                    let tabs = declared.map_or(&[][..], |v| v.tabs);
                    // [impl->dsn~page-tabs~4]
                    // The chosen part, for the sections to show another one (a context).
                    let chosen = RwSignal::new(tabs.first().map_or("", |t| t.key));
                    provide_context(crate::shell::ui::PageTabs(chosen));
                    let page = crate::shell::registry::page(&args, workspace, view);
                    if page.is_empty() {
                        return view! { <p class="notice">{text::no_workspace()}</p> }.into_any();
                    }
                    let Some(declared) = declared.filter(|_| !tabs.is_empty()) else {
                        return page.into_iter().map(|(_, v)| v).collect_view().into_any();
                    };
                    page_with_tabs(view, declared, page, chosen).into_any()
                }}
            </main>
            <crate::shell::ui::UndoBar offer=undo />
        </div>
        <palette::Palette core revision nav open=palette theme />
        <crate::shell::keys::KeyHelp open=key_help keymap />
        <p class="visually-hidden" role="status" aria-live="polite">
            {move || {
                revision.track();
                core.with_value(|c| c.announcement.clone())
            }}
        </p>
    }
    .into_any()
}

/// The element with `id`, once it shows: inside a hidden tab panel, its tab is chosen
/// first and `f` runs a moment later ([`crate::shell::ui::TabPanel`]).
fn when_shown(id: &str, f: impl Fn(web_sys::HtmlElement) + 'static) {
    use wasm_bindgen::JsCast;
    let find = {
        let id = id.to_owned();
        move || {
            web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id(&id))
                .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        }
    };
    let Some(el) = find() else { return };
    // [impl->dsn~record-overview~3]
    // [impl->dsn~page-tabs~4]
    // Inside a hidden part (a tab not chosen): that tab shows first.
    let hidden = el.offset_parent().is_none();
    let tab = el
        .closest("[role=tabpanel][hidden], .parts__part:not(.parts__part--chosen)")
        .ok()
        .flatten()
        .filter(|_| hidden)
        .and_then(|panel| panel.get_attribute("aria-labelledby"))
        .and_then(|tab| {
            web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id(&tab))
                .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        });
    match tab {
        Some(tab) => {
            tab.click();
            after_render(move || {
                if let Some(el) = find() {
                    f(el);
                }
            });
        }
        None => f(el),
    }
}

/// Moves keyboard focus to the element with `id` (after navigation).
/// Call inside a closure that rebuilds part of a page (a list after a new event): the
/// element that has the focus, if it has an ID, gets it back once the part is rebuilt, so
/// nobody typing or tabbing there loses their place.
pub(crate) fn keep_focus() {
    use wasm_bindgen::JsCast;
    let document = || web_sys::window().and_then(|w| w.document());
    let Some(id) = document()
        .and_then(|d| d.active_element())
        .map(|e| e.id())
        .filter(|id| !id.is_empty())
    else {
        return;
    };
    request_animation_frame(move || {
        let Some(d) = document() else { return };
        // Only focus that was lost (to the page itself) comes back.
        let body = d.body().map(web_sys::Element::from);
        if d.active_element()
            .is_some_and(|e| Some(&e) != body.as_ref())
        {
            return;
        }
        if let Some(el) = d
            .get_element_by_id(&id)
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = el.focus();
        }
    });
}

pub(crate) fn focus(id: &str) {
    when_shown(id, |el| {
        let _ = el.focus();
    });
}

/// Opens the collapsed inputs of a record section (`{section}-open`), if closed, and
/// moves the focus into them.
pub(crate) fn open_inputs(section: &str) {
    let panel = format!("{section}-inputs");
    when_shown(&format!("{section}-open"), move |button| {
        if button.get_attribute("aria-expanded").as_deref() != Some("true") {
            button.click();
        }
        let panel = panel.clone();
        after_render(move || {
            use wasm_bindgen::JsCast;
            if let Some(field) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| {
                    d.query_selector(&format!("#{panel} :is(input, select, textarea, button)"))
                        .ok()
                        .flatten()
                })
                .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
            {
                let _ = field.focus();
            }
        });
    });
}

/// Clicks the element with `id` (e.g. to open an editor after navigation).
pub(crate) fn click(id: &str) {
    when_shown(id, |el| el.click());
}

/// Connection and outbox state, as text (OFF-5).
// [impl->dsn~compact-navigation~2]
// [impl->dsn~ui-texts~2]
/// The device's language (I18N-1): "Automatisch" follows the browser; a shared tablet keeps
/// the language chosen here. Each language is named in itself. The page reloads to show it,
/// unless entries could not be stored: then it applies at the next start.
#[component]
fn LanguageSelect(core: Handle) -> impl IntoView {
    let value = RwSignal::new(core::preference(crate::i18n::PREFERENCE).unwrap_or_default());
    let later = RwSignal::new(false);
    let options: Vec<(&'static str, String)> =
        std::iter::once(("", text::language_auto().to_owned()))
            .chain(crate::i18n::languages().into_iter().map(|l| {
                // The language shown: its name from the texts already loaded.
                let name = if l == crate::i18n::language() {
                    text::language_name().to_owned()
                } else {
                    crate::i18n::name_of(l)
                };
                (l, name)
            }))
            .collect();
    let change = move |ev: leptos::ev::Event| {
        let chosen = event_target_value(&ev);
        core::set_preference(
            crate::i18n::PREFERENCE,
            (!chosen.is_empty()).then_some(chosen.as_str()),
        );
        value.set(chosen);
        if core.with_value(|c| c.storage_failed) {
            later.set(true);
        } else if let Some(w) = web_sys::window() {
            let _ = w.location().reload();
        }
    };
    view! {
        <label for="language">{text::language()}</label>
        <select id="language" prop:value=move || value.get() on:change=change>
            {options
                .into_iter()
                .map(|(v, name)| view! { <option value=v lang=(!v.is_empty()).then_some(v)>{name}</option> })
                .collect_view()}
        </select>
        <p class="hint" role="status">{move || later.get().then(text::language_after_reload)}</p>
    }
}

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
        Connection::Online => text::online(),
        Connection::Connecting => text::connecting(),
        Connection::Offline => text::offline(),
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
        Signal::derive(move || text::connection_label(words(), text::pending(state.get().1)));
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
        {move || state.get().3.then(|| view! { <p role="alert" class="error connection__error">{text::storage_failed()}</p> })}
    }
}
