use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub token: String,
    /// APNs environment this token was issued for: "sandbox" or "production".
    pub environment: String,
}
