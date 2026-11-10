use std::path::PathBuf;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod commands;
use commands::*;

#[derive(Parser)]
#[command(name = "depot")]
#[command(about = "Unified LLM-optimized monorepo manager fusing Google depot_tools, gitingest, graphify, and Git Scalar")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    passthrough: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Ingest a codebase and output an LLM-ready prompt digest
    Ingest {
        /// Target directory to ingest (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Format: markdown, xml, json
        #[arg(short, long, default_value = "markdown")]
        format: String,

        /// Max token budget (clamps and prioritizes important files)
        #[arg(short, long)]
        max_tokens: Option<usize>,

        /// Output file path (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include glob patterns
        #[arg(short, long)]
        include: Vec<String>,

        /// Exclude glob patterns
        #[arg(short, long)]
        exclude: Vec<String>,
    },

    /// AST / dependency graph generator and query tool for LLMs
    Graph {
        #[command(subcommand)]
        sub: GraphCommands,
    },

    /// Windows-safe worktree management without duplicating repo storage
    #[command(alias = "wt")]
    Worktree {
        #[command(subcommand)]
        sub: WorktreeCommands,
    },

    /// Fork and upstream tracking, divergence check, and fast-forward/rebase sync
    Fork {
        #[command(subcommand)]
        sub: ForkCommands,
    },

    /// Apply Git Scalar, commit-graph, multi-pack-index, and fsmonitor to minimize git weight
    Slim {
        /// Target repository path (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Minimal blobless clone with automated Git Scalar configuration
    Clone {
        /// Git repository URL
        url: String,

        /// Destination path
        dest: PathBuf,

        /// Specific branch
        #[arg(short, long)]
        branch: Option<String>,

        /// Shallow clone depth
        #[arg(short, long)]
        depth: Option<u32>,

        /// Sparse paths to include
        #[arg(long)]
        sparse: Vec<String>,
    },

    /// Unreal Engine GitDependencies + CIPD Blake3 content-addressed binary artifacts
    Artifacts {
        #[command(subcommand)]
        sub: ArtifactCommands,
    },

    /// Standard .gitignore and .llmignore templates
    Ignore {
        /// Template: rust, bun, chromium, python, llm
        #[arg(short, long, default_value = "llm")]
        template: String,

        /// Output path
        #[arg(short, long, default_value = ".gitignore")]
        output: PathBuf,
    },

    /// Display environment, depot_tools status, and feature flags
    Info,
}

#[derive(Subcommand)]
enum GraphCommands {
    /// Build graph.json, GRAPH_REPORT.md, and ARCHITECTURE.mermaid
    Build {
        #[arg(default_value = ".")]
        path: PathBuf,

        #[arg(short, long, default_value = "graphify-out")]
        out_dir: PathBuf,
    },
    /// Query a symbol or connection in the graph
    Query {
        term: String,

        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum WorktreeCommands {
    /// List all active worktrees
    List {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Add a new worktree
    Add {
        dest: PathBuf,
        branch: String,
        #[arg(short, long)]
        new: bool,
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// Remove an existing worktree
    Remove {
        dest: PathBuf,
        #[arg(short, long)]
        force: bool,
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
    /// Prune stale worktree links
    Prune {
        #[arg(default_value = ".")]
        repo: PathBuf,
    },
}

#[derive(Subcommand)]
enum ForkCommands {
    /// Inspect fork divergence from upstream
    Status {
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Set upstream repository URL
    SetUpstream {
        url: String,
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Sync current branch with upstream
    Sync {
        #[arg(short, long)]
        branch: Option<String>,
        #[arg(short, long, default_value = "ff")]
        strategy: String,
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum ArtifactCommands {
    /// Verify all artifacts against Blake3 hashes in manifest
    Verify {
        #[arg(default_value = "depot-artifacts.toml")]
        manifest: PathBuf,
    },
    /// Compute Blake3 hash and size for a file
    Hash {
        file: PathBuf,
    },
    /// Initialize a new depot-artifacts.toml manifest
    Init {
        #[arg(default_value = "depot-artifacts.toml")]
        manifest: PathBuf,
        #[arg(short, long, default_value = "project")]
        name: String,
    },
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Check if the first argument is a depot_tools tool pass-through
    if args.len() > 1 {
        let first = &args[1];
        if matches!(
            first.as_str(),
            "gclient" | "cipd" | "fetch" | "autoninja" | "siso" | "gn" | "ninja"
        ) {
            let passthrough_args = &args[2..];
            match depot_core::DepotTools::run_passthrough(first, passthrough_args) {
                Ok(code) => std::process::exit(code),
                Err(err) => {
                    eprintln!("{} {}", "[ERROR]".red().bold(), err);
                    std::process::exit(1);
                }
            }
        }
    }

    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Ingest {
            path,
            format,
            max_tokens,
            output,
            include,
            exclude,
        }) => ingest::run_ingest(path, format, max_tokens, output, include, exclude),

        Some(Commands::Graph { sub }) => match sub {
            GraphCommands::Build { path, out_dir } => graph::run_graph_build(path, out_dir),
            GraphCommands::Query { term, path } => graph::run_graph_query(path, term),
        },

        Some(Commands::Worktree { sub }) => match sub {
            WorktreeCommands::List { path } => worktree::run_worktree_list(path),
            WorktreeCommands::Add {
                dest,
                branch,
                new,
                repo,
            } => worktree::run_worktree_add(repo, dest, branch, new),
            WorktreeCommands::Remove { dest, force, repo } => {
                worktree::run_worktree_remove(repo, dest, force)
            }
            WorktreeCommands::Prune { repo } => worktree::run_worktree_prune(repo),
        },

        Some(Commands::Fork { sub }) => match sub {
            ForkCommands::Status { path } => fork::run_fork_status(path),
            ForkCommands::SetUpstream { url, path } => fork::run_fork_set_upstream(path, url),
            ForkCommands::Sync {
                branch,
                strategy,
                path,
            } => fork::run_fork_sync(path, branch, strategy),
        },

        Some(Commands::Slim { path }) => slim::run_slim(path),

        Some(Commands::Clone {
            url,
            dest,
            branch,
            depth,
            sparse,
        }) => slim::run_clone(url, dest, branch, depth, sparse),

        Some(Commands::Artifacts { sub }) => match sub {
            ArtifactCommands::Verify { manifest } => artifacts::run_artifacts_verify(manifest),
            ArtifactCommands::Hash { file } => artifacts::run_artifacts_hash(file),
            ArtifactCommands::Init { manifest, name } => artifacts::run_artifacts_init(manifest, name),
        },

        Some(Commands::Ignore { template, output }) => ignore::run_ignore_init(output, template),

        Some(Commands::Info) => info::run_info(),

        None => {
            println!("{}", "depot - Run `depot --help` for available commands.".cyan());
            Ok(())
        }
    };

    if let Err(err) = result {
        eprintln!("{} {}", "[ERROR]".red().bold(), err);
        std::process::exit(1);
    }
}
