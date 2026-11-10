use std::path::PathBuf;
use argh::FromArgs;
use depot_core::StyleExt;

mod commands;
use commands::*;

#[derive(FromArgs, PartialEq, Debug)]
/// Unified LLM-optimized monorepo manager fusing Google depot_tools, gitingest, graphify, and Git Scalar
struct Cli {
    #[argh(subcommand)]
    command: Commands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
enum Commands {
    Ingest(IngestArgs),
    Graph(GraphSubArgs),
    Worktree(WorktreeSubArgs),
    Fork(ForkSubArgs),
    Slim(SlimArgs),
    Clone(CloneArgs),
    Artifacts(ArtifactSubArgs),
    Ignore(IgnoreArgs),
    Info(InfoArgs),
}

// Ingest
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "ingest")]
/// Ingest a codebase and output an LLM-ready prompt digest
pub struct IngestArgs {
    /// target directory to ingest (defaults to ".")
    #[argh(positional)]
    pub path: Option<PathBuf>,

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
}

// Graph
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "graph")]
/// AST / dependency graph generator and query tool for LLMs
pub struct GraphSubArgs {
    #[argh(subcommand)]
    pub sub: GraphCommands,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum GraphCommands {
    Build(GraphBuildArgs),
    Query(GraphQueryArgs),
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "build")]
/// Build graph.json, GRAPH_REPORT.md, and ARCHITECTURE.mermaid
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
/// Query a symbol or connection in the graph
pub struct GraphQueryArgs {
    /// symbol term to query
    #[argh(positional)]
    pub term: String,

    /// target directory containing graph (default: ".")
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

fn main() {
    depot_core::enable_vt_processing();

    let mut raw_args: Vec<String> = std::env::args().collect();

    // Check pass-through for depot_tools
    if raw_args.len() > 1 {
        let first = &raw_args[1];
        if matches!(
            first.as_str(),
            "gclient" | "cipd" | "fetch" | "autoninja" | "siso" | "gn" | "ninja"
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
        println!("  ingest      Ingest a codebase into LLM-ready prompt digest");
        println!("  graph       AST / dependency graph generator and query tool");
        println!("  worktree    Windows-safe git worktree management (alias: wt)");
        println!("  fork        Fork and upstream tracking & synchronization");
        println!("  slim        Apply Git Scalar and maintenance optimizations");
        println!("  clone       Minimal blobless clone with Git Scalar");
        println!("  artifacts   Blake3 content-addressed binary artifact management");
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
        Commands::Ingest(args) => ingest::run_ingest(
            args.path.unwrap_or_else(|| PathBuf::from(".")),
            args.format,
            args.max_tokens,
            args.output,
            args.include,
            args.exclude,
        ),

        Commands::Graph(sub_args) => match sub_args.sub {
            GraphCommands::Build(b) => {
                graph::run_graph_build(b.path.unwrap_or_else(|| PathBuf::from(".")), b.out_dir)
            }
            GraphCommands::Query(q) => {
                graph::run_graph_query(q.path.unwrap_or_else(|| PathBuf::from(".")), q.term)
            }
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

        Commands::Slim(s) => {
            slim::run_slim(s.path.unwrap_or_else(|| PathBuf::from(".")))
        }

        Commands::Clone(c) => {
            slim::run_clone(c.url, c.dest, c.branch, c.depth, c.sparse)
        }

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

        Commands::Ignore(i) => ignore::run_ignore_init(i.output, i.template),

        Commands::Info(_) => info::run_info(),
    };

    if let Err(err) = result {
        eprintln!("{} {}", "[ERROR]".red().bold(), err);
        std::process::exit(1);
    }
}
