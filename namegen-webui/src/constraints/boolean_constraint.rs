use leptos::prelude::*;

#[component]
pub fn BooleanConstraint(
    id: String,
    display_name: String,
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
                        "none" => {
                            on_clear();
                        }
                        "true" | "false" => {
                            on_change(value.to_string());
                        }
                        _ => {}
                    }
                }
            >
                <option value="none">"Random"</option>
                <option value="true">"Yes"</option>
                <option value="false">"No"</option>
            </select>
        </div>
    }
}
