use std::path::PathBuf;
use argh::FromArgs;
use depot_core::StyleExt;
use depot_graph::{build_graph_for_dir, GraphExporter, GraphQuery, QueryOptions};

#[derive(FromArgs)]
/// Ultra-fast native Rust AST / dependency knowledge graph generator for LLMs (graphify compatible)
struct GraphCli {
    /// codebase directory (defaults to ".")
    #[argh(positional)]
    path: Option<PathBuf>,

    /// output directory for graph artifacts (default: graphify-out)
    #[argh(option, short = 'o', default = "PathBuf::from(\"graphify-out\")")]
    out_dir: PathBuf,

    /// query symbol instead of generating full graph
    #[argh(option, short = 'q')]
    query: Option<String>,
}

fn main() {
    depot_core::enable_vt_processing();
    let cli: GraphCli = argh::from_env();
    let target_path = cli.path.unwrap_or_else(|| PathBuf::from("."));

    if let Some(term) = cli.query {
        let out_json = target_path.join("graphify-out").join("graph.json");
        let graph = if out_json.exists() {
            let content = std::fs::read_to_string(&out_json).expect("Failed to read graph.json");
            serde_json::from_str(&content).expect("Failed to parse graph.json")
        } else {
            build_graph_for_dir(&target_path, &[])
        };

        let result = GraphQuery::query(&graph, &term, &QueryOptions::default());
        println!("{}", result);
        return;
    }

    let codebase_name = target_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "codebase".to_string());

    let mut graph = build_graph_for_dir(&target_path, &[]);
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
    println!("  - {}", cli.out_dir.join("graph.html").display());
    println!("  - {}", cli.out_dir.join("graph.svg").display());
    println!("  - {}", cli.out_dir.join("graph.graphml").display());
    println!("  - {}", cli.out_dir.join("cypher.txt").display());
}
