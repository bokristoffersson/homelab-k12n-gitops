use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use tracing::warn;

use crate::config::SmtpConfig;

/// Fire-and-forget email notifications. Sending failures are logged, never
/// surfaced - a booking must not fail because iCloud SMTP is down.
/// When SMTP is unconfigured (or the password is empty) the service is a no-op.
#[derive(Clone)]
pub struct EmailService {
    inner: Option<Inner>,
}

#[derive(Clone)]
struct Inner {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
    notify: Mailbox,
}

impl EmailService {
    pub fn new(config: Option<SmtpConfig>) -> Self {
        let inner = config.and_then(|c| {
            if c.password.trim().is_empty() {
                tracing::info!("SMTP password not set, email notifications disabled");
                return None;
            }
            let transport = match AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&c.host) {
                Ok(builder) => builder
                    .port(c.port)
                    .credentials(Credentials::new(c.username.clone(), c.password.clone()))
                    .build(),
                Err(e) => {
                    warn!("Failed to build SMTP transport: {} (email disabled)", e);
                    return None;
                }
            };
            let from: Mailbox = match c.from.parse() {
                Ok(mb) => mb,
                Err(e) => {
                    warn!("Invalid SMTP from address: {} (email disabled)", e);
                    return None;
                }
            };
            let notify: Mailbox = match c.notify_email.parse() {
                Ok(mb) => mb,
                Err(e) => {
                    warn!("Invalid notify email address: {} (email disabled)", e);
                    return None;
                }
            };
            tracing::info!("Email notifications enabled via {}", c.host);
            Some(Inner {
                transport,
                from,
                notify,
            })
        });
        Self { inner }
    }

    /// Send a notification to the admin (Bo).
    pub fn notify_admin(&self, subject: &str, body: &str) {
        if let Some(inner) = &self.inner {
            self.send(inner.notify.clone(), subject, body);
        }
    }

    /// Send a notification to an arbitrary address (e.g. the kid who had
    /// a booking cancelled). Silently skipped when the address is missing.
    pub fn notify_user(&self, to: Option<&str>, subject: &str, body: &str) {
        let Some(to) = to else { return };
        match to.parse::<Mailbox>() {
            Ok(mailbox) => self.send(mailbox, subject, body),
            Err(e) => warn!("Invalid recipient address {}: {}", to, e),
        }
    }

    fn send(&self, to: Mailbox, subject: &str, body: &str) {
        let Some(inner) = self.inner.clone() else {
            return;
        };
        let message = Message::builder()
            .from(inner.from.clone())
            .to(to)
            .subject(subject)
            .body(body.to_string());

        match message {
            Ok(message) => {
                tokio::spawn(async move {
                    if let Err(e) = inner.transport.send(message).await {
                        warn!("Failed to send email: {}", e);
                    }
                });
            }
            Err(e) => warn!("Failed to build email: {}", e),
        }
    }
}
