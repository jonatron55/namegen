use std::{collections::HashMap, io::Cursor, path::Path};

use itertools::Itertools;
use leptos::ev::Targeted;
use leptos::prelude::*;
use libnamegen::config::{ConfigSourceType, GeneratorConfig, ParseError, WriteXml};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{Event, FileReader, HtmlInputElement, ProgressEvent};
use xml::writer::EmitterConfig as XmlEmitterConfig;

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
    on_config_loaded: impl Callable<Result<GeneratorConfig, ParseError>> + Clone + 'static,
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

    let selected_config = Signal::derive_local(move || match config.get() {
        GeneratorConfig::Description { display_name, .. } => display_name.clone(),
        _ => String::new(),
    });

    let file_changed = {
        let on_config_loaded = on_config_loaded.clone();
        move |ev: Targeted<Event, HtmlInputElement>| {
            let input: HtmlInputElement = ev.target().dyn_into().unwrap();
            if let Some(file) = input.files().and_then(|files| files.get(0)) {
                let filename = file.name();
                let reader = FileReader::new().unwrap();
                let onload = Closure::once_into_js({
                    let reader = reader.clone();
                    let on_config_loaded = on_config_loaded.clone();
                    move |_: ProgressEvent| {
                        let data = js_sys::Uint8Array::new(&reader.result().unwrap()).to_vec();

                        let config = (|| {
                            let mut cursor = Cursor::new(data);
                            let config_type = ConfigSourceType::guess(Path::new(&filename), &mut cursor)?;
                            GeneratorConfig::read(cursor, config_type)
                        })();

                        if let Ok(config) = &config
                            && let GeneratorConfig::Description { display_name, .. } = config
                        {
                            let config = config.clone();
                            configs.update(|configs| {
                                configs.insert(display_name.clone(), config);
                            });
                        }
                        on_config_loaded.run(config);
                    }
                });

                reader.set_onload(Some(onload.unchecked_ref()));
                reader.read_as_array_buffer(&file).unwrap();
            }
        }
    };

    let export_file = {
        let config = config.clone();
        move |_| {
            let config = config.get();

            let name = match &config {
                GeneratorConfig::Description { display_name, .. } => display_name
                    .clone()
                    .chars()
                    .filter(|ch| ch.is_ascii_alphanumeric() || ch.is_whitespace())
                    .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
                    .flat_map(|ch| ch.to_lowercase())
                    .collect(),
                _ => "config".to_string(),
            };

            let mut xml = Vec::new();
            let mut cursor = Cursor::new(&mut xml);
            let mut writer = XmlEmitterConfig::new()
                .perform_indent(true)
                .line_separator("\n")
                .pad_self_closing(true)
                .indent_string("  ")
                .create_writer(&mut cursor);
            config.write_xml_root(&mut writer).unwrap();
            let xml = String::from_utf8(xml).unwrap();

            let props = web_sys::BlobPropertyBag::new();
            props.set_type("text/xml");

            let blob =
                web_sys::Blob::new_with_str_sequence_and_options(&js_sys::Array::of1(&JsValue::from_str(&xml)), &props)
                    .unwrap();

            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let a = document.create_element("a").unwrap();
            a.set_attribute("href", &url).unwrap();
            a.set_attribute("download", &format!("{}.xml", name)).unwrap();
            a.set_attribute("style", "display: none;").unwrap();
            document.body().unwrap().append_child(&a).unwrap();

            let a: web_sys::HtmlAnchorElement = a.dyn_into().unwrap();
            a.click();

            document.body().unwrap().remove_child(&a).unwrap();
            web_sys::Url::revoke_object_url(&url).unwrap();
        }
    };

    view! {
        <div class="toolbar panel">
            <div class="caption">
                <div class="toolbar-controls">
                    <label for="configs">"Configuration:"</label>
                    <select
                        id="configs"
                        on:input:target={
                            let on_config_loaded = on_config_loaded.clone();
                            move |ev| {
                                let name = ev.target().value();
                                let configs = configs.get();
                                let data = configs.get(&name).expect("Configuration not found");
                                on_config_loaded.run(Ok(data.clone()));
                            }
                        }
                        prop:value=move || selected_config.get()
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
                    <input
                        id="file-input"
                        class="button"
                        type="file"
                        accept=".xml,.txt"
                        on:change:target=file_changed
                    />
                    <label for="file-input" style="text-transform: none !important">
                        "↥ Import"
                    </label>
                    <button type="button" on:click=export_file>
                        "⤓ Export"
                    </button>
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
        </div>
    }
}
