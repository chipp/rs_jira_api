use chrono::{DateTime, TimeZone, Utc};
use serde::{self, de, Deserializer};
use std::fmt;

const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.3fZ";

pub fn deserialize_optional_date_without_tz<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_option(OptionalDateTimeFromCustomFormatVisitor)
}

struct OptionalDateTimeFromCustomFormatVisitor;
impl<'de> de::Visitor<'de> for OptionalDateTimeFromCustomFormatVisitor {
    type Value = Option<DateTime<Utc>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "null or a datetime string")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_some<D>(self, d: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        Ok(Some(d.deserialize_str(DateTimeFromCustomFormatVisitor)?))
    }
}

#[allow(dead_code)]
pub fn deserialize_date_without_tz<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_str(DateTimeFromCustomFormatVisitor)
}

struct DateTimeFromCustomFormatVisitor;
impl<'de> de::Visitor<'de> for DateTimeFromCustomFormatVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a datetime string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Utc.datetime_from_str(&value, FORMAT)
            .map_err(serde::de::Error::custom)
            .map(|d| d.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};

    #[test]
    fn date_format() {
        let date =
            Utc.with_ymd_and_hms(2020, 03, 10, 10, 20, 50).unwrap() + Duration::milliseconds(730);

        assert_eq!(date.format(FORMAT).to_string(), "2020-03-10T10:20:50.730Z");

        assert_eq!(
            Utc.datetime_from_str("2020-03-10T10:20:50.730Z", FORMAT)
                .unwrap(),
            date.with_timezone(&Utc)
        );
    }

    #[test]
    fn deserialize_optional_date() {
        use serde::Deserialize;

        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "super::deserialize_optional_date_without_tz")]
            date: Option<DateTime<Utc>>,
        }

        let json = r#"{"date": null}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert!(test.date.is_none());

        let json = r#"{"date": "2019-10-14T15:59:50.000Z"}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(
            test.date,
            Some(Utc.with_ymd_and_hms(2019, 10, 14, 15, 59, 50).unwrap())
        );
    }

    #[test]
    fn deserialize_date() {
        use serde::Deserialize;

        #[derive(Debug, Deserialize)]
        struct Test {
            #[serde(deserialize_with = "super::deserialize_date_without_tz")]
            date: DateTime<Utc>,
        }

        let json = r#"{"date": "2019-10-14T15:59:50.000Z"}"#;
        let test: Test = serde_json::from_str(json).unwrap();
        assert_eq!(
            test.date,
            Utc.with_ymd_and_hms(2019, 10, 14, 15, 59, 50).unwrap()
        );

        let json = r#"{"date": null}"#;
        assert!(serde_json::from_str::<'_, Test>(json).is_err());
    }
}
