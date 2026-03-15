use yew::prelude::*;
use crate::services::{MetricsClient, ModelMetrics};
use crate::components::{Card, LoadingSpinner, ErrorMessage};

#[function_component]
pub fn MetricsPage() -> Html {
    let models = use_state(|| Vec::<ModelMetrics>::new());
    let loading = use_state(|| true);
    let error = use_state(|| Option::<String>::None);

    {
        let models = models.clone();
        let loading = loading.clone();
        let error = error.clone();
        
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                let client = MetricsClient::new("benchmarks");
                match client.list_models().await {
                    Ok(data) => {
                        models.set(data);
                        error.set(None);
                    }
                    Err(e) => error.set(Some(e)),
                }
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

    let avg_latency = if models.is_empty() {
        0.0
    } else {
        models.iter().map(|m| m.inference_latency_ms).sum::<f64>() / models.len() as f64
    };

    let avg_memory = if models.is_empty() {
        0.0
    } else {
        models.iter().map(|m| m.memory_footprint_mb).sum::<f64>() / models.len() as f64
    };

    html! {
        <div class="metrics-page">
            <h1>{"Model Metrics"}</h1>
            
            <div class="metrics-summary">
                <Card title="Overview">
                    <div class="summary-stats">
                        <div class="summary-stat">
                            <div class="stat-value">{models.len()}</div>
                            <div class="stat-label">{"Models Tracked"}</div>
                        </div>
                        <div class="summary-stat">
                            <div class="stat-value">{avg_latency as u32}</div>
                            <div class="stat-label">{"Avg Latency (ms)"}</div>
                        </div>
                        <div class="summary-stat">
                            <div class="stat-value">{avg_memory as u32}</div>
                            <div class="stat-label">{"Avg Memory (MB)"}</div>
                        </div>
                    </div>
                </Card>
            </div>

            <Card title="Model Performance">
                <div class="models-table">
                    <table>
                        <thead>
                            <tr>
                                <th>{"Model"}</th>
                                <th>{"Inference Latency"}</th>
                                <th>{"Memory Footprint"}</th>
                                <th>{"Model Size"}</th>
                                <th>{"Platform"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {for models.iter().map(|model| {
                                html! {
                                    <tr>
                                        <td class="model-name">{&model.model_name}</td>
                                        <td>{model.inference_latency_ms}{" ms"}</td>
                                        <td>{model.memory_footprint_mb}{" MB"}</td>
                                        <td>{model.model_size_mb}{" MB"}</td>
                                        <td><code>{&model.platform}</code></td>
                                    </tr>
                                }
                            })}
                        </tbody>
                    </table>
                </div>
            </Card>

            <Card title="Optimization Recommendations">
                <div class="recommendations">
                    <p>{"Pro tip: Use the EdgeBot CLI to optimize models for your target platform:"}</p>
                    <pre class="code-block">
                        {r#"edgebot optimize --input model.onnx --output model.ebmodel --quantize int8 --fuse-layers"#}
                    </pre>
                    <p>{"This can reduce model size by up to 75% and improve inference speed by 2-4x."}</p>
                </div>
            </Card>
        </div>
    }
}
