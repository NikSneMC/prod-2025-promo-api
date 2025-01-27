use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::util::convertions::deserialize_opt_antifraud_datetime;

#[derive(Serialize, Debug)]
pub struct AntiFraudRequest {
    pub user_email: String,
    pub promo_id: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AntiFraudResponse {
    pub ok: bool,

    #[serde(deserialize_with = "deserialize_opt_antifraud_datetime")]
    pub cache_until: Option<DateTime<Utc>>,
}
