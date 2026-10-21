//! Per-IP limiter for failed token redemptions.

use std::collections::HashMap;
use std::net::IpAddr;

#[derive(Default)]
pub struct AttemptLimiter {
    failures_by_ip: HashMap<IpAddr, Vec<u64>>,
}

impl AttemptLimiter {
    pub fn is_blocked(&mut self, ip: IpAddr, now: u64, window_secs: u64, max: usize) -> bool {
        let failures = self.failures_by_ip.entry(ip).or_default();
        failures.retain(|at| now < at + window_secs);
        failures.len() >= max
    }

    pub fn record_failure(&mut self, ip: IpAddr, now: u64) {
        self.failures_by_ip.entry(ip).or_default().push(now);
    }

    pub fn purge(&mut self, now: u64, window_secs: u64) {
        self.failures_by_ip.retain(|_, failures| {
            failures.retain(|at| now < at + window_secs);
            !failures.is_empty()
        });
    }
}
