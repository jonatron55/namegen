use std::collections::HashMap;

use itertools::Itertools;
use lazy_static::lazy_static;
use leptos::prelude::*;
use libnamegen::config::{ConfigSourceType, GeneratorConfig};

lazy_static! {
    static ref builtins: HashMap<String, &'static [u8]> = {
        let mut map: HashMap<String, &'static [u8]> = HashMap::new();
        map.insert("silly".to_string(), include_bytes!("../../configs/silly.xml"));
        map.insert("gadgetry".to_string(), include_bytes!("../../configs/gadgetry.xml"));
        map.insert("elf".to_string(), include_bytes!("../../configs/elf.xml"));
        map.insert("dwarf".to_string(), include_bytes!("../../configs/dwarf.xml"));
        map.insert("goblin".to_string(), include_bytes!("../../configs/goblin.xml"));
        map.insert("abrahamic".to_string(), include_bytes!("../../configs/abrahamic.xml"));
        map.insert(
            "greco-roman".to_string(),
            include_bytes!("../../configs/greco-roman.xml"),
        );
        map
    };
}

lazy_static! {
    static ref builtin_display_names: HashMap<String, &'static str> = {
        let mut map: HashMap<String, &'static str> = HashMap::new();
        map.insert("silly".to_string(), "Silly Names");
        map.insert("gadgetry".to_string(), "Gadgetry");
        map.insert("elf".to_string(), "Elven Names");
        map.insert("dwarf".to_string(), "Dwarven Names");
        map.insert("goblin".to_string(), "Goblin Names");
        map.insert("abrahamic".to_string(), "Abrahamic Mythology");
        map.insert("greco-roman".to_string(), "Greco-Roman Mythology");
        map
    };
}

#[component]
pub fn Toolbar(
    #[prop(into)] config: Signal<GeneratorConfig, LocalStorage>,
    mut on_config_loaded: impl FnMut(GeneratorConfig) + 'static,
) -> impl IntoView {
    let display_name = Signal::derive_local(move || match config.get() {
        GeneratorConfig::Description { display_name, .. } => display_name.clone(),
        GeneratorConfig::Capitalizer { .. } => "Capitalizer".to_string(),
        GeneratorConfig::Joiner { .. } => "Joiner".to_string(),
        GeneratorConfig::Literal { .. } => "Literal".to_string(),
        GeneratorConfig::Markov { .. } => "Markov".to_string(),
        GeneratorConfig::Matcher { .. } => "Matcher".to_string(),
        GeneratorConfig::Numberer { .. } => "Numberer".to_string(),
        GeneratorConfig::Optional { .. } => "Optional".to_string(),
        GeneratorConfig::Repeater { .. } => "Repeater".to_string(),
        GeneratorConfig::Switcher { .. } => "Switcher".to_string(),
        GeneratorConfig::Words { .. } => "Words".to_string(),
    });

    let description = Signal::derive_local(move || match config.get() {
        GeneratorConfig::Description { description, .. } => description.clone(),
        _ => String::new(),
    });

    view! {
        <div class="toolbar panel">
            <div class="caption">
                <div class="toolbar-controls">
                    <label for="-builtin-configs">"Configuration:"</label>
                    <select
                        id="-builtin-configs"
                        on:input:target=move |ev| {
                            let name = ev.target().value();
                            let data = builtins[&name];
                            let config = GeneratorConfig::read(data, ConfigSourceType::Xml)
                                .unwrap();
                            on_config_loaded(config)
                        }
                        prop:value="silly"
                    >
                        {builtins
                            .keys()
                            .sorted()
                            .map(|key| {
                                view! {
                                    <option value=key.clone()>{builtin_display_names[key]}</option>
                                }
                            })
                            .collect_view()}
                    </select>
                // <button>"Load from file"</button>
                </div>
            </div>
            <div class="content">
                <h1>{display_name}</h1>
                <p>{description}</p>
            </div>
        </div>
    }
}
