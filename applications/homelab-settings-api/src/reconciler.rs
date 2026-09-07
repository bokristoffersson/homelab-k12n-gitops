//! Desired-state reconciler background task
//!
//! Level-based safety net under the event-based command path: whenever a
//! plug's reported state disagrees with its desired state, a new outbox
//! command is issued until the device converges. This survives failure modes
//! the command path cannot see on its own — a plug that was offline when the
//! command (and all its retries) went out gets fixed as soon as its telemetry
//! reappears.
//!
//! The reconciler only acts on plugs with a fresh telemetry-backed mismatch;
//! a fully silent plug is instead caught by the outbox confirmation timeout
//! and the failed-command alert. External changes (Homebridge/Siri publishing
//! straight to MQTT) are adopted as the new desired state by the Kafka
//! consumer before they ever look like a mismatch here.

use sqlx::PgPool;
use std::time::Duration;
use tokio::time::interval;

use crate::repositories::{outbox::OutboxRepository, plugs::PlugsRepository};

/// Configuration for the desired-state reconciler, from the
/// RECONCILE_INTERVAL_SECS / RECONCILE_GRACE_SECS env vars
#[derive(Debug, Clone)]
pub struct ReconcilerConfig {
    /// How often to look for mismatches. Also used as the per-plug backoff:
    /// at most one reconcile command per interval.
    pub interval_secs: u64,
    /// How long a desired-state change may sit unconverged before the
    /// reconciler steps in, leaving the normal command path room to finish
    pub grace_secs: u64,
}

/// Reconciler that runs as a background task
pub struct DesiredStateReconciler {
    pool: PgPool,
    config: ReconcilerConfig,
}

impl DesiredStateReconciler {
    pub fn new(pool: PgPool, config: ReconcilerConfig) -> Self {
        Self { pool, config }
    }

    /// Run the reconciler loop
    pub async fn run(&self) {
        tracing::info!(
            "Desired-state reconciler started (interval: {}s, grace: {}s)",
            self.config.interval_secs,
            self.config.grace_secs
        );

        let mut interval = interval(Duration::from_secs(self.config.interval_secs));

        // Consume the immediate first tick: reconciling at startup, before the
        // Kafka consumer has refreshed plug state, only produces noise. The
        // grace period would prevent premature commands anyway.
        interval.tick().await;

        loop {
            interval.tick().await;

            if let Err(e) = self.reconcile().await {
                tracing::error!("Reconciler error: {}", e);
            }
        }
    }

    /// Issue a command for every plug whose reported state disagrees with the
    /// desired state
    async fn reconcile(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let plugs_repo = PlugsRepository::new(self.pool.clone());
        let mismatches = plugs_repo
            .get_state_mismatches(self.config.grace_secs, self.config.interval_secs)
            .await?;

        for (plug_id, desired) in mismatches {
            let action = if desired { "ON" } else { "OFF" };
            tracing::warn!(
                "Plug {} has not converged to desired state {}, issuing reconcile command",
                plug_id,
                action
            );

            let mut tx = self.pool.begin().await?;
            let entry =
                OutboxRepository::insert_reconcile_plug_command_in_tx(&mut tx, &plug_id, desired)
                    .await?;
            tx.commit().await?;

            tracing::info!(
                "Created reconcile outbox entry {} for plug {} ({})",
                entry.id,
                plug_id,
                action
            );
        }

        Ok(())
    }
}
