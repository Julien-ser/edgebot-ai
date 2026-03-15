use yew::prelude::*;

#[derive(Clone, Properties)]
pub struct ErrorMessageProps {
    pub error: String,
    #[prop_or_default]
    pub on_retry: Option<Callback<MouseEvent>>,
}

#[function_component]
pub fn ErrorMessage(props: &ErrorMessageProps) -> Html {
    html! {
        <div class="error-message">
            <div class="error-icon">{"⚠️"}</div>
            <div class="error-content">
                <p class="error-text">{&props.error}</p>
                if let Some(retry) = &props.on_retry {
                    <button class="retry-button" onclick={retry}>
                        {"Retry"}
                    </button>
                }
            </div>
        </div>
    }
}
