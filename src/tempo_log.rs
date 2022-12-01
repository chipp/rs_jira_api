use chrono::NaiveDate;
use chrono::{offset::TimeZone, Utc};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct TempoLog {
    #[serde(rename = "dateStarted", deserialize_with = "deserialize_date_time")]
    pub date_started: NaiveDate,
}

use serde::{de, Deserializer};
use std::fmt;

const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.3f";

pub fn deserialize_date_time<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(DateTimeFromCustomFormatVisitor)
}

struct DateTimeFromCustomFormatVisitor;
impl<'de> de::Visitor<'de> for DateTimeFromCustomFormatVisitor {
    type Value = NaiveDate;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a datetime string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Utc.datetime_from_str(&value, FORMAT)
            .map_err(serde::de::Error::custom)
            .map(|dt| dt.date_naive())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parsing() {
        let json = json!({
            "author": {"displayName": "Pupkin, Vasiliy", "name": "vpupkin"},
            "issue": {"key": "RS-1", "summary": "Implement issues support for tempo"},
            "timeSpentSeconds": 3600,
            "dateStarted": "2019-03-11T00:00:00.000",
            "id": "1"
        });

        let log: TempoLog = serde_json::from_value(json).unwrap();
        assert_eq!(
            log.date_started,
            NaiveDate::from_ymd_opt(2019, 3, 11).unwrap()
        );
    }
}
