use yew::prelude::*;

#[function_component]
pub fn LoadingSpinner() -> Html {
    html! {
        <div class="loading-spinner">
            <div class="spinner"></div>
            <p>{"Loading..."}</p>
        </div>
    }
}

#[derive(Clone, Properties)]
pub struct LoadingOverlayProps {
    #[prop_or_default]
    pub message: String,
}

#[function_component]
pub fn LoadingOverlay(props: &LoadingOverlayProps) -> Html {
    html! {
        <div class="loading-overlay">
            <div class="spinner"></div>
            <p>{&props.message}</p>
        </div>
    }
}
