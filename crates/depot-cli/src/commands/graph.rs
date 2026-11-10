use std::path::PathBuf;
use colored::Colorize;
use depot_graph::{build_graph_for_dir, GraphExporter, GraphQuery};

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
        "{} Graph artifacts generated at {}:",
        "[OK]".green().bold(),
        out_dir.display().to_string().cyan()
    );
    println!("  - {}", out_dir.join("graph.json").display());
    println!("  - {}", out_dir.join("GRAPH_REPORT.md").display());
    println!("  - {}", out_dir.join("ARCHITECTURE.mermaid").display());

    Ok(())
}

pub fn run_graph_query(root_dir: PathBuf, term: String) -> Result<(), String> {
    let out_json = root_dir.join("graphify-out").join("graph.json");
    let graph = if out_json.exists() {
        let content = std::fs::read_to_string(&out_json).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())?
    } else {
        println!(
            "{} No graph.json found, building on the fly...",
            "->".yellow()
        );
        build_graph_for_dir(&root_dir, &[])
    };

    let results = GraphQuery::query_symbol(&graph, &term);

    if results.is_empty() {
        println!(
            "{} No symbols matching '{}' found in graph.",
            "[INFO]".yellow(),
            term
        );
    } else {
        println!(
            "{} Found {} matches for '{}':\n",
            "[MATCHES]".green().bold(),
            results.len(),
            term.cyan().bold()
        );
        for res in results {
            println!("{}\n", res);
        }
    }

    Ok(())
}
