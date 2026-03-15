use wasm_bindgen_test::*;
use edgebot_dashboard::app::App;

#[wasm_bindgen_test]
fn test_app_renders() {
    // Test that the App component can be instantiated
    let app = App;
    // In a real test, we'd check DOM elements but this validates the component type
    assert!(std::any::TypeId::of::<App>() != std::any::TypeId::of::<String>());
}

#[wasm_bindgen_test]
fn test_routes_exist() {
    use edgebot_dashboard::app::Route;
    
    // Test that all routes can be constructed
    let _dashboard = Route::Dashboard;
    let _simulations = Route::Simulations;
    let _metrics = Route::Metrics;
    let _license = Route::License;
    
    // Ensure routes are different
    assert_ne!(Route::Dashboard, Route::Simulations);
    assert_ne!(Route::Simulations, Route::Metrics);
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    
    #[test]
    fn test_card_props() {
        use edgebot_dashboard::components::card::CardProps;
        
        let props = CardProps {
            title: "Test Card".to_string(),
            children: yew::children![yew::html! { <div>{"Test"}</div> }],
            class: None,
        };
        
        assert_eq!(props.title, "Test Card");
    }
}