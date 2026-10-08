use anyhow::Result;

pub const FALLBACK_STANDUP: &str = "\
- Telemetry: Active compute cycles concentrated on low-latency Rust systems and autonomous agent frameworks.
- Platform: Advancing Kubernetes platform deployments and infrastructure as code automation.
- Momentum: Continuous integration verification and automated profile compiler pipeline operational.
- Focus: Scaling multi-agent coordination tooling and resilient cloud-native workflows.";

#[derive(serde::Serialize, Debug)]
pub struct GeminiRequest<'a> {
    pub contents: Vec<GeminiContent<'a>>,
    #[serde(rename = "generationConfig")]
    pub generation_config: GenerationConfig,
}

#[derive(serde::Serialize, Debug)]
pub struct GeminiContent<'a> {
    pub parts: Vec<GeminiPart<'a>>,
}

#[derive(serde::Serialize, Debug)]
pub struct GeminiPart<'a> {
    pub text: &'a str,
}

#[derive(serde::Serialize, Debug)]
pub struct GenerationConfig {
    pub temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    pub max_output_tokens: u32,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct GeminiResponse {
    pub candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct GeminiCandidate {
    pub content: Option<GeminiCandidateContent>,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct GeminiCandidateContent {
    pub parts: Option<Vec<GeminiCandidatePart>>,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct GeminiCandidatePart {
    pub text: Option<String>,
}

pub fn build_prompt(wakatime_summary: &str, github_summary: &str) -> String {
    format!(
        r#"You are the telemetry logging system for an autonomous AI systems engineer (MAHDTech).

Input Telemetry:
- WakaTime Compute (Past 7 Days):
{wakatime_summary}
- GitHub Activity:
{github_summary}

Requirements:
1. Generate exactly 3 to 4 concise bullet points summarizing recent momentum, development cycles, and platform engineering.
2. Tone: Crisp, technical, forward-looking, high momentum.
3. Highlight Rust systems, Kubernetes orchestration, and AI platform infrastructure.
4. CRITICAL RULE: DO NOT use em-dashes (Unicode U+2014 or &mdash;) anywhere in your response. Use hyphens (-), colons (:), or parentheses () instead.
5. Format output directly as a markdown bullet list (- Bullet point)."#
    )
}

pub fn sanitize_standup_bullets(raw_text: &str) -> String {
    let text = raw_text
        .replace(" \u{2014} ", " - ")
        .replace('\u{2014}', " - ")
        .replace(" \u{2013} ", " - ")
        .replace('\u{2013}', "-");

    let mut bullets: Vec<String> = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let cleaned = if let Some(stripped) = trimmed.strip_prefix("- ") {
            stripped.trim()
        } else if let Some(stripped) = trimmed.strip_prefix("* ") {
            stripped.trim()
        } else if let Some(stripped) = trimmed.strip_prefix("• ") {
            stripped.trim()
        } else {
            trimmed
                .find(". ")
                .filter(|&idx| idx <= 3 && trimmed[..idx].chars().all(|c| c.is_ascii_digit()))
                .map(|idx| trimmed[idx + 2..].trim())
                .unwrap_or(trimmed)
        };

        if !cleaned.is_empty() {
            bullets.push(format!("- {}", cleaned));
        }
    }

    if bullets.len() >= 3 && bullets.len() <= 4 {
        bullets.join("\n")
    } else if bullets.len() > 4 {
        bullets[..4].join("\n")
    } else {
        FALLBACK_STANDUP.to_string()
    }
}

pub async fn generate_standup_from_summaries(
    client: &reqwest::Client,
    api_key: Option<&str>,
    model: &str,
    wakatime_summary: &str,
    github_summary: &str,
    offline: bool,
) -> String {
    if offline {
        return FALLBACK_STANDUP.to_string();
    }

    let key = match api_key {
        Some(k) if !k.trim().is_empty() => k.trim(),
        _ => return FALLBACK_STANDUP.to_string(),
    };

    let prompt = build_prompt(wakatime_summary, github_summary);
    let request_body = GeminiRequest {
        contents: vec![GeminiContent {
            parts: vec![GeminiPart { text: &prompt }],
        }],
        generation_config: GenerationConfig {
            temperature: 0.4,
            max_output_tokens: 300,
        },
    };

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        model
    );

    let send_res = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("x-goog-api-key", key)
        .json(&request_body)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await;

    let response = match send_res {
        Ok(res) => {
            if !res.status().is_success() {
                eprintln!(
                    "[gemini] Upstream returned status {}: falling back to deterministic standup",
                    res.status()
                );
                return FALLBACK_STANDUP.to_string();
            }
            res
        }
        Err(err) => {
            eprintln!(
                "[gemini] Network error: {}, falling back to deterministic standup",
                err
            );
            return FALLBACK_STANDUP.to_string();
        }
    };

    let parsed: Result<GeminiResponse, _> = response.json().await;
    match parsed {
        Ok(gemini_resp) => {
            let candidate_text = gemini_resp
                .candidates
                .as_ref()
                .and_then(|c| c.first())
                .and_then(|c| c.content.as_ref())
                .and_then(|c| c.parts.as_ref())
                .and_then(|p| p.first())
                .and_then(|p| p.text.as_ref());

            if let Some(text) = candidate_text {
                sanitize_standup_bullets(text)
            } else {
                eprintln!("[gemini] No candidate text returned: using fallback standup");
                FALLBACK_STANDUP.to_string()
            }
        }
        Err(err) => {
            eprintln!(
                "[gemini] JSON deserialization error: {}, using fallback standup",
                err
            );
            FALLBACK_STANDUP.to_string()
        }
    }
}

pub async fn generate_standup(
    client: &reqwest::Client,
    api_key: Option<&str>,
    model: &str,
    wakatime_stats: &crate::wakatime::WakaTimeStats,
    github_activity: &crate::github::ActivitySummary,
    offline: bool,
) -> String {
    let waka_summary = wakatime_stats.format_for_prompt();
    let gh_summary = github_activity.format_for_prompt();
    generate_standup_from_summaries(client, api_key, model, &waka_summary, &gh_summary, offline)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_standup_bullet_count() {
        let bullets: Vec<&str> = FALLBACK_STANDUP
            .lines()
            .filter(|l| l.starts_with("- "))
            .collect();
        assert_eq!(
            bullets.len(),
            4,
            "Fallback standup must contain exactly 4 bullets"
        );
    }

    #[test]
    fn test_fallback_standup_zero_em_dashes() {
        assert!(
            !FALLBACK_STANDUP.contains('\u{2014}'),
            "Fallback standup must not contain em-dashes"
        );
        assert!(
            !FALLBACK_STANDUP.contains('\u{2013}'),
            "Fallback standup must not contain en-dashes"
        );
        assert!(!FALLBACK_STANDUP.contains("&mdash;"));
    }

    #[test]
    fn test_fallback_standup_keywords() {
        assert!(FALLBACK_STANDUP.contains("Rust"));
        assert!(FALLBACK_STANDUP.contains("Kubernetes"));
        assert!(FALLBACK_STANDUP.contains("Platform"));
        assert!(FALLBACK_STANDUP.contains("Telemetry"));
    }

    #[test]
    fn test_build_prompt_contains_constraints() {
        let prompt = build_prompt("Rust 28 hrs", "3 PRs merged");
        assert!(prompt.contains("DO NOT use em-dashes"));
        assert!(prompt.contains("Rust 28 hrs"));
        assert!(prompt.contains("3 PRs merged"));
        assert!(!prompt.contains('\u{2014}'));
    }

    #[test]
    fn test_sanitize_replaces_em_dashes() {
        let raw =
            "- Telemetry \u{2014} active cycles\n- Platform \u{2013} Kubernetes\n- Focus: scaling";
        let sanitized = sanitize_standup_bullets(raw);
        assert!(!sanitized.contains('\u{2014}'));
        assert!(!sanitized.contains('\u{2013}'));
        assert!(sanitized.contains("Telemetry - active cycles"));
    }

    #[test]
    fn test_sanitize_normalizes_bullet_markers() {
        let raw = "* First bullet\n• Second bullet\n- Third bullet\n1. Fourth bullet";
        let sanitized = sanitize_standup_bullets(raw);
        let lines: Vec<&str> = sanitized.lines().collect();
        assert_eq!(lines.len(), 4);
        for line in lines {
            assert!(line.starts_with("- "));
        }
    }

    #[test]
    fn test_sanitize_truncates_excess_bullets() {
        let raw = "- B1\n- B2\n- B3\n- B4\n- B5";
        let sanitized = sanitize_standup_bullets(raw);
        let count = sanitized.lines().filter(|l| l.starts_with("- ")).count();
        assert_eq!(count, 4);
    }

    #[test]
    fn test_sanitize_under_three_bullets_falls_back() {
        let raw = "- B1\n- B2";
        let sanitized = sanitize_standup_bullets(raw);
        assert_eq!(sanitized, FALLBACK_STANDUP);
    }

    #[test]
    fn test_deserialize_gemini_response() {
        let json_data = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {
                                "text": "- Bullet 1\n- Bullet 2\n- Bullet 3"
                            }
                        ]
                    }
                }
            ]
        }"#;
        let resp: GeminiResponse = serde_json::from_str(json_data).unwrap();
        let text = resp
            .candidates
            .as_ref()
            .and_then(|c| c.first())
            .and_then(|c| c.content.as_ref())
            .and_then(|c| c.parts.as_ref())
            .and_then(|p| p.first())
            .and_then(|p| p.text.as_ref())
            .unwrap();
        assert_eq!(text, "- Bullet 1\n- Bullet 2\n- Bullet 3");
    }

    #[tokio::test]
    async fn test_generate_standup_offline_mode() {
        let client = reqwest::Client::new();
        let res = generate_standup_from_summaries(
            &client,
            Some("dummy_key"),
            "gemini-3.8-flash",
            "mock waka",
            "mock gh",
            true,
        )
        .await;
        assert_eq!(res, FALLBACK_STANDUP);
    }

    #[tokio::test]
    async fn test_generate_standup_missing_key() {
        let client = reqwest::Client::new();
        let res = generate_standup_from_summaries(
            &client,
            None,
            "gemini-3.8-flash",
            "mock waka",
            "mock gh",
            false,
        )
        .await;
        assert_eq!(res, FALLBACK_STANDUP);
    }

    #[tokio::test]
    async fn test_generate_standup_empty_key() {
        let client = reqwest::Client::new();
        let res = generate_standup_from_summaries(
            &client,
            Some("   "),
            "gemini-3.8-flash",
            "mock waka",
            "mock gh",
            false,
        )
        .await;
        assert_eq!(res, FALLBACK_STANDUP);
    }
}
