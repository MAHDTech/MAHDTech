# MAHDTech Repository

## Overview

This repository generates the MAHDTech GitHub profile landing page via automated tooling.

## Structure

The repository is organized as follows:

- `Cargo.toml`: Root Cargo workspace definition.
- `crates/slop-cli/`: Cargo workspace member crate implementing the `slop` profile generator CLI:
  - `src/main.rs`: CLI entrypoint, argument parsing, and generation pipeline orchestration.
  - `src/wakatime.rs`: WakaTime API v1 stats client and ASCII retro-terminal compute HUD generator.
  - `src/github.rs`: GitHub user activity fetcher and commit/PR aggregation.
  - `src/gemini.rs`: Google Gemini Flash API client (`gemini-3.8-flash`), telemetry standup generator, and deterministic offline fallbacks.
  - `src/template.rs`: Tera template engine, dynamic context injection, and atomic file emitter.
- `README.template.md`: Single source of truth template for the profile layout.
- `README.md`: Generated profile markdown document.
- `.cargo-crap.toml`: Change Risk Anti-Patterns (CRAP) metric and test coverage configuration.
- `.github/workflows/`: GitHub Actions automation:
  - `update-readme.yaml`: Daily cron and manual dispatch profile compilation workflow.
  - `ci.yaml`: Pinned consumer devenv CI workflow (v3.3.0).
  - `cargo-crap.yaml`: Pinned consumer cargo-crap complexity and coverage workflow (v3.3.0).
  - `sec-codeql.yaml`: Pinned consumer CodeQL static analysis workflow (v3.3.0).
  - `sec-trivy.yaml`: Pinned consumer Trivy vulnerability scanner workflow (v3.3.0).
- `devenv.nix`: Hermetic development environment, packages, and `git-hooks` configured with `prek`.
- `project-words.txt`: Custom domain vocabulary for CSpell linting.
- `AGENTS.md`: Repository rules, governance, architecture, and behavioral guidelines.
- `.gitignore`: Git exclusion patterns for build artifacts, environment files, and agent metadata.

## Development

- Environment: This project requires `devenv`. Always check if `devenv` is active. Run ad-hoc commands inside the devenv shell with `devenv --no-tui shell -- <command>`.

- ⚠️ **CRITICAL WARNING**: Always pass the `--no-tui` flag when running `devenv` commands (e.g., `devenv --no-tui shell`, `devenv --no-tui test`) in automated or AI agent environments to disable the interactive terminal interface and prevent commands from getting stuck.

- **CRITICAL LINTING RULE:** NEVER run individual linters directly (e.g., `devenv --no-tui shell -- markdownlint .`). All linters and pre-commit hooks (managed via `prek`) MUST be configured and run inside `devenv.nix`.

- **CRITICAL HOOK RUNNER RULE:** The standalone `pre-commit` CLI tool and package are DEPRECATED and MUST NOT be used or added to `devenv.nix` (e.g., NEVER add `pkgs.pre-commit` or `pre-commit` package/input). Differentiate between "pre-commit" (the Git lifecycle hook stage) and `prek` (the actual CLI binary tool). Always use `prek` (e.g., `pkgs.prek` or `git-hooks`).

- **CRITICAL TESTING RULE:** ALWAYS run tests via `devenv --no-tui test` or the `run-tests` wrappers. This is the single guaranteed path.

- **CRITICAL ZERO EM-DASH RULE:** Never use em-dashes (Unicode U+2014) anywhere across code, templates, generated markdown, workflows, commit messages, or documentation. Always use hyphens (-), colons (:), or rewrite phrasing.

- If a specific linter or `prek` hook check does not exist, check the devenv MCP server or devenv agent docs. If you still do not find it, ask the user for confirmation.

- Runtime: Rust toolchain via devenv. The Cargo workspace member `crates/slop-cli` produces the `slop` binary.

- CLI Tool Execution:
  - Run via Cargo workspace package: `cargo run -p slop-cli -- [FLAGS]` (or `cargo run --bin slop -- [FLAGS]`).
  - Supported flags:
    - `--dry-run`: Preview rendered markdown to stdout without modifying destination file.
    - `--template <PATH>`: Path to input template (defaults to `README.template.md`).
    - `--output <PATH>`: Destination path for rendered markdown (defaults to `README.md`).
    - `--offline`: Bypass all network requests and force deterministic fallback telemetry.
    - `--model <MODEL>`: Gemini model identifier (defaults to `gemini-3.8-flash` or `GEMINI_MODEL` environment variable).
    - `--verbose`: Emit diagnostic logs to stderr.

- Deterministic Testing:
  - All unit tests in `crates/slop-cli` must run completely offline without external network dependencies.
  - When external API tokens are missing or network calls fail, `slop` must degrade gracefully to deterministic mock data and never panic.

## Behaviour

Follow the behavioural guidelines that reduce common LLM coding mistakes:

- think before coding (surface assumptions and tradeoffs instead of guessing)
- keep changes simple and surgical (minimum code, touch only what the request needs)
- drive every task to a verified success criterion.

They bias toward caution over speed; for trivial tasks, use judgment.
