use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone, Deserialize, Debug, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "camelCase")]
pub struct Sprint {
    pub id: u16,
    pub name: String,
    pub state: String,

    pub origin_board_id: u16,

    #[serde(default)]
    #[serde(deserialize_with = "crate::date_format::deserialize_optional_date_without_tz")]
    pub start_date: Option<DateTime<Utc>>,

    #[serde(default)]
    #[serde(deserialize_with = "crate::date_format::deserialize_optional_date_without_tz")]
    pub end_date: Option<DateTime<Utc>>,

    #[serde(default)]
    #[serde(deserialize_with = "crate::date_format::deserialize_optional_date_without_tz")]
    pub complete_date: Option<DateTime<Utc>>,
}

use std::fmt;
impl fmt::Display for Sprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone, Utc};
    use serde_json::json;

    #[test]
    fn no_dates_parsing() {
        let json = json!({
            "id": 1001,
            "state": "closed",
            "name": "Rust Sprint 1",
            "originBoardId": 123,
        });

        let sprint: super::Sprint = serde_json::from_value(json).unwrap();

        assert_eq!(sprint.id, 1001);
        assert_eq!(sprint.name, "Rust Sprint 1");
        assert_eq!(sprint.state, "closed");
        assert_eq!(sprint.origin_board_id, 123);
        assert_eq!(sprint.start_date, None);
        assert_eq!(sprint.end_date, None);
        assert_eq!(sprint.complete_date, None);
    }

    #[test]
    fn with_dates_parsing() {
        let json = json!({
            "id": 1001,
            "state": "closed",
            "name": "Rust Sprint 1",
            "originBoardId": 123,
            "completeDate": "2020-03-10T10:20:50.730Z",
            "endDate": "2020-03-02T22:01:00.000Z",
            "startDate": "2020-02-18T11:36:36.825Z"
        });

        let sprint: super::Sprint = serde_json::from_value(json).unwrap();

        assert_eq!(sprint.id, 1001);
        assert_eq!(sprint.name, "Rust Sprint 1");
        assert_eq!(sprint.state, "closed");
        assert_eq!(sprint.origin_board_id, 123);
        assert_eq!(
            sprint.start_date,
            Some(
                Utc.with_ymd_and_hms(2020, 02, 18, 11, 36, 36).unwrap()
                    + Duration::milliseconds(825)
            )
        );

        assert_eq!(
            sprint.end_date,
            Some(
                Utc.with_ymd_and_hms(2020, 03, 02, 22, 01, 00)
                    .unwrap()
                    .into()
            )
        );

        assert_eq!(
            sprint.complete_date,
            Some(
                Utc.with_ymd_and_hms(2020, 03, 10, 10, 20, 50).unwrap()
                    + Duration::milliseconds(730)
            )
        );
    }
}
