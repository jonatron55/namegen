use std::{collections::HashMap, io::BufReader};

use libnamegen::config::{ConfigSourceType, GeneratorConfig, IntoGenerator};
use rand::RngExt;
use web_sys::HtmlSelectElement;
use yew::prelude::*;

use crate::constraints::{ConstraintView, Constraints};

mod constraints;

const DEFAULT_CONFIG: &[u8] = include_bytes!("../../configs/default.xml");
const THING_CONFIG: &[u8] = include_bytes!("../../configs/thing.xml");
const ELF_CONFIG: &[u8] = include_bytes!("../../configs/elf.xml");
const DWARF_CONFIG: &[u8] = include_bytes!("../../configs/dwarf.xml");
const GOBLIN_CONFIG: &[u8] = include_bytes!("../../configs/goblin.xml");
const ABRAHAMIC_CONFIG: &[u8] = include_bytes!("../../configs/abrahamic.xml");
const MAX_NAMES: usize = 20;

#[derive(Debug, Default)]
pub struct App {
    current_config: Option<GeneratorConfig>,
    constraints: HashMap<String, String>,
    names: Vec<Result<(AttrValue, usize), AttrValue>>,
    name_index: usize,
    continuous_generation: bool,
    last_color_index: usize,
}

pub enum AppMessage {
    GenerateImmediate(usize),
    LoadPreset(&'static [u8]),
    LoadFromFile,
    StartContinuousGeneration,
    StopContinuousGeneration,
    ConstraintChanged(AttrValue, Option<String>),
}

impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let reader = BufReader::new(DEFAULT_CONFIG);
        let config = GeneratorConfig::read(reader, ConfigSourceType::Xml).expect("Failed to read default config");
        Self {
            current_config: Some(config),
            names: vec![Ok((AttrValue::from("\u{00A0}"), 0)); MAX_NAMES],
            name_index: 0,
            continuous_generation: false,
            last_color_index: 0,
            constraints: HashMap::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMessage::GenerateImmediate(count) => {
                if let Some(config) = &self.current_config {
                    let generator = config.clone().into_generator();

                    let mut rng = rand::rng();
                    for _ in 0..count {
                        let constraints = self.constraints.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
                        let name = match generator.generate(&mut rng, &constraints) {
                            Ok(names) => {
                                let mut color = rng.random_range(1..=6);
                                while color == self.last_color_index {
                                    color = rng.random_range(1..=6);
                                }
                                self.last_color_index = color;

                                Ok((AttrValue::from(names.join("")), color))
                            }
                            Err(err) => Err(AttrValue::from(format!("Error: {}", err))),
                        };

                        self.names[self.name_index] = name;
                        self.name_index = (self.name_index + 1) % MAX_NAMES;
                    }
                }
                true
            }
            AppMessage::StartContinuousGeneration => {
                self.continuous_generation = true;
                true
            }
            AppMessage::StopContinuousGeneration => {
                self.continuous_generation = false;
                true
            }
            AppMessage::LoadPreset(data) => {
                let reader = BufReader::new(data);
                match GeneratorConfig::read(reader, ConfigSourceType::Xml) {
                    Ok(config) => {
                        self.current_config = Some(config);
                        self.constraints.clear();
                    }
                    Err(err) => {}
                }
                true
            }
            AppMessage::LoadFromFile => true,
            AppMessage::ConstraintChanged(id, value) => {
                if let Some(value) = value {
                    self.constraints.insert(id.to_string(), value);
                } else {
                    self.constraints.remove(&id.to_string());
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_gen_one_click = &ctx.link().callback(|_| AppMessage::GenerateImmediate(1));
        let on_gen_20_click = &ctx.link().callback(|_| AppMessage::GenerateImmediate(20));
        let on_start_continuous_click = &ctx.link().callback(|_| AppMessage::StartContinuousGeneration);
        let on_stop_continuous_click = &ctx.link().callback(|_| AppMessage::StopContinuousGeneration);

        let (display_name, description, arg_display_names) = if let Some(GeneratorConfig::Description {
            display_name,
            description,
            arg_display_names,
            ..
        }) = &self.current_config
        {
            (
                AttrValue::from(display_name.clone()),
                AttrValue::from(description.clone()),
                arg_display_names
                    .iter()
                    .map(|(k, v)| (AttrValue::from(k.clone()), AttrValue::from(v.clone())))
                    .collect(),
            )
        } else {
            (AttrValue::from("No config loaded"), AttrValue::from(""), HashMap::new())
        };

        html! {
            <div class="app">
                <div class="panel">
                    <div class="caption">
                        <div class="toolbar">
                            <label for="-builtin-configs">{ "Configuration:" }</label>
                            <select id="-builtin-configs" onchange={ctx.link().callback(|e: Event|
                                match  e.target_unchecked_into::<HtmlSelectElement>().value().as_str() {
                                    "default" => AppMessage::LoadPreset(DEFAULT_CONFIG),
                                    "things" => AppMessage::LoadPreset(THING_CONFIG),
                                    "dwarf" => AppMessage::LoadPreset(DWARF_CONFIG),
                                    "elves" => AppMessage::LoadPreset(ELF_CONFIG),
                                    "goblins" => AppMessage::LoadPreset(GOBLIN_CONFIG),
                                    "abrahamic" => AppMessage::LoadPreset(ABRAHAMIC_CONFIG),
                                    _ => AppMessage::LoadPreset(DEFAULT_CONFIG),
                                })}>
                                <option value="default" selected={true}>{ "Default" }</option>
                                <option value="things">{ "Things" }</option>
                                <option value="dwarf">{ "Dwarven" }</option>
                                <option value="elves">{ "Elven" }</option>
                                <option value="goblins">{ "Goblinoid" }</option>
                                <option value="abrahamic">{ "Abrahamic" }</option>
                            </select>
                            <button>{ "Load from file" }</button>
                        </div>
                    </div>
                    <div class="content">
                        <h1>{ display_name }</h1>
                        <p>{ description }</p>
                    </div>
                </div>
                <div class="controls panel">
                    <div class="caption">
                        <h1>{ "Options" }</h1>
                    </div>
                    if let Some(config) = &self.current_config {
                        for c in config.constraints() {
                            <ConstraintView
                                constraint={c.clone()} display_name={arg_display_names.get(c.id()).cloned().unwrap_or_else(|| c.id().clone())}
                                value={self.constraints.get(&c.id().to_string()).cloned().map(AttrValue::from)}
                                on_change={ctx.link().callback(|(id, value): (AttrValue, Option<String>)| AppMessage::ConstraintChanged(id.clone(), value))}  />
                        }
                    }
                </div>
                <div class="output panel">
                    <div class="caption">
                        <div class="buttons">
                            <button class="secondary-button" onclick={on_gen_one_click}>{ "Generate one" }</button>
                            <button class="secondary-button" onclick={on_gen_20_click}>{ "Generate 20" }</button>
                            if !self.continuous_generation {
                                <button class="ok-button" onclick={on_start_continuous_click}>{ "Continuous generation" }</button>
                            }
                            if self.continuous_generation {
                                <button class="danger-button" onclick={on_stop_continuous_click}>{ "Stop generation" }</button>
                            }
                        </div>
                    </div>
                    <div class="content">
                        { for self.names.iter().enumerate().map(|(index, name)|
                            match name {
                                Ok((value, color)) => html! {
                                    <div id={value.clone()} class={classes!(format!("accent-{}", color), if self.name_index == (index + 1) % MAX_NAMES { "gen-name" } else { "" })}>
                                        { value.clone() }
                                    </div>
                                },
                                Err(err) => html! {<div class={"error"}>{ err.clone() }</div> },
                            })
                        }
                    </div>
                </div>
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
