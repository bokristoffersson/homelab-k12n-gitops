use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::Deserialize;
use std::sync::Arc;
use subtle::ConstantTimeEq;

use crate::{
    auth::CurrentUser,
    config::Config,
    email::EmailService,
    error::AppError,
    ics,
    models::{format_range_sv, CalendarUrlResponse, CreateSlotRequest, MeResponse, SlotResponse},
    repository::SlotsRepository,
};

#[derive(Clone)]
pub struct AppState {
    pub repository: SlotsRepository,
    pub config: Arc<Config>,
    pub email: EmailService,
}

pub async fn health() -> &'static str {
    "OK"
}

pub async fn me(Extension(user): Extension<CurrentUser>) -> Json<MeResponse> {
    Json(MeResponse {
        username: user.username,
        email: user.email,
        is_admin: user.is_admin,
    })
}

#[derive(Debug, Deserialize)]
pub struct RangeParams {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

pub async fn list_slots(
    State(state): State<AppState>,
    Query(params): Query<RangeParams>,
) -> Result<Json<Vec<SlotResponse>>, AppError> {
    let now = Utc::now();
    let from = params.from.unwrap_or(now - Duration::days(1));
    let to = params.to.unwrap_or(now + Duration::days(60));
    let slots = state.repository.list(from, to).await?;
    Ok(Json(slots.into_iter().map(SlotResponse::from).collect()))
}

/// Validation for a new slot, kept as a pure function for testability.
fn validate_new_slot(
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Result<(), String> {
    if ends_at <= starts_at {
        return Err("ends_at must be after starts_at".to_string());
    }
    if starts_at <= now {
        return Err("starts_at must be in the future".to_string());
    }
    Ok(())
}

pub async fn create_slot(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Json(req): Json<CreateSlotRequest>,
) -> Result<(StatusCode, Json<SlotResponse>), AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }
    validate_new_slot(req.starts_at, req.ends_at, Utc::now()).map_err(AppError::Validation)?;

    let slot = state
        .repository
        .create(
            req.starts_at,
            req.ends_at,
            req.note.as_deref(),
            &user.username,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(SlotResponse::from(slot))))
}

pub async fn delete_slot(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    if !user.is_admin {
        return Err(AppError::Forbidden);
    }
    let slot = state
        .repository
        .delete(id)
        .await?
        .ok_or(AppError::NotFound)?;

    if slot.booked_by_username.is_some() {
        state.email.notify_user(
            slot.booked_by_email.as_deref(),
            "Körpass inställt",
            &format!(
                "Ditt körpass {} har ställts in.",
                format_range_sv(slot.starts_at, slot.ends_at)
            ),
        );
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn book_slot(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<SlotResponse>, AppError> {
    let booked = state
        .repository
        .book(id, &user.username, user.email.as_deref())
        .await?;

    match booked {
        Some(slot) => {
            state.email.notify_admin(
                "Övningskörning bokad",
                &format!(
                    "{} har bokat övningskörning {}.",
                    slot.display_name().unwrap_or_else(|| user.username.clone()),
                    format_range_sv(slot.starts_at, slot.ends_at)
                ),
            );
            Ok(Json(SlotResponse::from(slot)))
        }
        None => match state.repository.get(id).await? {
            None => Err(AppError::NotFound),
            Some(slot) if slot.booked_by_username.is_some() => {
                Err(AppError::Conflict("slot_taken"))
            }
            Some(_) => Err(AppError::Conflict("slot_past")),
        },
    }
}

/// Who may cancel a booking, kept as a pure function for testability.
fn can_cancel(booked_by: Option<&str>, requester: &str, is_admin: bool) -> bool {
    match booked_by {
        Some(booker) => is_admin || booker == requester,
        None => false,
    }
}

pub async fn cancel_booking(
    State(state): State<AppState>,
    Extension(user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<SlotResponse>, AppError> {
    let slot = state.repository.get(id).await?.ok_or(AppError::NotFound)?;

    if slot.booked_by_username.is_none() {
        return Err(AppError::Conflict("not_booked"));
    }
    if !can_cancel(
        slot.booked_by_username.as_deref(),
        &user.username,
        user.is_admin,
    ) {
        return Err(AppError::Forbidden);
    }

    let cancelled = state
        .repository
        .cancel(id)
        .await?
        .ok_or(AppError::Conflict("not_booked"))?;

    let range = format_range_sv(cancelled.starts_at, cancelled.ends_at);
    let booker_cancelled_own = slot.booked_by_username.as_deref() == Some(user.username.as_str());
    if booker_cancelled_own {
        state.email.notify_admin(
            "Övningskörning avbokad",
            &format!(
                "{} har avbokat övningskörning {}.",
                slot.display_name().unwrap_or_else(|| user.username.clone()),
                range
            ),
        );
    } else {
        state.email.notify_user(
            slot.booked_by_email.as_deref(),
            "Bokning avbokad",
            &format!("Din bokning av körpasset {} har avbokats.", range),
        );
    }

    Ok(Json(SlotResponse::from(cancelled)))
}

pub async fn calendar_url(State(state): State<AppState>) -> Json<CalendarUrlResponse> {
    let base = state.config.app.base_url.trim_end_matches('/');
    let path = format!("/ical/{}/gasa.ics", state.config.app.ical_token);
    let https_url = format!("{}{}", base, path);
    let webcal_url = https_url.replacen("https://", "webcal://", 1);
    Json(CalendarUrlResponse {
        webcal_url,
        https_url,
    })
}

fn token_matches(provided: &str, expected: &str) -> bool {
    !expected.is_empty()
        && provided.len() == expected.len()
        && bool::from(provided.as_bytes().ct_eq(expected.as_bytes()))
}

pub async fn ical_feed(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> Result<Response, AppError> {
    if !token_matches(&token, &state.config.app.ical_token) {
        return Err(AppError::NotFound);
    }

    let now = Utc::now();
    let slots = state
        .repository
        .list(now - Duration::days(30), now + Duration::days(365))
        .await?;
    let calendar = ics::build_calendar(&slots);

    Ok((
        [
            (header::CONTENT_TYPE, "text/calendar; charset=utf-8"),
            (header::CACHE_CONTROL, "no-cache, max-age=300"),
            (header::CONTENT_DISPOSITION, "inline; filename=\"gasa.ics\""),
        ],
        calendar,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn t(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 18, hour, 0, 0).unwrap()
    }

    #[test]
    fn test_validate_new_slot_accepts_future_slot() {
        assert!(validate_new_slot(t(17), t(18), t(10)).is_ok());
    }

    #[test]
    fn test_validate_new_slot_rejects_reversed_times() {
        assert!(validate_new_slot(t(18), t(17), t(10)).is_err());
        assert!(validate_new_slot(t(17), t(17), t(10)).is_err());
    }

    #[test]
    fn test_validate_new_slot_rejects_past_start() {
        assert!(validate_new_slot(t(9), t(18), t(10)).is_err());
    }

    #[test]
    fn test_can_cancel_permission_matrix() {
        // Booker may cancel their own booking
        assert!(can_cancel(Some("erik"), "erik", false));
        // Admin may cancel anyone's booking
        assert!(can_cancel(Some("erik"), "bo", true));
        // Another non-admin user may not
        assert!(!can_cancel(Some("erik"), "alva", false));
        // Unbooked slot: nothing to cancel
        assert!(!can_cancel(None, "bo", true));
    }

    #[test]
    fn test_token_matches() {
        assert!(token_matches("abc123", "abc123"));
        assert!(!token_matches("abc124", "abc123"));
        assert!(!token_matches("abc12", "abc123"));
        assert!(!token_matches("", "abc123"));
        // An empty configured token must never open the feed
        assert!(!token_matches("", ""));
    }
}
