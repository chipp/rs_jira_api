use super::{changelog::Changelog, user::User, worklog::Worklogs};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Issue {
    pub id: String,
    pub key: String,
    pub fields: Fields,
    pub changelog: Option<Changelog>,
}

use std::fmt;
impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.key)
    }
}

pub const MANDATORY_ISSUE_FIELDS: &[&str] = &[
    "created",
    "creator",
    "issuetype",
    "priority",
    "status",
    "summary",
];

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Fields {
    pub summary: String,
    pub description: Option<String>,

    pub creator: User,

    #[serde(deserialize_with = "crate::date_format::deserialize_date_with_tz")]
    pub created: DateTime<Utc>,

    #[serde(deserialize_with = "crate::date_format::deserialize_optional_date_with_tz")]
    #[serde(rename = "resolutiondate", default)]
    pub resolution_date: Option<DateTime<Utc>>,

    pub assignee: Option<User>,

    #[serde(rename = "customfield_10182")]
    pub story_points: Option<f32>,
    #[serde(rename = "customfield_10231")]
    pub sprints: Option<Vec<String>>,

    #[serde(rename = "worklog")]
    pub work_logs: Option<Worklogs>,

    #[serde(rename = "issuetype")]
    pub issue_type: IssueType,
    pub status: IssueStatus,
    pub priority: Option<IssuePriority>,

    pub parent: Option<ShortIssue>,
    pub subtasks: Option<Vec<ShortIssue>>,

    #[serde(rename = "issuelinks")]
    pub issue_links: Option<Vec<IssueLink>>,

    #[serde(rename = "timeoriginalestimate")]
    pub original_estimate: Option<u32>,

    #[serde(rename = "timespent")]
    pub time_spent: Option<u32>,

    #[serde(rename = "aggregatetimespent")]
    pub total_time_spent: Option<u32>,

    #[serde(default)]
    pub labels: Vec<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct ModifyFields {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub description: Option<String>,

    #[serde(rename = "customfield_10182", skip_serializing_if = "Option::is_none")]
    pub story_points: Option<f32>,
    #[serde(rename = "customfield_10231", skip_serializing_if = "Option::is_none")]
    pub sprints: Option<Vec<String>>,

    #[serde(rename = "issuetype", skip_serializing_if = "Option::is_none")]
    pub issue_type: Option<IssueType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<IssuePriority>,

    #[serde(rename = "issuelinks", skip_serializing_if = "Option::is_none")]
    pub issue_links: Option<Vec<IssueLink>>,

    #[serde(
        rename = "timeoriginalestimate",
        skip_serializing_if = "Option::is_none"
    )]
    pub original_estimate: Option<u32>,

    #[serde(rename = "timespent", skip_serializing_if = "Option::is_none")]
    pub time_spent: Option<u32>,

    #[serde(rename = "aggregatetimespent", skip_serializing_if = "Option::is_none")]
    pub total_time_spent: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
}

impl ModifyFields {
    pub fn empty() -> ModifyFields {
        ModifyFields {
            summary: None,
            description: None,
            story_points: None,
            sprints: None,
            issue_type: None,
            priority: None,
            issue_links: None,
            original_estimate: None,
            time_spent: None,
            total_time_spent: None,
            labels: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct IssueType {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct IssueStatus {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct IssuePriority {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ShortIssue {
    pub id: String,
    pub key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct IssueLink {
    pub id: String,
    pub r#type: IssueLinkType,
    #[serde(rename = "inwardIssue")]
    pub inward_issue: Option<ShortIssue>,
    #[serde(rename = "outwardIssue")]
    pub outward_issue: Option<ShortIssue>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct IssueLinkType {
    pub id: String,
}
