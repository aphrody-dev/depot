use std::path::PathBuf;
use clap::Parser;
use colored::Colorize;
use depot_graph::{build_graph_for_dir, GraphExporter, GraphQuery};

#[derive(Parser)]
#[command(name = "depot-graph")]
#[command(about = "Ultra-fast native Rust AST / dependency knowledge graph generator for LLMs (graphify compatible)")]
#[command(version = "0.1.0")]
struct GraphCli {
    /// Codebase directory
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output directory for graph artifacts
    #[arg(short, long, default_value = "graphify-out")]
    out_dir: PathBuf,

    /// Query symbol instead of generating full graph
    #[arg(short, long)]
    query: Option<String>,
}

fn main() {
    let cli = GraphCli::parse();

    if let Some(term) = cli.query {
        let out_json = cli.path.join("graphify-out").join("graph.json");
        let graph = if out_json.exists() {
            let content = std::fs::read_to_string(&out_json).expect("Failed to read graph.json");
            serde_json::from_str(&content).expect("Failed to parse graph.json")
        } else {
            build_graph_for_dir(&cli.path, &[])
        };

        let results = GraphQuery::query_symbol(&graph, &term);
        if results.is_empty() {
            println!("{} No symbols matching '{}' found.", "[INFO]".yellow(), term);
        } else {
            println!("{} Found {} matches for '{}':\n", "[MATCHES]".green().bold(), results.len(), term.cyan().bold());
            for r in results {
                println!("{}\n", r);
            }
        }
        return;
    }

    let codebase_name = cli
        .path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "codebase".to_string());

    let mut graph = build_graph_for_dir(&cli.path, &[]);
    if let Err(e) = GraphExporter::export_all(&mut graph, &cli.out_dir, &codebase_name) {
        eprintln!("{} Export failed: {}", "[ERROR]".red().bold(), e);
        std::process::exit(1);
    }

    println!(
        "{} Knowledge graph generated at {}:",
        "[OK]".green().bold(),
        cli.out_dir.display().to_string().cyan()
    );
    println!("  - {}", cli.out_dir.join("graph.json").display());
    println!("  - {}", cli.out_dir.join("GRAPH_REPORT.md").display());
    println!("  - {}", cli.out_dir.join("ARCHITECTURE.mermaid").display());
}
