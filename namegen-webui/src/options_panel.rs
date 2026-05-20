use std::collections::HashMap;

use leptos::prelude::*;
use libnamegen::config::GeneratorConfig;

use crate::constraints::{Constraint, Constraints};

#[component]
pub fn OptionsPanel(
    #[prop(into)] config: Signal<GeneratorConfig, LocalStorage>,
    #[prop(into)] display_names: Signal<HashMap<String, String>, LocalStorage>,
    constraint_values: RwSignal<HashMap<String, String>, LocalStorage>,
) -> impl IntoView {
    let constraints = Signal::derive_local(move || config.get().constraints());
    let get_display_name = move |id: &str| display_names.get().get(id).cloned().unwrap_or_else(|| id.to_string());

    view! {
        <div class="controls panel">
            <div class="caption">
                <h1>"Options"</h1>
            </div>
            {move || {
                constraints
                    .get()
                    .into_iter()
                    .map(|constraint| {
                        match constraint {
                            Constraint::Number { id, min, max } => {
                                view! {
                                    <div class="constraint">
                                        <label for=id.clone()>{get_display_name(&id)}</label>
                                        <select
                                            id=id.clone()
                                            on:input:target=move |ev| {
                                                let value = ev.target().value();
                                                match value.as_str() {
                                                    "none" => {
                                                        constraint_values
                                                            .update(|values| {
                                                                values.remove(&id);
                                                            });
                                                    }
                                                    _ => {
                                                        if let Ok(n) = value.parse::<i32>() {
                                                            constraint_values
                                                                .update(|values| {
                                                                    values.insert(id.clone(), n.to_string());
                                                                });
                                                        }
                                                    }
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
                            Constraint::Text { id } => {
                                view! {
                                    <div class="constraint">
                                        <label for=id.clone()>{get_display_name(&id)}</label>
                                        <input
                                            type="text"
                                            placeholder="Prefix..."
                                            id=id.clone()
                                            on:input:target=move |ev| {
                                                let value = ev.target().value().clone();
                                                if !value.is_empty() {
                                                    constraint_values
                                                        .update(|values| {
                                                            values.insert(id.clone(), value.clone());
                                                        });
                                                } else {
                                                    constraint_values
                                                        .update(|values| {
                                                            values.remove(&id.clone());
                                                        });
                                                }
                                            }
                                        />
                                    </div>
                                }
                                    .into_any()
                            }
                            Constraint::Boolean { id } => {
                                view! {
                                    <div class="constraint">
                                        <label for=id.clone()>{get_display_name(&id)}</label>
                                        <select
                                            id=id.clone()
                                            on:input:target=move |ev| {
                                                let value = ev.target().value();
                                                match value.as_str() {
                                                    "none" => {
                                                        constraint_values
                                                            .update(|values| {
                                                                values.remove(&id);
                                                            });
                                                    }
                                                    "true" | "false" => {
                                                        constraint_values
                                                            .update(|values| {
                                                                values.insert(id.clone(), value.to_string());
                                                            });
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        >
                                            <option value="none">"Random"</option>
                                            <option value="true">"Present"</option>
                                            <option value="false">"Absent"</option>
                                        </select>
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
