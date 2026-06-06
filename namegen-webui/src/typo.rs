use std::{borrow::Cow, time::Duration};

use leptos::prelude::*;

use crate::accent_colors::ColoredString;

const TYPING_INTERVAL_MS: u64 = 45;

#[derive(Debug)]
pub struct State {
    displayed: String,
    target: Vec<char>,
    index: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Alphabet {
    Unchanged,
    Ascii,
    Futhorc,
    Tengwar,
}

impl Alphabet {
    pub fn apply<'a>(&self, string: &'a str) -> Cow<'a, str> {
        match self {
            Self::Unchanged => Cow::Borrowed(string),
            Self::Ascii => Cow::Owned(translit::to_ascii(string)),
            Self::Futhorc => Cow::Owned(translit::to_futhorc(string)),
            Self::Tengwar => Cow::Owned(translit::to_tengwar(string)),
        }
    }

    pub fn class(&self) -> Option<&'static str> {
        match self {
            Self::Unchanged | Self::Ascii => None,
            Self::Futhorc => Some("futhorc"),
            Self::Tengwar => Some("tengwar"),
        }
    }
}

#[component]
pub fn Typo(string: ColoredString, alphabet: Alphabet) -> impl IntoView {
    let state = RwSignal::new_local(State {
        displayed: String::new(),
        target: alphabet.apply(&string.text).chars().collect(),
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

    let span_class = match alphabet.class() {
        Some(class) => format!("name {} {}", string.class(), class),
        None => format!("name {}", string.class()),
    };

    view! { <span class=span_class>{move || state.with(|state| state.displayed.clone())}</span> }
}
