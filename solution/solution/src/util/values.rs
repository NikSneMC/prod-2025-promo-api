use chrono::{DateTime, Utc};

pub const MIN_DATETIME: DateTime<Utc> =
    DateTime::<Utc>::from_timestamp_millis(-210866803200).unwrap();

pub const MAX_DATETIME: DateTime<Utc> =
    DateTime::<Utc>::from_timestamp_millis(8210266876799999).unwrap();
