use serde::Serialize;

pub use crate::repositories::plugs::{PowerPlug, PowerPlugSchedule};

#[derive(Debug, Serialize)]
pub struct PlugsListResponse {
    pub plugs: Vec<PowerPlug>,
}

#[derive(Debug, Serialize)]
pub struct PlugResponse {
    #[serde(flatten)]
    pub plug: PowerPlug,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outbox_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outbox_status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SchedulesListResponse {
    pub schedules: Vec<PowerPlugSchedule>,
}

#[derive(Debug, Serialize)]
pub struct ScheduleResponse {
    #[serde(flatten)]
    pub schedule: PowerPlugSchedule,
}
