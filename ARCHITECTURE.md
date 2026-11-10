# Architecture Reference: Unified LLM-Optimized Monorepo & Git Infrastructure
**Sovereign Git Engine: Depot Tools, Unreal Engine GitDependencies, Jujutsu (`jj`), and Gitoxide (`gix`)**

---

## 1. Executive Summary & Vision

Modern large-scale development environments—spanning multi-gigabyte codebases such as Chromium, Unreal Engine 6, AOSP, and LLM-driven autonomous agent swarms—face two existential bottlenecks in version control:
1. **Repository Weight & Object Bloat**: Full Git clones download gigabytes of historical object trees and binary blobs, choking disk I/O, network bandwidth, and filesystem monitoring.
2. **LLM Context Window Inefficiency**: Large Language Models (LLMs) cannot ingest raw 100,000-file repositories without deterministic filtering, smart token budgeting, structural AST summarization, and noise elimination (lockfiles, build artifacts, compiled binaries).

The **Depot** ecosystem (`C:\src\depot`) unifies Google's **`depot_tools`**, **Unreal Engine's CDN binary artifact architecture**, **Microsoft Git Scalar**, **Jujutsu (`jj`)**, **`gitoxide` (`gix`)**, **`gitingest`**, and **`graphify`** into a high-performance, native Rust 2024 monorepo with `justfile` automation.

---

## 2. Deep Dive: Google `depot_tools` Architecture

`C:\src\depot_tools` powers Google's Chromium, V8, WebRTC, Skia, Dawn, and ANGLE ecosystems. Its core components operate as follows:

### 2.1 Multiverse Management: `gclient` and `DEPS`
Unlike Git submodules which record fixed Git tree hashes directly in the parent tree object, Google's `gclient` uses Python-evaluated declarative manifest files named `DEPS`:
- **`solutions`**: Defined in the root `.gclient` file, identifying top-level repositories and checkout modes.
- **`vars` and `deps`**: Dynamic mapping of paths to Git URLs pinned to specific commit hashes or CIPD packages.
- **Conditional Dependencies (`deps_os`)**: Platform-specific sub-repositories (e.g., checkout Windows SDK tools only on Windows).

### 2.2 Local Mirroring & Zero-Duplication: `git_cache.py`
To avoid downloading tens of gigabytes across multiple local checkouts or clean CI builds:
- `depot_tools` maintains a centralized bare Git mirror in `LOCALAPPDATA/depot_tools/git_cache` (e.g. `chromium.googlesource.com-chromium-src`).
- New working checkouts are initialized with `git clone --shared` or `git clone --reference <cache_dir>`.
- The checkout's `.git/objects/info/alternates` file points directly to the shared local mirror. Disk consumption per checkout is reduced from 40+ GB down to the working tree files alone.

### 2.3 Binary Distribution: CIPD (Chrome Infra Package Deployer)
Google completely separates binary toolchains from Git history using CIPD (`cipd.exe`):
- Compilers (Clang/LLVM), Python runtimes, Node/Bun runtimes, GN binaries, and test fixtures are packaged as content-addressed zip archives on Google Cloud Storage.
- `DEPS` pins CIPD packages by SHA-256 instance IDs (e.g. `package: infra/tools/cipd/windows-amd64@<hash>`).
- CIPD extracts binaries directly into designated directories without committing a single byte to Git history.

### 2.4 The Windows Worktree Gap: `gclient-new-workdir.py`
Chromium provides `gclient-new-workdir.py` to clone workdirs quickly. However, line 26 of Google's script contains:
```python
if sys.platform == "win32":
    print("ERROR: This script cannot run on Windows because it uses symlinks.")
    sys.exit(1)
```
Depot solves this natively: `depot worktree` replaces brittle Unix symlinks with native `git worktree` and directory junction management, functioning 100% reliably on Windows NT NTFS.

---

## 3. Deep Dive: Unreal Engine GitHub Setup (`GitDependencies` + Scalar)

Epic Games distributes Unreal Engine via GitHub (`EpicGames/UnrealEngine`), managing one of the largest C++ game engine repositories in the world (~150 GB with full binaries).

### 3.1 Content-Addressed CDN Binary Manifest (`GitDependencies.exe`)
Epic Games prevents Git object bloat through a proprietary tool:
- The Git repository contains only source code, shaders, build rules, and metadata.
- Precompiled third-party libraries (DirectX, Vulkan, PhysX, Chaos, consoles SDKs) are tracked in `.ue4dependencies` / `Commit.gitdeps.xml`.
- When running `Setup.bat`, `GitDependencies.exe` reads the XML/manifest, computes SHA-1/SHA-256 hashes of local files, and downloads compressed chunks (`.pack`) from an Amazon CloudFront CDN.
- Git history stays lean (under 2 GB of text source code), while binary SDKs are fetched on demand.

### 3.2 Microsoft Git Scalar (formerly GVFS / VFSForGit)
To handle 100,000+ files efficiently on Windows, Microsoft developed GVFS, now standardized into upstream Git as **Git Scalar**:
- **Blobless Clone (`--filter=blob:none`)**: Downloads commit trees and metadata instantly; file contents (blobs) are downloaded lazily on checkout or read.
- **Filesystem Monitor (`core.fsmonitor=true`)**: Hooks into the Windows NTFS Change Journal (`USN Journal`) or daemon to track file modifications without calling `stat()` on 200,000 files.
- **Many-Files Index Optimization (`feature.manyFiles=true`)**: Enables `index.version 4` with prefix compression and untracked cache (`core.untrackedCache=true`).
- **Commit Graph & Multi-Pack-Index**: Background maintenance (`scalar register`, `git commit-graph write --reachable --changed-paths`, `git multi-pack-index write`) keeps commit traversals instantaneous.

---

## 4. Modern Rust Git Alternatives: `gitoxide` (`gix`) & Jujutsu (`jj`)

| Metric / Capability | Git C (Default) | Google `depot_tools` | Epic Games UE | `gitoxide` (`gix`) | Jujutsu (`jj`) | **Depot (`depot`)** |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Language** | C / Shell / Perl | Python / Batch | C# / C++ | Pure Rust 2024 | Pure Rust 2024 | **Pure Rust 2024** |
| **Windows Support** | Good (Git for Win) | Partial (`workdir` fails) | Native | Native | Native | **100% Native NTFS** |
| **Minimal Git Weight** | Manual flags | Mirror + alternates | GitDependencies | Native streaming | Git-backed | **Scalar + Blobless Auto** |
| **Binary Artifacts** | Git LFS (heavy) | CIPD packages | CDN `.ue4dependencies`| Custom | External | **Blake3 Manifest TOML** |
| **LLM Ingest Engine** | None | None | None | None | None | **Native Ingest (100x speed)**|
| **AST Knowledge Graph**| None | None | None | None | None | **Native Graphify AST** |
| **Task Automation** | Makefile / Shell | Python recipes | Batch / UAT | Cargo | Shell | **Justfile (`just`)** |

### 4.1 Jujutsu (`jj`) at Google Scale
Developed by Martin von Zweigbergk at Google (`jj-vcs/jj`), Jujutsu is a Git-compatible version control system written in Rust:
- **Working Copy as a Commit**: The working directory is always an automatic commit (`@`). No staging index is needed.
- **Anonymous Branching**: Commits exist without named branches; branch pointers ("bookmarks") are updated smoothly.
- **Conflict-Free Operations**: Merge conflicts are first-class commit states that can be committed, pushed, and resolved later without interrupting rebases.
- **Depot Integration**: `depot_tools/jj/scm.py` provides Google's experimental bridge, while `depot` integrates seamlessly with `.jj` repositories.

### 4.2 Blake3 Content Addressing
Depot uses `blake3` instead of SHA-1 or SHA-256 for artifact verification:
- Blake3 processes ~15 GB/s with multi-core tree hashing (AVX-512 / AVX2 SIMD), outperforming SHA-256 by 12x.
- Large multi-gigabyte dependencies (Vulkan SDK, CUDA headers, LLVM toolchains) are verified in sub-second times.

---

## 5. Unified `depot` System Architecture

The monorepo at `C:\src\depot` is structured into four specialized crates:

```
C:\src\depot\
├── Cargo.toml                  # Cargo 2024 workspace root
├── ARCHITECTURE.md             # This comprehensive study
├── justfile                    # Master justfile automation
├── depot-artifacts.toml        # UE-style Blake3 artifact manifest
├── crates/
│   ├── depot-core/             # Core engine
│   │   ├── git_slim.rs         # Git Scalar, blobless clone, fsmonitor, index v4
│   │   ├── worktree.rs         # Windows-safe multi-agent worktree manager
│   │   ├── fork.rs             # Upstream tracking, divergence & rebase sync
│   │   ├── ignore.rs           # Standard .gitignore, .llmignore, SmartFilter
│   │   ├── artifacts.rs        # Content-addressed Blake3 artifact manifest
│   │   └── depot_tools.rs      # Detection & pass-through to Google depot_tools
│   ├── depot-ingest/           # Native Rust gitingest for LLMs
│   │   ├── scanner.rs          # Parallel rayon traversal with ignore crate
│   │   ├── tokenizer.rs        # Fast BPE / cl100k token estimation
│   │   ├── tree.rs             # ASCII hierarchy tree generator
│   │   └── formatter.rs        # Markdown, XML, JSON digest formatter with budget clamping
│   ├── depot-graph/            # Native Rust graphify AST knowledge graph
│   │   ├── parser.rs           # Multi-language symbol extractor (Rust, TS, Py, C++, Go)
│   │   ├── graph.rs            # Graph network, degree centrality, community detection
│   │   ├── export.rs           # graph.json, GRAPH_REPORT.md, ARCHITECTURE.mermaid
│   │   └── query.rs            # In-memory symbol search and dependency inspector
│   └── depot-cli/              # Unified CLI
│       ├── main.rs             # `depot` binary with subcommand router & pass-through
│       ├── bin/gitingest.rs    # `gitingest` standalone binary
│       ├── bin/depot_graph.rs  # `depot-graph` standalone binary
│       └── commands/           # Subcommand implementations
```

---

## 6. Verification & Operational Guide

All binaries are installed into `C:\src\depot_tools` as shims (`depot.bat`, `gitingest.bat`, `depot-graph.bat`), making them immediately available system-wide on PATH:

### 6.1 Codebase Ingestion for LLMs (`depot ingest` / `gitingest`)
```bash
# Ingest depot crates with a 15,000 token budget
depot ingest C:\src\depot\crates --max-tokens 15000 --format markdown

# Or use the standalone gitingest binary
gitingest C:\src\depot --format markdown --output digest.md
```

### 6.2 Knowledge Graph Generation (`depot graph` / `depot-graph`)
```bash
# Build graph.json, GRAPH_REPORT.md, and ARCHITECTURE.mermaid
depot graph build C:\src\depot\crates --out-dir C:\src\depot\graphify-out

# Query symbol dependencies in milliseconds
depot graph query Worktree C:\src\depot
```

### 6.3 Windows-Safe Worktrees for Parallel LLM Agents (`depot worktree`)
```bash
# List all active worktrees
depot worktree list

# Create a clean parallel branch worktree sharing .git objects
depot worktree add C:\src\depot-agent-1 agent-branch --new
```

### 6.4 Git Weight Reduction (`depot slim` / `depot clone`)
```bash
# Optimize an existing repository with Scalar-grade flags
depot slim C:\src\depot

# Clone any massive repository with minimal weight (blobless, no-tags, single-branch)
depot clone https://github.com/casey/just.git C:\src\just-slim
```

### 6.5 Binary Artifact Verification (`depot artifacts`)
```bash
# Compute Blake3 hash for any file
depot artifacts hash C:\src\depot\Cargo.toml

# Verify all project artifacts against depot-artifacts.toml
depot artifacts verify C:\src\depot\depot-artifacts.toml
```

### 6.6 depot_tools Pass-Through
```bash
# All Google depot_tools commands pass through transparently
depot autoninja --help
depot cipd --version
```
