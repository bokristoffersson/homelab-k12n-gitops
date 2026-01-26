use crate::api::ApiClient;
use leptos::*;

use super::heatpump_status_card::HeatpumpStatusCard;
use super::hourly_total_card::HourlyTotalCard;
use super::temperatures_card::TemperaturesCard;

/// Dashboard page component with data fetching
#[component]
pub fn Dashboard() -> impl IntoView {
    let client = ApiClient::new();
    let client_hourly = client.clone();
    let client_heatpump = client.clone();
    let client_temp = client.clone();

    // Resources for async data fetching (using create_local_resource for CSR)
    // Hourly total - refetch every 60 seconds
    let (hourly_trigger, set_hourly_trigger) = create_signal(0);
    let hourly_total = create_local_resource(
        move || hourly_trigger.get(),
        move |_| {
            let client = client_hourly.clone();
            async move { client.get_hourly_total().await }
        },
    );

    // Heatpump status - refetch every 5 seconds
    let (heatpump_trigger, set_heatpump_trigger) = create_signal(0);
    let heatpump_status = create_local_resource(
        move || heatpump_trigger.get(),
        move |_| {
            let client = client_heatpump.clone();
            async move { client.get_heatpump_status().await }
        },
    );

    // Indoor temperature - refetch every 60 seconds
    let (temp_trigger, set_temp_trigger) = create_signal(0);
    let indoor_temp = create_local_resource(
        move || temp_trigger.get(),
        move |_| {
            let client = client_temp.clone();
            async move { client.get_temperature_latest().await }
        },
    );

    // Set up polling intervals
    #[cfg(target_arch = "wasm32")]
    {
        use gloo_timers::callback::Interval;

        // Hourly total: 60 seconds
        let hourly_interval = Interval::new(60_000, move || {
            set_hourly_trigger.update(|n| *n += 1);
        });

        // Heatpump status: 5 seconds
        let heatpump_interval = Interval::new(5_000, move || {
            set_heatpump_trigger.update(|n| *n += 1);
        });

        // Temperature: 60 seconds
        let temp_interval = Interval::new(60_000, move || {
            set_temp_trigger.update(|n| *n += 1);
        });

        on_cleanup(move || {
            drop(hourly_interval);
            drop(heatpump_interval);
            drop(temp_interval);
        });
    }

    view! {
        <div class="dashboard">
            <div class="dashboard-grid">
                // Power gauge placeholder (Phase 6)
                <div class="card">
                    <h3>"Live Power"</h3>
                    <p class="placeholder-text">"WebSocket gauge coming in Phase 6"</p>
                </div>

                // Hourly total card
                <Suspense fallback=move || view! { <LoadingCard title="This Hour" /> }>
                    {move || {
                        hourly_total.get().map(|result| {
                            view! { <HourlyTotalCard data=result /> }
                        })
                    }}
                </Suspense>

                // Heatpump status card
                <Suspense fallback=move || view! { <LoadingCard title="Heatpump Status" /> }>
                    {move || {
                        heatpump_status.get().map(|result| {
                            view! { <HeatpumpStatusCard data=result.clone() /> }
                        })
                    }}
                </Suspense>

                // Temperatures card
                <Suspense fallback=move || view! { <LoadingCard title="Temperatures" /> }>
                    {move || {
                        let heatpump = heatpump_status.get();
                        let temp = indoor_temp.get();
                        match (heatpump, temp) {
                            (Some(hp_result), Some(temp_result)) => {
                                Some(view! {
                                    <TemperaturesCard
                                        heatpump=hp_result
                                        indoor_temp=temp_result
                                    />
                                })
                            }
                            _ => None
                        }
                    }}
                </Suspense>

                // Chart placeholders (Phase 7)
                <div class="card chart-card">
                    <h3>"24h Energy History"</h3>
                    <p class="placeholder-text">"Charts coming in Phase 7"</p>
                </div>

                <div class="card chart-card">
                    <h3>"24h Temperature History"</h3>
                    <p class="placeholder-text">"Charts coming in Phase 7"</p>
                </div>
            </div>
        </div>
    }
}

/// Loading card placeholder
#[component]
fn LoadingCard(title: &'static str) -> impl IntoView {
    view! {
        <div class="card">
            <h3>{title}</h3>
            <div class="loading">"Loading..."</div>
        </div>
    }
}
