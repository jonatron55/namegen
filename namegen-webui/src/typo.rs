use std::time::Duration;

use leptos::prelude::*;

use crate::accent_colors::ColoredString;

const TYPING_INTERVAL_MS: u64 = 45;

#[derive(Debug)]
pub struct State {
    displayed: String,
    target: Vec<char>,
    index: usize,
}

#[component]
pub fn Typo(string: ColoredString) -> impl IntoView {
    let state = RwSignal::new_local(State {
        displayed: String::new(),
        target: string.text.chars().collect(),
        index: 0,
    });

    set_interval(
        move || {
            state.update(|state| {
                if state.index < state.target.len() {
                    state.displayed.push(state.target[state.index]);
                    state.index += 1;
                }
            });
        },
        Duration::from_millis(TYPING_INTERVAL_MS),
    );

    view! {
        <span class=format!(
            "name {}",
            string.class(),
        )>{move || state.with(|state| state.displayed.clone())}</span>
    }
}
