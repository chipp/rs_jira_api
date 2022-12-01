use serde::Deserialize;

#[derive(Deserialize)]
pub struct DevStatus {
    #[serde(rename = "pullRequests")]
    pub pull_requests: Vec<PullRequest>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct PullRequest {
    pub id: String,
    pub destination: Branch,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Branch {
    pub repository: Repo,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Repo {
    pub url: String,
}
