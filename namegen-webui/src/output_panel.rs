use leptos::prelude::*;

use crate::{typo::Typo, GenerationResult};

#[component]
pub fn OutputPanel(
    #[prop(into)] started: Signal<bool, LocalStorage>,
    names: StoredValue<Vec<RwSignal<GenerationResult, LocalStorage>>, LocalStorage>,
    mut on_generate_single: impl FnMut() + 'static,
    mut on_generate_all: impl FnMut() + 'static,
    mut on_start: impl FnMut() + 'static,
    mut on_stop: impl FnMut() + 'static,
) -> impl IntoView {
    view! {
        <div class="output panel">
            <div class="caption">
                <div class="buttons">
                    <button on:click=move |_| on_generate_single() disabled=move || started.get()>
                        "Generate one"
                    </button>
                    <button on:click=move |_| on_generate_all() disabled=move || started.get()>
                        "Generate 20"
                    </button>
                    <button
                        class=move || { if started.get() { "danger" } else { "secondary" } }
                        on:click=move |_| if started.get() { on_stop() } else { on_start() }
                    >
                        {move || {
                            if started.get() { "Stop generation" } else { "Generate continuously" }
                        }}
                    </button>
                </div>
            </div>
            <div class="content">
                {names
                    .get_value()
                    .iter()
                    .map(|name| {
                        let name = name.clone();
                        view! {
                            <div>
                                {move || match name.get() {
                                    Ok(string) => {
                                        view! { <Typo string=string.clone() /> }.into_any()
                                    }
                                    Err(err) => {
                                        let err = err.to_string();
                                        view! { <div class="name red-background badge">{err}</div> }
                                            .into_any()
                                    }
                                }}
                            </div>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
    }
}
