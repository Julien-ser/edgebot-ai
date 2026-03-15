use yew::prelude::*;
use crate::services::LicensingClient;
use crate::components::{Card, LoadingSpinner, ErrorMessage};

#[derive(Clone, PartialEq)]
struct LicenseState {
    has_license: bool,
    has_pro: bool,
    features: Vec<String>,
    key_preview: Option<String>,
    error: Option<String>,
}

#[function_component]
pub fn LicensePage() -> Html {
    let license_state = use_state(|| Option::<LicenseState>::None);
    let loading = use_state(|| true);

    {
        let license_state = license_state.clone();
        let loading = loading.clone();
        
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                let client = LicensingClient;
                let result = check_license(&client).await;
                license_state.set(result);
                loading.set(false);
            });
            || {}
        });
    }

    if *loading {
        return html! { <LoadingSpinner /> };
    }

    let state = match &*license_state {
        Some(s) => s,
        None => return html! { <ErrorMessage error="Failed to load license info".to_string() /> },
    };

    html! {
        <div class="license-page">
            <h1>{"License Management"}</h1>
            
            <div class="license-grid">
                <Card title="Current Status">
                    <div class="license-status">
                        if state.has_license {
                            <div class={classes!("status-badge", if state.has_pro { "pro" } else { "free" })}>
                                {if state.has_pro { "✅ Pro License Active" } else { "🆓 Free Tier" }}
                            </div>
                        } else {
                            <div class="status-badge free">
                                {"No License Key Set"}
                            </div>
                        }
                    </div>
                    
                    if state.has_pro {
                        <div class="pro-features">
                            <h3>{"Enabled Features"}</h3>
                            <ul>
                                {for state.features.iter().map(|feature| {
                                    html! { <li>{"✅ "}{feature}</li> }
                                })}
                            </ul>
                        </div>
                    } else {
                        <div class="free-tier">
                            <h3>{"Free Tier Features"}</h3>
                            <ul>
                                <li>{"✅ Core inference engine"}</li>
                                <li>{"✅ ROS2 integration"}</li>
                                <li>{"✅ WebAssembly compilation"}</li>
                                <li>{"✅ Local Webots simulation"}</li>
                                <li>{"❌ Cloud simulations (100+ runs)"}</li>
                                <li>{"❌ Advanced model optimization"}</li>
                            </ul>
                            <div class="upgrade-cta">
                                <p>{"Upgrade to Pro ($29/month) to unlock:"}</p>
                                <ul>
                                    <li>{"Cloud simulation with batch processing"}</li>
                                    <li>{"Int8 and FP16 quantization"}</li>
                                    <li>{"Model pruning and layer fusion"}</li>
                                    <li>{"Priority support"}</li>
                                </ul>
                                <a href="https://edgebot.ai/pricing" target="_blank" class="upgrade-button">
                                    {"Upgrade to Pro"}
                                </a>
                            </div>
                        </div>
                    }
                </Card>

                <Card title="License Details">
                    if let Some(key_preview) = &state.key_preview {
                        <div class="license-info">
                            <div class="info-row">
                                <span class="label">{"Key:"}</span>
                                <span class="value">{key_preview}</span>
                            </div>
                            <div class="info-row">
                                <span class="label">{"Status:"}</span>
                                <span class="value text-success">{"Valid"}</span>
                            </div>
                        </div>
                    } else {
                        <div class="no-license">
                            <p>{"No license key is currently set in your environment."}</p>
                            <h4>{"To activate pro features:"}</h4>
                            <ol>
                                <li>{"Subscribe at edgebot.ai/pricing"}</li>
                                <li>{"Receive license key via email"}</li>
                                <li>{"Set environment variable:"}</li>
                            </ol>
                            <pre class="code-block">
                                {r#"export EDGEBOT_LICENSE_KEY="your_key_here""#}
                            </pre>
                        </div>
                    }
                </Card>

                <Card title="Testing">
                    <div class="testing-section">
                        <h3>{"Development License"}</h3>
                        <p>{"Generate a test license for development (debug builds only):"}</p>
                        
                        #[cfg(debug_assertions)]
                        {
                            if state.has_license {
                                <button class="test-button" onclick={|_| {
                                    // Would call a dev license generation function
                                    // This is a placeholder
                                }}>
                                    {"Generate New Dev License"}
                                </button>
                            }
                        }
                        
                        #[cfg(not(debug_assertions))]
                        {
                            <p class="note">{"Dev license generation only available in debug builds"}</p>
                        }
                    </div>
                </Card>
            </div>
        </div>
    }
}

async fn check_license(client: &LicensingClient) -> Option<LicenseState> {
    let has_pro = client.check_pro_access().is_ok();
    let has_license = match client.get_license_info() {
        Ok(Some(info)) => info.has_key,
        _ => false,
    };
    
    let key_preview = client.get_license_info()
        .ok()
        .flatten()
        .map(|info| info.key_preview);
    
    // Extract features from license (would need full verification)
    let features = if has_pro {
        vec!["cloud_sim".to_string(), "optimization".to_string()]
    } else {
        vec![]
    };
    
    Some(LicenseState {
        has_license,
        has_pro,
        features,
        key_preview,
        error: None,
    })
}
