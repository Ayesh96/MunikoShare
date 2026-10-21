use std::sync::Arc;

use super::incoming::IncomingTransfers;
use crate::error::Result;

/// Cloneable handle for answering incoming offers.
#[derive(Clone)]
pub struct OfferResponder {
    transfers: Arc<IncomingTransfers>,
}

impl OfferResponder {
    pub(super) fn new(transfers: Arc<IncomingTransfers>) -> Self {
        Self { transfers }
    }

    /// Accepts or declines a pending offer; errors if it is unknown or already closed.
    pub fn respond_to_offer(&self, offer_id: &str, accept: bool) -> Result<()> {
        self.transfers.respond(offer_id, accept)
    }
}
