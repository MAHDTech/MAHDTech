# Goal: Automated High-Impact GitHub Profile with `slop`

- Status: active
- Context: MAHDTech/MAHDTech on branch feat/fancy-pants
- Updated: 2026-10-08T21:16:00+11:00
- Budget: none

## Objective

Implement the automated GitHub profile in `.tars/scratch/GOAL.md`.

## Sources and scope changes

- Source: `.tars/scratch/GOAL.md`
- Referenced consumers guide: `/home/mahdtech/Sync/Projects/GitHub/tars-cloud/actions/.tars/scratch/CONSUMERS.md`
- Target shared actions release: `tars-cloud/actions` v3.3.0 (`822009e983b73316cbf67a3606842502c7be4599`)

## Requirements and evidence

| Requirement | State | Evidence |
| --- | --- | --- |
| 1. Cargo workspace with `crates/slop-cli` producing binary `slop` | unverified | `Cargo.toml` and `crates/slop-cli/Cargo.toml` |
| 2. Implement `wakatime.rs` with API client and ASCII terminal HUD | unverified | Unit tests in `wakatime.rs` |
| 3. Implement `github.rs` with GitHub events client | unverified | Unit tests in `github.rs` |
| 4. Implement `gemini.rs` with Gemini API client and fallback telemetry | unverified | Unit tests in `gemini.rs` |
| 5. Implement `template.rs` with Tera rendering of `README.template.md` | unverified | Unit tests in `template.rs` |
| 6. Draft `README.template.md` and generate initial `README.md` | unverified | File presence and rendered output |
| 7. Create/Update CI workflows (`update-readme.yaml`, `ci.yaml`, `cargo-crap.yaml`, `sec-codeql.yaml`, `sec-trivy.yaml`) | unverified | Pinned workflow files matching CONSUMERS.md |
| 8. Update `AGENTS.md` for this repository | unverified | `AGENTS.md` content and formatting |
| 9. Verification via `devenv test` and pre-commit hooks (`prek`) | unverified | Clean run of `devenv --no-tui test` |
| 10. Commit, push branch, open PR, and notify user | unverified | GitHub PR URL |

## Checkpoint

- Progress: Goal recorded and initialized.
- Live work: none
- Blocker: none
- Next action: Set up root `Cargo.toml` workspace and scaffold `crates/slop-cli`.
