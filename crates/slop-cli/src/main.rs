use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;

mod gemini;
mod github;
mod template;
mod wakatime;

#[derive(Parser, Debug)]
#[command(
    name = "slop",
    author = "MAHDTech",
    version,
    about = "Automated high-impact GitHub profile compiler"
)]
pub struct CliArgs {
    /// Preview rendered output to stdout without modifying target files
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,

    /// Path to the source profile template file
    #[arg(short, long, default_value = "README.template.md")]
    pub template: PathBuf,

    /// Destination path for the compiled output file
    #[arg(short, long, default_value = "README.md")]
    pub output: PathBuf,

    /// Bypass external network calls and force deterministic offline mock data
    #[arg(long, default_value_t = false)]
    pub offline: bool,

    /// Override the Gemini model name
    #[arg(long, env = "GEMINI_MODEL", default_value = "gemini-3.8-flash")]
    pub model: String,

    /// Enable verbose logging to stderr
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

/// Resolves the template path, falling back to sample fixture if available.
pub fn resolve_template_path(template: &Path) -> Result<PathBuf, String> {
    let default_template = Path::new("README.template.md");
    let sample_template = Path::new("tests/fixtures/sample_template.md");

    let path = if !template.exists() && template == default_template && sample_template.exists() {
        sample_template.to_path_buf()
    } else {
        template.to_path_buf()
    };

    if !path.exists() {
        Err(format!("Template file not found: {}", template.display()))
    } else {
        Ok(path)
    }
}

/// Resolves an optional environment variable token, returning None in offline mode.
pub fn resolve_token(key_name: &str, offline: bool) -> Option<String> {
    if offline {
        return None;
    }
    std::env::var(key_name)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Resolves the GitHub token from primary or secondary environment variables.
pub fn resolve_github_token(offline: bool) -> Option<String> {
    if offline {
        return None;
    }
    std::env::var("MAHDTECH_GITHUB_TOKEN")
        .or_else(|_| std::env::var("GITHUB_TOKEN"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Verifies that the rendered markdown output is free of rogue em-dashes.
pub fn check_rogue_em_dash(content: &str) -> Result<(), &'static str> {
    if content.contains('\u{2014}') {
        Err("Fatal: Rogue em-dash detected in output stream")
    } else {
        Ok(())
    }
}

/// Runs the profile compilation pipeline and returns the rendered markdown.
pub async fn run_pipeline(args: &CliArgs) -> Result<String, Box<dyn std::error::Error>> {
    let template_path = resolve_template_path(&args.template).map_err(|e| e)?;
    let template_content = fs::read_to_string(&template_path)?;

    let is_offline = args.offline;
    let wakatime_key = resolve_token("WAKATIME_API_KEY", is_offline);
    let gemini_key = resolve_token("GEMINI_API_KEY", is_offline);
    let github_token = resolve_github_token(is_offline);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("slop-cli/0.1.0")
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());

    let (wakatime_stats, github_activity) = tokio::join!(
        wakatime::fetch_stats(&client, wakatime_key.as_deref(), is_offline),
        github::fetch_recent_activity(&client, github_token.as_deref(), is_offline),
    );

    let ai_standup = gemini::generate_standup(
        &client,
        gemini_key.as_deref(),
        &args.model,
        &wakatime_stats,
        &github_activity,
        is_offline,
    )
    .await;

    let wakatime_hud = wakatime::render_hud(&wakatime_stats);
    let now_utc = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();
    let cli_version = env!("CARGO_PKG_VERSION");

    let context = template::create_context(&wakatime_hud, &ai_standup, &now_utc, cli_version);
    let rendered = template::render_profile(&template_content, &context)?;

    check_rogue_em_dash(&rendered).map_err(|e| e)?;

    Ok(rendered)
}

/// Orchestrates execution based on CLI arguments.
pub async fn execute(args: &CliArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.verbose {
        eprintln!("[slop] Active model: {}", args.model);
        eprintln!("[slop] Offline mode: {}", args.offline);
        eprintln!("[slop] Template path: {}", args.template.display());
        eprintln!("[slop] Output destination: {}", args.output.display());
    }

    let rendered = run_pipeline(args).await?;

    if args.dry_run {
        print!("{}", rendered);
        return Ok(());
    }

    template::write_atomic(&args.output, &rendered)?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();
    if let Err(e) = execute(&args).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args_defaults() {
        let args = CliArgs::try_parse_from(["slop"]).expect("Default args should parse");
        assert!(!args.dry_run);
        assert!(!args.offline);
        assert!(!args.verbose);
        assert_eq!(args.template, PathBuf::from("README.template.md"));
        assert_eq!(args.output, PathBuf::from("README.md"));
        assert_eq!(args.model, "gemini-3.8-flash");
    }

    #[test]
    fn test_cli_args_flags() {
        let args = CliArgs::try_parse_from([
            "slop",
            "--dry-run",
            "--offline",
            "--verbose",
            "-t",
            "custom_tmpl.md",
            "-o",
            "custom_out.md",
            "--model",
            "gemini-test",
        ])
        .expect("Flags should parse correctly");

        assert!(args.dry_run);
        assert!(args.offline);
        assert!(args.verbose);
        assert_eq!(args.template, PathBuf::from("custom_tmpl.md"));
        assert_eq!(args.output, PathBuf::from("custom_out.md"));
        assert_eq!(args.model, "gemini-test");
    }

    #[test]
    fn test_cli_args_missing_flag_value() {
        let res = CliArgs::try_parse_from(["slop", "--template"]);
        assert!(res.is_err(), "Missing flag value must error");
    }

    #[test]
    fn test_cli_args_unrecognized_flag() {
        let res = CliArgs::try_parse_from(["slop", "--non-existent-flag"]);
        assert!(res.is_err(), "Unrecognized flag must error");
    }

    #[test]
    fn test_resolve_template_path() {
        let missing = Path::new("/non/existent/path/template.md");
        assert!(resolve_template_path(missing).is_err());

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let resolved = resolve_template_path(temp_file.path());
        assert!(resolved.is_ok());
    }

    #[test]
    fn test_resolve_tokens_offline() {
        assert!(resolve_token("WAKATIME_API_KEY", true).is_none());
        assert!(resolve_token("NON_EXISTENT_VAR_XYZ", false).is_none());
        assert!(resolve_github_token(true).is_none());
    }

    #[test]
    fn test_check_rogue_em_dash() {
        assert!(check_rogue_em_dash("Clean output - valid hyphen").is_ok());
        assert!(check_rogue_em_dash("Output with \u{2014} rogue dash").is_err());
    }

    #[tokio::test]
    async fn test_execute_dry_run_offline() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), "Hello {{ cli_version }}").unwrap();

        let args = CliArgs {
            dry_run: true,
            template: temp_file.path().to_path_buf(),
            output: PathBuf::from("README.md"),
            offline: true,
            model: "gemini-3.8-flash".to_string(),
            verbose: true,
        };

        let result = execute(&args).await;
        assert!(result.is_ok());
    }
}
