use leptos::prelude::*;

#[component]
pub fn NumberConstraint(
    id: String,
    display_name: String,
    min: usize,
    max: usize,
    on_change: impl Fn(String) + 'static,
    on_clear: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <div class="constraint">
            <label for=id.clone()>{display_name}</label>
            <select
                id=id.clone()
                on:input:target=move |ev| {
                    let value = ev.target().value();
                    match value.as_str() {
                        "none" => on_clear(),
                        _ => on_change(value),
                    }
                }
            >
                <option value="none">"Random"</option>
                {(min..=max)
                    .map(|n| {
                        view! { <option value=n>{n}</option> }
                    })
                    .collect_view()}
            </select>
        </div>
    }
    .into_any()
}
