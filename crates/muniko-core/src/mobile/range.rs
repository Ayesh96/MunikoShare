//! Parses a single-range `Range: bytes=...` header.

use crate::constants::mobile::RANGE_PREFIX;

#[derive(Debug, PartialEq, Eq)]
pub enum RangeRequest {
    Whole,
    /// Inclusive byte positions.
    Part { start: u64, end: u64 },
    Unsatisfiable,
}

pub fn parse_range(header: Option<&str>, size: u64) -> RangeRequest {
    let Some(spec) = header.and_then(|value| value.trim().strip_prefix(RANGE_PREFIX)) else { return RangeRequest::Whole };
    let Some((first, last)) = spec.split_once('-').filter(|_| !spec.contains(',')) else { return RangeRequest::Whole };
    match (first.trim().parse::<u64>(), last.trim().parse::<u64>()) {
        (Ok(start), Ok(end)) => bounded(start, end.min(size.saturating_sub(1)), size),
        (Ok(start), Err(_)) if last.trim().is_empty() => bounded(start, size.saturating_sub(1), size),
        (Err(_), Ok(count)) if first.trim().is_empty() && count > 0 => bounded(size.saturating_sub(count), size.saturating_sub(1), size),
        _ => RangeRequest::Unsatisfiable,
    }
}

fn bounded(start: u64, end: u64, size: u64) -> RangeRequest {
    if size == 0 || start >= size || start > end {
        RangeRequest::Unsatisfiable
    } else {
        RangeRequest::Part { start, end }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_forms() {
        assert_eq!(parse_range(None, 10), RangeRequest::Whole);
        assert_eq!(parse_range(Some("bytes=2-4"), 10), RangeRequest::Part { start: 2, end: 4 });
        assert_eq!(parse_range(Some("bytes=5-"), 10), RangeRequest::Part { start: 5, end: 9 });
        assert_eq!(parse_range(Some("bytes=-3"), 10), RangeRequest::Part { start: 7, end: 9 });
        assert_eq!(parse_range(Some("bytes=8-99"), 10), RangeRequest::Part { start: 8, end: 9 });
        assert_eq!(parse_range(Some("bytes=10-"), 10), RangeRequest::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=0-1,3-4"), 10), RangeRequest::Whole);
    }
}
