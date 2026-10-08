use reqwest::header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::{Deserialize, Serialize};

/// GitHub event object from /users/{user}/events API.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub repo: GitHubRepoRef,
    #[serde(default)]
    pub payload: serde_json::Value,
    #[serde(default)]
    pub created_at: String,
}

/// Repository reference inside a GitHub event.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitHubRepoRef {
    pub name: String,
}

/// Aggregated engineering activity summary for template and Gemini prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySummary {
    pub pull_requests_opened: Vec<String>,
    pub pull_requests_merged: Vec<String>,
    pub commits_pushed: usize,
    pub recent_repos: Vec<String>,
}

impl ActivitySummary {
    /// Formats the activity summary for LLM prompt ingestion.
    pub fn format_for_prompt(&self) -> String {
        format!(
            "Commits pushed: {}. PRs opened: {}. PRs merged: {}. Active repositories: {}.",
            self.commits_pushed,
            self.pull_requests_opened.len(),
            self.pull_requests_merged.len(),
            if self.recent_repos.is_empty() {
                "MAHDTech/MAHDTech".to_string()
            } else {
                self.recent_repos.join(", ")
            }
        )
    }
}

/// Provides deterministic fallback activity summary when offline or API is rate-limited.
pub fn mock_activity() -> ActivitySummary {
    ActivitySummary {
        pull_requests_opened: vec!["Automated Profile Telemetry Pipeline".to_string()],
        pull_requests_merged: vec!["Hermetic Devenv Infrastructure & Workflows".to_string()],
        commits_pushed: 42,
        recent_repos: vec![
            "MAHDTech/MAHDTech".to_string(),
            "MAHDTech/skills-hub".to_string(),
        ],
    }
}

/// Aggregates a raw vector of GitHub events into an ActivitySummary.
pub fn summarize_events(events: &[GitHubEvent]) -> ActivitySummary {
    let mut pull_requests_opened = Vec::new();
    let mut pull_requests_merged = Vec::new();
    let mut commits_pushed = 0;
    let mut recent_repos = Vec::new();

    for event in events {
        if !recent_repos.contains(&event.repo.name) && recent_repos.len() < 5 {
            recent_repos.push(event.repo.name.clone());
        }

        match event.event_type.as_str() {
            "PushEvent" => {
                if let Some(commits) = event.payload.get("commits").and_then(|c| c.as_array()) {
                    commits_pushed += commits.len();
                } else if let Some(size) = event.payload.get("size").and_then(|s| s.as_u64()) {
                    commits_pushed += size as usize;
                } else {
                    commits_pushed += 1;
                }
            }
            "PullRequestEvent" => {
                let action = event
                    .payload
                    .get("action")
                    .and_then(|a| a.as_str())
                    .unwrap_or("");
                let pr_obj = event.payload.get("pull_request");
                let title = pr_obj
                    .and_then(|pr| pr.get("title"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("Engineering Improvement");
                let merged = pr_obj
                    .and_then(|pr| pr.get("merged"))
                    .and_then(|m| m.as_bool())
                    .unwrap_or(false);

                if action == "opened" {
                    pull_requests_opened.push(format!("{}: {}", event.repo.name, title));
                } else if action == "closed" && merged {
                    pull_requests_merged.push(format!("{}: {}", event.repo.name, title));
                }
            }
            _ => {}
        }
    }

    if commits_pushed == 0 && pull_requests_opened.is_empty() && pull_requests_merged.is_empty() {
        return mock_activity();
    }

    ActivitySummary {
        pull_requests_opened,
        pull_requests_merged,
        commits_pushed,
        recent_repos,
    }
}

/// Fetches recent public activity from GitHub API with graceful fallback.
pub async fn fetch_recent_activity(
    client: &reqwest::Client,
    token: Option<&str>,
    offline: bool,
) -> ActivitySummary {
    if offline {
        return mock_activity();
    }

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("slop-cli/0.1.0"));
    headers.insert(
        ACCEPT,
        HeaderValue::from_static("application/vnd.github+json"),
    );
    headers.insert(
        HeaderName::from_static("x-github-api-version"),
        HeaderValue::from_static("2022-11-28"),
    );

    if let Some(tok) = token {
        let trimmed = tok.trim();
        if !trimmed.is_empty() {
            if let Ok(hv) = HeaderValue::from_str(&format!("Bearer {}", trimmed)) {
                headers.insert(AUTHORIZATION, hv);
            }
        }
    }

    let url = "https://api.github.com/users/MAHDTech/events?per_page=30";
    let resp = match client
        .get(url)
        .headers(headers)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[github] Network error fetching events: {}", e);
            return mock_activity();
        }
    };

    if !resp.status().is_success() {
        eprintln!(
            "[github] Upstream returned status {}: falling back to mock activity",
            resp.status()
        );
        return mock_activity();
    }

    match resp.json::<Vec<GitHubEvent>>().await {
        Ok(events) => summarize_events(&events),
        Err(e) => {
            eprintln!("[github] Error deserializing events JSON: {}", e);
            mock_activity()
        }
    }
}

/// Alias for fetch_recent_activity.
#[allow(dead_code)]
pub async fn fetch_activity(
    client: &reqwest::Client,
    token: Option<&str>,
    offline: bool,
) -> ActivitySummary {
    fetch_recent_activity(client, token, offline).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_push_and_pr_events() {
        let events = vec![
            GitHubEvent {
                id: "1".to_string(),
                event_type: "PushEvent".to_string(),
                repo: GitHubRepoRef {
                    name: "MAHDTech/MAHDTech".to_string(),
                },
                payload: serde_json::json!({
                    "commits": [{"sha": "abc"}, {"sha": "def"}]
                }),
                created_at: "2026-10-08T10:00:00Z".to_string(),
            },
            GitHubEvent {
                id: "2".to_string(),
                event_type: "PullRequestEvent".to_string(),
                repo: GitHubRepoRef {
                    name: "MAHDTech/MAHDTech".to_string(),
                },
                payload: serde_json::json!({
                    "action": "opened",
                    "pull_request": {
                        "title": "feat: Profile Automation CLI",
                        "merged": false
                    }
                }),
                created_at: "2026-10-08T11:00:00Z".to_string(),
            },
            GitHubEvent {
                id: "3".to_string(),
                event_type: "PullRequestEvent".to_string(),
                repo: GitHubRepoRef {
                    name: "MAHDTech/skills-hub".to_string(),
                },
                payload: serde_json::json!({
                    "action": "closed",
                    "pull_request": {
                        "title": "chore: dependency updates",
                        "merged": true
                    }
                }),
                created_at: "2026-10-08T12:00:00Z".to_string(),
            },
        ];

        let summary = summarize_events(&events);
        assert_eq!(summary.commits_pushed, 2);
        assert_eq!(summary.pull_requests_opened.len(), 1);
        assert_eq!(summary.pull_requests_merged.len(), 1);
        assert_eq!(summary.recent_repos.len(), 2);
    }

    #[test]
    fn test_prompt_format_zero_em_dash() {
        let summary = mock_activity();
        let prompt_text = summary.format_for_prompt();
        assert!(!prompt_text.contains('\u{2014}'));
        assert!(prompt_text.contains("Commits pushed:"));
        assert!(prompt_text.contains("Active repositories:"));
    }

    #[tokio::test]
    async fn test_offline_mode_returns_mock_activity() {
        let client = reqwest::Client::new();
        let summary = fetch_activity(&client, None, true).await;
        assert_eq!(summary.commits_pushed, 42);
        assert!(!summary.recent_repos.is_empty());
    }

    #[test]
    fn test_deserialize_github_event_json() {
        let raw_json = r#"[
            {
                "id": "12345",
                "type": "PushEvent",
                "repo": {"name": "MAHDTech/MAHDTech"},
                "payload": {"size": 3},
                "created_at": "2026-10-08T10:00:00Z"
            }
        ]"#;

        let parsed: Result<Vec<GitHubEvent>, _> = serde_json::from_str(raw_json);
        assert!(parsed.is_ok());
        let events = parsed.unwrap();
        assert_eq!(events[0].event_type, "PushEvent");
        assert_eq!(events[0].repo.name, "MAHDTech/MAHDTech");
    }

    #[test]
    fn test_mock_activity_contents() {
        let summary = mock_activity();
        assert_eq!(summary.commits_pushed, 42);
        assert_eq!(summary.pull_requests_opened.len(), 1);
        assert_eq!(summary.pull_requests_merged.len(), 1);
        assert_eq!(summary.recent_repos.len(), 2);
    }
}
