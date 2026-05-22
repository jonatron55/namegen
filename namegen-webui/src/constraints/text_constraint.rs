use leptos::prelude::*;

#[component]
pub fn TextConstraint(
    id: String,
    display_name: String,
    on_change: impl Fn(String) + 'static,
    on_clear: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <div class="constraint">
            <label for=id.clone()>{display_name}</label>
            <input
                type="text"
                placeholder="Prefix..."
                autocomplete="off"
                id=id.clone()
                on:input:target=move |ev| {
                    let value = ev.target().value().clone();
                    if !value.is_empty() {
                        on_change(value);
                    } else {
                        on_clear();
                    }
                }
            />
        </div>
    }
}
