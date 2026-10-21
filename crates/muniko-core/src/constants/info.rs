use std::time::Duration;

/// Returns this device's DeviceInfo; needs a client certificate.
pub const ROUTE_INFO: &str = "/api/v1/info";
/// Upper bound for a whole add-by-IP probe request.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
