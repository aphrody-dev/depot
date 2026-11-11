pub mod auth;
pub mod error;
pub mod github;
pub mod gitlab;
pub mod models;

pub use auth::ForgeAuth;
pub use error::ForgeError;
pub use github::GitHubClient;
pub use gitlab::GitLabClient;
pub use models::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_debug_masking() {
        let auth = ForgeAuth::new(Some("ghp_secret_token_12345".to_string()));
        let dbg = format!("{:?}", auth);
        assert!(!dbg.contains("ghp_secret_token_12345"));
        assert!(dbg.contains("***"));
    }

    #[test]
    fn test_models_deserialization() {
        let json_repo = r#"{
            "id": 12345,
            "name": "tree-sitter-rust",
            "full_name": "tree-sitter/tree-sitter-rust",
            "description": "Rust grammar for tree-sitter",
            "html_url": "https://github.com/tree-sitter/tree-sitter-rust",
            "clone_url": "https://github.com/tree-sitter/tree-sitter-rust.git",
            "default_branch": "master",
            "stargazers_count": 850,
            "open_issues_count": 12,
            "forks_count": 150,
            "language": "C"
        }"#;

        let repo: GitHubRepo = serde_json::from_str(json_repo).unwrap();
        assert_eq!(repo.name, "tree-sitter-rust");
        assert_eq!(repo.default_branch, "master");
        assert_eq!(repo.stargazers_count, 850);
    }

    #[test]
    fn test_gitlab_project_deserialization() {
        let json_proj = r#"{
            "id": 42,
            "name": "aphrody-core",
            "path_with_namespace": "aphrody/aphrody-core",
            "web_url": "https://git.aphrody.com/aphrody/aphrody-core",
            "star_count": 10,
            "forks_count": 2
        }"#;

        let proj: GitLabProject = serde_json::from_str(json_proj).unwrap();
        assert_eq!(proj.id, 42);
        assert_eq!(proj.name, "aphrody-core");
    }
}
