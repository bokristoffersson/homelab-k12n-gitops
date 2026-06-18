use crate::api::models::prices::{LatestResponse, PriceExtreme, PricePoint, PricesResponse};
use crate::repositories::{SpotPrice, SpotPriceRepository};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use chrono::{Local, NaiveDate};

/// Today's SE3 prices (local delivery date).
pub async fn get_today(
    State((pool, config, _validator)): State<crate::auth::AppState>,
) -> Result<Json<PricesResponse>, StatusCode> {
    let date = Local::now().date_naive();
    let response = build_prices(&pool, &config, date)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(response))
}

/// Tomorrow's SE3 prices. Returns 204 until they have been fetched.
pub async fn get_tomorrow(
    State((pool, config, _validator)): State<crate::auth::AppState>,
) -> Result<Response, StatusCode> {
    let Some(date) = Local::now().date_naive().succ_opt() else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let response = build_prices(&pool, &config, date)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if response.prices.is_empty() {
        return Ok(StatusCode::NO_CONTENT.into_response());
    }
    Ok(Json(response).into_response())
}

/// Combined view: today's prices plus tomorrow's when available.
pub async fn get_latest(
    State((pool, config, _validator)): State<crate::auth::AppState>,
) -> Result<Json<LatestResponse>, StatusCode> {
    let today_date = Local::now().date_naive();
    let today = build_prices(&pool, &config, today_date)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let tomorrow = match today_date.succ_opt() {
        Some(date) => {
            let resp = build_prices(&pool, &config, date)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            if resp.prices.is_empty() {
                None
            } else {
                Some(resp)
            }
        }
        None => None,
    };

    Ok(Json(LatestResponse { today, tomorrow }))
}

async fn build_prices(
    pool: &crate::db::DbPool,
    config: &crate::config::Config,
    date: NaiveDate,
) -> Result<PricesResponse, crate::error::AppError> {
    let area = &config.nordpool.delivery_area;
    let rows = SpotPriceRepository::get_for_local_date(pool, area, date).await?;
    Ok(to_response(area, &config.nordpool.currency, date, rows))
}

fn to_response(
    area: &str,
    currency: &str,
    date: NaiveDate,
    rows: Vec<SpotPrice>,
) -> PricesResponse {
    let source_updated_at = rows.iter().filter_map(|r| r.source_updated_at).max();

    let min = rows
        .iter()
        .min_by(|a, b| a.price_per_kwh.total_cmp(&b.price_per_kwh))
        .map(|r| PriceExtreme {
            time: r.time,
            price_ore_per_kwh: r.price_per_kwh * 100.0,
        });
    let max = rows
        .iter()
        .max_by(|a, b| a.price_per_kwh.total_cmp(&b.price_per_kwh))
        .map(|r| PriceExtreme {
            time: r.time,
            price_ore_per_kwh: r.price_per_kwh * 100.0,
        });

    let prices = rows
        .into_iter()
        .map(|r| PricePoint {
            time: r.time,
            price_per_kwh: r.price_per_kwh,
            price_ore_per_kwh: r.price_per_kwh * 100.0,
        })
        .collect();

    PricesResponse {
        delivery_area: area.to_string(),
        currency: currency.to_string(),
        date,
        source_updated_at,
        min,
        max,
        prices,
    }
}
