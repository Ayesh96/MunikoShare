/// What the latest poll means relative to the previous one.
#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    /// First reading after idling; remembered but never sent, so enabling sync leaks nothing.
    Baseline,
    Unchanged,
    Changed,
}

/// Tracks the last seen hash so the same content is never reported twice in a row.
#[derive(Default)]
pub struct ChangeDetector {
    last_hash: Option<String>,
    primed: bool,
}

impl ChangeDetector {
    pub fn observe(&mut self, hash: &str) -> Change {
        let unchanged = self.last_hash.as_deref() == Some(hash);
        self.last_hash = Some(hash.to_string());
        if !self.primed {
            self.primed = true;
            return Change::Baseline;
        }
        if unchanged {
            Change::Unchanged
        } else {
            Change::Changed
        }
    }

    /// Call when polling pauses; the next reading becomes a fresh baseline.
    pub fn go_idle(&mut self) {
        self.primed = false;
    }
}
