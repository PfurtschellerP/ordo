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
