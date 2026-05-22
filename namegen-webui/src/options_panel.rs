use std::collections::HashMap;

use leptos::prelude::*;
use libnamegen::config::GeneratorConfig;

use crate::constraints::{BooleanConstraint, Constraint, Constraints, NumberConstraint, TextConstraint};

#[component]
pub fn OptionsPanel(
    #[prop(into)] config: Signal<GeneratorConfig, LocalStorage>,
    #[prop(into)] display_names: Signal<HashMap<String, String>, LocalStorage>,
    constraint_values: RwSignal<HashMap<String, String>, LocalStorage>,
) -> impl IntoView {
    let constraints = Signal::derive_local(move || config.get().constraints());
    let get_display_name = move |id: &str| display_names.get().get(id).cloned().unwrap_or_else(|| id.to_string());

    let update_value = Callback::<(String, String)>::new(move |(id, value)| {
        constraint_values.update(move |values| {
            values.insert(id, value);
        })
    });

    let clear_value = Callback::<String>::new(move |id| {
        constraint_values.update(move |values| {
            values.remove(&id);
        })
    });

    view! {
        <div class="controls panel">
            <div class="caption">
                <h1>"Options"</h1>
            </div>
            <div class="content">
                {move || {
                    constraints
                        .get()
                        .into_iter()
                        .map(|constraint| {
                            match constraint {
                                Constraint::Number { id, min, max } => {
                                    view! {
                                        <NumberConstraint
                                            id=id.clone()
                                            display_name=get_display_name(&id)
                                            min
                                            max
                                            on_change={
                                                let id = id.clone();
                                                move |value| { update_value.run((id.clone(), value)) }
                                            }
                                            on_clear={
                                                let id = id.clone();
                                                move || clear_value.run(id.clone())
                                            }
                                        />
                                    }
                                        .into_any()
                                }
                                Constraint::Text { id } => {
                                    view! {
                                        <TextConstraint
                                            id=id.clone()
                                            display_name=get_display_name(&id)
                                            on_change={
                                                let id = id.clone();
                                                move |value| { update_value.run((id.clone(), value)) }
                                            }
                                            on_clear={
                                                let id = id.clone();
                                                move || clear_value.run(id.clone())
                                            }
                                        />
                                    }
                                        .into_any()
                                }
                                Constraint::Boolean { id } => {
                                    view! {
                                        <BooleanConstraint
                                            id=id.clone()
                                            display_name=get_display_name(&id)
                                            on_change={
                                                let id = id.clone();
                                                move |value| { update_value.run((id.clone(), value)) }
                                            }
                                            on_clear={
                                                let id = id.clone();
                                                move || clear_value.run(id.clone())
                                            }
                                        />
                                    }
                                        .into_any()
                                }
                            }
                        })
                        .collect_view()
                }}
            </div>
        </div>
    }
}
