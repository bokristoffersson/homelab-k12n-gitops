use crate::api::ApiClient;
use crate::models::{HeatpumpMode, HeatpumpSetting, SettingPatch};
use leptos::*;

use super::adjustable_field::AdjustableField;

/// Settings page component
#[component]
pub fn Settings() -> impl IntoView {
    let client = ApiClient::new();
    let client_fetch = client.clone();
    let client_update = client.clone();

    // Trigger for refetching settings
    let (trigger, set_trigger) = create_signal(0);

    // Fetch settings
    let settings_resource = create_local_resource(
        move || trigger.get(),
        move |_| {
            let client = client_fetch.clone();
            async move { client.get_settings().await }
        },
    );

    // Create action for updates
    let update_setting = create_action(move |input: &(String, SettingPatch)| {
        let (device_id, patch) = input.clone();
        let client = client_update.clone();
        async move {
            let result = client.update_settings(&device_id, &patch).await;
            result
        }
    });

    // When update completes, refetch settings
    create_effect(move |_| {
        if update_setting.value().get().is_some() {
            set_trigger.update(|n| *n += 1);
        }
    });

    // Manual refresh button handler
    let refresh = move |_| {
        set_trigger.update(|n| *n += 1);
    };

    // Set up polling interval (30 seconds)
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_timers::callback::Interval;

        let settings_interval = Interval::new(30_000, move || {
            set_trigger.update(|n| *n += 1);
        });

        on_cleanup(move || drop(settings_interval));
    }

    view! {
        <div class="settings-page">
            <div class="settings-header">
                <h2>"Heatpump Settings"</h2>
                <button class="refresh-button" on:click=refresh>
                    "Refresh"
                </button>
            </div>

            <Suspense fallback=move || view! {
                <div class="loading">"Loading settings..."</div>
            }>
                {move || {
                    settings_resource.get().map(|result| {
                        match result {
                            Ok(response) if response.settings.is_empty() => {
                                view! {
                                    <div class="no-data">"No heatpump devices found"</div>
                                }.into_view()
                            }
                            Ok(response) => {
                                view! {
                                    <div class="settings-grid">
                                        <For
                                            each=move || response.settings.clone()
                                            key=|setting| setting.device_id.clone()
                                            children=move |setting| {
                                                let device_id = setting.device_id.clone();
                                                let is_pending = update_setting.pending();

                                                view! {
                                                    <SettingsCard
                                                        setting=setting
                                                        device_id=device_id
                                                        on_adjust=update_setting
                                                        is_pending=is_pending.into()
                                                    />
                                                }
                                            }
                                        />
                                    </div>
                                }.into_view()
                            }
                            Err(e) => {
                                view! {
                                    <div class="error-banner">
                                        <strong>"Error loading settings:"</strong>
                                        <div>{format!("{}", e)}</div>
                                    </div>
                                }.into_view()
                            }
                        }
                    })
                }}
            </Suspense>

            <div class="settings-footer">
                <div class="info-box">
                    <strong>"Increment/Decrement Control"</strong>
                    <p>
                        "Use the +/- buttons to adjust settings one step at a time. Each change is queued in the outbox and sent to your heatpump via MQTT."
                    </p>
                </div>
            </div>
        </div>
    }
}

/// Settings card for a single device
#[component]
fn SettingsCard(
    setting: HeatpumpSetting,
    device_id: String,
    on_adjust: Action<(String, SettingPatch), Result<HeatpumpSetting, crate::api::ApiError>>,
    is_pending: Signal<bool>,
) -> impl IntoView {
    let device_id = device_id.clone();

    // Get mode display string
    let mode_display = setting
        .mode
        .and_then(HeatpumpMode::from_i32)
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| {
            setting
                .mode
                .map(|m| format!("Unknown ({})", m))
                .unwrap_or("N/A".to_string())
        });

    view! {
        <div class="settings-card">
            <div class="settings-card-header">
                <h3>{device_id.clone()}</h3>
                <span class="last-updated">
                    "Updated: " {format_timestamp(&setting.updated_at)}
                </span>
            </div>

            <div class="settings-section">
                <h4>"Temperature Control"</h4>
                <AdjustableField
                    label="Indoor Target Temperature"
                    value=setting.indoor_target_temp
                    device_id=device_id.clone()
                    field_name="indoor_target_temp"
                    unit="°C"
                    on_adjust=on_adjust
                    is_pending=is_pending
                />
                <div class="setting-item">
                    <span class="setting-label">"Mode"</span>
                    <span class="setting-value setting-badge">{mode_display}</span>
                </div>
            </div>

            <div class="settings-section">
                <h4>"Heating Curve"</h4>
                <AdjustableField
                    label="Curve"
                    value=setting.curve.map(|v| v as f64)
                    device_id=device_id.clone()
                    field_name="curve"
                    unit=""
                    on_adjust=on_adjust
                    is_pending=is_pending
                />
                <AdjustableField
                    label="Curve Min"
                    value=setting.curve_min.map(|v| v as f64)
                    device_id=device_id.clone()
                    field_name="curve_min"
                    unit="°C"
                    on_adjust=on_adjust
                    is_pending=is_pending
                />
                <AdjustableField
                    label="Curve Max"
                    value=setting.curve_max.map(|v| v as f64)
                    device_id=device_id.clone()
                    field_name="curve_max"
                    unit="°C"
                    on_adjust=on_adjust
                    is_pending=is_pending
                />
            </div>

            <div class="settings-section">
                <h4>"Other Settings"</h4>
                <AdjustableField
                    label="Heat Stop"
                    value=setting.heatstop.map(|v| v as f64)
                    device_id=device_id.clone()
                    field_name="heatstop"
                    unit=""
                    on_adjust=on_adjust
                    is_pending=is_pending
                />
                <AdjustableField
                    label="Integral (d73)"
                    value=setting.integral_setting
                    device_id=device_id.clone()
                    field_name="integral_setting"
                    unit=""
                    on_adjust=on_adjust
                    is_pending=is_pending
                />
            </div>
        </div>
    }
}

/// Format timestamp for display
fn format_timestamp(ts: &str) -> String {
    // Simple formatting - in production, use chrono
    ts.split('T')
        .next()
        .map(|s| s.to_string())
        .unwrap_or_else(|| ts.to_string())
}
