//! Administration: upstream login providers (ORG-11, ADR-0063). Administrators connect the
//! organisation's OpenID Connect provider to Dex: they enter it, log in once through it in
//! a new browser tab, see the identity ORDO would take, and only then save it. Deleting
//! asks in place, as for accounts.

use leptos::prelude::*;
use ordo_sync::message::{
    ClientMessage, LoginProvider, LoginProviderDraft, LoginProviders, LoginTest, LoginTestState,
};

use crate::core::{self, Handle};
use crate::text;
use crate::views::{after_render, focus};

/// The change waiting for its answer, as in the accounts view.
#[derive(Clone, Copy)]
struct Pending {
    since: RwSignal<Option<u64>>,
    on_done: StoredValue<Option<Box<dyn Fn() + Send + Sync>>>,
    /// What to say when it was done (saved, removed), if anything.
    success: StoredValue<Option<&'static str>>,
    outcome: RwSignal<Option<&'static str>>,
}

impl Pending {
    fn busy(self) -> bool {
        self.since.get().is_some()
    }

    fn send(
        self,
        core: Handle,
        message: ClientMessage,
        success: Option<&'static str>,
        on_done: impl Fn() + Send + Sync + 'static,
    ) {
        if self.since.get_untracked().is_some() {
            return;
        }
        self.outcome.set(None);
        self.success.set_value(success);
        self.since
            .set(Some(core.with_value(|c| c.provider_answers)));
        self.on_done.set_value(Some(Box::new(on_done)));
        core::login_providers(core, message);
    }
}

// [impl->dsn~login-providers~1]
/// The providers, the form for a new or changed one, and its test login.
#[component]
pub fn LoginProvidersView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let ready = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.replica.is_ready())
    });
    let welcomes = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.welcomes)
    });
    Effect::new(move |_| {
        if ready.get() && welcomes.get() > 0 {
            core::login_providers(core, ClientMessage::GetLoginProviders);
        }
    });
    let providers = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.login_providers.clone())
    });
    let answers = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.provider_answers)
    });
    let outcome = RwSignal::new(None::<&'static str>);
    let pending = Pending {
        since: RwSignal::new(None),
        on_done: StoredValue::new(None),
        success: StoredValue::new(None),
        outcome,
    };
    let failed = RwSignal::new(false);
    Effect::new(move |_| {
        let seen = answers.get();
        if let Some(since) = pending.since.get_untracked()
            && seen > since
        {
            pending.since.set(None);
            let error = providers.get_untracked().and_then(|p| p.error);
            failed.set(error.is_some());
            outcome.set(
                error
                    .map(text::provider_error)
                    .or_else(|| pending.success.get_value()),
            );
            if error.is_none()
                && let Some(done) = pending.on_done.try_update_value(Option::take).flatten()
            {
                done();
            }
        }
    });
    // While a test login waits for the browser tab, ask again now and then.
    let alive = StoredValue::new(true);
    on_cleanup(move || alive.set_value(false));
    leptos::task::spawn_local(async move {
        loop {
            core::sleep_ms(2_000).await;
            if !alive.try_get_value().unwrap_or(false) {
                return;
            }
            let waiting = providers
                .get_untracked()
                .is_some_and(|p| p.test.is_some_and(|t| t.state == LoginTestState::Waiting));
            if waiting && pending.since.get_untracked().is_none() {
                core::login_providers(core, ClientMessage::GetLoginProviders);
            }
        }
    });
    // The provider being changed (its ID), or a new one.
    let editing = RwSignal::new(None::<LoginProvider>);
    let deleting = RwSignal::new(None::<String>);
    view! {
        <section aria-labelledby="providers-title" class="card">
            <h2 id="providers-title" tabindex="-1">{text::PROVIDERS_TITLE}</h2>
            <p>{text::PROVIDERS_INTRO}</p>
            <p role="status" class="notice" hidden=move || failed.get() || outcome.get().is_none()>
                {move || outcome.get().filter(|_| !failed.get())}
            </p>
            <p role="alert" class="error">{move || outcome.get().filter(|_| failed.get())}</p>
            {move || {
                if !ready.get() {
                    return view! { <p class="notice">{text::ACCOUNTS_OFFLINE}</p> }.into_any();
                }
                match providers.get() {
                    None => view! { <p>{text::PROVIDERS_LOADING}</p> }.into_any(),
                    Some(p) if !p.managed => view! { <p class="notice">{text::PROVIDERS_NOT_MANAGED}</p> }.into_any(),
                    Some(p) => view! {
                        <ul class="roster provider-list" data-keys="list">
                            {p.list
                                .into_iter()
                                .map(|provider| view! { <ProviderItem core provider pending editing deleting /> })
                                .collect_view()}
                        </ul>
                    }
                    .into_any(),
                }
            }}
            <Show when=move || providers.get().is_some_and(|p| p.managed)>
                <ProviderForm core providers pending editing />
                {move || providers.get().and_then(|p| p.test).map(|test| view! { <TestPanel core test pending /> })}
            </Show>
        </section>
    }
}

/// One provider: name, kind, issuer; change and delete for those ORDO may change.
#[component]
fn ProviderItem(
    core: Handle,
    provider: LoginProvider,
    pending: Pending,
    editing: RwSignal<Option<LoginProvider>>,
    deleting: RwSignal<Option<String>>,
) -> impl IntoView {
    let id = provider.id.clone();
    let key = core::slug(&id);
    let (delete_id, confirm_id, question_id) = (
        format!("provider-delete-{key}"),
        format!("provider-confirm-{key}"),
        format!("provider-question-{key}"),
    );
    let is_deleting = {
        let id = id.clone();
        move || deleting.get().as_deref() == Some(id.as_str())
    };
    let detail = match (&provider.issuer, provider.kind.as_str()) {
        (Some(issuer), _) => issuer.clone(),
        (None, "local") => text::PROVIDER_LOCAL.to_owned(),
        (None, kind) => text::provider_kind(kind),
    };
    let delete = {
        let id = id.clone();
        move |_| {
            deleting.set(None);
            pending.send(
                core,
                ClientMessage::DeleteLoginProvider(id.clone()),
                Some(text::PROVIDER_REMOVED),
                || (),
            );
            after_render(|| focus("providers-title"));
        }
    };
    let editable = provider.editable;
    let (change_label, delete_label, question) = (
        text::provider_change_label(&provider.name),
        text::provider_delete_label(&provider.name),
        text::provider_delete_question(&provider.name),
    );
    view! {
        <li class="roster__entry" data-provider=id.clone()>
            <span class="roster__person">{provider.name.clone()}</span>
            <span class="roster__id">{detail}</span>
            {editable.then(|| view! {
                <div class="row">
                    <button type="button" class="button button--small" aria-label=change_label disabled=move || pending.busy()
                        on:click={
                            let provider = provider.clone();
                            move |_| {
                                editing.set(Some(provider.clone()));
                                after_render(|| focus("provider-name"));
                            }
                        }>
                        {text::PROVIDER_CHANGE}
                    </button>
                    <button id=delete_id.clone() type="button" class="button button--small" aria-label=delete_label disabled=move || pending.busy()
                        on:click={
                            let (id, confirm_id) = (id.clone(), confirm_id.clone());
                            move |_| {
                                deleting.set(Some(id.clone()));
                                let confirm_id = confirm_id.clone();
                                after_render(move || focus(&confirm_id));
                            }
                        }>
                        {text::PROVIDER_DELETE}
                    </button>
                </div>
            })}
            <Show when=is_deleting.clone()>
                <div class="device-card__confirm" role="group" aria-labelledby=question_id.clone()>
                    <p id=question_id.clone() class="notice notice--warning">{question.clone()}</p>
                    <div class="device-card__buttons">
                        <button id=confirm_id.clone() type="button" class="button button--danger" disabled=move || pending.busy() on:click=delete.clone()>
                            {text::PROVIDER_DELETE_CONFIRM}
                        </button>
                        <button type="button" class="button" on:click={
                            let delete_id = delete_id.clone();
                            move |_| {
                                deleting.set(None);
                                let delete_id = delete_id.clone();
                                after_render(move || focus(&delete_id));
                            }
                        }>{text::CANCEL}</button>
                    </div>
                </div>
            </Show>
        </li>
    }
}

/// A new provider, or a change of the one in `editing`: tested before it is saved.
#[component]
fn ProviderForm(
    core: Handle,
    providers: Memo<Option<LoginProviders>>,
    pending: Pending,
    editing: RwSignal<Option<LoginProvider>>,
) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let issuer = RwSignal::new(String::new());
    let client_id = RwSignal::new(String::new());
    let secret = RwSignal::new(String::new());
    let skip = RwSignal::new(false);
    let error = RwSignal::new(None::<&'static str>);
    // Changing one fills the form with what it has (never the secret).
    Effect::new(move |_| {
        let p = editing.get();
        name.set(p.as_ref().map(|p| p.name.clone()).unwrap_or_default());
        issuer.set(
            p.as_ref()
                .and_then(|p| p.issuer.clone())
                .unwrap_or_default(),
        );
        client_id.set(
            p.as_ref()
                .and_then(|p| p.client_id.clone())
                .unwrap_or_default(),
        );
        skip.set(p.as_ref().is_some_and(|p| p.skip_email_verified));
        secret.set(String::new());
    });
    let changing = move || editing.with(Option::is_some);
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let (n, i, c, s) = (
            name.get_untracked().trim().to_owned(),
            issuer.get_untracked().trim().to_owned(),
            client_id.get_untracked().trim().to_owned(),
            secret.get_untracked().trim().to_owned(),
        );
        let id = editing.get_untracked().map(|p| p.id);
        if n.is_empty() || !i.starts_with("http") || c.is_empty() || (s.is_empty() && id.is_none())
        {
            error.set(Some(text::PROVIDER_INVALID));
            return;
        }
        error.set(None);
        pending.send(
            core,
            ClientMessage::TestLoginProvider(LoginProviderDraft {
                id,
                name: n,
                issuer: i,
                client_id: c,
                client_secret: (!s.is_empty()).then_some(s),
                skip_email_verified: skip.get_untracked(),
            }),
            None,
            move || {
                secret.set(String::new());
                after_render(|| focus("provider-test-title"));
            },
        );
    };
    view! {
        <form class="form" novalidate on:submit=submit aria-labelledby="provider-form-title">
            <h3 id="provider-form-title">
                {move || editing.get().map_or_else(|| text::PROVIDER_NEW.to_owned(), |p| text::provider_change_title(&p.name))}
            </h3>
            <div class="form--inline">
                <div class="field">
                    <label for="provider-name">{text::PROVIDER_NAME}</label>
                    <input id="provider-name" type="text" autocomplete="off" aria-describedby="provider-name-hint" bind:value=name />
                    <p id="provider-name-hint" class="hint">{text::PROVIDER_NAME_HINT}</p>
                </div>
                <div class="field">
                    <label for="provider-issuer">{text::PROVIDER_ISSUER}</label>
                    <input id="provider-issuer" type="url" inputmode="url" autocomplete="off" aria-describedby="provider-issuer-hint" bind:value=issuer />
                    <p id="provider-issuer-hint" class="hint">{text::PROVIDER_ISSUER_HINT}</p>
                </div>
                <div class="field">
                    <label for="provider-client">{text::PROVIDER_CLIENT_ID}</label>
                    <input id="provider-client" type="text" autocomplete="off" bind:value=client_id />
                </div>
                <div class="field">
                    <label for="provider-secret">{text::PROVIDER_SECRET}</label>
                    <input id="provider-secret" type="password" autocomplete="off" aria-describedby="provider-secret-hint" bind:value=secret />
                    <p id="provider-secret-hint" class="hint">
                        {move || if changing() { text::PROVIDER_SECRET_KEEP } else { text::PROVIDER_SECRET_HINT }}
                    </p>
                </div>
            </div>
            <div class="field">
                <label class="choice chip">
                    <input id="provider-skip-email" type="checkbox" bind:checked=skip />
                    {text::PROVIDER_SKIP_EMAIL}
                </label>
            </div>
            <p class="hint">
                {text::PROVIDER_REDIRECT} " "
                <code id="provider-redirect">{move || providers.get().and_then(|p| p.redirect_uri)}</code>
            </p>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="row">
                <button type="submit" class="button button--primary" disabled=move || pending.busy()>{text::PROVIDER_TEST}</button>
                <Show when=changing>
                    <button type="button" class="button" on:click=move |_| editing.set(None)>{text::CANCEL}</button>
                </Show>
            </div>
        </form>
    }
}

/// The running test login: the link to open, then its result and saving.
#[component]
fn TestPanel(core: Handle, test: LoginTest, pending: Pending) -> impl IntoView {
    let confirmed = RwSignal::new(false);
    let LoginTest {
        token,
        url,
        name,
        state,
    } = test;
    let body = match state {
        LoginTestState::Waiting => view! {
            <p>{text::provider_test_waiting(&name)}</p>
            <p><a id="provider-test-link" class="button" href=url target="_blank" rel="noopener">{text::PROVIDER_TEST_OPEN}</a></p>
            <p class="hint">{text::PROVIDER_TEST_HINT}</p>
        }
        .into_any(),
        LoginTestState::Passed { user, name: person, email } => {
            let save = move |_| {
                pending.send(
                    core,
                    ClientMessage::SaveLoginProvider(token.clone()),
                    Some(text::PROVIDER_SAVED),
                    || after_render(|| focus("providers-title")),
                );
            };
            view! {
                <p class="notice" id="provider-test-result">{text::provider_test_passed(&user, person.as_deref(), email.as_deref())}</p>
                <div class="field">
                    <label class="choice chip">
                        <input id="provider-confirm" type="checkbox" bind:checked=confirmed />
                        {text::provider_confirm(&name)}
                    </label>
                </div>
                <div class="row">
                    <button type="button" class="button button--primary" disabled=move || !confirmed.get() || pending.busy() on:click=save>
                        {text::PROVIDER_SAVE}
                    </button>
                </div>
            }
            .into_any()
        }
        LoginTestState::Failed(error) => view! {
            <p role="alert" class="error" id="provider-test-result">{text::provider_test_failed(&error)}</p>
        }
        .into_any(),
    };
    view! {
        <section class="provider-test" aria-labelledby="provider-test-title">
            <h3 id="provider-test-title" tabindex="-1">{text::PROVIDER_TEST_TITLE}</h3>
            {body}
        </section>
    }
}
