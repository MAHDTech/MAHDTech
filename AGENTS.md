# MAHDTech Repository

## Overview

This repo generates the MAHDTech profile landing content via automation.

## Structure

TODO

## Development

- Environment: This project requires `devenv`. Always check if `devenv` is active. Run ad-hoc commands inside the devenv shell with `devenv --no-tui shell -- <command>`.

- ⚠️ **CRITICAL WARNING**: Always pass the `--no-tui` flag when running `devenv` commands (e.g., `devenv --no-tui shell`, `devenv --no-tui test`) in automated or AI agent environments to disable the interactive terminal interface and prevent commands from getting stuck.

- **CRITICAL LINTING RULE:** NEVER run individual linters directly (e.g., `devenv --no-tui shell -- markdownlint .`). All linters and pre-commit hooks (managed via `prek`) MUST be configured and run inside `devenv.nix`.

- **CRITICAL HOOK RUNNER RULE:** The standalone `pre-commit` CLI tool and package are DEPRECATED and MUST NOT be used or added to `devenv.nix` (e.g., NEVER add `pkgs.pre-commit` or `pre-commit` package/input). Differentiate between "pre-commit" (the Git lifecycle hook stage) and `prek` (the actual CLI binary tool). Always use `prek` (e.g., `pkgs.prek` or `git-hooks`).

- **CRITICAL TESTING RULE:** ALWAYS run tests via `devenv --no-tui test` or the `run-tests` wrappers. This is the single guaranteed path.

- If a specific linter or `prek` hook check doesn't exist, check the devenv MCP server or devenv agent docs. If you STILL don't find it, ask the user for confirmation.

- Runtime: Rust toolchain via devenv. The native `ask` CLI binary in `crates/ask-cli` manages skills and the dashboard.

- CLI Tool: `ask` (or `cargo run -p slop --`).

## Behaviour

Follow the behavioural guidelines that reduce common LLM coding mistakes;

- think before coding (surface assumptions and tradeoffs instead of guessing)
- keep changes simple and surgical (minimum code, touch only what the request needs)
- drive every task to a verified success criterion.

They bias toward caution over speed, for trivial tasks, use judgment.
