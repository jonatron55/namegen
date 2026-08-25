use std::collections::HashMap;

use itertools::Itertools;
use leptos::html;
use leptos::prelude::*;
use libnamegen::config::ParseError;
use libnamegen::config::{ConfigSourceType, GeneratorConfig};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;
use web_sys::FileReader;
use web_sys::HtmlInputElement;

const BUILTINS: [&[u8]; 8] = [
    include_bytes!("../../configs/abrahamic.xml"),
    include_bytes!("../../configs/dwarf.xml"),
    include_bytes!("../../configs/elf.xml"),
    include_bytes!("../../configs/epick.xml"),
    include_bytes!("../../configs/gadgetry.xml"),
    include_bytes!("../../configs/goblin.xml"),
    include_bytes!("../../configs/greco-roman.xml"),
    include_bytes!("../../configs/silly.xml"),
];

#[component]
pub fn Toolbar(
    #[prop(into)] config: Signal<GeneratorConfig, LocalStorage>,
    mut on_config_loaded: impl FnMut(Result<GeneratorConfig, ParseError>) + 'static,
) -> impl IntoView {
    let configs: HashMap<String, GeneratorConfig> = BUILTINS
        .iter()
        .map(|data| {
            let config =
                GeneratorConfig::read(*data, ConfigSourceType::Xml).expect("Builtin configuration could not be parsed");
            let name = match &config {
                GeneratorConfig::Description { display_name, .. } => display_name.clone(),
                _ => panic!("Builtin configuration does not include a description"),
            };
            (name, config)
        })
        .collect();

    let configs = RwSignal::new_local(configs);

    let description = Signal::derive_local(move || match config.get() {
        GeneratorConfig::Description { description, .. } => Some(description.clone()),
        _ => None,
    });

    let file_input_ref = NodeRef::<html::Input>::new();

    view! {
        <div class="toolbar panel">
            <div class="caption">
                <div class="toolbar-controls">
                    <label for="configs">"Configuration:"</label>
                    <select
                        id="configs"
                        on:input:target=move |ev| {
                            let name = ev.target().value();
                            let configs = configs.get();
                            let data = configs.get(&name).expect("Configuration not found");
                            on_config_loaded(Ok(data.clone()))
                        }
                        prop:value="Silly Names"
                    >
                        {move || {
                            configs
                                .get()
                                .keys()
                                .into_iter()
                                .sorted()
                                .map(|name| {
                                    view! { <option value=name.clone()>{name.clone()}</option> }
                                })
                                .collect_view()
                        }}
                    </select>
                    <button>"↥ Import"</button>
                    <button>"⤓ Export"</button>
                </div>
            </div>
            {move || {
                description
                    .get()
                    .map(|description| {
                        view! {
                            <div class="content">
                                <p>{description}</p>
                            </div>
                        }
                    })
            }}

            <input
                id="file-input"
                type="file"
                accept=".xml,.txt"
                on:change:target=move |ev| {
                    let input: HtmlInputElement = ev.target().dyn_into().unwrap();
                    if let Some(file) = input.files().and_then(|files| files.get(0)) {
                        let reader = FileReader::new().unwrap();
                        let onload = Closure::once_into_js(move |_: web_sys::ProgressEvent| {
                            let result = reader.result().unwrap();
                            let config = GeneratorConfig::read(data, ConfigSourceType::Xml);
                            on_config_loaded(config);
                        });
                    }
                }
                node_ref=file_input_ref.clone()
                hidden
            />
        </div>
    }
}
