use leptos::prelude::*;

pub enum Severity {
    None,
    Ok,
    Caution,
    Danger,
}

#[component]
pub fn Dialog(
    #[prop(into)] title: String,
    #[prop(into)] content: String,
    #[prop(into)] yes_caption: String,
    severity: Severity,
    #[prop(into, optional)] no_caption: Option<String>,
    #[prop(into, optional)] cancel_caption: Option<String>,
    #[prop(into)] on_yes: Callback<()>,
    #[prop(into, optional)] on_no: Option<Callback<()>>,
    #[prop(into, optional)] on_cancel: Option<Callback<()>>,
) -> impl IntoView {
    let severity_class = match severity {
        Severity::None => "",
        Severity::Ok => "ok",
        Severity::Caution => "caution",
        Severity::Danger => "danger",
    };

    view! {
        <div class="dialog-overlay">
            <dialog class="card" closedby="none" open>
                <div class=format!("caption {severity_class}")>
                    <h2>{title}</h2>
                </div>
                <div class="content">
                    <p>{content}</p>
                </div>
                <div class="caption buttons">
                    <button class="primary" on:click=move |_| on_yes.run(())>
                        {yes_caption}
                    </button>
                    {move || {
                        if let Some(no_caption) = no_caption.clone() {
                            let on_no = on_no.clone();
                            view! {
                                <button
                                    class="secondary"
                                    on:click=move |_| {
                                        if let Some(on_no) = &on_no {
                                            on_no.run(());
                                        }
                                    }
                                >
                                    {no_caption}
                                </button>
                            }
                                .into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}
                    {move || {
                        if let Some(cancel_caption) = cancel_caption.clone() {
                            let on_cancel = on_cancel.clone();
                            view! {
                                <button
                                    class="tertiary"
                                    on:click=move |_| {
                                        if let Some(on_cancel) = &on_cancel {
                                            on_cancel.run(());
                                        }
                                    }
                                >
                                    {cancel_caption}
                                </button>
                            }
                                .into_any()
                        } else {
                            view! {}.into_any()
                        }
                    }}
                </div>
            </dialog>
        </div>
    }
}
