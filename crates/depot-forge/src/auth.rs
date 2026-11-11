use std::fmt;
use depot_core::VaultEnv;

#[derive(Clone)]
pub struct ForgeAuth {
    token: Option<String>,
}

impl fmt::Debug for ForgeAuth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ForgeAuth")
            .field("has_token", &self.token.is_some())
            .field("token", &self.token.as_ref().map(|_| "***"))
            .finish()
    }
}

impl ForgeAuth {
    pub fn new(token: Option<String>) -> Self {
        Self {
            token: token.map(|t| t.trim().to_string()),
        }
    }

    pub fn github_from_vault() -> Self {
        let tok = VaultEnv::get_first(&["GITHUB_TOKEN", "GH_TOKEN"]);
        Self::new(tok)
    }

    pub fn gitlab_from_vault(host: Option<&str>) -> Self {
        let is_self_hosted = host.map(|h| h.contains("aphrody.com")).unwrap_or(false);
        let keys: &[&str] = if is_self_hosted {
            &["GITLAB_SELF_TOKEN", "GITLAB_TOKEN"]
        } else {
            &["GITLAB_TOKEN", "GITLAB_SELF_TOKEN"]
        };
        let tok = VaultEnv::get_first(keys);
        Self::new(tok)
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }
}
