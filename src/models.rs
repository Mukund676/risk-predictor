use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEnvelope {
    pub request_id: String,
    pub portfolio_id: String,
    pub base_currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseEnvelope {
    pub request_id: String,
    pub portfolio_id: String,
    pub base_currency: String,
    pub generated_at: String,
    pub engine_version: String,
}