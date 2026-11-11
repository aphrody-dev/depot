use depot_core::StyleExt;
use depot_forge::{GitHubClient, GitLabClient};

pub fn run_forge_repo(forge: String, target: String, json_output: bool) -> Result<(), String> {
    if forge.eq_ignore_ascii_case("github") {
        let parts: Vec<&str> = target.split('/').collect();
        if parts.len() != 2 {
            return Err("GitHub repository must be in format 'owner/repo'".to_string());
        }
        let (owner, repo_name) = (parts[0], parts[1]);
        let client = GitHubClient::from_vault();
        let r = client.get_repo(owner, repo_name).map_err(|e| e.to_string())?;

        if json_output {
            println!("{}", serde_json::to_string_pretty(&r).unwrap());
            return Ok(());
        }

        println!("{} GitHub Repository Details:\n", "[FORGE]".cyan().bold());
        println!("  - Name:             {}", r.full_name.yellow().bold());
        println!("  - Description:      {}", r.description.as_deref().unwrap_or(""));
        println!("  - Default Branch:   {}", r.default_branch.green());
        println!("  - Stars:            {}", r.stargazers_count);
        println!("  - Open Issues:      {}", r.open_issues_count);
        println!("  - URL:              {}", r.html_url);
    } else {
        let client = GitLabClient::from_vault(None);
        let p = client.get_project(&target).map_err(|e| e.to_string())?;

        if json_output {
            println!("{}", serde_json::to_string_pretty(&p).unwrap());
            return Ok(());
        }

        println!("{} GitLab Project Details:\n", "[FORGE]".cyan().bold());
        println!("  - Name:             {}", p.path_with_namespace.yellow().bold());
        println!("  - Default Branch:   {}", p.default_branch.green());
        println!("  - URL:              {}", p.web_url);
    }
    Ok(())
}

pub fn run_forge_org_repos(
    forge: String,
    org: String,
    limit: usize,
    json_output: bool,
) -> Result<(), String> {
    if forge.eq_ignore_ascii_case("github") {
        let client = GitHubClient::from_vault();
        let repos = client.list_org_repos(&org, Some(limit)).map_err(|e| e.to_string())?;

        if json_output {
            println!("{}", serde_json::to_string_pretty(&repos).unwrap());
            return Ok(());
        }

        println!(
            "{} Found {} repositories for org '{}':\n",
            "[FORGE]".cyan().bold(),
            repos.len(),
            org.yellow().bold()
        );

        for (i, r) in repos.iter().enumerate() {
            let stars_str = format!("* {}", r.stargazers_count);
            let lang = r.language.as_deref().unwrap_or("Unknown");
            println!(
                "  {:>3}. {:<32} {:<10} {:<12} {}",
                i + 1,
                r.name.cyan(),
                stars_str.yellow(),
                lang.magenta(),
                r.html_url
            );
        }
    } else {
        let client = GitLabClient::from_vault(None);
        let projects = client.list_group_projects(&org).map_err(|e| e.to_string())?;

        if json_output {
            println!("{}", serde_json::to_string_pretty(&projects).unwrap());
            return Ok(());
        }

        println!(
            "{} Found {} projects for group '{}':\n",
            "[FORGE]".cyan().bold(),
            projects.len(),
            org.yellow().bold()
        );

        for (i, p) in projects.iter().enumerate().take(limit) {
            println!("  {:>3}. {:<35} {}", i + 1, p.path_with_namespace.cyan(), p.web_url);
        }
    }

    Ok(())
}
