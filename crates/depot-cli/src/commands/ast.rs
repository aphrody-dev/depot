use std::path::PathBuf;
use depot_core::StyleExt;
use depot_treesitter::TreeSitterEngine;

#[allow(dead_code)]
pub fn run_ast(file: PathBuf, query: Option<String>, json: bool) -> Result<(), String> {
    if let Some(q) = query {
        run_ast_query(file, q)
    } else {
        run_ast_parse(file, json)
    }
}

pub fn run_ast_parse(file_path: PathBuf, json_output: bool) -> Result<(), String> {
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

    let summary = TreeSitterEngine::parse_file(&file_path, &content)
        .ok_or_else(|| format!("Unsupported language or parse failed for {}", file_path.display()))?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&summary).unwrap());
        return Ok(());
    }

    println!(
        "{} AST Summary for {}:\n",
        "[AST]".cyan().bold(),
        file_path.display().to_string().yellow()
    );
    println!("  - Language:     {}", summary.language.green());
    println!("  - Lines:        {}", summary.line_count);
    println!("  - Symbols:      {}", summary.symbols.len());
    println!("  - Imports:      {}", summary.imports.len());
    println!("  - Calls:        {}", summary.calls.len());
    println!("  - Parse errors: {}", summary.parse_errors);

    if !summary.imports.is_empty() {
        println!("\n  Imports:");
        for imp in &summary.imports {
            println!("    - {} [{}]", imp.source_module.cyan(), imp.span.source_location());
        }
    }

    if !summary.symbols.is_empty() {
        println!("\n  Symbols ({}):", summary.symbols.len());
        for s in &summary.symbols {
            let sig = s.signature.as_deref().unwrap_or(&s.name);
            println!(
                "    - {:<10} {:<24} [{}] {}",
                s.kind.as_str().magenta(),
                s.name.bold(),
                s.span.source_location().yellow(),
                sig.green()
            );
        }
    }

    if !summary.calls.is_empty() {
        println!("\n  Calls (first 15 of {}):", summary.calls.len());
        for c in summary.calls.iter().take(15) {
            let caller = c.caller_scope.as_deref().unwrap_or("<root>");
            println!("    - in {}() -> {}() [{}]", caller.yellow(), c.callee.cyan(), c.span.source_location());
        }
    }

    Ok(())
}

pub fn run_ast_query(file_path: PathBuf, query_sexpr: String) -> Result<(), String> {
    let content = std::fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

    let lang = TreeSitterEngine::detect_language(&file_path)
        .ok_or_else(|| format!("Cannot detect language for {}", file_path.display()))?;

    let matches = TreeSitterEngine::query_sexpr(lang, &content, &query_sexpr)?;

    println!(
        "{} Executing S-expression query on {}:\n",
        "[QUERY]".cyan().bold(),
        file_path.display().to_string().yellow()
    );

    println!("Query: {}\n", query_sexpr.cyan());
    println!("Matches ({}):", matches.len());

    for (i, m) in matches.iter().enumerate() {
        println!(
            "  {:>3}. @{:<15} [{}] \"{}\"",
            i + 1,
            m.capture_name.green().bold(),
            m.span.source_location().yellow(),
            m.text.trim()
        );
    }

    Ok(())
}
