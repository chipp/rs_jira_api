use std::collections::HashSet;

use url::Url;
use {
    super::board::Board,
    super::issue::{Issue, ShortIssue},
    super::project::Project,
    super::sprint::Sprint,
    super::tempo_log::TempoLog,
    super::user::User,
    super::worklog::Worklogs,
};

use chipp_http::{
    curl::easy::{Easy, List},
    Error, HttpClient, HttpMethod, Interceptor, Request,
};
use log::trace;
use serde::{Deserialize, Serialize};

use crate::issue::{ModifyFields, MANDATORY_ISSUE_FIELDS};

pub struct Client {
    inner: HttpClient<Authenticator>,
}

struct Authenticator(String);

impl Interceptor for Authenticator {
    fn modify(&self, _: &mut Easy, _: &Request) {}

    fn add_headers(&self, headers: &mut List, _: &Request) {
        let token = jira_credentials(&self.0);
        let header = format!("Authorization: Bearer {token}");
        headers.append(&header).unwrap();
    }
}

impl<'a> Client {
    pub fn new<U>(jira_base_url: U) -> Option<Client>
    where
        U: AsRef<str>,
    {
        let mut jira_base_url = Url::parse(jira_base_url.as_ref()).ok()?;
        jira_base_url.path_segments_mut().unwrap().push("rest");

        let domain = jira_base_url.domain().map(ToOwned::to_owned)?;
        let inner = HttpClient::new(&jira_base_url)
            .unwrap()
            .with_interceptor(Authenticator(domain));

        Some(Client { inner })
    }
}

#[cfg(target_os = "macos")]
fn jira_credentials(domain: &str) -> String {
    chipp_auth::token(domain, "access_token")
}

#[cfg(target_os = "linux")]
fn jira_credentials(_: &str) -> String {
    std::env::var("JIRA_ACCESS_TOKEN").expect("should have JIRA_ACCESS_TOKEN")
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgilePageResponse<V> {
    pub is_last: bool,
    pub max_results: u16,

    #[serde(bound(deserialize = "V: Deserialize<'de>"))]
    pub values: Vec<V>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuesPageResponse {
    pub max_results: u32,
    pub total: u32,
    pub issues: Vec<Issue>,
}

impl Client {
    pub async fn myself(&self) -> Result<User, Error> {
        self.inner.get(["api", "2", "myself"]).await
    }

    pub async fn get_project(&self, key: &str) -> Result<Project, Error> {
        let mut request = self.inner.new_request(&["api", "2", "project", key]);
        request.set_retry_count(3);

        self.inner
            .perform_request(request, chipp_http::json::parse_json)
            .await
    }

    pub async fn get_issue(
        &self,
        key: &str,
        fields: Option<&[&str]>,
        expand: Option<&[&str]>,
    ) -> Result<Issue, Error> {
        let fields = fields.unwrap_or_default();
        let mut all_fields = HashSet::<&str>::new();
        all_fields.extend(fields);
        all_fields.extend(MANDATORY_ISSUE_FIELDS);

        let fields = all_fields.into_iter().collect::<Vec<_>>().join(",");

        let expand = expand.unwrap_or_default().join(",");

        let mut request = self.inner.new_request_with_params(
            &["api", "2", "issue", key],
            [("fields", fields), ("expand", expand)],
        );
        request.set_retry_count(3);

        self.inner
            .perform_request(request, chipp_http::json::parse_json)
            .await
    }

    pub async fn get_board(&self, board_id: u16) -> Result<Board, Error> {
        let mut request =
            self.inner
                .new_request(&["agile", "1.0", "board", &format!("{}", board_id)]);
        request.set_retry_count(3);

        self.inner
            .perform_request(request, chipp_http::json::parse_json)
            .await
    }

    pub async fn get_sprints_for_board(
        &self,
        board_id: u16,
        start_at: u16,
    ) -> Result<AgilePageResponse<Sprint>, Error> {
        let mut request = self.inner.new_request_with_params(
            &["agile", "1.0", "board", &format!("{}", board_id), "sprint"],
            &[("startAt", format!("{}", start_at))],
        );
        request.set_retry_count(3);

        self.inner
            .perform_request(request, chipp_http::json::parse_json)
            .await
    }

    pub async fn search_issues(
        &self,
        jql: &str,
        start_at: u32,
        max_results: u32,
        fields: Option<&[&str]>,
        expand: Option<&[&str]>,
    ) -> Result<IssuesPageResponse, Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Body<'a> {
            jql: &'a str,
            start_at: u32,
            max_results: u32,
            fields: Option<&'a [&'a str]>,
            expand: Option<&'a [&'a str]>,
        }

        let fields = fields.unwrap_or_default();
        let mut all_fields = HashSet::<&str>::new();
        all_fields.extend(fields);
        all_fields.extend(MANDATORY_ISSUE_FIELDS);

        let fields = all_fields.into_iter().collect::<Vec<_>>();

        let body = Body {
            jql,
            start_at,
            max_results,
            fields: Some(&fields),
            expand,
        };

        let mut request = self.inner.new_request(&["api", "2", "search"]);
        request.set_method(HttpMethod::Post);
        request.set_retry_count(3);
        request.set_json_body(&body);

        self.inner
            .perform_request(request, chipp_http::json::parse_json)
            .await
    }

    pub async fn get_user_by_username<U>(&self, username: U) -> Result<User, Error>
    where
        U: std::fmt::Display + AsRef<str>,
    {
        trace!("loading user information {}", username);

        let result = self
            .inner
            .get_with_params(
                &["api", "2", "user"],
                &[("username", username.as_ref()), ("expand", "groups")],
            )
            .await;

        trace!("loaded user information {}", username);

        result
    }

    pub async fn get_user_by_key<U>(&self, key: U) -> Result<User, Error>
    where
        U: std::fmt::Display + AsRef<str>,
    {
        let mut request = self
            .inner
            .new_request_with_params(&["api", "2", "user"], &[("key", key)]);
        request.set_retry_count(3);

        self.inner
            .perform_request(request, chipp_http::json::parse_json)
            .await
    }

    pub async fn get_worklogs_for_issue(
        &self,
        issue_id: &str,
        start_at: u32,
    ) -> Result<Worklogs, Error> {
        let mut request = self.inner.new_request_with_params(
            &["api", "2", "issue", issue_id, "worklog"],
            &[
                ("startAt", format!("{}", start_at).as_str()),
                ("maxResults", "500"),
            ],
        );
        request.set_retry_count(3);

        self.inner
            .perform_request(request, chipp_http::json::parse_json)
            .await
    }

    pub async fn get_subtasks_for_issue(&self, issue_id: &str) -> Result<Vec<ShortIssue>, Error> {
        let mut request = self
            .inner
            .new_request(&["api", "2", "issue", issue_id, "subtask"]);
        request.set_retry_count(3);

        self.inner
            .perform_request(request, chipp_http::json::parse_json)
            .await
    }

    pub async fn get_logs_for_user(
        &self,
        user: &str,
        date_from: &str,
        date_to: &str,
    ) -> Result<Vec<TempoLog>, Error> {
        let mut request = self.inner.new_request_with_params(
            &["tempo-timesheets", "3", "worklogs"],
            &[
                ("dateFrom", date_from),
                ("dateTo", date_to),
                ("username", user),
            ],
        );
        request.set_retry_count(3);

        self.inner
            .perform_request(request, chipp_http::json::parse_json)
            .await
    }

    pub async fn update_issue(&self, key: &str, modify: ModifyFields) -> Result<(), Error> {
        #[derive(Debug, Serialize)]
        struct RequestBody {
            fields: ModifyFields,
        }

        let mut request = self.inner.new_request(&["api", "2", "issue", key]);
        request.method = HttpMethod::Put;

        let body = RequestBody { fields: modify };
        request.body = Some(serde_json::to_vec(&body).unwrap());
        request.add_header("Content-Type", "application/json; charset=utf-8");

        self.inner
            .perform_request(request, chipp_http::parse_void)
            .await
    }

    pub async fn update_issue_labels(&self, key: &str, labels: &[String]) -> Result<(), Error> {
        let mut request = self.inner.new_request(&["api", "2", "issue", key]);
        request.method = HttpMethod::Put;

        let body = serde_json::json!({
            "fields": {
                "labels": labels
            }
        });
        request.body = Some(serde_json::to_vec(&body).unwrap());
        request.add_header("Content-Type", "application/json; charset=utf-8");

        self.inner
            .perform_request(request, chipp_http::parse_void)
            .await
    }
}
