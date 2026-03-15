use yew::prelude::*;
use crate::services::{SimServerClient, SimulationJob, SimulationMetrics};
use crate::components::{Card, LoadingSpinner, ErrorMessage};

#[function_component]
pub fn SimulationsPage() -> Html {
    let jobs = use_state(|| Vec::<SimulationJob>::new());
    let selected_job = use_state(|| Option::<SimulationJob>::None);
    let loading = use_state(|| true);
    let error = use_state(|| Option::<String>::None);
    let refreshing = use_state(|| false);

    // Load jobs on mount
    {
        let jobs = jobs.clone();
        let loading = loading.clone();
        let error = error.clone();
        
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                match load_jobs().await {
                    Ok(data) => jobs.set(data),
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
            });
            || {}
        });
    }

    async fn load_jobs() -> Result<Vec<SimulationJob>, String> {
        let client = SimServerClient::from_env();
        client.list_jobs().await
    }

    let on_refresh = {
        let jobs = jobs.clone();
        let selected_job = selected_job.clone();
        let loading = loading.clone();
        let error = error.clone();
        let refreshing = refreshing.clone();
        
        Callback::from(move |_| {
            let jobs = jobs.clone();
            let selected_job = selected_job.clone();
            let loading = loading.clone();
            let error = error.clone();
            let refreshing = refreshing.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                refreshing.set(true);
                match load_jobs().await {
                    Ok(data) => {
                        jobs.set(data);
                        selected_job.set(None);
                        error.set(None);
                    }
                    Err(e) => error.set(Some(e)),
                }
                refreshing.set(false);
            });
        })
    };

    let on_select_job = {
        let selected_job = selected_job.clone();
        let jobs = jobs.clone();
        Callback::from(move |job_id: String| {
            selected_job.set(
                jobs.iter()
                    .find(|j| j.id == job_id)
                    .cloned()
            );
        })
    };

    if *loading {
        return html! { <LoadingSpinner /> };
    }

    if let Some(err) = &*error {
        return html! { <ErrorMessage error={err.clone()} on_retry={Some(on_refresh.clone())} /> };
    }

    html! {
        <div class="simulations-page">
            <div class="page-header">
                <h1>{"Simulation Results"}</h1>
                <button class="refresh-button" onclick={on_refresh}>
                    {if *refreshing { "Refreshing..." } else { "Refresh" }}
                </button>
            </div>

            <div class="simulations-layout">
                <Card title="Jobs">
                    if jobs.is_empty() {
                        <p class="no-data">{"No simulation jobs found. Run a simulation to get started."}</p>
                    } else {
                        <div class="job-list">
                            {for jobs.iter().map(|job| {
                                let is_selected = selected_job.as_ref().map_or(false, |s| s.id == job.id);
                                let status_class = match job.status.as_str() {
                                    "Completed" => "status-completed",
                                    "Failed" => "status-failed",
                                    _ => "status-running"
                                };
                                
                                html! {
                                    <div class={classes!("job-item", if is_selected { "selected" } else { ""})} 
                                         onclick={on_select_job.reform(move || job.id.clone())}>
                                        <div class="job-header">
                                            <span class="job-id">{job.id.chars().take(12).collect::<String>()}...</span>
                                            <span class={classes!("job-status", status_class)}>
                                                {&job.status}
                                            </span>
                                        </div>
                                        <div class="job-meta">
                                            <span>{"Model: "}{&job.model_name}</span>
                                            if let Some(metrics) = &job.metrics {
                                                <span>{"FPS: "}{metrics.fps}{" | Latency: "}{metrics.inference_latency_ms}{"ms"}</span>
                                            }
                                        </div>
                                        <div class="job-time">
                                            {format_job_time(&job.id)} // Placeholder - would have timestamp in real impl
                                        </div>
                                    </div>
                                }
                            })}
                        </div>
                    }
                </Card>

                if let Some(job) = &*selected_job {
                    <Card title={format!("Job Details: {}", &job.id.chars().take(8).collect::<String>())}>
                        <div class="job-details">
                            <div class="detail-row">
                                <span class="label">{"Status:"}</span>
                                <span class={classes!("value", match job.status.as_str() {
                                    "Completed" => "text-success",
                                    "Failed" => "text-error",
                                    _ => "text-running"
                                })}>
                                    {&job.status}
                                </span>
                            </div>
                            <div class="detail-row">
                                <span class="label">{"Model:"}</span>
                                <span class="value">{&job.model_name}</span>
                            </div>
                            if let Some(world) = &job.world_file {
                                <div class="detail-row">
                                    <span class="label">{"World:"}</span>
                                    <span class="value">{world}</span>
                                </div>
                            }
                            if let Some(scenes) = &job.scenes.as_option().cloned() {
                                <div class="detail-row">
                                    <span class="label">{"Scenes:"}</span>
                                    <span class="value">{scenes.join(", ")}</span>
                                </div>
                            }
                            if let Some(metrics) = &job.metrics {
                                <div class="metrics-section">
                                    <h4>{"Performance Metrics"}</h4>
                                    <div class="metrics-grid">
                                        <div class="metric">
                                            <div class="metric-value">{metrics.fps}</div>
                                            <div class="metric-label">{"FPS"}</div>
                                        </div>
                                        <div class="metric">
                                            <div class="metric-value">{metrics.inference_latency_ms}</div>
                                            <div class="metric-label">{"Latency (ms)"}</div>
                                        </div>
                                        <div class="metric">
                                            <div class="metric-value">{metrics.memory_peak_mb}</div>
                                            <div class="metric-label">{"Memory (MB)"}</div>
                                        </div>
                                        <div class="metric">
                                            <div class="metric-value">{metrics.total_frames}</div>
                                            <div class="metric-label">{"Frames"}</div>
                                        </div>
                                    </div>
                                </div>
                            }
                            if let Some(error_msg) = &job.error {
                                <div class="error-details">
                                    <h4>{"Error"}</h4>
                                    <p class="error-text">{error_msg}</p>
                                </div>
                            }
                        </div>
                    </Card>
                } else {
                    <Card title="Details">
                        <div class="empty-state">
                            <p>{"Select a job to view details"}</p>
                        </div>
                    </Card>
                }
            </div>
        </div>
    }
}

fn format_job_time(job_id: &str) -> String {
    // Placeholder - would extract timestamp from job.id or have a timestamp field
    "Recent".to_string()
}
