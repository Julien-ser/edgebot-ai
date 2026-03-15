use yew::prelude::*;

#[derive(Clone, Properties)]
pub struct CardProps {
    pub title: String,
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub class: Option<String>,
}

#[function_component]
pub fn Card(props: &CardProps) -> Html {
    let class = props.class.as_deref().unwrap_or("card");
    
    html! {
        <div class={classes!("dashboard-card", class)}>
            <h3>{&props.title}</h3>
            <div class="card-content">
                {for props.children.iter()}
            </div>
        </div>
    }
}
