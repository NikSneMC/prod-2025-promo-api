use base64::{engine::GeneralPurpose, Engine};
use chrono::{DateTime, Utc};
use serde::{de::value::StrDeserializer, Deserialize, Deserializer, Serializer};
use uuid::Uuid;

pub fn decode_string(engine: &GeneralPurpose, value: &str) -> Result<String, ()> {
    let string = engine.decode(value).map_err(|_| ())?;
    String::from_utf8(string).map_err(|_| ())
}

pub fn decode_uuid(engine: &GeneralPurpose, value: &str) -> Result<Uuid, ()> {
    let uuid = engine.decode(value).map_err(|_| ())?;
    Uuid::from_slice(uuid.as_slice()).map_err(|_| ())
}

pub mod promo_date_format {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{Deserialize, Serializer};

    const DATE_FORMAT: &'static str = "%F %H:%M:%S";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = format!("{} 00:00:00", String::deserialize(deserializer)?);
        let dt =
            NaiveDateTime::parse_from_str(&s, DATE_FORMAT).map_err(serde::de::Error::custom)?;
        Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc)))
    }

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(DATE_FORMAT));
        let s = s.split_whitespace().collect::<Vec<&str>>()[0];
        serializer.serialize_str(&s)
    }
}
pub fn serialize_opt_promo_date<S>(
    dt: &Option<DateTime<Utc>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match dt {
        Some(dt) => promo_date_format::serialize(dt, serializer),
        _ => unreachable!(),
    }
}

pub mod antifraud_datetime_format {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    const FORMAT: &'static str = "%FT%H:%M:%S%.3f";

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}
pub fn deserialize_opt_antifraud_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;

    Ok(match s {
        Some(s) => Some(antifraud_datetime_format::deserialize(
            StrDeserializer::new(&s),
        )?),
        None => None,
    })
}
