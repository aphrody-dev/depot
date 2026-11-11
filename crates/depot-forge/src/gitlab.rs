use crate::auth::ForgeAuth;
use crate::error::ForgeError;
use crate::models::{GitLabIssue, GitLabMergeRequest, GitLabPipeline, GitLabProject};

pub struct GitLabClient {
    auth: ForgeAuth,
    base_url: String,
}

impl GitLabClient {
    pub fn new(base_url: Option<&str>, token: Option<String>) -> Self {
        let base = base_url
            .unwrap_or("https://gitlab.com")
            .trim_end_matches('/')
            .to_string();
        Self {
            auth: ForgeAuth::new(token),
            base_url: base,
        }
    }

    pub fn from_vault(host: Option<&str>) -> Self {
        let base = host
            .unwrap_or("https://gitlab.com")
            .trim_end_matches('/')
            .to_string();
        Self {
            auth: ForgeAuth::gitlab_from_vault(Some(&base)),
            base_url: base,
        }
    }

    pub fn auth(&self) -> &ForgeAuth {
        &self.auth
    }

    fn prepare_request(&self, req: ureq::Request) -> ureq::Request {
        let mut r = req
            .set("User-Agent", "depot-forge/0.2.0")
            .set("Accept", "application/json");
        if let Some(tok) = self.auth.token() {
            r = r.set("PRIVATE-TOKEN", tok);
        }
        r
    }

    fn encode_path(p: &str) -> String {
        p.replace('/', "%2F")
    }

    pub fn get_project(&self, project: &str) -> Result<GitLabProject, ForgeError> {
        let enc = Self::encode_path(project);
        let url = format!("{}/api/v4/projects/{}", self.base_url, enc);
        let resp = self
            .prepare_request(ureq::get(&url))
            .call()
            .map_err(|e| map_ureq_error(e, &url))?;

        resp.into_json()
            .map_err(|e| ForgeError::Json(e.to_string()))
    }

    pub fn list_group_projects(&self, group: &str) -> Result<Vec<GitLabProject>, ForgeError> {
        let enc = Self::encode_path(group);
        let url = format!(
            "{}/api/v4/groups/{}/projects?per_page=100&include_subgroups=true",
            self.base_url, enc
        );
        let resp = self
            .prepare_request(ureq::get(&url))
            .call()
            .map_err(|e| map_ureq_error(e, &url))?;

        resp.into_json()
            .map_err(|e| ForgeError::Json(e.to_string()))
    }

    pub fn list_merge_requests(
        &self,
        project: &str,
        state: Option<&str>,
    ) -> Result<Vec<GitLabMergeRequest>, ForgeError> {
        let enc = Self::encode_path(project);
        let st = state.unwrap_or("opened");
        let url = format!(
            "{}/api/v4/projects/{}/merge_requests?state={}&per_page=50",
            self.base_url, enc, st
        );
        let resp = self
            .prepare_request(ureq::get(&url))
            .call()
            .map_err(|e| map_ureq_error(e, &url))?;

        resp.into_json()
            .map_err(|e| ForgeError::Json(e.to_string()))
    }

    pub fn list_issues(&self, project: &str) -> Result<Vec<GitLabIssue>, ForgeError> {
        let enc = Self::encode_path(project);
        let url = format!(
            "{}/api/v4/projects/{}/issues?state=opened&per_page=50",
            self.base_url, enc
        );
        let resp = self
            .prepare_request(ureq::get(&url))
            .call()
            .map_err(|e| map_ureq_error(e, &url))?;

        resp.into_json()
            .map_err(|e| ForgeError::Json(e.to_string()))
    }

    pub fn list_pipelines(
        &self,
        project: &str,
        limit: usize,
    ) -> Result<Vec<GitLabPipeline>, ForgeError> {
        let enc = Self::encode_path(project);
        let url = format!(
            "{}/api/v4/projects/{}/pipelines?per_page={}",
            self.base_url,
            enc,
            limit.min(100)
        );
        let resp = self
            .prepare_request(ureq::get(&url))
            .call()
            .map_err(|e| map_ureq_error(e, &url))?;

        resp.into_json()
            .map_err(|e| ForgeError::Json(e.to_string()))
    }

    pub fn get_raw_file(
        &self,
        project: &str,
        r#ref: &str,
        path: &str,
    ) -> Result<String, ForgeError> {
        let enc_project = Self::encode_path(project);
        let enc_path = Self::encode_path(path.trim_start_matches('/'));
        let url = format!(
            "{}/api/v4/projects/{}/repository/files/{}/raw?ref={}",
            self.base_url, enc_project, enc_path, r#ref
        );
        let resp = self
            .prepare_request(ureq::get(&url))
            .call()
            .map_err(|e| map_ureq_error(e, &url))?;

        resp.into_string()
            .map_err(|e| ForgeError::Network(e.to_string()))
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
