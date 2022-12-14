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

use http_client::{curl::easy::Auth, Error, HttpClient, HttpMethod};
use log::debug;
use serde::{Deserialize, Serialize};

pub struct Client<'a> {
    inner: HttpClient<'a>,
}

impl<'a> Client<'a> {
    pub fn new<U>(jira_base_url: U) -> Option<Client<'a>>
    where
        U: AsRef<str>,
    {
        let mut jira_base_url = Url::parse(jira_base_url.as_ref()).ok()?;
        jira_base_url.set_path("/rest");

        let mut inner = HttpClient::new(&jira_base_url).unwrap();

        if let Some(domain) = jira_base_url.domain().map(ToOwned::to_owned) {
            inner.set_interceptor(move |easy| {
                let mut auth = Auth::new();

                auth.basic(true);
                easy.http_auth(&auth).unwrap();

                let (username, password) = auth::user_and_password(&domain);

                easy.username(&username).unwrap();
                easy.password(&password).unwrap();
            });
        }

        Some(Client { inner })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgilePageResponse<V> {
    pub is_last: bool,
    pub max_results: u16,

    #[serde(bound(deserialize = "V: Deserialize<'de>"))]
    pub values: Vec<V>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssuesPageResponse {
    pub max_results: u32,
    pub total: u32,
    pub issues: Vec<Issue>,
}

impl Client<'_> {
    pub async fn get_project(&self, key: &str) -> Result<Project, Error> {
        let mut request = self.inner.new_request(vec!["api", "2", "project", key]);
        request.set_retry_count(3);

        self.inner
            .perform_request(request, http_client::json::parse_json)
            .await
    }

    pub async fn get_board(&self, board_id: u16) -> Result<Board, Error> {
        let mut request =
            self.inner
                .new_request(vec!["agile", "1.0", "board", &format!("{}", board_id)]);
        request.set_retry_count(3);

        self.inner
            .perform_request(request, http_client::json::parse_json)
            .await
    }

    pub async fn get_sprints_for_board(
        &self,
        board_id: u16,
        start_at: u16,
    ) -> Result<AgilePageResponse<Sprint>, Error> {
        let mut request = self.inner.new_request_with_params(
            vec!["agile", "1.0", "board", &format!("{}", board_id), "sprint"],
            &[("startAt", format!("{}", start_at))],
        );
        request.set_retry_count(3);

        self.inner
            .perform_request(request, http_client::json::parse_json)
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

        let body = Body {
            jql,
            start_at,
            max_results,
            fields,
            expand,
        };

        let mut request = self.inner.new_request(vec!["api", "2", "search"]);
        request.set_method(HttpMethod::Post);
        request.set_retry_count(3);
        request.set_json_body(&body);

        self.inner
            .perform_request(request, http_client::json::parse_json)
            .await
    }

    pub async fn get_user_by_key(&self, key: &str) -> Result<User, Error> {
        let mut request = self
            .inner
            .new_request_with_params(vec!["api", "2", "user"], &[("key", key)]);
        request.set_retry_count(3);

        self.inner
            .perform_request(request, http_client::json::parse_json)
            .await
    }

    pub async fn get_worklogs_for_issue(
        &self,
        issue_id: &str,
        start_at: u32,
    ) -> Result<Worklogs, Error> {
        let mut request = self.inner.new_request_with_params(
            vec!["api", "2", "issue", issue_id, "worklog"],
            &[
                ("startAt", format!("{}", start_at).as_str()),
                ("maxResults", "500"),
            ],
        );
        request.set_retry_count(3);

        self.inner
            .perform_request(request, http_client::json::parse_json)
            .await
    }

    pub async fn get_subtasks_for_issue(&self, issue_id: &str) -> Result<Vec<ShortIssue>, Error> {
        let mut request = self
            .inner
            .new_request(vec!["api", "2", "issue", issue_id, "subtask"]);
        request.set_retry_count(3);

        self.inner
            .perform_request(request, http_client::json::parse_json)
            .await
    }

    pub async fn search_issues_in_project_by_users_worklogs<U: AsRef<str>>(
        &self,
        project: &str,
        date_from: &str,
        date_to: &str,
        users: &[U],
        start_at: u32,
    ) -> Result<IssuesPageResponse, Error> {
        let users = users
            .iter()
            .map(|u| u.as_ref().to_owned())
            .collect::<Vec<_>>()
            .join(",");

        let jql = format!(
            "project = {} AND worklogAuthor in ({}) \
            AND worklogDate >= \"{}\" AND worklogDate <= \"{}\"",
            project, users, date_from, date_to
        );

        debug!("performing JQL: {}", jql);

        self.inner
            .get_with_params(
                &["api", "2", "search"],
                &[
                    ("jql", jql.as_str()),
                    ("startAt", &format!("{}", start_at)),
                    ("maxResults", "500"),
                ],
            )
            .await
    }

    pub async fn get_logs_for_user(
        &self,
        user: &str,
        date_from: &str,
        date_to: &str,
    ) -> Result<Vec<TempoLog>, Error> {
        let mut request = self.inner.new_request_with_params(
            vec!["tempo-timesheets", "3", "worklogs"],
            &[
                ("dateFrom", date_from),
                ("dateTo", date_to),
                ("username", user),
            ],
        );
        request.set_retry_count(3);

        self.inner
            .perform_request(request, http_client::json::parse_json)
            .await
    }
}
