pub mod artifacts;
pub mod depot_tools;
pub mod ecosystems;
pub mod fork;
pub mod git_slim;
pub mod gix_engine;
pub mod ignore;
pub mod just_engine;
pub mod magika;
pub mod path;
pub mod style;
pub mod sys;
pub mod vault;
pub mod worktree;

pub use artifacts::{Artifact, ArtifactBinaryHeader, ArtifactEntry, ArtifactManifest, ArtifactReport};
pub use depot_tools::{DepotTools, DepotToolsInfo};
pub use ecosystems::{
    AospProjectEntry, DepsPin, EcosystemGovernanceStatus, EcosystemManifests,
    KernelMaintainerSubsystem,
};
pub use fork::{ForkManager, ForkStatus, SyncStrategy};
pub use git_slim::{CloneOptions, GitSlim, SlimOptions, SlimReport};
pub use gix_engine::{GixCommitSummary, GixEngine, GixRepoTelemetry, GixTreeEntry};
pub use ignore::{IgnoreTemplate, SmartFilter};
pub use just_engine::{JustEngine, JustParam, JustRecipe, JustfileSpec};
pub use magika::{ContentCategory, ContentSignature, FeatureHeader, detect_content};
pub use path::normalize_posix;
pub use style::StyleExt;
pub use sys::*;
pub use vault::VaultEnv;
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
    fn test_magika_detection() {
        let p = std::path::Path::new("main.rs");
        let sig = detect_content(p, b"fn main() { println!(\"hello\"); }\n");
        assert_eq!(sig.category, ContentCategory::Rust);
        assert!(!sig.is_binary);

        let pe_sample = b"MZ\x90\x00\x03\x00\x00\x00";
        let pe_sig = detect_content(std::path::Path::new("kernel.exe"), pe_sample);
        assert_eq!(pe_sig.category, ContentCategory::PeExecutable);
        assert!(pe_sig.is_binary);
    }

    #[test]
    fn test_depot_tools_detection() {
        let info = DepotTools::detect();
        assert!(info.root_path.is_some());
        assert!(info.gclient_found);
        assert!(info.repo_found);
    }

    #[test]
    fn test_four_pillar_manifest_parsers() {
        let deps = "deps = {\n  'third_party/chromium/src': 'https://chromium.googlesource.com/chromium/src.git@abcdef1234',\n}\n";
        let pins = EcosystemManifests::parse_chromium_deps(deps);
        assert_eq!(pins.len(), 1);
        assert_eq!(
            pins.get("third_party/chromium/src")
                .and_then(|p| p.revision.as_deref()),
            Some("abcdef1234")
        );

        let xml = "<manifest><default remote=\"aosp\" revision=\"main\"/><project path=\"build/kleaf\" name=\"kernel/build\" groups=\"aosp,gki\" /></manifest>";
        let projs = EcosystemManifests::parse_aosp_manifest(xml);
        assert_eq!(projs.len(), 1);
        assert_eq!(projs[0].path, "build/kleaf");
        assert_eq!(projs[0].remote, "aosp");

        let maint = "LINUX KERNEL SUBSYSTEM\nM:\tTest <contact@aphrody.com>\nS:\tSupported\nT:\tgit https://git.kernel.org/\nF:\tthird_party/linux/\n";
        let subs = EcosystemManifests::parse_kernel_maintainers(maint);
        assert_eq!(subs.len(), 1);
        let hits = EcosystemManifests::lookup_maintainers_for_path(&subs, "third_party/linux/Makefile");
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn test_justfile_dag_parser() {
        let jf = r#"
set shell := ["pwsh", "-NoLogo", "-Command"]
export RUST_LOG := "info"
alias b := build

# Compile release workspace
[group('rust')]
build target="release": lint
    cargo build --{{target}}

[private]
lint:
    cargo clippy
"#;
        let spec = JustEngine::parse_str("justfile", jf);
        assert_eq!(spec.settings.get("shell").map(|s| s.as_str()), Some("[\"pwsh\", \"-NoLogo\", \"-Command\"]"));
        assert_eq!(spec.variables.get("RUST_LOG").map(|s| s.as_str()), Some("\"info\""));
        assert_eq!(spec.aliases.get("b").map(|s| s.as_str()), Some("build"));
        assert_eq!(spec.recipes.len(), 2);
        assert_eq!(spec.recipes[0].name, "build");
        assert_eq!(spec.recipes[0].dependencies, vec!["lint".to_string()]);
        assert_eq!(spec.recipes[0].group.as_deref(), Some("rust"));
        assert!(spec.recipes[1].is_private);
    }

    #[test]
    fn test_gix_engine_inspects_monorepo() {
        let telemetry = GixEngine::inspect(std::path::Path::new(r"C:\src"), 3).unwrap();
        assert!(!telemetry.branch.is_empty());
        assert!(telemetry.head_commit.is_some());
        assert!(!telemetry.recent_commits.is_empty());
    }
}
