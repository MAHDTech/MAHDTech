use anyhow::{Context, Result, bail};
use std::path::Path;

pub const TEMPLATE_NAME: &str = "profile";

pub fn create_context(
    wakatime_hud: &str,
    ai_standup: &str,
    last_updated: &str,
    cli_version: &str,
) -> tera::Context {
    let mut context = tera::Context::new();
    context.insert("wakatime_hud", wakatime_hud);
    context.insert("ai_standup", ai_standup);
    context.insert("last_updated", last_updated);
    context.insert("cli_version", cli_version);
    context
}

pub fn render_template(template_content: &str, context: &tera::Context) -> Result<String> {
    let mut tera = tera::Tera::default();
    tera.autoescape_on(vec![]);
    tera.add_raw_template(TEMPLATE_NAME, template_content)
        .context("Failed to parse Tera template syntax")?;

    let rendered = tera
        .render(TEMPLATE_NAME, context)
        .context("Failed to render Tera template")?;

    if rendered.contains('\u{2014}') {
        bail!("Fatal: Rogue em-dash detected in output stream");
    }

    Ok(rendered)
}

pub fn render_profile(template_content: &str, context: &tera::Context) -> Result<String> {
    render_template(template_content, context)
}

#[allow(dead_code)]
pub fn load_and_render_template(template_path: &Path, context: &tera::Context) -> Result<String> {
    if !template_path.exists() {
        bail!("Template file not found: {}", template_path.display());
    }

    let template_content = std::fs::read_to_string(template_path)
        .with_context(|| format!("Failed to read template file: {}", template_path.display()))?;

    render_template(&template_content, context)
}

pub fn write_atomic(output_path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = output_path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }

    let file_stem = output_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output");
    let tmp_file_name = format!("{}.tmp", file_stem);
    let tmp_path = match output_path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.join(tmp_file_name),
        _ => std::path::PathBuf::from(tmp_file_name),
    };

    if let Err(write_err) = std::fs::write(&tmp_path, content.as_bytes()) {
        if tmp_path.exists() {
            let _ = std::fs::remove_file(&tmp_path);
        }
        return Err(write_err).with_context(|| {
            format!(
                "Failed to write temporary output file: {}",
                tmp_path.display()
            )
        });
    }

    if let Err(rename_err) = std::fs::rename(&tmp_path, output_path) {
        if tmp_path.exists() {
            let _ = std::fs::remove_file(&tmp_path);
        }
        return Err(rename_err).with_context(|| {
            format!(
                "Failed to atomically rename {} to {}",
                tmp_path.display(),
                output_path.display()
            )
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_template_basic_substitution() {
        let tmpl = "# Test\n{{ ai_standup }}\n{{ wakatime_hud }}\nLast Updated: {{ last_updated }}\nv{{ cli_version }}";
        let ctx = create_context("HUD_MOCK", "STANDUP_MOCK", "2026-10-08 21:00 UTC", "0.1.0");
        let rendered = render_template(tmpl, &ctx).unwrap();
        assert!(rendered.contains("STANDUP_MOCK"));
        assert!(rendered.contains("HUD_MOCK"));
        assert!(rendered.contains("2026-10-08 21:00 UTC"));
        assert!(rendered.contains("v0.1.0"));
        assert!(!rendered.contains("{{"));
    }

    #[test]
    fn test_render_template_preserves_html_details() {
        let tmpl = "<details><summary>Summary</summary>\n{{ wakatime_hud }}\n</details>";
        let ctx = create_context("HUD_MOCK", "", "", "");
        let rendered = render_template(tmpl, &ctx).unwrap();
        assert!(rendered.contains("<details>"));
        assert!(rendered.contains("<summary>Summary</summary>"));
        assert!(rendered.contains("</details>"));
        assert!(!rendered.contains("&lt;details&gt;"));
    }

    #[test]
    fn test_render_template_empty_template() {
        let tmpl = "";
        let ctx = create_context("HUD", "STANDUP", "NOW", "0.1.0");
        let rendered = render_template(tmpl, &ctx).unwrap();
        assert_eq!(rendered, "");
    }

    #[test]
    fn test_render_template_syntax_error() {
        let tmpl = "Broken {{ unclosed_tag";
        let ctx = create_context("HUD", "STANDUP", "NOW", "0.1.0");
        let err = render_template(tmpl, &ctx);
        assert!(err.is_err(), "Invalid Tera syntax should return an Err");
    }

    #[test]
    fn test_render_template_detects_em_dash() {
        let tmpl = "Text with rogue em-dash \u{2014} here";
        let ctx = create_context("HUD", "STANDUP", "NOW", "0.1.0");
        let err = render_template(tmpl, &ctx);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Rogue em-dash"));
    }

    #[test]
    fn test_write_atomic_creates_directories_and_writes() {
        let temp_dir = tempfile::tempdir().unwrap();
        let target_path = temp_dir.path().join("nested").join("sub").join("README.md");
        let content = "# Hello World\n";

        write_atomic(&target_path, content).unwrap();
        assert!(target_path.exists());
        let read_back = std::fs::read_to_string(&target_path).unwrap();
        assert_eq!(read_back, content);

        let parent = target_path.parent().unwrap();
        let entries: Vec<_> = std::fs::read_dir(parent).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_load_and_render_missing_template() {
        let ctx = create_context("HUD", "STANDUP", "NOW", "0.1.0");
        let err = load_and_render_template(Path::new("/non_existent/path/template.md"), &ctx);
        assert!(err.is_err());
    }
}
