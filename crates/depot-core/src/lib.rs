pub mod artifacts;
pub mod depot_tools;
pub mod fork;
pub mod git_slim;
pub mod ignore;
pub mod path;
pub mod style;
pub mod sys;
pub mod worktree;

pub use artifacts::{Artifact, ArtifactBinaryHeader, ArtifactEntry, ArtifactManifest, ArtifactReport};
pub use depot_tools::{DepotTools, DepotToolsInfo};
pub use fork::{ForkManager, ForkStatus, SyncStrategy};
pub use git_slim::{CloneOptions, GitSlim, SlimOptions, SlimReport};
pub use ignore::{IgnoreTemplate, SmartFilter};
pub use path::normalize_posix;
pub use style::StyleExt;
pub use sys::*;
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
    fn test_zerocopy_binary_header() {
        let header = ArtifactBinaryHeader::new(42, [0xabu8; 32], 1024 * 1024);
        let bytes = header.as_bytes();
        assert_eq!(bytes.len(), std::mem::size_of::<ArtifactBinaryHeader>());
        assert_eq!(&bytes[..8], b"DEPOTART");

        let (parsed, remaining) = ArtifactBinaryHeader::parse_from_prefix(bytes).unwrap();
        assert_eq!(parsed, header);
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_posix_path_normalization() {
        use std::path::Path;
        let p1 = Path::new("foo\\bar\\baz.rs");
        assert_eq!(normalize_posix(p1), "foo/bar/baz.rs");

        let p2 = Path::new("a/b/../c/./d.txt");
        assert_eq!(normalize_posix(p2), "a/c/d.txt");
    }

    #[test]
    fn test_native_sys_calls() {
        assert!(enable_vt_processing());
        let (cpus, arch) = get_cpu_info();
        assert!(cpus > 0);
        let _ = arch;

        let perf_cnt = query_perf_counter();
        let perf_freq = query_perf_frequency();
        assert!(perf_freq > 0);
        let _ = perf_cnt;

        #[cfg(windows)]
        {
            let mem = get_memory_status();
            assert!(mem.is_some());
            let (total, avail) = mem.unwrap();
            assert!(total > 0);
            assert!(avail > 0);
        }
    }

    #[test]
    fn test_style_ext() {
        let text = "STATUS";
        assert_eq!(text.green(), "\x1b[32mSTATUS\x1b[0m");
        assert_eq!(text.red().bold(), "\x1b[1m\x1b[31mSTATUS\x1b[0m\x1b[0m");
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
