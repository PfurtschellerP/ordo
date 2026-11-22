//! The bell in the header (TASK-3): what needs the person's attention, gathered from every
//! package of the registry.

use leptos::prelude::*;

use crate::core::Handle;
use crate::text;

// [impl->dsn~tasks~8]
// [impl->dsn~package-registry~1]
/// The bell in the header (TASK-3), for everyone in an incident, on every page: what every
/// package says needs the person's attention now, each item linking to where it is dealt
/// with. The number is text, never colour alone.
#[component]
pub fn AttentionBell(
    core: Handle,
    revision: RwSignal<u64>,
    /// Called before opening, to close the other header menus.
    on_open: Callback<()>,
) -> impl IntoView {
    let in_incident = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.incident.is_some())
    });
    let items = Memo::new(move |_| {
        revision.track();
        crate::shell::registry::PACKAGES
            .iter()
            .flat_map(|p| (p.bell)(core))
            .collect::<Vec<_>>()
    });
    let open = RwSignal::new(false);
    view! {
        <Show when=move || in_incident.get()>
            <crate::shell::ui::Disclosure id="attention" open class="attention"
                button_class="attention__button" panel_class="attention__panel"
                on_open=Some(on_open)
                label=Some(Signal::derive(move || text::attention_label(items.with(Vec::len))))
                button=Box::new(move || view! {
                    <crate::shell::ui::Icon name="bell" />
                    {move || {
                        let n = items.with(Vec::len);
                        (n > 0).then(|| view! { <span class="attention__count" aria-hidden="true">{n}</span> })
                    }}
                }.into_any())>
                <h2 class="launcher__title">{text::attention()}</h2>
                {move || {
                    let list = items.get();
                    if list.is_empty() {
                        return view! { <p>{text::attention_none()}</p> }.into_any();
                    }
                    view! {
                        <ul class="attention__list">
                            {list.into_iter().map(|(item, route)| {
                                view! {
                                    <li><a href=route.to_path() on:click={
                                        let follow = crate::shell::views::follow_route(route.clone());
                                        move |ev| { open.set(false); follow(ev) }
                                    }>{item}</a></li>
                                }
                            }).collect_view()}
                        </ul>
                    }
                    .into_any()
                }}
            </crate::shell::ui::Disclosure>
        </Show>
    }
}
