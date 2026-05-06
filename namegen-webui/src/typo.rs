use yew::prelude::*;
use yew_hooks::use_interval;

#[derive(PartialEq, Clone, Properties)]
pub struct TypoProps {
    pub text: AttrValue,
}

#[derive(PartialEq, Clone, Properties)]
struct TypoState {
    displayed: AttrValue,
    target: AttrValue,
}

#[function_component]
pub fn Typo(props: &TypoProps) -> Html {
    let state = use_state(|| TypoState {
        displayed: String::new().into(),
        target: props.text.clone(),
    });

    let _ = use_interval(
        {
            let props = props.clone();
            let state = state.clone();
            move || {
                if state.target != props.text {
                    state.set(TypoState {
                        displayed: String::new().into(),
                        target: props.text.clone(),
                    });
                    return;
                }

                let target = &state.target;
                let displayed = &state.displayed;

                if displayed != target {
                    let mut new_displayed = displayed.chars().collect::<Vec<_>>();
                    if new_displayed.len() < target.chars().count() {
                        new_displayed.push(target.chars().nth(new_displayed.len()).unwrap());
                    } else {
                        new_displayed.pop();
                    }
                    state.set(TypoState {
                        displayed: AttrValue::from(new_displayed.into_iter().collect::<String>()),
                        target: target.clone(),
                    });
                }
            }
        },
        32,
    );

    html! {
        <span>{ if state.displayed.len() == 0 { "\u{00A0}" } else { &state.displayed } }</span>
    }
}
