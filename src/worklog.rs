use super::user::User;

use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Worklogs {
    #[serde(rename = "startAt")]
    pub start_at: u32,
    #[serde(rename = "maxResults")]
    pub max_results: u32,
    pub total: u32,

    #[serde(rename = "worklogs")]
    pub worklogs: Vec<Worklog>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Worklog {
    pub id: String,
    pub author: User,
    #[serde(rename = "timeSpentSeconds")]
    pub time_spent: u32,
    #[serde(rename = "started", with = "jira_datetime_format")]
    pub date_started: NaiveDate,
}

mod jira_datetime_format {
    use chrono::{DateTime, NaiveDate};
    use serde::{self, Deserialize, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S%.3f%z";

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_str(&s, FORMAT)
            .map(|datetime| datetime.date_naive())
            .map_err(serde::de::Error::custom)
    }
}

use std::hash::{Hash, Hasher};

impl Hash for Worklog {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Worklog {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Worklog {}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parsing() {
        let json = json!({
            "author": {"key": "JIRAUSER1", "displayName": "Pupkin, Vasiliy", "name": "vpupkin"},
            "issue": {"key": "RS-1", "summary": "Implement issues support for tempo"},
            "timeSpentSeconds": 3600u32,
            "started": "2019-03-11T00:00:00.000-0500",
            "id": "1"
        });

        let worklog: super::Worklog = serde_json::from_value(json).unwrap();

        assert_eq!(
            worklog.author.display_name,
            Some("Pupkin, Vasiliy".to_owned())
        );
        assert_eq!(worklog.time_spent, 3600);
        assert_eq!(worklog.id, "1");
        assert_eq!(
            worklog.date_started,
            NaiveDate::from_ymd_opt(2019, 3, 11).unwrap()
        );
    }
}
