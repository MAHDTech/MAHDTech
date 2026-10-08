use clap::Parser;
use std::fs;
use std::path::PathBuf;
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
    #[arg(long, env = "GEMINI_MODEL", default_value = "gemini-2.5-flash")]
    pub model: String,

    /// Enable verbose logging to stderr
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
}

#[tokio::main]
async fn main() {
    let args = CliArgs::parse();

    if args.verbose {
        eprintln!("[slop] Active model: {}", args.model);
        eprintln!("[slop] Offline mode: {}", args.offline);
        eprintln!("[slop] Template path: {}", args.template.display());
        eprintln!("[slop] Output destination: {}", args.output.display());
    }

    let default_template = PathBuf::from("README.template.md");
    let sample_template = PathBuf::from("tests/fixtures/sample_template.md");

    let template_path =
        if !args.template.exists() && args.template == default_template && sample_template.exists()
        {
            sample_template
        } else {
            args.template.clone()
        };

    if !template_path.exists() {
        eprintln!(
            "Error: Template file not found: {}",
            args.template.display()
        );
        process::exit(1);
    }

    let template_content = match fs::read_to_string(&template_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading template {}: {}", template_path.display(), e);
            process::exit(1);
        }
    };

    let is_offline = args.offline;

    let wakatime_key = if is_offline {
        None
    } else {
        std::env::var("WAKATIME_API_KEY")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };

    let gemini_key = if is_offline {
        None
    } else {
        std::env::var("GEMINI_API_KEY")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };

    let github_token = if is_offline {
        None
    } else {
        std::env::var("MAHDTECH_GITHUB_TOKEN")
            .or_else(|_| std::env::var("GITHUB_TOKEN"))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };

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

    let rendered = match template::render_profile(&template_content, &context) {
        Ok(rendered) => rendered,
        Err(e) => {
            eprintln!("Template rendering error: {}", e);
            process::exit(1);
        }
    };

    if rendered.contains('\u{2014}') {
        eprintln!("Fatal: Rogue em-dash detected in output stream");
        process::exit(1);
    }

    if args.dry_run {
        print!("{}", rendered);
        process::exit(0);
    }

    if let Err(e) = template::write_atomic(&args.output, &rendered) {
        eprintln!("Error writing output to {}: {}", args.output.display(), e);
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
        assert_eq!(args.model, "gemini-2.5-flash");
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
}
