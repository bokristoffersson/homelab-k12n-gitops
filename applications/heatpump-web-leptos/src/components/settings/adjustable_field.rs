use leptos::*;
use crate::api::ApiError;
use crate::models::{HeatpumpSetting, SettingPatch};

/// Adjustable field with +/- buttons
#[component]
pub fn AdjustableField(
    label: &'static str,
    value: Option<f64>,
    device_id: String,
    field_name: &'static str,
    unit: &'static str,
    on_adjust: Action<(String, SettingPatch), Result<HeatpumpSetting, ApiError>>,
    is_pending: Signal<bool>,
) -> impl IntoView {
    match value {
        Some(val) => {
            let device_id_dec = device_id.clone();
            let device_id_inc = device_id.clone();

            let on_decrement = move |_| {
                let patch = create_patch(field_name, val - 1.0);
                on_adjust.dispatch((device_id_dec.clone(), patch));
            };

            let on_increment = move |_| {
                let patch = create_patch(field_name, val + 1.0);
                on_adjust.dispatch((device_id_inc.clone(), patch));
            };

            view! {
                <div class="setting-item adjustable">
                    <span class="setting-label">{label}</span>
                    <div class="setting-value-controls">
                        <button
                            class="adjust-button"
                            on:click=on_decrement
                            disabled=move || is_pending.get()
                            aria-label=format!("Decrease {}", label)
                        >
                            "-"
                        </button>
                        <span class="setting-value">
                            {format_value(val)}{unit}
                        </span>
                        <button
                            class="adjust-button"
                            on:click=on_increment
                            disabled=move || is_pending.get()
                            aria-label=format!("Increase {}", label)
                        >
                            "+"
                        </button>
                    </div>
                </div>
            }.into_view()
        }
        None => {
            view! {
                <div class="setting-item">
                    <span class="setting-label">{label}</span>
                    <span class="setting-value">"N/A"</span>
                </div>
            }.into_view()
        }
    }
}

/// Create a SettingPatch for the given field
fn create_patch(field_name: &str, value: f64) -> SettingPatch {
    let mut patch = SettingPatch::default();

    match field_name {
        "indoor_target_temp" => patch.indoor_target_temp = Some(value),
        "mode" => patch.mode = Some(value as i32),
        "curve" => patch.curve = Some(value as i32),
        "curve_min" => patch.curve_min = Some(value as i32),
        "curve_max" => patch.curve_max = Some(value as i32),
        "curve_plus_5" => patch.curve_plus_5 = Some(value as i32),
        "curve_zero" => patch.curve_zero = Some(value as i32),
        "curve_minus_5" => patch.curve_minus_5 = Some(value as i32),
        "heatstop" => patch.heatstop = Some(value as i32),
        "integral_setting" => patch.integral_setting = Some(value),
        _ => {}
    }

    patch
}

/// Format value for display
fn format_value(val: f64) -> String {
    if val.fract() == 0.0 {
        format!("{:.0}", val)
    } else {
        format!("{:.1}", val)
    }
}
