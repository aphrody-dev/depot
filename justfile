# Depot Unified Justfile — LLM-Optimized Monorepo & depot_tools Automation
set shell := ["powershell", "-NoProfile", "-Command"]

# Default recipe: list available recipes
default:
    just --list

# Build release binaries (depot.exe, gitingest.exe, depot-graph.exe)
build:
    cargo build --release --workspace --target-dir target

# Check workspace code
check:
    cargo check --workspace --target-dir target

# Run all automated tests
test:
    cargo test --workspace --target-dir target

# Install shims and link to C:\src\depot_tools so depot is globally available on PATH
install-shims:
    @Write-Host "Installing depot_tools shims..." -ForegroundColor Cyan
    @Set-Content -Path "C:\src\depot_tools\depot.bat" -Value '@"C:\src\depot\target\release\depot.exe" %*'
    @Set-Content -Path "C:\src\depot_tools\gitingest.bat" -Value '@"C:\src\depot\target\release\gitingest.exe" %*'
    @Set-Content -Path "C:\src\depot_tools\depot-graph.bat" -Value '@"C:\src\depot\target\release\depot-graph.exe" %*'
    @Add-Content -Path "C:\src\depot_tools\.git\info\exclude" -Value "`ndepot.bat`ngitingest.bat`ndepot-graph.bat" -ErrorAction SilentlyContinue
    @Write-Host "[OK] Shims installed in C:\src\depot_tools and excluded from git tracking." -ForegroundColor Green

# Ingest codebase and generate LLM digest (100x faster than python gitingest)
ingest path="." format="markdown" max_tokens="":
    @if ("{{max_tokens}}" -ne "") { cargo run --release --bin depot --target-dir target -- ingest "{{path}}" --format "{{format}}" --max-tokens {{max_tokens}} } else { cargo run --release --bin depot --target-dir target -- ingest "{{path}}" --format "{{format}}" }

# Generate AST knowledge graph (graph.json, GRAPH_REPORT.md, ARCHITECTURE.mermaid)
graph path="." out="graphify-out":
    cargo run --release --bin depot --target-dir target -- graph build "{{path}}" --out-dir "{{out}}"

# Query a symbol or connection in the AST graph
graph-query term path=".":
    cargo run --release --bin depot --target-dir target -- graph query "{{term}}" "{{path}}"

# Optimize repository using Git Scalar, commit-graph, and multi-pack-index
slim path=".":
    cargo run --release --bin depot --target-dir target -- slim "{{path}}"

# List active worktrees
worktree-list path=".":
    cargo run --release --bin depot --target-dir target -- worktree list "{{path}}"

# Add parallel worktree for LLM/multi-agent branching
worktree-add dest branch repo=".":
    cargo run --release --bin depot --target-dir target -- worktree add "{{dest}}" "{{branch}}" --new "{{repo}}"

# Inspect upstream fork status
fork-status path=".":
    cargo run --release --bin depot --target-dir target -- fork status "{{path}}"

# Verify Unreal Engine / CIPD style content-addressed Blake3 artifacts
artifacts-verify manifest="depot-artifacts.toml":
    cargo run --release --bin depot --target-dir target -- artifacts verify "{{manifest}}"

# Compute Blake3 hash and byte size of a file
artifacts-hash file:
    cargo run --release --bin depot --target-dir target -- artifacts hash "{{file}}"

# Display system info and depot_tools bridge status
info:
    cargo run --release --bin depot --target-dir target -- info
