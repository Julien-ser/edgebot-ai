use yew::prelude::*;
use crate::services::{SimServerClient, LicensingClient, ModelMetrics};
use crate::components::{Card, LoadingSpinner, ErrorMessage};

#[derive(Clone, PartialEq)]
struct DashboardState {
    jobs_count: usize,
    recent_jobs: Vec<crate::services::SimulationJob>,
    license_status: String,
    models_count: usize,
    has_pro_license: bool,
}

#[function_component]
pub fn DashboardPage() -> Html {
    let state = use_state(|| Option::<DashboardState>::None);
    let error = use_state(|| Option::<String>::None);
    let loading = use_state(|| true);

    // Fetch dashboard data on mount
    {
        let state = state.clone();
        let error = error.clone();
        let loading = loading.clone();
        
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                error.set(None);
                
                // Fetch jobs
                let sim_client = SimServerClient::from_env();
                let license_client = LicensingClient;
                
                let jobs_result = sim_client.list_jobs().await;
                let jobs = match jobs_result {
                    Ok(jobs) => jobs,
                    Err(e) => {
                        error.set(Some(format!("Failed to load jobs: {}", e)));
                        loading.set(false);
                        return;
                    }
                };
                
                // Check license status
                let has_pro_license = license_client.check_pro_access().is_ok();
                let license_status = if has_pro_license {
                    "✅ Pro".to_string()
                } else {
                    "🆓 Free".to_string()
                };
                
                // Fetch model metrics (simplified)
                let models_result = license_client.get_license_info().ok().flatten();
                
                state.set(Some(DashboardState {
                    jobs_count: jobs.len(),
                    recent_jobs: jobs.iter().take(5).cloned().collect(),
                    license_status: license_status.clone(),
                    models_count: 2, // Placeholder
                    has_pro_license,
                }));
                
                loading.set(false);
            });
            
            || {}
        });
    }

    if *loading {
        return html! { <LoadingSpinner /> };
    }

    if let Some(err) = &*error {
        return html! { <ErrorMessage error={err.clone()} /> };
    }

    let Some(dashboard) = &*state else {
        return html! { <div>{"No data"}</div> };
    };

    html! {
        <div class="dashboard-page">
            <h1>{"EdgeBot Dashboard"}</h1>
            
            <div class="dashboard-grid">
                <Card title="License Status">
                    <div class="license-badge">
                        <span class={classes!("status-badge", if dashboard.has_pro_license { "pro" } else { "free" })}>
                            {&dashboard.license_status}
                        </span>
                    </div>
                    <p class="license-info">
                        if dashboard.has_pro_license {
                            {"All pro features enabled. Access to cloud simulations and advanced optimizations."}
                        } else {
                            {"Free tier active. Upgrade to unlock cloud simulations and advanced optimizations."}
                        }
                    </p>
                </Card>

                <Card title="Simulation Jobs">
                    <div class="stat-value">{dashboard.jobs_count}</div>
                    <p>{"Total jobs"}</p>
                    if !dashboard.recent_jobs.is_empty() {
                        <div class="recent-jobs">
                            <h4>{"Recent Jobs"}</h4>
                            {for dashboard.recent_jobs.iter().map(|job| {
                                html! {
                                    <div class="job-item">
                                        <span class="job-id">{job.id.chars().take(8).collect::<String>()}...</span>
                                        <span class={classes!("job-status", match job.status.as_str() {
                                            "Completed" => "success",
                                            "Failed" => "error",
                                            _ => "running"
                                        })}>
                                            {&job.status}
                                        </span>
                                    </div>
                                }
                            })}
                        </div>
                    } else {
                        <p class="no-data">{"No simulations run yet."}</p>
                    }
                </Card>

                <Card title="Model Metrics">
                    <div class="stat-value">{dashboard.models_count}</div>
                    <p>{"Models tracked"}</p>
                    <div class="quick-links">
                        <a href="#/metrics">{"View All Metrics →"}</a>
                    </div>
                </Card>

                <Card title="Quick Actions">
                    <div class="quick-actions">
                        <a href="#/simulations" class="action-button">{"Run Simulation"}</a>
                        <a href="#/metrics" class="action-button">{"View Metrics"}</a>
                        <a href="#/license" class="action-button">{"Manage License"}</a>
                    </div>
                </Card>
            </div>
        </div>
    )
}
