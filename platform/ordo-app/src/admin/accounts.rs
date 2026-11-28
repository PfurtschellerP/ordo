//! Organisation workspace: local login accounts (ORG-10, ADR-0060). Administrators create
//! an account with a first password, set a new password, or delete it; the server makes
//! the change in Dex. Deleting asks in place, as revoking a device does (ADR-0029).

use leptos::prelude::*;
use ordo_sync::message::{Account, AccountChange, password_fits};

use crate::admin::text;
use crate::core::{self, Handle};
use crate::shell::views::{after_render, focus};

/// The change waiting for its answer: one at a time, so each answer belongs to it.
#[derive(Clone, Copy)]
struct Pending {
    /// Answers seen when the change was sent; `None` while no change waits.
    since: RwSignal<Option<u64>>,
    /// Called when the answer says it was done.
    on_done: StoredValue<Option<Box<dyn Fn() + Send + Sync>>>,
    /// What the last change came to, shown until the next one is sent.
    outcome: RwSignal<Option<&'static str>>,
}

impl Pending {
    /// Whether a change waits for its answer (the actions are disabled meanwhile).
    fn busy(self) -> bool {
        self.since.get().is_some()
    }

    fn send(self, core: Handle, change: AccountChange, on_done: impl Fn() + Send + Sync + 'static) {
        if self.since.get_untracked().is_some() {
            return;
        }
        self.outcome.set(None);
        self.since.set(Some(core.with_value(|c| c.account_answers)));
        self.on_done.set_value(Some(Box::new(on_done)));
        core::change_account(core, change);
    }
}

// [impl->dsn~local-accounts~1]
/// The accounts, their actions, and the form for a new one.
#[component]
pub fn AccountsView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
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
            core::get_accounts(core);
        }
    });
    let accounts = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.accounts.clone())
    });
    let answers = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.account_answers)
    });
    // The outcome of the last change: shown once its answer arrives.
    let outcome = RwSignal::new(None::<&'static str>);
    let pending = Pending {
        since: RwSignal::new(None),
        on_done: StoredValue::new(None),
        outcome,
    };
    let failed = RwSignal::new(false);
    Effect::new(move |_| {
        let seen = answers.get();
        if let Some(since) = pending.since.get_untracked()
            && seen > since
        {
            pending.since.set(None);
            let error = accounts.get_untracked().and_then(|a| a.error);
            failed.set(error.is_some());
            outcome.set(Some(
                error.map_or(text::account_done(), text::account_error),
            ));
            if error.is_none()
                && let Some(done) = pending.on_done.try_update_value(Option::take).flatten()
            {
                done();
            }
        }
    });
    let resetting = RwSignal::new(None::<String>);
    let deleting = RwSignal::new(None::<String>);
    view! {
        <section aria-labelledby="accounts-title" class="card">
            // [impl->dsn~building-blocks~1]
            // "+ Konto" at the card's head, the form in the dialog (ADR-0102).
            <div class="section-head">
                <h2 id="accounts-title" class="section-head__title" tabindex="-1">{text::accounts_title()}</h2>
                <Show when=move || accounts.get().is_some_and(|a| a.managed)>
                    <div class="section-head__action">
                        <crate::shell::ui::CreateButton id="account-new" word=text::account_new_word() title=text::account_create_title()>
                            <div class="edit-dialog__part"><CreateForm core pending /></div>
                        </crate::shell::ui::CreateButton>
                    </div>
                </Show>
            </div>
            <p>{text::accounts_intro()}</p>
            <p role="status" class="notice" hidden=move || failed.get() || outcome.get().is_none()>
                {move || outcome.get().filter(|_| !failed.get())}
            </p>
            <p role="alert" class="error">{move || outcome.get().filter(|_| failed.get())}</p>
            {move || {
                if !ready.get() {
                    return view! { <p class="notice">{text::accounts_offline()}</p> }.into_any();
                }
                match accounts.get() {
                    None => view! { <p>{text::accounts_loading()}</p> }.into_any(),
                    Some(a) if !a.managed => view! { <p class="notice">{text::accounts_not_managed()}</p> }.into_any(),
                    Some(a) if a.list.is_empty() => view! { <p>{text::accounts_empty()}</p> }.into_any(),
                    Some(a) => view! {
                        <ul class="roster account-list" data-keys="list">
                            {a.list
                                .into_iter()
                                .map(|account| view! { <AccountItem core account pending resetting deleting /> })
                                .collect_view()}
                        </ul>
                    }
                    .into_any(),
                }
            }}
        </section>
    }
}

/// One account: name, login name, new password, delete.
#[component]
fn AccountItem(
    core: Handle,
    account: Account,
    pending: Pending,
    resetting: RwSignal<Option<String>>,
    deleting: RwSignal<Option<String>>,
) -> impl IntoView {
    let Account { email, name } = account;
    let key = core::slug(&email);
    let (reset_id, password_id, delete_id, confirm_id, question_id) = (
        format!("account-reset-{key}"),
        format!("account-password-{key}"),
        format!("account-delete-{key}"),
        format!("account-confirm-{key}"),
        format!("account-question-{key}"),
    );
    let password = RwSignal::new(String::new());
    let short = RwSignal::new(false);
    let is_resetting = {
        let email = email.clone();
        move || resetting.get().as_deref() == Some(email.as_str())
    };
    let is_deleting = {
        let email = email.clone();
        move || deleting.get().as_deref() == Some(email.as_str())
    };
    let save = {
        let email = email.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let value = password.get_untracked();
            if !password_fits(&value) {
                short.set(true);
                return;
            }
            short.set(false);
            password.set(String::new());
            pending.send(
                core,
                AccountChange::SetPassword {
                    email: email.clone(),
                    password: value,
                },
                move || resetting.set(None),
            );
        }
    };
    let delete = {
        let email = email.clone();
        move |_| {
            deleting.set(None);
            pending.send(
                core,
                AccountChange::Delete {
                    email: email.clone(),
                },
                || (),
            );
            after_render(|| focus("accounts-title"));
        }
    };
    let question = text::account_delete_question(&name, &email);
    let (reset_label, delete_label) = (
        text::account_set_password_label(&email),
        text::account_delete_label(&email),
    );
    view! {
        <li class="roster__entry" data-account=email.clone()>
            <span class="roster__person">{name.clone()}</span>
            <span class="roster__id">{email.clone()}</span>
            <div class="row">
                <button id=reset_id.clone() type="button" class="button button--small" aria-label=reset_label disabled=move || pending.busy()
                    aria-expanded={
                        let is_resetting = is_resetting.clone();
                        move || is_resetting().to_string()
                    }
                    on:click={
                        let (email, password_id) = (email.clone(), password_id.clone());
                        move |_| {
                            resetting.set(Some(email.clone()));
                            let password_id = password_id.clone();
                            after_render(move || focus(&password_id));
                        }
                    }>
                    {text::account_set_password()}
                </button>
                <button id=delete_id.clone() type="button" class="button button--small" aria-label=delete_label disabled=move || pending.busy()
                    on:click={
                        let (email, confirm_id) = (email.clone(), confirm_id.clone());
                        move |_| {
                            deleting.set(Some(email.clone()));
                            let confirm_id = confirm_id.clone();
                            after_render(move || focus(&confirm_id));
                        }
                    }>
                    {text::account_delete()}
                </button>
            </div>
            <Show when=is_resetting.clone()>
                <form class="form account-password" novalidate on:submit=save.clone()>
                    <div class="field">
                        <label for=password_id.clone()>{text::account_new_password()}</label>
                        <input id=password_id.clone() type="password" autocomplete="new-password"
                            aria-describedby=format!("{password_id}-hint") bind:value=password />
                        <p id=format!("{password_id}-hint") class="hint">{text::account_password_hint()}</p>
                        <p role="alert" class="error">{move || short.get().then_some(text::account_password_length())}</p>
                    </div>
                    <div class="row">
                        <button type="submit" class="button button--primary" disabled=move || pending.busy()>{text::account_save_password()}</button>
                        <button type="button" class="button" on:click={
                            let reset_id = reset_id.clone();
                            move |_| {
                                resetting.set(None);
                                let reset_id = reset_id.clone();
                                after_render(move || focus(&reset_id));
                            }
                        }>{text::cancel()}</button>
                    </div>
                </form>
            </Show>
            <Show when=is_deleting.clone()>
                <div class="device-card__confirm" role="group" aria-labelledby=question_id.clone()>
                    <p id=question_id.clone() class="notice notice--warning">{question.clone()}</p>
                    <div class="device-card__buttons">
                        <button id=confirm_id.clone() type="button" class="button button--danger" disabled=move || pending.busy() on:click=delete.clone()>
                            {text::account_delete_confirm()}
                        </button>
                        <button type="button" class="button" on:click={
                            let delete_id = delete_id.clone();
                            move |_| {
                                deleting.set(None);
                                let delete_id = delete_id.clone();
                                after_render(move || focus(&delete_id));
                            }
                        }>{text::cancel()}</button>
                    </div>
                </div>
            </Show>
        </li>
    }
}

/// A new account: login name, name, first password.
#[component]
fn CreateForm(core: Handle, pending: Pending) -> impl IntoView {
    let email = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(None::<&'static str>);
    let created = use_context::<crate::shell::ui::CreateOpen>();
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let (e, n, p) = (
            email.get_untracked(),
            name.get_untracked(),
            password.get_untracked(),
        );
        if !e.contains('@') || n.trim().is_empty() {
            error.set(Some(text::account_invalid()));
            focus(if e.contains('@') {
                "account-name"
            } else {
                "account-email"
            });
            return;
        }
        if !password_fits(&p) {
            error.set(Some(text::account_password_length()));
            focus("account-password");
            return;
        }
        error.set(None);
        // The server's answer shows on the page, at the list; what was typed stays in the
        // form until the account is made.
        crate::shell::ui::CreateOpen::done(created);
        pending.send(
            core,
            AccountChange::Create {
                email: e.trim().to_owned(),
                name: n.trim().to_owned(),
                password: p,
            },
            move || {
                email.set(String::new());
                name.set(String::new());
                password.set(String::new());
            },
        );
    };
    view! {
        <form class="form" novalidate on:submit=submit aria-labelledby="account-new-title">
            <div class="form--inline">
                <div class="field">
                    <label for="account-email">{text::account_email()}</label>
                    <input id="account-email" type="email" autocomplete="off" bind:value=email />
                </div>
                <div class="field">
                    <label for="account-name">{text::account_name()}</label>
                    <input id="account-name" type="text" autocomplete="off" bind:value=name />
                </div>
                <div class="field">
                    <label for="account-password">{text::account_password()}</label>
                    <input id="account-password" type="password" autocomplete="new-password"
                        aria-describedby="account-password-hint" bind:value=password />
                </div>
            </div>
            <p id="account-password-hint" class="hint">{text::account_password_hint()}</p>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="row">
                <button type="submit" class="button button--primary" disabled=move || pending.busy()>{text::account_create()}</button>
            </div>
        </form>
    }
}
