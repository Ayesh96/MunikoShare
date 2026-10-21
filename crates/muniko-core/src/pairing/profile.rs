use serde::{Deserialize, Serialize};

/// Name and OS a device announces about itself during pairing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairingProfile {
    pub name: String,
    pub os: String,
}

/// Body of `POST /api/v1/pairing/confirm`.
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmBody {
    pub code: String,
    #[serde(flatten)]
    pub profile: PairingProfile,
}
