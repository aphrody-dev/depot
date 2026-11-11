use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRepo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub html_url: String,
    pub clone_url: String,
    #[serde(default = "default_branch_master")]
    pub default_branch: String,
    #[serde(default)]
    pub stargazers_count: u64,
    #[serde(default)]
    pub open_issues_count: u64,
    #[serde(default)]
    pub forks_count: u64,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub pushed_at: Option<String>,
}

fn default_branch_master() -> String {
    "main".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPullRequest {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub user: Option<GitHubUser>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub merged_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssue {
    pub id: u64,
    pub number: u64,
    pub title: String,
    pub state: String,
    pub html_url: String,
    pub user: Option<GitHubUser>,
    #[serde(default)]
    pub comments: u64,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCommitAuthor {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCommitDetails {
    pub message: String,
    #[serde(default)]
    pub author: Option<GitHubCommitAuthor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCommit {
    pub sha: String,
    pub html_url: String,
    pub commit: GitHubCommitDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubWorkflowRun {
    pub id: u64,
    pub name: Option<String>,
    pub status: String,
    #[serde(default)]
    pub conclusion: Option<String>,
    pub html_url: String,
    #[serde(default)]
    pub head_branch: Option<String>,
    #[serde(default)]
    pub head_sha: Option<String>,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

// GitLab Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabProject {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
    #[serde(default)]
    pub description: Option<String>,
    pub web_url: String,
    #[serde(default)]
    pub http_url_to_repo: Option<String>,
    #[serde(default = "default_branch_master")]
    pub default_branch: String,
    #[serde(default)]
    pub star_count: u64,
    #[serde(default)]
    pub forks_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabMergeRequest {
    pub id: u64,
    pub iid: u64,
    pub title: String,
    pub state: String,
    pub web_url: String,
    pub source_branch: String,
    pub target_branch: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabIssue {
    pub id: u64,
    pub iid: u64,
    pub title: String,
    pub state: String,
    pub web_url: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabPipeline {
    pub id: u64,
    pub iid: u64,
    pub status: String,
    pub r#ref: String,
    pub sha: String,
    pub web_url: String,
    #[serde(default)]
    pub created_at: Option<String>,
}
