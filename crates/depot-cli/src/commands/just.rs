use std::path::PathBuf;
use std::process::Command;
use depot_core::{JustEngine, StyleExt};

pub fn run_just_list(dir: PathBuf, json_output: bool) -> Result<(), String> {
    let p = if dir.is_file() {
        dir
    } else {
        JustEngine::find_justfile(&dir)
            .ok_or_else(|| format!("No justfile found in '{}'", dir.display()))?
    };
    let spec = JustEngine::parse_file(&p)?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&spec).unwrap());
        return Ok(());
    }

    println!(
        "{} Justfile recipes in {}:\n",
        "[JUST]".cyan().bold(),
        p.display().to_string().yellow()
    );

    for r in &spec.recipes {
        let params_str = if r.params.is_empty() {
            String::new()
        } else {
            let names: Vec<String> = r.params.iter().map(|p| p.name.clone()).collect();
            format!(" ({})", names.join(", "))
        };

        let deps_str = if r.dependencies.is_empty() {
            String::new()
        } else {
            format!(" -> [{}]", r.dependencies.join(", "))
        };

        let doc_str = r.doc.as_deref().unwrap_or("");

        println!(
            "  - {}{}{} [L{}]",
            r.name.green().bold(),
            params_str.cyan(),
            deps_str.magenta(),
            r.line_number
        );
        if !doc_str.is_empty() {
            println!("      {}", doc_str);
        }
    }

    Ok(())
}

#[allow(dead_code)]
pub fn run_just_exec(dir: PathBuf, recipe_args: Vec<String>) -> Result<(), String> {
    println!(
        "{} Executing just recipe: {}...",
        "->".cyan().bold(),
        recipe_args.join(" ").yellow()
    );

    let mut cmd = Command::new("just");
    cmd.current_dir(&dir);
    cmd.args(&recipe_args);

    let status = cmd
        .status()
        .map_err(|e| format!("Failed to execute 'just' command: {}", e))?;

    if !status.success() {
        return Err(format!("Recipe exited with status: {}", status));
    }

    Ok(())
}
