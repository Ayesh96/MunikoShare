use std::time::Duration;

pub const ROUTE_REQUEST: &str = "/api/v1/pairing/request";
pub const ROUTE_CONFIRM: &str = "/api/v1/pairing/confirm";

pub const CODE_LENGTH: u32 = 6;
pub const CODE_TTL: Duration = Duration::from_secs(120);
pub const MAX_ATTEMPTS: u32 = 5;
