use base64::prelude::*;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};

/// Heavy horizontal line character used for terminal divider.
const DIVIDER_CHAR: char = '━';
/// Exact width of the HUD horizontal divider in characters.
const DIVIDER_WIDTH: usize = 56;
/// Number of progress blocks in the retro progress bar.
const BAR_WIDTH: usize = 20;

/// Upstream envelope for WakaTime stats endpoint.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WakaTimeResponse {
    pub data: WakaTimeStats,
}

/// Aggregated coding statistics over the queried interval.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WakaTimeStats {
    #[serde(default)]
    pub languages: Vec<WakaLanguage>,
    #[serde(default)]
    pub editors: Vec<WakaEditor>,
    #[serde(default)]
    pub operating_systems: Vec<WakaOs>,
    #[serde(default)]
    pub human_readable_total: Option<String>,
    #[serde(default)]
    pub human_readable_daily_average: Option<String>,
    #[serde(default)]
    pub total_seconds: Option<f64>,
}

/// Language telemetry record.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WakaLanguage {
    pub name: String,
    pub percent: f64,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub total_seconds: f64,
    #[serde(default)]
    pub hours: i64,
    #[serde(default)]
    pub minutes: i64,
}

/// Editor telemetry record.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WakaEditor {
    pub name: String,
    pub percent: f64,
}

/// Operating system telemetry record.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WakaOs {
    pub name: String,
    pub percent: f64,
}

impl WakaTimeStats {
    /// Formats a high-signal summary of active languages for LLM prompt ingestion.
    pub fn format_for_prompt(&self) -> String {
        let langs: Vec<String> = self
            .languages
            .iter()
            .take(4)
            .map(|l| format!("{}: {:.1}% ({})", l.name, l.percent, l.text))
            .collect();
        format!(
            "Total weekly compute: {}. Languages: {}.",
            self.human_readable_total.as_deref().unwrap_or("active"),
            langs.join(", ")
        )
    }
}

/// Provides deterministic fallback statistics when offline or upstream fails.
pub fn mock_stats() -> WakaTimeStats {
    WakaTimeStats {
        languages: vec![
            WakaLanguage {
                name: "Rust".to_string(),
                percent: 58.2,
                text: "28 hrs 40 mins".to_string(),
                total_seconds: 103200.0,
                hours: 28,
                minutes: 40,
            },
            WakaLanguage {
                name: "Python".to_string(),
                percent: 16.7,
                text: "8 hrs 15 mins".to_string(),
                total_seconds: 29700.0,
                hours: 8,
                minutes: 15,
            },
            WakaLanguage {
                name: "YAML / K8s".to_string(),
                percent: 10.5,
                text: "5 hrs 10 mins".to_string(),
                total_seconds: 18600.0,
                hours: 5,
                minutes: 10,
            },
            WakaLanguage {
                name: "Nix".to_string(),
                percent: 7.6,
                text: "3 hrs 45 mins".to_string(),
                total_seconds: 13500.0,
                hours: 3,
                minutes: 45,
            },
        ],
        editors: vec![WakaEditor {
            name: "Neovim / Helix".to_string(),
            percent: 100.0,
        }],
        operating_systems: vec![WakaOs {
            name: "Linux (NixOS)".to_string(),
            percent: 100.0,
        }],
        human_readable_total: Some("45 hrs 50 mins".to_string()),
        human_readable_daily_average: Some("6 hrs 32 mins".to_string()),
        total_seconds: Some(165000.0),
    }
}

/// Renders a single 20-block Unicode progress bar with safety clamping.
pub fn render_progress_bar(percent: f64) -> String {
    let safe_percent = if percent.is_nan() || percent < 0.0 {
        0.0
    } else if percent > 100.0 {
        100.0
    } else {
        percent
    };

    let filled = ((safe_percent / 100.0) * (BAR_WIDTH as f64))
        .round()
        .clamp(0.0, BAR_WIDTH as f64) as usize;
    let unfilled = BAR_WIDTH.saturating_sub(filled);

    format!("[{}{}]", "█".repeat(filled), "░".repeat(unfilled))
}

/// Renders a formatted language row for the retro compute HUD.
fn render_language_row(lang: &WakaLanguage) -> String {
    let name_clean: String = if lang.name.chars().count() > 14 {
        lang.name.chars().take(14).collect()
    } else {
        lang.name.clone()
    };
    let name_col = format!("{:<14}", name_clean);

    let time_str = if lang.hours > 0 || lang.minutes > 0 {
        format!("{:>2} hrs {:>2} mins", lang.hours, lang.minutes)
    } else if !lang.text.is_empty() {
        format!("{:<14}", lang.text)
    } else {
        " 0 hrs  0 mins".to_string()
    };
    let time_col = format!("{:<14}", time_str);

    let bar = render_progress_bar(lang.percent);
    let safe_percent = if lang.percent.is_nan() || lang.percent < 0.0 {
        0.0
    } else if lang.percent > 100.0 {
        100.0
    } else {
        lang.percent
    };
    let pct_str = format!("{:.1}%", safe_percent);
    let pct_col = format!("{:>6}", pct_str);

    format!("{}{}  {}{}", name_col, time_col, bar, pct_col)
}

/// Generates the complete 56-character monospaced ASCII retro compute HUD block.
pub fn render_hud(stats: &WakaTimeStats) -> String {
    let divider = DIVIDER_CHAR.to_string().repeat(DIVIDER_WIDTH);
    let mut lines = Vec::new();

    lines.push("⚡ WEEKLY COMPUTE CYCLES (via WakaTime API)".to_string());
    lines.push(divider.clone());

    let languages = if stats.languages.is_empty() {
        mock_stats().languages
    } else {
        stats.languages.clone()
    };

    for lang in languages.iter().take(4) {
        lines.push(render_language_row(lang));
    }

    lines.push(divider);

    let os_name = stats
        .operating_systems
        .first()
        .map(|o| o.name.as_str())
        .unwrap_or("Linux (NixOS)");
    let editor_name = stats
        .editors
        .first()
        .map(|e| e.name.as_str())
        .unwrap_or("Neovim / Helix");

    let footer = format!("OS: {} | Primary Editor: {}", os_name, editor_name);
    lines.push(footer);

    let output = lines.join("\n");
    // Strictly verify zero em-dashes
    output.replace('\u{2014}', "-")
}

/// Builds HTTP headers including base64 Basic auth for WakaTime API.
pub fn build_auth_headers(api_key: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("slop-cli/0.1.0"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

    let encoded_key = BASE64_STANDARD.encode(api_key.as_bytes());
    let auth_header_val = format!("Basic {}", encoded_key);
    if let Ok(hv) = HeaderValue::from_str(&auth_header_val) {
        headers.insert(AUTHORIZATION, hv);
    }
    headers
}

/// Validates and extracts WakaTime stats from the API response payload.
pub fn extract_stats(parsed: WakaTimeResponse) -> Option<WakaTimeStats> {
    if parsed.data.languages.is_empty() {
        eprintln!("[wakatime] Upstream returned empty languages: falling back to mock stats");
        None
    } else {
        Some(parsed.data)
    }
}

/// Dispatches network request to WakaTime API endpoint and parses response.
async fn fetch_stats_from_network(
    client: &reqwest::Client,
    api_key: &str,
) -> Option<WakaTimeStats> {
    let headers = build_auth_headers(api_key);
    let url = "https://wakatime.com/api/v1/users/current/stats/last_7_days";
    let resp = client
        .get(url)
        .headers(headers)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        eprintln!(
            "[wakatime] Upstream API returned status {}: falling back to mock stats",
            resp.status()
        );
        return None;
    }

    let parsed = resp.json::<WakaTimeResponse>().await.ok()?;
    extract_stats(parsed)
}

/// Fetches weekly stats from the WakaTime API v1 with graceful fallback.
pub async fn fetch_stats(
    client: &reqwest::Client,
    api_key: Option<&str>,
    offline: bool,
) -> WakaTimeStats {
    if offline {
        return mock_stats();
    }

    let trimmed = api_key.map(str::trim).filter(|k| !k.is_empty());
    match trimmed {
        Some(key) => fetch_stats_from_network(client, key)
            .await
            .unwrap_or_else(mock_stats),
        None => mock_stats(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divider_width_is_exactly_56_chars() {
        let divider = DIVIDER_CHAR.to_string().repeat(DIVIDER_WIDTH);
        assert_eq!(divider.chars().count(), 56);
    }

    #[test]
    fn test_progress_bar_length_and_glyphs() {
        let bar = render_progress_bar(50.0);
        assert_eq!(bar.chars().count(), 22); // '[' + 20 blocks + ']'
        assert!(bar.starts_with('['));
        assert!(bar.ends_with(']'));
        assert_eq!(bar.chars().filter(|&c| c == '█').count(), 10);
        assert_eq!(bar.chars().filter(|&c| c == '░').count(), 10);
    }

    #[test]
    fn test_progress_bar_clamping_and_zero_safety() {
        let bar_zero = render_progress_bar(0.0);
        assert_eq!(bar_zero, "[░░░░░░░░░░░░░░░░░░░░]");

        let bar_neg = render_progress_bar(-20.0);
        assert_eq!(bar_neg, "[░░░░░░░░░░░░░░░░░░░░]");

        let bar_full = render_progress_bar(100.0);
        assert_eq!(bar_full, "[████████████████████]");

        let bar_overflow = render_progress_bar(250.0);
        assert_eq!(bar_overflow, "[████████████████████]");

        let bar_nan = render_progress_bar(f64::NAN);
        assert_eq!(bar_nan, "[░░░░░░░░░░░░░░░░░░░░]");
    }

    #[test]
    fn test_hud_zero_em_dash_compliance() {
        let stats = mock_stats();
        let hud = render_hud(&stats);
        assert!(!hud.contains('\u{2014}'));
        assert!(!hud.contains('\u{2013}'));
        assert!(!hud.contains("&mdash;"));
    }

    #[test]
    fn test_hud_structure_and_bounds() {
        let stats = mock_stats();
        let hud = render_hud(&stats);

        let lines: Vec<&str> = hud.lines().collect();
        assert_eq!(lines.len(), 8);
        assert_eq!(lines[0], "⚡ WEEKLY COMPUTE CYCLES (via WakaTime API)");
        assert_eq!(lines[1].chars().count(), 56);
        assert_eq!(lines[6].chars().count(), 56);
        assert_eq!(
            lines[7],
            "OS: Linux (NixOS) | Primary Editor: Neovim / Helix"
        );

        for line in &lines {
            assert!(
                line.chars().count() <= 60,
                "Line exceeded 60 characters: {}",
                line
            );
        }
    }

    #[test]
    fn test_hud_with_zero_stats() {
        let zero_stats = WakaTimeStats {
            languages: vec![WakaLanguage {
                name: "Rust".to_string(),
                percent: 0.0,
                text: "0 secs".to_string(),
                total_seconds: 0.0,
                hours: 0,
                minutes: 0,
            }],
            editors: vec![],
            operating_systems: vec![],
            human_readable_total: Some("0 secs".to_string()),
            human_readable_daily_average: Some("0 secs".to_string()),
            total_seconds: Some(0.0),
        };

        let hud = render_hud(&zero_stats);
        assert!(hud.contains("[░░░░░░░░░░░░░░░░░░░░]  0.0%"));
        assert!(hud.contains("OS: Linux (NixOS) | Primary Editor: Neovim / Helix"));
    }

    #[tokio::test]
    async fn test_offline_mode_returns_mock_stats() {
        let client = reqwest::Client::new();
        let stats = fetch_stats(&client, None, true).await;
        assert_eq!(stats.languages.len(), 4);
        assert_eq!(stats.languages[0].name, "Rust");
    }

    #[test]
    fn test_deserialize_wakatime_json() {
        let raw_json = r#"{
            "data": {
                "languages": [
                    {"name": "Rust", "percent": 75.5, "text": "15 hrs 10 mins", "hours": 15, "minutes": 10}
                ],
                "editors": [{"name": "Neovim", "percent": 100.0}],
                "operating_systems": [{"name": "Linux", "percent": 100.0}],
                "human_readable_total": "15 hrs 10 mins"
            }
        }"#;

        let parsed: Result<WakaTimeResponse, _> = serde_json::from_str(raw_json);
        assert!(parsed.is_ok());
        let stats = parsed.unwrap().data;
        assert_eq!(stats.languages[0].name, "Rust");
        assert_eq!(stats.languages[0].percent, 75.5);
    }

    #[test]
    fn test_format_for_prompt() {
        let stats = mock_stats();
        let prompt_text = stats.format_for_prompt();
        assert!(prompt_text.contains("Total weekly compute:"));
        assert!(prompt_text.contains("Rust: 58.2%"));
        assert!(!prompt_text.contains('\u{2014}'));
    }

    #[test]
    fn test_build_auth_headers() {
        let headers = build_auth_headers("test-secret-key");
        assert_eq!(headers.get(USER_AGENT).unwrap(), "slop-cli/0.1.0");
        assert_eq!(headers.get(ACCEPT).unwrap(), "application/json");
        let auth_val = headers.get(AUTHORIZATION).unwrap().to_str().unwrap();
        assert!(auth_val.starts_with("Basic "));
    }

    #[test]
    fn test_extract_stats() {
        let empty_resp = WakaTimeResponse {
            data: WakaTimeStats {
                languages: vec![],
                editors: vec![],
                operating_systems: vec![],
                human_readable_total: None,
                human_readable_daily_average: None,
                total_seconds: None,
            },
        };
        assert!(extract_stats(empty_resp).is_none());

        let valid_resp = WakaTimeResponse { data: mock_stats() };
        let extracted = extract_stats(valid_resp);
        assert!(extracted.is_some());
        assert_eq!(extracted.unwrap().languages.len(), 4);
    }

    #[tokio::test]
    async fn test_fetch_stats_empty_and_whitespace_keys() {
        let client = reqwest::Client::new();
        let stats_none = fetch_stats(&client, None, false).await;
        assert_eq!(stats_none.languages.len(), 4);

        let stats_empty = fetch_stats(&client, Some(""), false).await;
        assert_eq!(stats_empty.languages.len(), 4);

        let stats_whitespace = fetch_stats(&client, Some("   "), false).await;
        assert_eq!(stats_whitespace.languages.len(), 4);
    }
}
