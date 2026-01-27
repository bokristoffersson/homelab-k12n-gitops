use crate::api::ApiError;
use crate::models::{HeatpumpStatus, TemperatureLatest};
use leptos::*;

#[component]
pub fn TemperaturesCard(
    heatpump: Result<HeatpumpStatus, ApiError>,
    indoor_temp: Result<Vec<TemperatureLatest>, ApiError>,
) -> impl IntoView {
    match heatpump {
        Ok(hp) => {
            // Get indoor temperature if available
            let indoor = indoor_temp.ok().and_then(|temps| {
                temps
                    .into_iter()
                    .find(|t| t.location.as_ref().map(|l| l == "indoor").unwrap_or(false))
            });

            view! {
                <div class="card">
                    <h3>"Temperatures"</h3>
                    <div class="temp-grid">
                        <TempItem label="Indoor" value=indoor.as_ref().and_then(|t| t.temperature_c) />
                        <TempItem label="Outdoor" value=hp.outdoor_temp />
                        <TempItem label="Supply" value=hp.supplyline_temp />
                        <TempItem label="Return" value=hp.returnline_temp />
                        <TempItem label="Hot Water" value=hp.hotwater_temp />
                        <TempItem label="Brine Out" value=hp.brine_out_temp />
                        <TempItem label="Brine In" value=hp.brine_in_temp />
                        <TempItem label="Integral" value=hp.integral />
                    </div>
                    {indoor.as_ref().and_then(|t| t.humidity).map(|h| {
                        view! {
                            <div class="humidity-info">
                                {format!("Indoor Humidity: {:.1}%", h)}
                            </div>
                        }
                    })}
                </div>
            }.into_view()
        }
        Err(e) => view! {
            <div class="card card-error">
                <h3>"Temperatures"</h3>
                <div class="error-message">
                    {format!("Error: {}", e)}
                </div>
            </div>
        }
        .into_view(),
    }
}

#[component]
fn TempItem(label: &'static str, value: Option<f64>) -> impl IntoView {
    view! {
        <div class="temp-item">
            <span class="temp-label">{label}":"</span>
            <span class="temp-value">
                {value.map(|v| format!("{:.1}°C", v)).unwrap_or_else(|| "N/A".to_string())}
            </span>
        </div>
    }
}
