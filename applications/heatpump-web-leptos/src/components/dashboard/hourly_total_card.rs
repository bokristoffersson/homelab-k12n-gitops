use leptos::*;
use crate::api::ApiError;
use crate::models::HourlyTotal;

#[component]
pub fn HourlyTotalCard(
    data: Result<HourlyTotal, ApiError>,
) -> impl IntoView {
    match data {
        Ok(hourly) => {
            view! {
                <div class="card">
                    <h3>"This Hour"</h3>
                    <div class="energy-value">
                        {format!("{:.2}", hourly.total_kwh)}
                        <span class="unit">" kWh"</span>
                    </div>
                    <div class="subtitle">"(so far)"</div>
                </div>
            }.into_view()
        }
        Err(e) => {
            view! {
                <div class="card card-error">
                    <h3>"This Hour"</h3>
                    <div class="error-message">
                        {format!("Error: {}", e)}
                    </div>
                </div>
            }.into_view()
        }
    }
}
