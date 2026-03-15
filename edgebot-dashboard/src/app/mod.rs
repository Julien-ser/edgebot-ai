use yew::prelude::*;
use yew_router::prelude::*;
use crate::pages::{dashboard::DashboardPage, simulations::SimulationsPage, metrics::MetricsPage, license::LicensePage};

#[derive(Routable, PartialEq, Clone, Debug)]
enum Route {
    #[at("/")]
    Dashboard,
    #[at("/simulations")]
    Simulations,
    #[at("/metrics")]
    Metrics,
    #[at("/license")]
    License,
}

#[function_component]
pub fn App() -> Html {
    html! {
        <BrowserRouter<Route>>
            <Navbar />
            <main class="main-content">
                <Switch<Route> render=|route| {
                    match route {
                        Route::Dashboard => html! { <DashboardPage /> },
                        Route::Simulations => html! { <SimulationsPage /> },
                        Route::Metrics => html! { <MetricsPage /> },
                        Route::License => html! { <LicensePage /> },
                    }
                } />
            </main>
            <Footer />
        </BrowserRouter<Route>>
    }
}

#[function_component]
fn Navbar() -> Html {
    let navigator = use_navigator().unwrap();

    html! {
        <nav class="navbar">
            <div class="nav-brand">
                <h1>{"EdgeBot AI"}</h1>
            </div>
            <ul class="nav-links">
                <li class={classes!("nav-item", if is_active(&navigator, Route::Dashboard) { "active" } else { ""})}
                    onclick={navigator.callback(move |_| Route::Dashboard)}>
                    {"Dashboard"}
                </li>
                <li class={classes!("nav-item", if is_active(&navigator, Route::Simulations) { "active" } else { ""})}
                    onclick={navigator.callback(move |_| Route::Simulations)}>
                    {"Simulations"}
                </li>
                <li class={classes!("nav-item", if is_active(&navigator, Route::Metrics) { "active" } else { ""})}
                    onclick={navigator.callback(move |_| Route::Metrics)}>
                    {"Metrics"}
                </li>
                <li class={classes!("nav-item", if is_active(&navigator, Route::License) { "active" } else { ""})}
                    onclick={navigator.callback(move |_| Route::License)}>
                    {"License"}
                </li>
            </ul>
        </nav>
    }
}

fn is_active(navigator: &Navigator<Route>, route: Route) -> bool {
    navigator.route() == Ok(&route)
}

#[function_component]
fn Footer() -> Html {
    html! {
        <footer class="footer">
            <p>{"EdgeBot AI Dashboard • Built with Yew"}</p>
        </footer>
    }
}
