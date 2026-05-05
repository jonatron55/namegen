use std::collections::HashSet;

use libnamegen::config::GeneratorConfig;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

pub trait Constraints {
    fn constraints(&self) -> Vec<Constraint>;
}

#[derive(PartialEq, Clone)]
pub enum Constraint {
    Number { id: AttrValue, min: usize, max: usize },
    Text { id: AttrValue },
    Boolean { id: AttrValue },
}

impl Constraints for GeneratorConfig {
    fn constraints(&self) -> Vec<Constraint> {
        match self {
            GeneratorConfig::Description { subpart, .. } => subpart.constraints(),
            GeneratorConfig::Capitalizer { subpart, .. } => subpart.constraints(),
            GeneratorConfig::Joiner { subparts, .. } => {
                let mut views: Vec<_> = subparts.iter().flat_map(|s| s.constraints()).collect();
                let mut seen = HashSet::new();
                views.retain(|v| seen.insert(v.id().clone()));
                views
            }
            GeneratorConfig::Markov { id: Some(id), .. } => vec![Constraint::Text {
                id: AttrValue::from(id.clone()),
            }],
            GeneratorConfig::Matcher {
                base, cases, default, ..
            } => {
                let mut views = base.constraints();

                for (_, config) in cases {
                    views.extend(config.constraints());
                }

                if let Some(default) = default {
                    views.extend(default.constraints());
                }

                let mut seen = HashSet::new();
                views.retain(|v| seen.insert(v.id().clone()));
                views
            }
            GeneratorConfig::Numberer {
                id: Some(id), min, max, ..
            } => vec![Constraint::Number {
                id: AttrValue::from(id.clone()),
                min: *min,
                max: *max,
            }],
            GeneratorConfig::Optional { id, generator, .. } => {
                let mut views = generator.constraints();
                if let Some(id) = id {
                    views.push(Constraint::Boolean {
                        id: AttrValue::from(id.clone()),
                    });
                }
                let mut seen = HashSet::new();
                views.retain(|v| seen.insert(v.id().clone()));
                views
            }
            GeneratorConfig::Repeater {
                id,
                generator,
                min,
                max,
            } => {
                let mut views = generator.constraints();
                if let Some(id) = id {
                    views.push(Constraint::Number {
                        id: AttrValue::from(id.clone()),
                        min: *min,
                        max: *max,
                    });
                }
                let mut seen = HashSet::new();
                views.retain(|v| seen.insert(v.id().clone()));
                views
            }
            GeneratorConfig::Switcher { id, subparts, .. } => {
                let mut views: Vec<_> = subparts.iter().flat_map(|s| s.constraints()).collect();
                let mut seen = HashSet::new();
                if let Some(id) = id {
                    views.push(Constraint::Number {
                        id: AttrValue::from(id.clone()),
                        min: 0,
                        max: subparts.len() - 1,
                    });
                }
                views.retain(|v| seen.insert(v.id().clone()));
                views
            }
            GeneratorConfig::Words { id: Some(id), .. } => vec![Constraint::Text {
                id: AttrValue::from(id.clone()),
            }],
            _ => vec![],
        }
    }
}

impl Constraint {
    pub fn id(&self) -> &AttrValue {
        match self {
            Constraint::Number { id, .. } => id,
            Constraint::Text { id, .. } => id,
            Constraint::Boolean { id, .. } => id,
        }
    }
}

#[derive(PartialEq, Clone, Properties)]
pub struct ConstraintViewProps {
    pub constraint: Constraint,
    pub display_name: AttrValue,
    pub value: Option<AttrValue>,
    pub on_change: Callback<(AttrValue, Option<String>)>,
}

#[component]
pub fn ConstraintView(props: &ConstraintViewProps) -> Html {
    let constraint = &props.constraint;
    match constraint {
        Constraint::Number { id, min, max } => html! {
            <div class="constraint">
                <label for={id}>{ &props.display_name }</label>
                <select id={id} onchange={props.on_change.reform({
                    let id = id.clone();
                    move |e: Event| {
                        let input: HtmlSelectElement = e.target_unchecked_into();
                        let value = input.value();
                        let value = if value == "none" { None } else { Some(value) };
                        (id.clone(), value)
                    }
                })}>
                    <option value="none" selected={props.value.is_none()}>{ "Random" }</option>
                    { for (*min..=*max).map(|n| html! {
                        <option value={n.to_string()} selected={props.value.clone().map_or(false, |v| v == n.to_string())}>{ n }</option>
                    }) }
                </select>
            </div>
        },
        Constraint::Text { id } => html! {
            <div class="constraint">
                <label for={id}>{ &props.display_name }</label>
                <input id={id} type="text" value={props.value.clone().unwrap_or_default()} placeholder="Prefix..." onchange={props.on_change.reform({
                    let id = id.clone();
                    move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        let value = input.value();
                        let value = if value.is_empty() { None } else { Some(value) };
                        (id.clone(), value)
                    }
                })} />
            </div>
        },
        Constraint::Boolean { id } => html! {
            <div class="constraint">
                <label for={id}>{ &props.display_name }</label>
                <select id={id} onchange={props.on_change.reform({
                    let id = id.clone();
                    move |e: Event| {
                    let input: HtmlSelectElement = e.target_unchecked_into();
                    let value = input.value();
                    (id.clone(), match value.as_str() {
                        "none" => None,
                        "true" => Some("true".to_string()),
                        "false" => Some("false".to_string()),
                        _ => None,
                    })
                }})}>
                    <option value="none" selected={props.value.is_none()}>{ "Random" }</option>
                    <option value="true" selected={props.value.clone().map_or(false, |v| v == "true")}>{ "Present" }</option>
                    <option value="false" selected={props.value.clone().map_or(false, |v| v == "false")}>{ "Absent" }</option>
                </select>
            </div>
        },
    }
}
