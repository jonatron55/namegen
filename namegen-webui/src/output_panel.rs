use leptos::prelude::*;

use crate::{
    GenerationResult,
    typo::{Alphabet, Typo},
};

#[component]
pub fn OutputPanel(
    #[prop(into)] started: Signal<bool, LocalStorage>,
    names: StoredValue<Vec<RwSignal<GenerationResult, LocalStorage>>, LocalStorage>,
    mut on_generate_single: impl FnMut() + 'static,
    mut on_generate_all: impl FnMut() + 'static,
    mut on_start: impl FnMut() + 'static,
    mut on_stop: impl FnMut() + 'static,
) -> impl IntoView {
    let (alphabet, set_alphabet) = signal_local(Alphabet::Unchanged);

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
                {
                    let names = names.get_value();
                    names
                        .into_iter()
                        .map(move |name| {
                            view! {
                                <div>
                                    {move || {
                                        let alphabet = alphabet.get();
                                        match name.get() {
                                            Ok(string) => view! { <Typo string alphabet /> }.into_any(),
                                            Err(err) => {
                                                let err = err.to_string();
                                                view! { <div class="name red-background badge">{err}</div> }
                                                    .into_any()
                                            }
                                        }
                                    }}
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()
                }
            </div>
            <div class="caption">
                <div class="footer">
                    <label for="alphabet-select">"Output alphabet: "</label>
                    <select
                        id="alphabet-select"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            match value.as_str() {
                                "ascii" => set_alphabet.set(Alphabet::Ascii),
                                "futhorc" => set_alphabet.set(Alphabet::Futhorc),
                                "tengwar" => set_alphabet.set(Alphabet::Tengwar),
                                _ => set_alphabet.set(Alphabet::Unchanged),
                            }
                        }
                    >
                        <option value="unchanged">"Unchanged"</option>
                        <option value="ascii">"ASCII only"</option>
                        <option value="futhorc">"Futhorc runes"</option>
                        <option value="tengwar">"Tengwar"</option>
                    </select>
                </div>
            </div>
        </div>
    }
}
