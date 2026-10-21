//! Device info endpoint used to add a peer by IP address.
//!
//! API: `GET /api/v1/info` returns this device's `DeviceInfo` as JSON
//! (`id`, `name`, `os`, `port`, `protocol_version`; `addresses` is always empty).
//! A client certificate is required: 401 without one, so browsers never see it.

mod local_device;
mod routes;

#[cfg(test)]
mod tests;

pub use local_device::LocalDevice;
pub use routes::InfoRoutes;
