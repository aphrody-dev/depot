use std::path::PathBuf;
use depot_core::StyleExt;
use depot_graph::{build_graph_for_dir, GraphExporter, GraphQuery, McpServer, QueryOptions};

pub fn run_graph_build(root_dir: PathBuf, out_dir: PathBuf) -> Result<(), String> {
    println!(
        "{} Building AST / dependency graph for {}...",
        "->".cyan().bold(),
        root_dir.display().to_string().yellow()
    );

    let codebase_name = root_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "codebase".to_string());

    let mut graph = build_graph_for_dir(&root_dir, &[]);
    GraphExporter::export_all(&mut graph, &out_dir, &codebase_name)?;

    println!(
        "{} Complete Graphify artifacts generated at {}:",
        "[OK]".green().bold(),
        out_dir.display().to_string().cyan()
    );
    println!("  - {}", out_dir.join("graph.json").display());
    println!("  - {}", out_dir.join("GRAPH_REPORT.md").display());
    println!("  - {}", out_dir.join("ARCHITECTURE.mermaid").display());
    println!("  - {}", out_dir.join("graph.html").display());
    println!("  - {}", out_dir.join("graph.svg").display());
    println!("  - {}", out_dir.join("graph.graphml").display());
    println!("  - {}", out_dir.join("cypher.txt").display());

    Ok(())
}

fn load_or_build_graph(root_dir: &std::path::Path) -> Result<depot_graph::CodeGraph, String> {
    let out_json = root_dir.join("graphify-out").join("graph.json");
    if out_json.exists() {
        let content = std::fs::read_to_string(&out_json).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    } else {
        Ok(build_graph_for_dir(root_dir, &[]))
    }
}

pub fn run_graph_query_advanced(
    root_dir: PathBuf,
    term: String,
    opts: QueryOptions,
) -> Result<(), String> {
    let graph = load_or_build_graph(&root_dir)?;
    let result = GraphQuery::query(&graph, &term, &opts);
    println!("{}", result);
    Ok(())
}

#[allow(dead_code)]
pub fn run_graph_query(
    root_dir: PathBuf,
    term: String,
    dfs: bool,
    budget: Option<usize>,
) -> Result<(), String> {
    let opts = QueryOptions {
        dfs,
        max_tokens: budget.unwrap_or(2000),
        max_depth: 3,
    };
    run_graph_query_advanced(root_dir, term, opts)
}

pub fn run_graph_path(root_dir: PathBuf, start: String, end: String) -> Result<(), String> {
    let graph = load_or_build_graph(&root_dir)?;
    let result = GraphQuery::path(&graph, &start, &end)?;
    println!("{}", result);
    Ok(())
}

pub fn run_graph_explain(root_dir: PathBuf, term: String) -> Result<(), String> {
    let graph = load_or_build_graph(&root_dir)?;
    let result = GraphQuery::explain(&graph, &term)?;
    println!("{}", result);
    Ok(())
}

pub fn run_graph_mcp(root_dir: PathBuf) -> Result<(), String> {
    let graph = load_or_build_graph(&root_dir)?;
    McpServer::run_stdio(graph).map_err(|e| e.to_string())
}
