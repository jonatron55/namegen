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

const DEFAULT_CONFIG: &[u8] = include_bytes!("../../configs/silly.xml");
const MAX_NAMES: usize = 20;
const GENERATION_RATE_MS: u64 = 2500;
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

    let generator = StoredValue::new_local(default_generator);
    _ = Effect::new({
        let config = config.clone();
        let generator = generator.clone();
        move || {
            generator.set_value(config.get().build_generator());
        }
    });

    let accent_colors = StoredValue::new_local(RefCell::new(AccentColors::new(rand::rng())));

    let interval_handle = RwSignal::new_local(None::<IntervalHandle>);

    let names = StoredValue::new_local(
        (0..MAX_NAMES)
            .map(|_| {
                let color = accent_colors.with_value(|value| value.borrow_mut().next());
                RwSignal::new_local(Ok(NBSP.to_string().with_accent_color(color)))
            })
            .collect::<Vec<_>>(),
    );

    let name_index = RwSignal::new_local(0);

    let constraint_values = RwSignal::new_local(HashMap::<String, String>::new());

    let generate = Callback::new(move |count: usize| {
        generator.with_value(move |generator| {
            rng.update_value(move |mut rng| {
                for _ in 0..count {
                    let color = accent_colors.with_value(|value| value.borrow_mut().next());

                    let name = generator
                        .generate(&mut rng, &constraint_values.get())
                        .map(|parts| parts.join(""));

                    name_index.update(move |i| {
                        names.with_value(|names| names[*i].set(name.map(|n| n.with_accent_color(color))));
                        *i = (*i + 1) % MAX_NAMES;
                    });
                }
            });
        });
    });

    let start_continuous = Callback::new(move |_: ()| {
        interval_handle.update(move |handle| {
            if handle.is_none() {
                generate.run(1);
                *handle =
                    set_interval_with_handle(move || generate.run(1), Duration::from_millis(GENERATION_RATE_MS)).ok();
            }
        });
    });

    let stop_continuous = Callback::new(move |_: ()| {
        interval_handle.update(move |handle| {
            if let Some(handle) = handle.take() {
                handle.clear();
            }
        });
    });

    view! {
        <div class="app">
            <Toolbar config on_config_loaded=move |config| set_config.set(config) />
            <OutputPanel
                started=move || interval_handle.with(|h| h.is_some())
                names=names.clone()
                on_generate_single=move || generate.run(1)
                on_generate_all=move || generate.run(MAX_NAMES)
                on_start=move || start_continuous.run(())
                on_stop=move || stop_continuous.run(())
            />
            <OptionsPanel
                config
                display_names=arg_display_names
                constraint_values=constraint_values
            />
        </div>
    }
}

fn main() {
    mount_to_body(App);
}
