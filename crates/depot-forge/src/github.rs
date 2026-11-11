use serde_json::json;
use crate::auth::ForgeAuth;
use crate::error::ForgeError;
use crate::models::{
    GitHubCommit, GitHubIssue, GitHubPullRequest, GitHubRepo, GitHubWorkflowRun,
};

pub struct GitHubClient {
    auth: ForgeAuth,
    base_url: String,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self {
        Self {
            auth: ForgeAuth::new(token),
            base_url: "https://api.github.com".to_string(),
        }
    }

    pub fn from_vault() -> Self {
        Self {
            auth: ForgeAuth::github_from_vault(),
            base_url: "https://api.github.com".to_string(),
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into().trim_end_matches('/').to_string();
        self
    }

    pub fn auth(&self) -> &ForgeAuth {
        &self.auth
    }

    fn prepare_request(&self, req: ureq::Request) -> ureq::Request {
        let mut r = req
            .set("User-Agent", "depot-forge/0.2.0")
            .set("Accept", "application/vnd.github+json")
            .set("X-GitHub-Api-Version", "2022-11-28");
        if let Some(tok) = self.auth.token() {
            r = r.set("Authorization", &format!("Bearer {}", tok));
        }
        r
    }

    pub fn get_repo(&self, owner: &str, repo: &str) -> Result<GitHubRepo, ForgeError> {
        let url = format!("{}/repos/{}/{}", self.base_url, owner, repo);
        let resp = self
            .prepare_request(ureq::get(&url))
            .call()
            .map_err(|e| map_ureq_error(e, &url))?;

        resp.into_json::<GitHubRepo>()
            .map_err(|e| ForgeError::Json(e.to_string()))
    }

    pub fn list_org_repos(
        &self,
        org: &str,
        limit: Option<usize>,
    ) -> Result<Vec<GitHubRepo>, ForgeError> {
        let max_repos = limit.unwrap_or(usize::MAX);
        let mut all_repos = Vec::new();
        let mut page = 1;
        let per_page = 100;

        loop {
            let url = format!(
                "{}/orgs/{}/repos?per_page={}&page={}&sort=pushed&direction=desc",
                self.base_url, org, per_page, page
            );
            let resp = self
                .prepare_request(ureq::get(&url))
                .call()
                .map_err(|e| map_ureq_error(e, &url))?;

            let batch: Vec<GitHubRepo> = resp
                .into_json()
                .map_err(|e| ForgeError::Json(e.to_string()))?;

            if batch.is_empty() {
                break;
            }

            let batch_len = batch.len();
            for r in batch {
                all_repos.push(r);
                if all_repos.len() >= max_repos {
                    return Ok(all_repos);
                }
            }

            if batch_len < per_page {
                break;
            }
            page += 1;
        }

        Ok(all_repos)
    }

    pub fn list_pull_requests(
        &self,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> Result<Vec<GitHubPullRequest>, ForgeError> {
        let st = state.unwrap_or("open");
        let url = format!(
            "{}/repos/{}/{}/pulls?state={}&per_page=50",
            self.base_url, owner, repo, st
        );
        let resp = self
            .prepare_request(ureq::get(&url))
            .call()
            .map_err(|e| map_ureq_error(e, &url))?;

        resp.into_json()
            .map_err(|e| ForgeError::Json(e.to_string()))
    }

    pub fn get_pull_request(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
    ) -> Result<GitHubPullRequest, ForgeError> {
        let url = format!("{}/repos/{}/{}/pulls/{}", self.base_url, owner, repo, number);
        let resp = self
            .prepare_request(ureq::get(&url))
            .call()
            .map_err(|e| map_ureq_error(e, &url))?;

        resp.into_json()
            .map_err(|e| ForgeError::Json(e.to_string()))
    }

    pub fn list_issues(
        &self,
        owner: &str,
        repo: &str,
        state: Option<&str>,
    ) -> Result<Vec<GitHubIssue>, ForgeError> {
        let st = state.unwrap_or("open");
        let url = format!(
            "{}/repos/{}/{}/issues?state={}&per_page=50",
            self.base_url, owner, repo, st
        );
        let resp = self
            .prepare_request(ureq::get(&url))
            .call()
            .map_err(|e| map_ureq_error(e, &url))?;

        resp.into_json()
            .map_err(|e| ForgeError::Json(e.to_string()))
    }

    pub fn list_commits(
        &self,
        owner: &str,
        repo: &str,
        branch: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GitHubCommit>, ForgeError> {
        let mut url = format!(
            "{}/repos/{}/{}/commits?per_page={}",
            self.base_url,
            owner,
            repo,
            limit.min(100)
        );
        if let Some(b) = branch {
            url.push_str(&format!("&sha={}", b));
        }

        let resp = self
            .prepare_request(ureq::get(&url))
            .call()
            .map_err(|e| map_ureq_error(e, &url))?;

        resp.into_json()
            .map_err(|e| ForgeError::Json(e.to_string()))
    }

    pub fn list_workflow_runs(
        &self,
        owner: &str,
        repo: &str,
        limit: usize,
    ) -> Result<Vec<GitHubWorkflowRun>, ForgeError> {
        let url = format!(
            "{}/repos/{}/{}/actions/runs?per_page={}",
            self.base_url,
            owner,
            repo,
            limit.min(100)
        );
        let resp = self
            .prepare_request(ureq::get(&url))
            .call()
            .map_err(|e| map_ureq_error(e, &url))?;

        #[derive(serde::Deserialize)]
        struct WorkflowRunsResp {
            workflow_runs: Vec<GitHubWorkflowRun>,
        }

        let body: WorkflowRunsResp = resp
            .into_json()
            .map_err(|e| ForgeError::Json(e.to_string()))?;

        Ok(body.workflow_runs)
    }

    pub fn get_raw_blob(
        &self,
        owner: &str,
        repo: &str,
        r#ref: &str,
        path: &str,
    ) -> Result<String, ForgeError> {
        let raw_url = format!(
            "https://raw.githubusercontent.com/{}/{}/{}/{}",
            owner,
            repo,
            r#ref,
            path.trim_start_matches('/')
        );
        let mut req = ureq::get(&raw_url).set("User-Agent", "depot-forge/0.2.0");
        if let Some(tok) = self.auth.token() {
            req = req.set("Authorization", &format!("Bearer {}", tok));
        }

        let resp = req.call().map_err(|e| map_ureq_error(e, &raw_url))?;
        resp.into_string()
            .map_err(|e| ForgeError::Network(e.to_string()))
    }

    pub fn graphql_query(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, ForgeError> {
        if !self.auth.is_authenticated() {
            return Err(ForgeError::MissingAuth(
                "GitHub GraphQL API requires authentication".to_string(),
            ));
        }

        let url = format!("{}/graphql", self.base_url);
        let payload = json!({
            "query": query,
            "variables": variables.unwrap_or_else(|| json!({}))
        });

        let resp = self
            .prepare_request(ureq::post(&url))
            .send_json(payload)
            .map_err(|e| map_ureq_error(e, &url))?;

        resp.into_json()
            .map_err(|e| ForgeError::Json(e.to_string()))
    }
}

fn map_ureq_error(err: ureq::Error, url: &str) -> ForgeError {
    match err {
        ureq::Error::Status(code, resp) => {
            let msg = resp.into_string().unwrap_or_else(|_| "Unknown HTTP error".to_string());
            ForgeError::Http(code, format!("URL: {} -> {}", url, msg))
        }
        ureq::Error::Transport(t) => ForgeError::Network(format!("URL: {} -> {}", url, t)),
    }
}
