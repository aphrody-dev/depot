pub mod artifacts;
pub mod depot_tools;
pub mod fork;
pub mod git_slim;
pub mod ignore;
pub mod worktree;

pub use artifacts::{Artifact, ArtifactManifest, ArtifactReport};
pub use depot_tools::{DepotTools, DepotToolsInfo};
pub use fork::{ForkManager, ForkStatus, SyncStrategy};
pub use git_slim::{CloneOptions, GitSlim, SlimOptions, SlimReport};
pub use ignore::{IgnoreTemplate, SmartFilter};
pub use worktree::{WorktreeInfo, WorktreeManager};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_computation() {
        let tmp_path = std::env::temp_dir().join("test_depot_blake3.txt");
        std::fs::write(&tmp_path, b"hello depot universe").unwrap();
        let (hash, size) = Artifact::compute_blake3(&tmp_path).unwrap();
        let _ = std::fs::remove_file(&tmp_path);
        assert_eq!(size, 20);
        assert!(!hash.is_empty());
    }

    #[test]
    fn test_ignore_template() {
        let tmpl = SmartFilter::template(IgnoreTemplate::Rust);
        assert!(tmpl.contains("/target/"));
        assert!(tmpl.contains("*.pdb"));
    }

    #[test]
    fn test_depot_tools_detection() {
        let info = DepotTools::detect();
        assert!(info.root_path.is_some());
        assert!(info.gclient_found);
    }
}
