use leptos::leptos_dom::helpers::IntervalHandle;
use leptos::prelude::*;

use crate::typo::Typo;

pub const MAX_NAMES: usize = 20;

pub type NameEntry = Result<(String, usize), String>;

#[component]
pub fn NamesPanel(
    names: RwSignal<Vec<NameEntry>>,
    interval_handle: RwSignal<Option<IntervalHandle>>,
    generate: Callback<usize>,
    start_continuous: Callback<()>,
    stop_continuous: Callback<()>,
) -> impl IntoView {
    view! {
        <div class="output panel">
            <div class="caption">
                <div class="buttons">
                    <button class="secondary-button"
                            on:click=move |_| generate.run(1)>
                        "Generate one"
                    </button>
                    <button class="secondary-button"
                            on:click=move |_| generate.run(20)>
                        "Generate 20"
                    </button>
                    {move || {
                        if interval_handle.get().is_none() {
                            view! {
                                <button class="ok-button"
                                        on:click=move |_| start_continuous.run(())>
                                    "Continuous generation"
                                </button>
                            }.into_any()
                        } else {
                            view! {
                                <button class="danger-button"
                                        on:click=move |_| stop_continuous.run(())>
                                    "Stop generation"
                                </button>
                            }.into_any()
                        }
                    }}
                </div>
            </div>
            <div class="content">
                <For each=move || 0..MAX_NAMES
                     key=|i| *i
                     children=move |index| {
                        let text = Signal::derive(move || {
                            names.with(|name| match &name[index] {
                                Ok((v, _)) => v.clone(),
                                Err(_) => String::new(),
                            })
                        });

                        let class_attr = Signal::derive(move || {
                            names.with(|name| match &name[index] {
                                Ok((_, color)) => format!("accent-{color}"),
                                Err(_) => "danger".to_string(),
                            })
                        });

                        let is_err = Memo::new(move |_| {
                            names.with(|name| name[index].is_err())
                        });

                        let err_text = Memo::new(move |_| {
                            names.with(|name| match &name[index] {
                                Ok(_) => String::new(),
                                Err(err) => err.clone(),
                            })
                        });

                        view! {
                            <div class=move || class_attr.get()>
                                {move || {
                                    if is_err.get() {
                                        view! { {err_text.get()} }.into_any()
                                    } else {
                                        view! { <Typo text /> }.into_any()
                                    }
                                }}
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
