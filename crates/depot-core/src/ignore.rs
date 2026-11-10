use std::path::Path;
use ignore::WalkBuilder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnoreTemplate {
    Rust,
    BunNode,
    ChromiumCpp,
    Python,
    LlmUniversal,
}

pub struct SmartFilter;

impl SmartFilter {
    pub const DEFAULT_LLM_IGNORES: &'static [&'static str] = &[
        // Git and VCS
        ".git",
        ".git/*",
        ".jj",
        ".svn",
        ".hg",
        // Package managers & dependencies
        "node_modules",
        "vendor",
        "bower_components",
        ".pnpm-store",
        // Locks (too many tokens with minimal semantic value)
        "package-lock.json",
        "yarn.lock",
        "pnpm-lock.yaml",
        "bun.lockb",
        "Cargo.lock",
        "poetry.lock",
        "uv.lock",
        "Gemfile.lock",
        // Build outputs & caches
        "target",
        "dist",
        "build",
        "out",
        ".next",
        ".nuxt",
        ".turbo",
        "__pycache__",
        "*.pyc",
        ".pytest_cache",
        ".ruff_cache",
        ".mypy_cache",
        // Binary files & compiled objects
        "*.exe",
        "*.dll",
        "*.dylib",
        "*.so",
        "*.a",
        "*.lib",
        "*.o",
        "*.obj",
        "*.pdb",
        "*.wasm",
        // Heavy media & assets
        "*.png",
        "*.jpg",
        "*.jpeg",
        "*.gif",
        "*.webp",
        "*.ico",
        "*.svg",
        "*.mp4",
        "*.webm",
        "*.mp3",
        "*.wav",
        "*.zip",
        "*.tar*",
        "*.gz",
        "*.7z",
        // ML models & heavy data
        "*.onnx",
        "*.safetensors",
        "*.bin",
        "*.pt",
        "*.pth",
        "*.gguf",
        "*.parquet",
        "*.arrow",
        // Graphify / analysis outputs
        "graphify-out",
        "graphify-out/*",
    ];

    pub fn template(t: IgnoreTemplate) -> &'static str {
        match t {
            IgnoreTemplate::Rust => {
                r#"# Rust .gitignore (Depot Standard)
/target/
**/*.rs.bk
*.pdb
*.lib
Cargo.lock
"#
            }
            IgnoreTemplate::BunNode => {
                r#"# Bun / Node .gitignore (Depot Standard)
node_modules/
dist/
.next/
.turbo/
.cache/
*.log
bun.lock
bun.lockb
package-lock.json
yarn.lock
"#
            }
            IgnoreTemplate::ChromiumCpp => {
                r#"# Chromium / C++ .gitignore (Depot Standard)
out/
build/
*.obj
*.o
*.ninja
*.ninja_deps
*.ninja_log
*.ilk
*.pdb
"#
            }
            IgnoreTemplate::Python => {
                r#"# Python .gitignore (Depot Standard)
__pycache__/
*.py[cod]
*$py.class
*.so
.Python
build/
develop-eggs/
dist/
downloads/
eggs/
.eggs/
lib/
lib64/
parts/
sdist/
var/
wheels/
*.egg-info/
.installed.cfg
*.egg
.pytest_cache/
.ruff_cache/
.mypy_cache/
.venv/
venv/
"#
            }
            IgnoreTemplate::LlmUniversal => {
                r#"# Universal LLM Ignore (.llmignore - Depot Standard)
# VCS
.git/
.jj/

# Dependencies & Lockfiles
node_modules/
target/
dist/
build/
out/
.next/
__pycache__/
*.lock
*.lockb
package-lock.json

# Binaries & Media
*.exe
*.dll
*.so
*.dylib
*.a
*.lib
*.pdb
*.png
*.jpg
*.jpeg
*.gif
*.webp
*.mp4
*.wasm
*.onnx
*.safetensors
*.gguf

# Temp & Artifacts
graphify-out/
*.tmp
*.log
"#
            }
        }
    }

    pub fn build_walker(
        root: &Path,
        respect_gitignore: bool,
        respect_llmignore: bool,
        custom_excludes: &[String],
    ) -> WalkBuilder {
        let mut builder = WalkBuilder::new(root);
        builder
            .git_ignore(respect_gitignore)
            .git_global(respect_gitignore)
            .git_exclude(respect_gitignore)
            .hidden(true)
            .parents(true);

        // Add standard LLM ignore overrides
        if respect_llmignore {
            let mut overrides = ignore::overrides::OverrideBuilder::new(root);
            for pattern in Self::DEFAULT_LLM_IGNORES {
                let neg = format!("!{}", pattern);
                let _ = overrides.add(&neg);
            }
            for custom in custom_excludes {
                let neg = format!("!{}", custom);
                let _ = overrides.add(&neg);
            }
            if let Ok(ov) = overrides.build() {
                builder.overrides(ov);
            }
        }

        // Also check if .llmignore file exists in root
        let llmignore_file = root.join(".llmignore");
        if respect_llmignore && llmignore_file.exists() {
            builder.add_custom_ignore_filename(".llmignore");
        }

        builder
    }
}
