use std::path::PathBuf;
use argh::FromArgs;
use depot_core::StyleExt;

mod commands;
use commands::*;

#[derive(FromArgs, PartialEq, Debug)]
/// Unified LLM-optimized monorepo manager fusing Google depot_tools, gitingest, graphify, Git Scalar, Tree-Sitter, Just, Gitoxide & Tri-Forge
struct Cli {
    #[argh(subcommand)]
    command: Commands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Commands {
    Ingest(IngestArgs),
    Graph(GraphSubArgs),
    Ast(AstArgs),
    Lua(LuaSubArgs),
    Re(ReArgs),
    Migrate(MigrateArgs),
    Rope(RopeArgs),
    Just(JustArgs),
    Gix(GixArgs),
    Forge(ForgeSubArgs),
    Analyze(AnalyzeArgs),
    Worktree(WorktreeSubArgs),
    Fork(ForkSubArgs),
    Slim(SlimArgs),
    Clone(CloneArgs),
    Artifacts(ArtifactSubArgs),
    Manifests(ManifestsArgs),
    Maintainer(MaintainerArgs),
    Ignore(IgnoreArgs),
    Info(InfoArgs),
}

// Ingest
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ingest")]
/// Ingest a codebase or remote repo into an LLM-ready prompt digest
pub struct IngestArgs {
    /// target directory or remote URL to ingest (defaults to ".")
    #[argh(positional, default = "String::from(\".\")")]
    pub target: String,

    /// format: markdown, xml, json (default: markdown)
    #[argh(option, short = 'f', default = "String::from(\"markdown\")")]
    pub format: String,

    /// max token budget (clamps and prioritizes important files)
    #[argh(option, short = 'm')]
    pub max_tokens: Option<usize>,

    /// output file path (defaults to stdout)
    #[argh(option, short = 'o')]
    pub output: Option<PathBuf>,

    /// include glob patterns
    #[argh(option, short = 'i')]
    pub include: Vec<String>,

    /// exclude glob patterns
    #[argh(option, short = 'e')]
    pub exclude: Vec<String>,

    /// produce high-density symbol outline only
    #[argh(switch)]
    pub outline: bool,
}

// Graph
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "graph")]
/// AST / dependency graph generator and query tool for LLMs (Graphify)
pub struct GraphSubArgs {
    #[argh(subcommand)]
    pub sub: GraphCommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum GraphCommands {
    Build(GraphBuildArgs),
    Query(GraphQueryArgs),
    Path(GraphPathArgs),
    Explain(GraphExplainArgs),
    Mcp(GraphMcpArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "build")]
/// Build graph.json, graph.html, GRAPH_REPORT.md, and ARCHITECTURE.mermaid
pub struct GraphBuildArgs {
    /// target path (default: ".")
    #[argh(positional)]
    pub path: Option<PathBuf>,

    /// output directory (default: graphify-out)
    #[argh(option, short = 'o', default = "PathBuf::from(\"graphify-out\")")]
    pub out_dir: PathBuf,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "query")]
/// Query a symbol or connection in the graph with LLM token budget
pub struct GraphQueryArgs {
    /// symbol term to query
    #[argh(positional)]
    pub term: String,

    /// target directory containing graph (default: ".")
    #[argh(positional)]
    pub path: Option<PathBuf>,

    /// use DFS traversal instead of default BFS
    #[argh(switch)]
    pub dfs: bool,

    /// max token budget for LLM context output (default: 2000)
    #[argh(option, short = 'b', default = "2000")]
    pub budget: usize,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "path")]
/// Find shortest Dijkstra/BFS connection path between two symbols
pub struct GraphPathArgs {
    /// source symbol name
    #[argh(positional)]
    pub start: String,

    /// destination symbol name
    #[argh(positional)]
    pub end: String,

    /// target directory containing graph (default: ".")
    #[argh(positional)]
    pub path: Option<PathBuf>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "explain")]
/// Explain a symbol in rich dossier format (degree, community, callers, callees)
pub struct GraphExplainArgs {
    /// symbol name to explain
    #[argh(positional)]
    pub term: String,

    /// target directory containing graph (default: ".")
    #[argh(positional)]
    pub path: Option<PathBuf>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "mcp")]
/// Run Graphify Model Context Protocol (MCP) JSON-RPC 2.0 stdio server
pub struct GraphMcpArgs {
    /// target directory containing graph (default: ".")
    #[argh(positional)]
    pub path: Option<PathBuf>,
}

// AST
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ast")]
/// Inspect multi-language Tree-Sitter AST symbols or run S-expression queries
pub struct AstArgs {
    /// file path to parse
    #[argh(positional)]
    pub file: PathBuf,

    /// tree-sitter S-expression query (e.g. "(function_item) @fn")
    #[argh(option, short = 'q')]
    pub query: Option<String>,

    /// output as JSON
    #[argh(switch)]
    pub json: bool,
}

// RE (Reverse Engineering & Binary Analysis)
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "re")]
/// Reverse-engineering & runtime execution analysis (PE/ELF/Mach-O triage, MinGW CRT filter, x86-64 disasm, ABI classification)
pub struct ReArgs {
    /// target binary to inspect (e.g. .exe, .dll, .so)
    #[argh(positional)]
    pub binary: PathBuf,

    /// output as JSON
    #[argh(switch)]
    pub json: bool,
}

// Migrate (AST Codemods targeting Bun v1.4.4 & Rust LTO)
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "migrate")]
/// Universal AST codemods & migrations targeting Bun v1.4.4 and Rust LTO
pub struct MigrateArgs {
    /// root directory to migrate (default: ".")
    #[argh(positional, default = "PathBuf::from(\".\")")]
    pub root: PathBuf,

    /// dry run without modifying files on disk
    #[argh(switch)]
    pub dry_run: bool,
}

// Rope (Helix Transactional Span Editor)
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "rope")]
/// Helix-style transactional Rope text replacement
pub struct RopeArgs {
    /// target file to edit
    #[argh(positional)]
    pub file: PathBuf,

    /// replacement specification: START_BYTE:END_BYTE:REPLACEMENT
    #[argh(option, short = 'r')]
    pub replace: Vec<String>,
}

// Just
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "just")]
/// Inspect or run recipes from justfile
pub struct JustArgs {
    /// working directory or justfile path (default: ".")
    #[argh(option, short = 'd', default = "PathBuf::from(\".\")")]
    pub dir: PathBuf,

    /// output recipes as JSON
    #[argh(switch)]
    pub json: bool,
}

// Gix
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "gix")]
/// Pure-Rust Gitoxide zero-subprocess repository status & telemetry
pub struct GixArgs {
    /// target repository path (default: ".")
    #[argh(positional)]
    pub path: Option<PathBuf>,

    /// commit history limit (default: 5)
    #[argh(option, short = 'c', default = "5")]
    pub commits: usize,

    /// output as JSON
    #[argh(switch)]
    pub json: bool,
}

// Forge
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "forge")]
/// Strongly typed GitHub & GitLab API client and org inspector
pub struct ForgeSubArgs {
    #[argh(subcommand)]
    pub sub: ForgeCommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum ForgeCommands {
    Repo(ForgeRepoArgs),
    Org(ForgeOrgArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "repo")]
/// Inspect a repository on GitHub or GitLab
pub struct ForgeRepoArgs {
    /// forge: github or gitlab
    #[argh(positional)]
    pub forge: String,

    /// target repository in 'owner/repo' format
    #[argh(positional)]
    pub target: String,

    /// output as JSON
    #[argh(switch)]
    pub json: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "org")]
/// List repositories for a GitHub org or GitLab group
pub struct ForgeOrgArgs {
    /// forge: github or gitlab
    #[argh(positional)]
    pub forge: String,

    /// organization or group name (e.g. tree-sitter)
    #[argh(positional)]
    pub org: String,

    /// limit results (default: 30)
    #[argh(option, short = 'l', default = "30")]
    pub limit: usize,

    /// output as JSON
    #[argh(switch)]
    pub json: bool,
}

// Analyze
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "analyze")]
/// Comprehensive end-to-end analysis: Tree-Sitter AST + Graphify + Gitingest in 1 command
pub struct AnalyzeArgs {
    /// target path or repository (default: ".")
    #[argh(positional)]
    pub path: Option<PathBuf>,
}

// Worktree
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "worktree")]
/// Windows-safe worktree management without duplicating repo storage
pub struct WorktreeSubArgs {
    #[argh(subcommand)]
    pub sub: WorktreeCommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum WorktreeCommands {
    List(WorktreeListArgs),
    Add(WorktreeAddArgs),
    Remove(WorktreeRemoveArgs),
    Prune(WorktreePruneArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "list")]
/// List all active worktrees
pub struct WorktreeListArgs {
    /// repo path (default: ".")
    #[argh(positional)]
    pub path: Option<PathBuf>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "add")]
/// Add a new worktree
pub struct WorktreeAddArgs {
    /// destination path for the worktree
    #[argh(positional)]
    pub dest: PathBuf,

    /// branch name
    #[argh(positional)]
    pub branch: String,

    /// create a new branch (-n / --new)
    #[argh(switch, short = 'n')]
    pub new: bool,

    /// base repository path (default: ".")
    #[argh(option, short = 'r', default = "PathBuf::from(\".\")")]
    pub repo: PathBuf,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "remove")]
/// Remove an existing worktree
pub struct WorktreeRemoveArgs {
    /// worktree path to remove
    #[argh(positional)]
    pub dest: PathBuf,

    /// force removal even if untracked changes exist
    #[argh(switch, short = 'f')]
    pub force: bool,

    /// base repository path (default: ".")
    #[argh(option, short = 'r', default = "PathBuf::from(\".\")")]
    pub repo: PathBuf,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "prune")]
/// Prune stale worktree links
pub struct WorktreePruneArgs {
    /// base repository path (default: ".")
    #[argh(option, short = 'r', default = "PathBuf::from(\".\")")]
    pub repo: PathBuf,
}

// Fork
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "fork")]
/// Fork and upstream tracking, divergence check, and fast-forward/rebase sync
pub struct ForkSubArgs {
    #[argh(subcommand)]
    pub sub: ForkCommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum ForkCommands {
    Status(ForkStatusArgs),
    SetUpstream(ForkSetUpstreamArgs),
    Sync(ForkSyncArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "status")]
/// Inspect fork divergence from upstream
pub struct ForkStatusArgs {
    /// repo path (default: ".")
    #[argh(positional)]
    pub path: Option<PathBuf>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "set-upstream")]
/// Set upstream repository URL
pub struct ForkSetUpstreamArgs {
    /// upstream Git repository URL
    #[argh(positional)]
    pub url: String,

    /// repo path (default: ".")
    #[argh(positional)]
    pub path: Option<PathBuf>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "sync")]
/// Sync current branch with upstream
pub struct ForkSyncArgs {
    /// specific branch to sync
    #[argh(option, short = 'b')]
    pub branch: Option<String>,

    /// strategy: ff, rebase, merge (default: ff)
    #[argh(option, short = 's', default = "String::from(\"ff\")")]
    pub strategy: String,

    /// repo path (default: ".")
    #[argh(positional)]
    pub path: Option<PathBuf>,
}

// Slim
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "slim")]
/// Apply Git Scalar, commit-graph, multi-pack-index, and fsmonitor to minimize git weight
pub struct SlimArgs {
    /// target repository path (default: ".")
    #[argh(positional)]
    pub path: Option<PathBuf>,
}

// Clone
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "clone")]
/// Minimal blobless clone with automated Git Scalar configuration
pub struct CloneArgs {
    /// git repository URL
    #[argh(positional)]
    pub url: String,

    /// destination path
    #[argh(positional)]
    pub dest: PathBuf,

    /// specific branch
    #[argh(option, short = 'b')]
    pub branch: Option<String>,

    /// shallow clone depth
    #[argh(option, short = 'd')]
    pub depth: Option<u32>,

    /// sparse paths to include
    #[argh(option)]
    pub sparse: Vec<String>,
}

// Artifacts
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "artifacts")]
/// Unreal Engine GitDependencies + CIPD Blake3 content-addressed binary artifacts
pub struct ArtifactSubArgs {
    #[argh(subcommand)]
    pub sub: ArtifactCommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum ArtifactCommands {
    Verify(ArtifactVerifyArgs),
    Hash(ArtifactHashArgs),
    Init(ArtifactInitArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "verify")]
/// Verify all artifacts against Blake3 hashes in manifest
pub struct ArtifactVerifyArgs {
    /// manifest path (default: depot-artifacts.toml)
    #[argh(positional)]
    pub manifest: Option<PathBuf>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "hash")]
/// Compute Blake3 hash and size for a file
pub struct ArtifactHashArgs {
    /// file to hash
    #[argh(positional)]
    pub file: PathBuf,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "init")]
/// Initialize a new depot-artifacts.toml manifest
pub struct ArtifactInitArgs {
    /// manifest path (default: depot-artifacts.toml)
    #[argh(positional)]
    pub manifest: Option<PathBuf>,

    /// project name (default: project)
    #[argh(option, short = 'n', default = "String::from(\"project\")")]
    pub name: String,
}

// Manifests & Maintainer
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "manifests")]
/// Verify Chromium DEPS, AOSP .repo/manifests/default.xml, Kernel MAINTAINERS, and Quilt patch series
pub struct ManifestsArgs {
    /// monorepo root path (default: C:\src)
    #[argh(positional)]
    pub root: Option<PathBuf>,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "maintainer")]
/// Lookup Linux Kernel style MAINTAINERS ownership for any monorepo path
pub struct MaintainerArgs {
    /// relative path inside monorepo (e.g. third_party/linux/Makefile)
    #[argh(positional)]
    pub target: String,

    /// monorepo root path (default: C:\src)
    #[argh(option, short = 'r', default = "PathBuf::from(\"C:\\\\src\")")]
    pub root: PathBuf,
}

// Ignore
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ignore")]
/// Standard .gitignore and .llmignore templates
pub struct IgnoreArgs {
    /// template: rust, bun, chromium, python, llm (default: llm)
    #[argh(option, short = 't', default = "String::from(\"llm\")")]
    pub template: String,

    /// output path (default: .gitignore)
    #[argh(option, short = 'o', default = "PathBuf::from(\".gitignore\")")]
    pub output: PathBuf,
}

// Info
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "info")]
/// Display environment, native Windows NT kernel32 status, and feature flags
pub struct InfoArgs {}

// Lua 5.2 Sovereign Suite
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "lua")]
/// Lua 5.2 AST inspector, REA bytecode disassembler, x64<->wasm32 transcoder & project initializer
pub struct LuaSubArgs {
    #[argh(subcommand)]
    pub sub: LuaCommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum LuaCommands {
    Ast(LuaAstArgs),
    Disasm(LuaDisasmArgs),
    Transcode(LuaTranscodeArgs),
    Migrate(LuaMigrateArgs),
    Init(LuaInitArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ast")]
/// Parse Lua 5.2 source AST (functions, methods, ::labels::, _ENV, imports)
pub struct LuaAstArgs {
    /// path to .lua source file
    #[argh(positional)]
    pub file: PathBuf,

    /// emit structured JSON output
    #[argh(switch, short = 'j')]
    pub json: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "disasm")]
/// Disassemble PUC-Rio Lua 5.2 bytecode (.luac / .lua.bin) and audit _ENV globals
pub struct LuaDisasmArgs {
    /// path to compiled .luac or .lua.bin file
    #[argh(positional)]
    pub file: PathBuf,

    /// emit structured JSON output
    #[argh(switch, short = 'j')]
    pub json: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "transcode")]
/// Transcode Lua 5.2 bytecode header & size_t strings between x64 (8B) and wasm32 (4B)
pub struct LuaTranscodeArgs {
    /// input .luac / .lua.bin path
    #[argh(positional)]
    pub input: PathBuf,

    /// output .luac path
    #[argh(option, short = 'o')]
    pub output: PathBuf,

    /// target architecture: wasm32 or x64 (default: wasm32)
    #[argh(option, short = 'a', default = "String::from(\"wasm32\")")]
    pub arch: String,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "migrate")]
/// Migrate Lua 5.1 idioms (unpack, bit.*, loadstring) to Lua 5.2 via Helix Rope
pub struct LuaMigrateArgs {
    /// path to .lua file
    #[argh(positional)]
    pub file: PathBuf,

    /// apply edits in-place
    #[argh(switch, short = 'w')]
    pub write: bool,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "init")]
/// Scaffold a Lua 5.2 project (.luarc.json, stylua.toml, selene.toml, src/main.lua)
pub struct LuaInitArgs {
    /// target directory (default: ".")
    #[argh(positional, default = "PathBuf::from(\".\")")]
    pub dir: PathBuf,
}

fn main() {
    depot_core::enable_vt_processing();

    let mut raw_args: Vec<String> = std::env::args().collect();

    // Check pass-through for depot_tools
    if raw_args.len() > 1 {
        let first = &raw_args[1];
        if matches!(
            first.as_str(),
            "gclient" | "repo" | "cipd" | "fetch" | "autoninja" | "siso" | "gn" | "ninja"
        ) {
            let passthrough_args = &raw_args[2..];
            match depot_core::DepotTools::run_passthrough(first, passthrough_args) {
                Ok(code) => std::process::exit(code),
                Err(err) => {
                    eprintln!("{} {}", "[ERROR]".red().bold(), err);
                    std::process::exit(1);
                }
            }
        }
    }

    if raw_args.len() <= 1 {
        println!("{}", "depot - Run `depot --help` for available commands.".cyan());
        println!("\nAvailable commands:");
        println!("  analyze     End-to-end AST graphify + prompt ingestion in 1 shot");
        println!("  ast         Multi-language Tree-Sitter AST inspection & S-expression queries");
        println!("  lua         Lua 5.2 AST, REA bytecode disassembler, wasm32 transcoder & init");
        println!("  forge       Typed GitHub & GitLab API client and org repository inspector");
        println!("  gix         Pure-Rust Gitoxide zero-subprocess status and ODB telemetry");
        println!("  graph       AST / dependency graph generator and query tool (Graphify)");
        println!("  ingest      Ingest a codebase into LLM-ready prompt digest (Gitingest)");
        println!("  just        Justfile recipe runner, DAG parser, and structured inspector");
        println!("  worktree    Windows-safe git worktree management (alias: wt)");
        println!("  fork        Fork and upstream tracking & synchronization");
        println!("  slim        Apply Git Scalar and maintenance optimizations");
        println!("  clone       Minimal blobless clone with Git Scalar");
        println!("  artifacts   Blake3 content-addressed binary artifact management");
        println!("  manifests   Verify Chromium DEPS, AOSP repo XML & Kernel MAINTAINERS");
        println!("  maintainer  Query subsystem ownership via root MAINTAINERS");
        println!("  ignore      Standard .gitignore and .llmignore templates");
        println!("  info        Display environment, native NT kernel32 telemetry");
        return;
    }

    // Support 'wt' alias
    if raw_args[1] == "wt" {
        raw_args[1] = "worktree".to_string();
    }

    let args_strs: Vec<&str> = raw_args.iter().map(|s| s.as_str()).collect();
    let cli = match Cli::from_args(&[args_strs[0]], &args_strs[1..]) {
        Ok(c) => c,
        Err(early_exit) => {
            println!("{}", early_exit.output);
            std::process::exit(if early_exit.status.is_ok() { 0 } else { 1 });
        }
    };

    let result = match cli.command {
        Commands::Analyze(a) => analyze::run_analyze(a.path),

        Commands::Ast(a) => {
            if let Some(q) = a.query {
                ast::run_ast_query(a.file, q)
            } else {
                ast::run_ast_parse(a.file, a.json)
            }
        }

        Commands::Re(r) => re::execute_re(&r.binary, r.json),

        Commands::Lua(l) => match l.sub {
            LuaCommands::Ast(a) => lua::run_lua_ast(&a.file, a.json),
            LuaCommands::Disasm(d) => lua::run_lua_disasm(&d.file, d.json),
            LuaCommands::Transcode(t) => lua::run_lua_transcode(&t.input, &t.output, &t.arch),
            LuaCommands::Migrate(m) => lua::run_lua_migrate(&m.file, m.write),
            LuaCommands::Init(i) => lua::run_lua_init(&i.dir),
        },

        Commands::Migrate(m) => migrate::execute_migrate(&m.root, m.dry_run),

        Commands::Rope(rp) => rope::execute_rope(&rp.file, &rp.replace),

        Commands::Just(j) => just::run_just_list(j.dir, j.json),

        Commands::Gix(g) => gix::run_gix_status(g.path, g.commits, g.json),

        Commands::Forge(f) => match f.sub {
            ForgeCommands::Repo(r) => forge::run_forge_repo(r.forge, r.target, r.json),
            ForgeCommands::Org(o) => forge::run_forge_org_repos(o.forge, o.org, o.limit, o.json),
        },

        Commands::Ingest(args) => ingest::run_ingest(
            args.target,
            args.format,
            args.max_tokens,
            args.output,
            args.include,
            args.exclude,
            args.outline,
        ),

        Commands::Graph(sub_args) => match sub_args.sub {
            GraphCommands::Build(b) => {
                graph::run_graph_build(b.path.unwrap_or_else(|| PathBuf::from(".")), b.out_dir)
            }
            GraphCommands::Query(q) => {
                let opts = depot_graph::query::QueryOptions {
                    dfs: q.dfs,
                    max_tokens: q.budget,
                    max_depth: 4,
                };
                graph::run_graph_query_advanced(
                    q.path.unwrap_or_else(|| PathBuf::from(".")),
                    q.term,
                    opts,
                )
            }
            GraphCommands::Path(p) => graph::run_graph_path(
                p.path.unwrap_or_else(|| PathBuf::from(".")),
                p.start,
                p.end,
            ),
            GraphCommands::Explain(e) => graph::run_graph_explain(
                e.path.unwrap_or_else(|| PathBuf::from(".")),
                e.term,
            ),
            GraphCommands::Mcp(m) => graph::run_graph_mcp(
                m.path.unwrap_or_else(|| PathBuf::from(".")),
            ),
        },

        Commands::Worktree(sub_args) => match sub_args.sub {
            WorktreeCommands::List(l) => {
                worktree::run_worktree_list(l.path.unwrap_or_else(|| PathBuf::from(".")))
            }
            WorktreeCommands::Add(a) => {
                worktree::run_worktree_add(a.repo, a.dest, a.branch, a.new)
            }
            WorktreeCommands::Remove(r) => {
                worktree::run_worktree_remove(r.repo, r.dest, r.force)
            }
            WorktreeCommands::Prune(p) => worktree::run_worktree_prune(p.repo),
        },

        Commands::Fork(sub_args) => match sub_args.sub {
            ForkCommands::Status(s) => {
                fork::run_fork_status(s.path.unwrap_or_else(|| PathBuf::from(".")))
            }
            ForkCommands::SetUpstream(u) => {
                fork::run_fork_set_upstream(u.path.unwrap_or_else(|| PathBuf::from(".")), u.url)
            }
            ForkCommands::Sync(s) => fork::run_fork_sync(
                s.path.unwrap_or_else(|| PathBuf::from(".")),
                s.branch,
                s.strategy,
            ),
        },

        Commands::Slim(s) => slim::run_slim(s.path.unwrap_or_else(|| PathBuf::from("."))),

        Commands::Clone(c) => slim::run_clone(c.url, c.dest, c.branch, c.depth, c.sparse),

        Commands::Artifacts(sub_args) => match sub_args.sub {
            ArtifactCommands::Verify(v) => artifacts::run_artifacts_verify(
                v.manifest
                    .unwrap_or_else(|| PathBuf::from("depot-artifacts.toml")),
            ),
            ArtifactCommands::Hash(h) => artifacts::run_artifacts_hash(h.file),
            ArtifactCommands::Init(i) => artifacts::run_artifacts_init(
                i.manifest
                    .unwrap_or_else(|| PathBuf::from("depot-artifacts.toml")),
                i.name,
            ),
        },

        Commands::Manifests(m) => manifests::run_manifests_verify(
            &m.root.unwrap_or_else(|| PathBuf::from(r"C:\src")),
        ),

        Commands::Maintainer(m) => manifests::run_maintainer_lookup(&m.root, &m.target),

        Commands::Ignore(i) => ignore::run_ignore_init(i.output, i.template),

        Commands::Info(_) => info::run_info(),
    };

    if let Err(err) = result {
        eprintln!("{} {}", "[ERROR]".red().bold(), err);
        std::process::exit(1);
    }
}
