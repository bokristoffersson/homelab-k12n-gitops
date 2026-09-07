//! Schedule executor background task
//!
//! Runs every minute to check for due power plug schedules and creates
//! outbox entries for scheduled actions.
//!
//! A schedule is due when its time_of_day has passed today (local time, so
//! the pod's TZ must be set correctly) and it has not fired since today's
//! scheduled instant. This is restart-safe: if a tick drifts across a minute
//! boundary or the pod restarts at the scheduled minute, the schedule still
//! fires on the next tick instead of being skipped for the day. Catch-up
//! fires run in time_of_day order, so the plug converges to the state the
//! latest passed schedule intended.

use chrono::{DateTime, Local, TimeZone, Utc};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::interval;

use crate::repositories::plugs::SchedulesRepository;

/// Configuration for the schedule executor
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// How often to check for due schedules (default: 60 seconds)
    pub check_interval_secs: u64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 60,
        }
    }
}

/// Schedule executor that runs as a background task
pub struct ScheduleExecutor {
    pool: PgPool,
    config: SchedulerConfig,
}

impl ScheduleExecutor {
    pub fn new(pool: PgPool, config: SchedulerConfig) -> Self {
        Self { pool, config }
    }

    /// Run the scheduler loop
    ///
    /// This task checks every minute for schedules that are due and creates
    /// outbox entries to trigger MQTT commands.
    pub async fn run(&self) {
        tracing::info!(
            "Schedule executor started (interval: {}s, local timezone: {})",
            self.config.check_interval_secs,
            Local::now().format("%Z %z")
        );

        let mut interval = interval(Duration::from_secs(self.config.check_interval_secs));

        loop {
            interval.tick().await;

            if let Err(e) = self.check_and_execute_schedules().await {
                tracing::error!("Schedule executor error: {}", e);
            }
        }
    }

    /// Check for due schedules and create outbox entries
    async fn check_and_execute_schedules(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let now = Local::now();

        let local_midnight_utc = match local_midnight_utc(&now) {
            Some(t) => t,
            None => {
                tracing::warn!("Failed to resolve local midnight, skipping tick");
                return Ok(());
            }
        };

        let schedules_repo = SchedulesRepository::new(self.pool.clone());
        let due_schedules = schedules_repo
            .get_due_schedules(now.time(), local_midnight_utc)
            .await?;

        if due_schedules.is_empty() {
            tracing::debug!("No schedules due at {}", now.format("%H:%M"));
            return Ok(());
        }

        tracing::info!(
            "Found {} schedule(s) due at {}",
            due_schedules.len(),
            now.format("%H:%M")
        );

        // Process each due schedule
        for schedule in due_schedules {
            if let Err(e) = self.execute_schedule(&schedule).await {
                tracing::error!(
                    "Failed to execute schedule {} for plug {}: {}",
                    schedule.id,
                    schedule.plug_id,
                    e
                );
                // Continue processing other schedules
            }
        }

        Ok(())
    }

    /// Execute a single schedule by creating an outbox entry
    async fn execute_schedule(
        &self,
        schedule: &crate::repositories::plugs::PowerPlugSchedule,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let status = schedule.action == "on";

        tracing::info!(
            "Executing schedule {} for plug {}: action={}, time={}",
            schedule.id,
            schedule.plug_id,
            schedule.action,
            schedule.time_of_day.format("%H:%M:%S")
        );

        // Begin transaction
        let mut tx = self.pool.begin().await?;

        // Insert outbox entry for the scheduled command
        let outbox_entry =
            crate::repositories::outbox::OutboxRepository::insert_scheduled_plug_command_in_tx(
                &mut tx,
                &schedule.plug_id,
                status,
                schedule.id,
            )
            .await?;

        // Record the fire atomically with the outbox insert
        SchedulesRepository::mark_fired_in_tx(&mut tx, schedule.id).await?;

        // Commit transaction
        tx.commit().await?;

        tracing::info!(
            "Created outbox entry {} for scheduled action on plug {}",
            outbox_entry.id,
            schedule.plug_id
        );

        Ok(())
    }
}

/// UTC instant of today's local midnight; None only if the local midnight
/// does not exist in the timezone (never the case for CET/CEST)
fn local_midnight_utc(now: &DateTime<Local>) -> Option<DateTime<Utc>> {
    let midnight = now.date_naive().and_hms_opt(0, 0, 0)?;
    Local
        .from_local_datetime(&midnight)
        .earliest()
        .map(|dt| dt.with_timezone(&Utc))
}
