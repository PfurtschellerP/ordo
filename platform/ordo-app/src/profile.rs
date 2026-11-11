//! Profile pictures (ADR-0047): shown next to people's names, set by each person for
//! themselves. The server resolves them: uploaded in ORDO, else from the identity provider,
//! else from Gravatar; without a picture the person's initials are shown.

use leptos::prelude::*;
use ordo_core::UserId;
use ordo_sync::message::MAX_AVATAR_BYTES;

use crate::core::{self, Handle};
use crate::text;

/// Profile pictures: longest edge and JPEG quality, tried in this order.
const AVATAR_STEPS: [(f64, f64); 3] = [(256.0, 0.8), (192.0, 0.7), (128.0, 0.6)];

/// Up to two initials of a user name, e.g. `anna.schmidt` → `AS`.
pub fn initials(user: &UserId) -> String {
    user.localpart()
        .split(['.', '-', '_'])
        .filter_map(|p| p.chars().next())
        .take(2)
        .flat_map(char::to_uppercase)
        .collect()
}

// [impl->dsn~display-names~2]
/// A person's display name (ORG-7), else their user ID. An unknown name is asked for, as
/// pictures are; the view shows it once it arrives.
pub fn display_name(core: Handle, user: &UserId) -> String {
    match core.with_value(|c| c.names.get(user).cloned()) {
        Some(Some(name)) => name,
        Some(None) => user.to_string(),
        None => {
            core::request_avatars(core, std::slice::from_ref(user));
            user.to_string()
        }
    }
}

/// The app's revision, provided to the views: texts naming people follow it, as names
/// may arrive after a text is first shown.
#[derive(Clone, Copy)]
pub struct Revision(pub RwSignal<u64>);

/// A text that names people (with [`display_name`] or [`display_text`]), kept current.
pub fn naming<T>(make: impl Fn() -> T + Send + Sync + 'static) -> Signal<T>
where
    T: Clone + Send + Sync + 'static,
{
    let revision = use_context::<Revision>();
    Signal::derive(move || {
        if let Some(Revision(r)) = revision {
            r.track();
        }
        make()
    })
}

/// For text that holds either a user ID or a name as written (staff lists, leaders): the
/// display name if it is a user ID, else the text.
pub fn display_text(core: Handle, text: &str) -> String {
    text.trim()
        .parse::<UserId>()
        .map_or_else(|_| text.to_owned(), |user| display_name(core, &user))
}

/// A person's picture, or their initials. Decorative: the name is always written next
/// to it.
#[component]
pub fn Avatar(core: Handle, revision: RwSignal<u64>, user: UserId) -> impl IntoView {
    // Asked after every new connection: a question sent while offline is lost.
    let welcomes = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.welcomes)
    });
    {
        let user = user.clone();
        Effect::new(move |_| {
            if welcomes.get() > 0 {
                core::request_avatars(core, std::slice::from_ref(&user));
            }
        });
    }
    let initials = initials(&user);
    let image = move || {
        revision.track();
        core.with_value(|c| c.avatars.get(&user).cloned().flatten())
    };
    move || {
        match image() {
        Some(url) => view! { <img class="avatar" src=url alt="" /> }.into_any(),
        None => view! { <span class="avatar avatar--initials" aria-hidden="true">{initials.clone()}</span> }.into_any(),
    }
    }
}

/// The own profile picture: take or choose a photo, or remove it.
#[component]
pub fn ProfilePicture(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let user = core.with_value(|c| c.replica.user().clone());
    let error = RwSignal::new(None::<&'static str>);
    let choose = move |ev: leptos::ev::Event| {
        use wasm_bindgen::JsCast;
        let Some(file) = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
            .and_then(|i| i.files())
            .and_then(|f| f.get(0))
        else {
            return;
        };
        leptos::task::spawn_local(async move {
            match crate::images::downscale(&file, &AVATAR_STEPS, MAX_AVATAR_BYTES).await {
                Ok(bytes) => {
                    error.set(None);
                    core::set_avatar(core, Some(bytes));
                }
                Err(e) => error.set(Some(e)),
            }
        });
    };
    view! {
        <div class="profile-picture">
            <Avatar core revision user />
            <label class="button button--small" for="avatar-upload">{text::AVATAR_CHANGE}</label>
            <input id="avatar-upload" class="visually-hidden" type="file" accept="image/*" capture="user" on:change=choose />
            <button type="button" class="button button--small" on:click=move |_| core::set_avatar(core, None)>
                {text::AVATAR_REMOVE}
            </button>
            <p role="alert" class="error">{move || error.get()}</p>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initials_come_from_the_user_name() {
        let user = |s: &str| s.parse::<UserId>().unwrap();
        assert_eq!(initials(&user("anna.schmidt@ordo.example.org")), "AS");
        assert_eq!(initials(&user("ben@ordo.example.org")), "B");
        assert_eq!(initials(&user("dora-2@ordo.example.org")), "D2");
    }
}

// [impl->dsn~display-names~2]
/// The own display name (ORG-7): as the identity provider gave it, or chosen here.
#[component]
pub fn DisplayName(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let me = core.with_value(|c| c.replica.user().clone());
    let current = {
        let me = me.clone();
        move || {
            revision.track();
            core.with_value(|c| c.names.get(&me).cloned().flatten())
        }
    };
    let value = RwSignal::new(String::new());
    // The field starts with the known name and follows it until the person types.
    let typed = RwSignal::new(false);
    Effect::new({
        let current = current.clone();
        move || {
            if let Some(name) = current()
                && !typed.get_untracked()
            {
                value.set(name);
            }
        }
    });
    let save = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name = value.get_untracked().trim().to_owned();
        core::set_name(core, (!name.is_empty()).then_some(name));
        typed.set(false);
    };
    let clear = move |_| {
        core::set_name(core, None);
        value.set(String::new());
        typed.set(false);
    };
    view! {
        <form class="form display-name" novalidate on:submit=save>
            <label for="display-name">{text::DISPLAY_NAME}</label>
            <input id="display-name" type="text" autocomplete="name" maxlength="80" prop:value=move || value.get()
                on:input=move |ev| {
                    typed.set(true);
                    value.set(event_target_value(&ev));
                } />
            <p class="hint">{text::DISPLAY_NAME_HINT}</p>
            <div class="row">
                <button type="submit" class="button button--small">{text::DISPLAY_NAME_SAVE}</button>
                <button type="button" class="button button--small button--quiet" on:click=clear>{text::DISPLAY_NAME_RESET}</button>
            </div>
        </form>
    }
}
