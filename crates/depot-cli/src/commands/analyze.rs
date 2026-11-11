use std::path::PathBuf;
use depot_core::{GixEngine, JustEngine, StyleExt};
use depot_graph::{build_graph_for_dir, GraphExporter};
use depot_ingest::{ingest_codebase, DigestFormat, IngestConfig};

pub fn run_analyze(path: Option<PathBuf>) -> Result<(), String> {
    let root = path.unwrap_or_else(|| PathBuf::from("."));

    println!(
        "{} Performing unified analysis of {}...\n",
        "[ANALYZE]".cyan().bold(),
        root.display().to_string().yellow()
    );

    // 1. Gix Telemetry
    println!("=== 1. Git Repository Telemetry ===");
    match GixEngine::inspect(&root, 5) {
        Ok(t) => {
            println!("  Branch:       {}", t.branch.green().bold());
            if let Some(ref c) = t.head_commit {
                let subj = t.head_subject.as_deref().unwrap_or("");
                println!("  HEAD:         {} ({})", &c[..c.len().min(8)].cyan(), subj);
            }
            println!("  Entries:      {} tracked", t.tracked_entry_count);
            println!("  Jujutsu:      {}", if t.has_jujutsu { "active".green() } else { "none".into() });
        }
        Err(_) => println!("  (Not a git repository or head unborn)"),
    }

    // 2. Justfile
    println!("\n=== 2. Justfile Automation ===");
    let just_p = root.join("justfile");
    if just_p.exists() {
        if let Ok(spec) = JustEngine::parse_file(&just_p) {
            println!("  Found {} recipes:", spec.recipes.len());
            for r in spec.recipes.iter().take(6) {
                let deps = if r.dependencies.is_empty() { String::new() } else { format!(" -> [{}]", r.dependencies.join(", ")) };
                println!("    - {}{}", r.name.green(), deps.magenta());
            }
        }
    } else {
        println!("  No justfile present");
    }

    // 3. Tree-sitter CodeGraph
    println!("\n=== 3. AST Knowledge Graph ===");
    let mut graph = build_graph_for_dir(&root, &[]);
    let god_nodes = graph.compute_degrees_and_god_nodes(8);
    let comms = graph.detect_communities();

    println!("  Total Nodes:       {}", graph.nodes.len().to_string().cyan());
    println!("  Total Edges:       {}", graph.edges.len().to_string().cyan());
    println!("  Communities:       {}", comms);

    println!("\n  Core God Nodes:");
    for (i, (id, deg)) in god_nodes.iter().enumerate() {
        let node = graph.nodes.iter().find(|n| &n.id == id);
        let label = node.map(|n| n.label.as_str()).unwrap_or(id.as_str());
        let kind = node.map(|n| n.file_type.as_str()).unwrap_or("sym");
        println!("    {}. `{}` [{}] - {} connections", i + 1, label.yellow().bold(), kind, deg);
    }

    // Export graph artifacts to graphify-out
    let out_dir = root.join("graphify-out");
    let codebase_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "codebase".to_string());
    let _ = GraphExporter::export_all(&mut graph, &out_dir, &codebase_name);
    println!("  Artifacts written to: {}", out_dir.display().to_string().cyan());

    // 4. Ingest LLM sizing
    println!("\n=== 4. LLM Digest Estimation ===");
    let config = IngestConfig {
        max_file_size_bytes: 512 * 1024,
        format: DigestFormat::Markdown,
        ..Default::default()
    };
    let (_, stats) = ingest_codebase(&root, &config);
    println!("  Files:             {}", stats.total_files);
    println!("  Lines:             {}", stats.total_lines);
    println!("  Size:              {:.2} KB", stats.total_bytes as f64 / 1024.0);
    println!("  Estimated Tokens:  ~{}", stats.estimated_tokens.to_string().green().bold());

    println!(
        "\n{} Codebase analysis complete. Ready for autonomous LLM pairing.",
        "[COMPLETE]".green().bold()
    );

    Ok(())
}
