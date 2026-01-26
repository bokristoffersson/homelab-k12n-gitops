use leptos::*;
use crate::api::ApiError;
use crate::models::HeatpumpStatus;

#[component]
pub fn HeatpumpStatusCard(
    data: Result<HeatpumpStatus, ApiError>,
) -> impl IntoView {
    match data {
        Ok(heatpump) => {
            view! {
                <div class="card">
                    <h3>"Heatpump Status"</h3>
                    <div class="status-grid">
                        <StatusBadge label="Compressor" value=heatpump.compressor_on />
                        <StatusBadge label="Hot Water" value=heatpump.hotwater_production />
                        <StatusBadge label="Flow Pump" value=heatpump.flowlinepump_on />
                        <StatusBadge label="Brine Pump" value=heatpump.brinepump_on />
                        <StatusBadge label="Aux 3kW" value=heatpump.aux_heater_3kw_on />
                        <StatusBadge label="Aux 6kW" value=heatpump.aux_heater_6kw_on />
                    </div>
                </div>
            }.into_view()
        }
        Err(e) => {
            view! {
                <div class="card card-error">
                    <h3>"Heatpump Status"</h3>
                    <div class="error-message">
                        {format!("Error: {}", e)}
                    </div>
                </div>
            }.into_view()
        }
    }
}

#[component]
fn StatusBadge(
    label: &'static str,
    value: Option<bool>,
) -> impl IntoView {
    let is_on = value.unwrap_or(false);
    let dot_class = if is_on { "status-dot on" } else { "status-dot off" };

    view! {
        <div class="status-item">
            <span class="status-label">
                <span class=dot_class aria-hidden="true"></span>
                <span class="status-text">{label}</span>
            </span>
        </div>
    }
}
