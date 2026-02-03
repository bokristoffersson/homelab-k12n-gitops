//! Schedule executor background task
//!
//! Runs every minute to check for due power plug schedules and creates
//! outbox entries for scheduled actions.

use chrono::{Local, Timelike};
use sqlx::PgPool;
use std::time::Duration;
use tokio::time::interval;

use crate::repositories::plugs::SchedulesRepository;

/// Configuration for the schedule executor
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// How often to check for due schedules (default: 60 seconds)
    pub check_interval_secs: u64,
    /// Timezone offset in hours from UTC (for local time matching)
    /// Not currently used - we use the system's local time
    #[allow(dead_code)]
    pub timezone_offset_hours: i32,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            check_interval_secs: 60,
            timezone_offset_hours: 0,
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
            "Schedule executor started (interval: {}s)",
            self.config.check_interval_secs
        );

        let mut interval = interval(Duration::from_secs(self.config.check_interval_secs));

        // Skip the first immediate tick to align with minute boundaries
        interval.tick().await;

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
        // Get current local time (truncated to the current minute)
        let now = Local::now();
        let current_time = now.time().with_second(0).and_then(|t| t.with_nanosecond(0));

        let current_time = match current_time {
            Some(t) => t,
            None => {
                tracing::warn!("Failed to truncate current time");
                return Ok(());
            }
        };

        tracing::debug!(
            "Checking for due schedules at {}",
            current_time.format("%H:%M")
        );

        // Get schedules due in the current minute
        let schedules_repo = SchedulesRepository::new(self.pool.clone());
        let due_schedules = schedules_repo.get_due_schedules(current_time).await?;

        if due_schedules.is_empty() {
            tracing::debug!("No schedules due at {}", current_time.format("%H:%M"));
            return Ok(());
        }

        tracing::info!(
            "Found {} schedule(s) due at {}",
            due_schedules.len(),
            current_time.format("%H:%M")
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
