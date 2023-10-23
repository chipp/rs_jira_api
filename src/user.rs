use serde::Deserialize;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct User {
    pub key: String,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(default)]
    pub groups: Groups,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq)]
pub struct Groups {
    pub size: usize,
    pub items: Vec<Group>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Group {
    pub name: String,
}

use std::fmt;
impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name.as_ref().unwrap_or(&self.name))
    }
}

use std::cmp::Ordering;
impl PartialOrd for User {
    fn partial_cmp(&self, other: &User) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for User {
    fn cmp(&self, other: &User) -> Ordering {
        self.display_name.cmp(&other.display_name)
    }
}

impl Hash for User {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn model_parsing() {
        let json = json!({
            "active": true,
            "applicationRoles": {
                "items": [],
                "size": 1
            },
            "avatarUrls": {
                "16x16": "https://jira.example.io/secure/useravatar?size=xsmall&ownerId=chipp&avatarId=16609",
                "24x24": "https://jira.example.io/secure/useravatar?size=small&ownerId=chipp&avatarId=16609",
                "32x32": "https://jira.example.io/secure/useravatar?size=medium&ownerId=chipp&avatarId=16609",
                "48x48": "https://jira.example.io/secure/useravatar?ownerId=chipp&avatarId=16609"
            },
            "displayName": "Vladimir Burdukov",
            "emailAddress": "me@chipp.dev",
            "expand": "groups,applicationRoles",
            "groups": {
                "items": [],
                "size": 15
            },
            "key": "chipp",
            "locale": "en_US",
            "name": "chipp",
            "self": "https://jira.example.io/rest/api/2/user?username=chipp",
            "timeZone": "Europe/Vilnius",
            "groups": {
                "size": 0,
                "items": []
            }
        });

        let issue: super::User = serde_json::from_value(json).unwrap();

        assert_eq!(issue.name, "chipp");
        assert_eq!(issue.display_name, Some("Vladimir Burdukov".to_owned()));
    }

    #[test]
    fn model_parsing_no_display_name() {
        let json = json!({"key": "JIRAUSER1", "name":"karumuga","active":false});

        let issue: super::User = serde_json::from_value(json).unwrap();

        assert_eq!(issue.name, "karumuga");
        assert_eq!(issue.display_name, None);
    }
}
