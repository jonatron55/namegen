use std::{cell::RefCell, collections::HashMap, time::Duration};

use leptos::prelude::*;
use libnamegen::{
    config::{BuildGenerator, ConfigSourceType, GeneratorConfig},
    generator::Error as GenerationError,
};

use crate::{
    accent_colors::{AccentColors, ColoredString, WithAccentColor},
    options_panel::OptionsPanel,
    output_panel::OutputPanel,
    toolbar::Toolbar,
};

mod accent_colors;
mod constraints;
mod options_panel;
mod output_panel;
mod toolbar;
mod typo;

const DEFAULT_CONFIG: &[u8] = include_bytes!("../../configs/default.xml");
const MAX_NAMES: usize = 20;
const GENERATION_RATE_MS: u32 = 2500;
const NBSP: &str = "\u{00A0}";

pub type GenerationResult = Result<ColoredString, GenerationError>;

#[component]
fn App() -> impl IntoView {
    let rng = StoredValue::new_local(rand::rng());
    let default_config = GeneratorConfig::read(DEFAULT_CONFIG, ConfigSourceType::Xml).unwrap();
    let default_generator = default_config.build_generator();
    let (config, set_config) = signal_local(default_config);
    let arg_display_names = Signal::derive_local(move || match config.get() {
        GeneratorConfig::Description { arg_display_names, .. } => arg_display_names.clone(),
        _ => HashMap::new(),
    });
    let accent_colors = StoredValue::new_local(RefCell::new(AccentColors::new(rand::rng())));
    let generator = StoredValue::new_local(default_generator);
    _ = Effect::new({
        let config = config.clone();
        let generator = generator.clone();
        move || {
            generator.set_value(config.get().build_generator());
        }
    });

    let (continuous, set_continuous) = signal_local(false);
    let names: StoredValue<Vec<RwSignal<GenerationResult, LocalStorage>>, LocalStorage> = StoredValue::new_local(
        (0..MAX_NAMES)
            .map(|_| {
                let color = accent_colors.with_value(|value| value.borrow_mut().next());
                RwSignal::new_local(Ok(NBSP.to_string().with_accent_color(color)))
            })
            .collect::<Vec<_>>(),
    );
    let name_index = RwSignal::new_local(0);
    let constraint_values: RwSignal<HashMap<String, String>, LocalStorage> = RwSignal::new_local(HashMap::new());

    set_interval(
        move || {
            if continuous.get() {
                rng.update_value(|mut rng| {
                    generator.with_value(|generator| {
                        let color = accent_colors.with_value(|value| value.borrow_mut().next());
                        let name = generator
                            .generate(&mut rng, &constraint_values.get())
                            .map(|parts| parts.join(""));
                        names.get_value()[name_index.get()].set(name.map(|n| n.with_accent_color(color)));
                    });
                    name_index.set((name_index.get() + 1) % MAX_NAMES);
                })
            }
        },
        Duration::from_millis(GENERATION_RATE_MS as u64),
    );

    view! {
        <div class="app">
            <Toolbar config on_config_loaded=move |config| set_config.set(config) />
            <OptionsPanel
                config
                display_names=arg_display_names
                constraint_values=constraint_values
            />
            <OutputPanel
                started=continuous
                names=names.clone()
                on_generate_single=move || {
                    rng.update_value(|mut rng| {
                        let color = accent_colors.with_value(|value| value.borrow_mut().next());
                        generator
                            .with_value(|generator| {
                                let name = generator
                                    .generate(&mut rng, &constraint_values.get())
                                    .map(|parts| parts.join(""));
                                names
                                    .get_value()[name_index.get()]
                                    .set(name.map(|n| n.with_accent_color(color)));
                                name_index.set((name_index.get() + 1) % MAX_NAMES);
                            });
                    })
                }
                on_generate_all={
                    let names = names.clone();
                    let name_index = name_index.clone();
                    move || {
                        rng.update_value(|mut rng| {
                            generator
                                .with_value(|generator| {
                                    for i in 0..MAX_NAMES {
                                        let color = accent_colors
                                            .with_value(|value| value.borrow_mut().next());
                                        let name = generator
                                            .generate(&mut rng, &constraint_values.get())
                                            .map(|parts| parts.join(""));
                                        names
                                            .get_value()[i]
                                            .set(name.map(|n| n.with_accent_color(color)));
                                    }
                                });
                            name_index.set(0);
                        })
                    }
                }
                on_start=move || {
                    set_continuous.set(true);
                }
                on_stop=move || {
                    set_continuous.set(false);
                }
            />
        </div>
    }
}

fn main() {
    mount_to_body(App);
}
