<!-- cspell:words wakatime Platane skillicons actionlint Tera codeql trivy clippy actionlint neovim -->

# End-to-End Test Infrastructure Specification

## 1. Overview and Architecture

This document defines the comprehensive end-to-end (E2E) testing infrastructure for the MAHDTech Profile Automation project (`MAHDTech/MAHDTech`). The testing methodology follows a strict 4-tier validation model to guarantee functional correctness, boundary resiliency, cross-feature coherence, and real-world execution integrity.

### Core Testing Tenets

- **Deterministic Execution**: All E2E tests are capable of executing completely offline without relying on external network access or external credentials.
- **Progressive Testability**: The test harness provides robust verification across all project milestones, supporting both early structural verification and compiled binary integration.
- **Strict Typography Integrity**: Zero tolerance for em-dash characters (Unicode `U+2014`) across all source templates, rendered outputs, telemetry logs, and documentation.
- **Opaque-Box Verification**: The test suite treats the CLI tool (`slop`) as an opaque binary, validating inputs, flags, exit codes, stdout/stderr streams, and filesystem modifications.

---

## 2. Feature Inventory Under Test

The E2E testing framework covers seven core features:

| Feature ID    | Feature Name                    | Primary Scope                                                                                                   |
| ------------- | ------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| `F-CLI`       | CLI Arguments and Options       | Flag parsing (`--dry-run`, `--template`, `--output`, `--offline`, `--model`, `--verbose`, `--help`), exit codes |
| `F-HUD`       | WakaTime ASCII Compute HUD      | 56-character line width, Unicode bar rendering (`█`/`░`), percentage alignment, footer format                   |
| `F-STANDUP`   | Gemini Standup Generator        | 3 to 4 bullet points, systems engineering tone, model overrides, prompt guardrails                              |
| `F-FALLBACK`  | Deterministic Offline Fallbacks | Graceful degradation on missing API keys, rate limits (HTTP 429), server errors (5xx), no panic                 |
| `F-TERA`      | Tera Template Engine            | Dynamic variable injection (`wakatime_hud`, `ai_standup`, `last_updated`), markdown preserving                  |
| `F-EMDASH`    | Zero Em-Dash Enforcement        | Complete prevention and sanitation of em-dash characters in all generated artifacts                             |
| `F-WORKFLOWS` | CI/CD Workflows and Actionlint  | Workflow syntax, trigger configurations, secret parameterization, actionlint compliance                         |

---

## 3. Tier 1: Feature Coverage

Tier 1 validates primary functional behavior (happy paths) for every feature under test. Each feature contains at least five distinct test cases.

### Feature 1: CLI Arguments and Options (`F-CLI`)

#### TC-CLI-01: Help Flag Documentation

- **Input**: `slop --help`
- **Authoritative Source**: Project CLI Contract (`PROJECT.md` § CLI Binary Contract)
- **Expected Output**: Standard output displays usage synopsis, documentation for all flags (`--dry-run`, `--template`, `--output`, `--offline`, `--model`, `--verbose`), and exits with code `0`.
- **Verification Method**: Execute CLI with `--help`, capture stdout, grep for flag descriptions, assert exit code `0`.

#### TC-CLI-02: Dry-Run Mode Execution

- **Input**: `slop --dry-run`
- **Authoritative Source**: `ORIGINAL_REQUEST.md` R1, `PROJECT.md` § CLI Binary Contract
- **Expected Output**: Renders compiled markdown directly to standard output, makes zero write operations to `README.md`, and exits with code `0`.
- **Verification Method**: Execute with `--dry-run`, verify stdout is non-empty markdown, confirm target file timestamp/hash is unchanged.

#### TC-CLI-03: Custom Template Path Specification

- **Input**: `slop --template custom_template.md --dry-run`
- **Authoritative Source**: `PROJECT.md` § CLI Binary Contract
- **Expected Output**: CLI parses custom template file, substitutes context variables, outputs rendered text to stdout, and exits with code `0`.
- **Verification Method**: Create temporary custom template file with known sentinel text, run CLI, verify sentinel text in stdout.

#### TC-CLI-04: Custom Output Destination Path

- **Input**: `slop --offline --output custom_output.md`
- **Authoritative Source**: `PROJECT.md` § CLI Binary Contract
- **Expected Output**: CLI renders profile content and writes directly to `custom_output.md`, creating the file cleanly, and exits with code `0`.
- **Verification Method**: Run CLI with custom output path, verify file exists at destination, verify contents match expected template structure.

#### TC-CLI-05: Explicit Offline Flag Invocation

- **Input**: `slop --offline --dry-run`
- **Authoritative Source**: `ORIGINAL_REQUEST.md` R1, `PROJECT.md` § CLI Binary Contract
- **Expected Output**: CLI bypasses all HTTP socket initializations, loads deterministic fallback data for WakaTime and Gemini, and exits with code `0`.
- **Verification Method**: Run CLI with `--offline` in sandboxed environment without network access, verify immediate exit with code `0`.

#### TC-CLI-06: Gemini Model Flag Override

- **Input**: `slop --model gemini-3.8-flash --dry-run`
- **Authoritative Source**: `GOAL.md` § 4, `PROJECT.md` § CLI Binary Contract
- **Expected Output**: CLI registers configured model parameter, overrides default `gemini-2.5-flash`, and processes request without error.
- **Verification Method**: Inspect model resolution or verbose logs to confirm active model string matches `gemini-3.8-flash`.

---

### Feature 2: WakaTime ASCII Compute HUD (`F-HUD`)

#### TC-HUD-01: Header Line Content

- **Input**: WakaTime stats rendering pipeline
- **Authoritative Source**: `GOAL.md` § 3, `PROJECT.md` Feature 10
- **Expected Output**: HUD block begins with exact header string: `⚡ WEEKLY COMPUTE CYCLES (via WakaTime API)`.
- **Verification Method**: Extract first line of generated HUD block, assert equality with expected header text.

#### TC-HUD-02: Boundary Divider Width Integrity

- **Input**: Generated WakaTime HUD block
- **Authoritative Source**: `PROJECT.md` Feature 10 (56-character ASCII HUD layout)
- **Expected Output**: Horizontal divider lines consist of heavy box-drawing characters (`━`) with exact character count equal to 56.
- **Verification Method**: Measure character count (Unicode code points) of divider lines; assert length equals 56.

#### TC-HUD-03: Unicode Progress Bar Glyphs

- **Input**: Standard language distribution (e.g. Rust 58.2%, Python 16.7%)
- **Authoritative Source**: `GOAL.md` § 3
- **Expected Output**: Progress bars use filled blocks `█` (`\u{2588}`) and empty blocks `░` (`\u{2591}`) enclosed in square brackets `[...]` totaling 20 blocks.
- **Verification Method**: Extract bar slice from language lines, assert presence of `█` and `░`, verify total enclosed bar length is 20 glyphs.

#### TC-HUD-04: Language Row Column Alignment

- **Input**: Language statistics dataset
- **Authoritative Source**: `GOAL.md` § 3
- **Expected Output**: Language names padded to 14 characters, duration strings formatted with hrs/mins, percentages right-aligned to 5 characters with 1 decimal place.
- **Verification Method**: Parse columns across all language rows, verify uniform character offsets for bar start bracket `[` and percent sign `%`.

#### TC-HUD-05: Metadata Footer Format and Typography

- **Input**: Rendered HUD block footer
- **Authoritative Source**: `GOAL.md` § 3, `PROJECT.md` Feature 11
- **Expected Output**: Footer displays `OS: Linux (NixOS) | Primary Editor: Neovim / Helix` using pipe separator with zero em-dashes.
- **Verification Method**: Grep footer line, verify text content, verify pipe delimiter, assert absence of em-dash (`U+2014`).

---

### Feature 3: Gemini Standup Generator and Model Configuration (`F-STANDUP`)

#### TC-STAND-01: Telemetry Bullet Count

- **Input**: Generated AI standup summary
- **Authoritative Source**: `ORIGINAL_REQUEST.md` R1, `GOAL.md` § 4
- **Expected Output**: Standup block consists of exactly 3 to 4 markdown bullet items starting with a dash bullet (`-`).
- **Verification Method**: Count lines beginning with a dash bullet (`-`) in `ai_standup` block; assert count is in range `[3, 4]`.

#### TC-STAND-02: Autonomous Systems Tone and Content

- **Input**: Generated AI standup summary
- **Authoritative Source**: `GOAL.md` § 4, `PROJECT.md` Feature 16
- **Expected Output**: Standup bullets highlight low-latency Rust systems, Kubernetes orchestration, platform automation, and multi-agent systems.
- **Verification Method**: Inspect keywords across standup text; verify presence of relevant technical engineering terminology.

#### TC-STAND-03: Default Model Configuration

- **Input**: Invocation without `--model` or `GEMINI_MODEL`
- **Authoritative Source**: `ORIGINAL_REQUEST.md` R1, `PROJECT.md` Feature 15
- **Expected Output**: System defaults to `gemini-2.5-flash`.
- **Verification Method**: Verify default configuration value in binary metadata or debug output.

#### TC-STAND-04: Environment Variable Model Override

- **Input**: `GEMINI_MODEL=gemini-2.5-pro slop --dry-run`
- **Authoritative Source**: `GOAL.md` § Architectural Decision 1
- **Expected Output**: Binary picks up model selection from environment variable when `--model` CLI flag is not provided.
- **Verification Method**: Set environment variable, run dry-run, assert active model is `gemini-2.5-pro`.

#### TC-STAND-05: Markdown Bullet Formatting

- **Input**: Injected `ai_standup` text
- **Authoritative Source**: `PROJECT.md` Feature 16
- **Expected Output**: Bullets formatted cleanly with proper leading hyphen, single space, bold prefix or descriptive prefix, and standard punctuation.
- **Verification Method**: Regex match each line against `^- [A-Z][a-zA-Z0-9 /()]+: .+`.

---

### Feature 4: Deterministic Offline Fallbacks (`F-FALLBACK`)

#### TC-FALL-01: Execution with Unset API Credentials

- **Input**: `env -u GEMINI_API_KEY -u WAKATIME_API_KEY slop --dry-run`
- **Authoritative Source**: `ORIGINAL_REQUEST.md` R1, Acceptance Criteria
- **Expected Output**: Binary executes cleanly without error, outputs deterministic mock HUD and standup, exits with code `0`.
- **Verification Method**: Unset all tokens in isolated subshell, execute command, assert exit code `0` and non-empty output.

#### TC-FALL-02: Deterministic Output Stability

- **Input**: Two consecutive offline runs: `slop --offline --dry-run`
- **Authoritative Source**: `ORIGINAL_REQUEST.md` R1
- **Expected Output**: Both invocations produce identical telemetry text (ignoring dynamic timestamp in `last_updated`).
- **Verification Method**: Diff the two outputs filtering out timestamp line; assert identical diff.

#### TC-FALL-03: Network Isolation Resilience

- **Input**: Offline execution under disconnected network namespace or mock failure
- **Authoritative Source**: `ORIGINAL_REQUEST.md` Acceptance Criteria
- **Expected Output**: CLI detects offline condition, activates fallback paths, completes in under 2 seconds without timing out.
- **Verification Method**: Execute with `--offline`, measure execution duration; assert runtime < 2.0s and exit code `0`.

#### TC-FALL-04: Fallback Standup Consistency

- **Input**: Offline telemetry generator
- **Authoritative Source**: `PROJECT.md` Feature 17
- **Expected Output**: Emits pre-computed 4-bullet telemetry log covering Rust systems, Kubernetes platform, CI pipeline, and agent coordination.
- **Verification Method**: Compare generated standup with canonical fallback bullet text; assert match.

#### TC-FALL-05: Fallback HUD Data Integrity

- **Input**: Offline HUD generator
- **Authoritative Source**: `PROJECT.md` Feature 17
- **Expected Output**: Emits pre-computed HUD representing typical active compute stats (Rust ~58%, Python ~16%, YAML/K8s ~10%, Nix ~7%).
- **Verification Method**: Verify language rows and percentage values match canonical mock metrics.

---

### Feature 5: Tera Template Engine and Dynamic Context Injection (`F-TERA`)

#### TC-TERA-01: Profile Template Rendering

- **Input**: `README.template.md` containing Tera expressions
- **Authoritative Source**: `ORIGINAL_REQUEST.md` R1, `PROJECT.md` § Template Context Contract
- **Expected Output**: Compiles template into valid markdown document, fully replacing placeholders.
- **Verification Method**: Run compiler, verify output contains no unresolved `{{ ... }}` expressions.

#### TC-TERA-02: WakaTime HUD Placeholder Replacement

- **Input**: Template containing `{{ wakatime_hud }}`
- **Authoritative Source**: `PROJECT.md` § Template Context Contract
- **Expected Output**: Placeholder replaced with complete 56-character monospaced HUD code block.
- **Verification Method**: Check compiled markdown for presence of triple backticks code block enclosing WakaTime compute cycles.

#### TC-TERA-03: AI Standup Placeholder Replacement

- **Input**: Template containing `{{ ai_standup }}`
- **Authoritative Source**: `PROJECT.md` § Template Context Contract
- **Expected Output**: Placeholder replaced with 3 to 4 markdown bullet points.
- **Verification Method**: Verify compiled markdown includes rendered standup bullet items at expected location.

#### TC-TERA-04: Last Updated Timestamp Injection

- **Input**: Template containing `{{ last_updated }}`
- **Authoritative Source**: `PROJECT.md` § Template Context Contract
- **Expected Output**: Injected string represents current UTC timestamp formatted as `YYYY-MM-DD HH:MM UTC`.
- **Verification Method**: Regex match rendered timestamp against `^[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2} UTC$`.

#### TC-TERA-05: Markdown Autoescaping Prevention

- **Input**: Template containing markdown characters (backticks, brackets, ampersands) in injected variables
- **Authoritative Source**: `PROJECT.md` Feature 18 (disable markdown autoescaping)
- **Expected Output**: Special characters like `&`, `<`, `>`, `"` are NOT escaped as HTML entities (`&amp;`, `&lt;`) in markdown contexts.
- **Verification Method**: Verify rendered output retains raw markdown characters without HTML entity replacement.

---

### Feature 6: Zero Em-Dash Enforcement (`F-EMDASH`)

#### TC-DASH-01: Rendered Output Em-Dash Audit

- **Input**: Rendered `README.md`
- **Authoritative Source**: `ORIGINAL_REQUEST.md` R4, `PROJECT.md` Architecture
- **Expected Output**: Zero instances of Unicode em-dash character (`\u{2014}`) anywhere in the document.
- **Verification Method**: Execute character search for `\u2014`; assert match count is 0.

#### TC-DASH-02: WakaTime HUD Em-Dash Audit

- **Input**: Generated HUD block string
- **Authoritative Source**: `PROJECT.md` Feature 11
- **Expected Output**: Zero instances of em-dash in HUD header, borders, language rows, or footer.
- **Verification Method**: Scan HUD text for byte sequence `\xe2\x80\x94`; assert zero occurrences.

#### TC-DASH-03: AI Standup Em-Dash Audit

- **Input**: Generated AI standup text
- **Authoritative Source**: `ORIGINAL_REQUEST.md` R4
- **Expected Output**: Standup bullets use hyphens (`-`) or colons (`:`) exclusively; zero em-dashes.
- **Verification Method**: Scan standup text for byte sequence `\xe2\x80\x94`; assert zero occurrences.

#### TC-DASH-04: Source Template Em-Dash Audit

- **Input**: `README.template.md`
- **Authoritative Source**: `PROJECT.md` Feature 10
- **Expected Output**: Source template contains zero em-dash characters in copy, headers, badges, or comments.
- **Verification Method**: Scan source template file for em-dash; assert 0 matches.

#### TC-DASH-05: CLI Help Text and Diagnostics Em-Dash Audit

- **Input**: CLI stdout and stderr streams
- **Authoritative Source**: `PROJECT.md` Governance
- **Expected Output**: Help text, version strings, and error messages use standard hyphens and colons; zero em-dashes.
- **Verification Method**: Capture CLI help and verbose logs, scan for em-dash; assert 0 matches.

---

### Feature 7: GitHub Actions Workflows and Actionlint (`F-WORKFLOWS`)

#### TC-WF-01: Update Readme Workflow Validation

- **Input**: `.github/workflows/update-readme.yaml`
- **Authoritative Source**: `PROJECT.md` Feature 30
- **Expected Output**: `actionlint` verifies workflow syntax, step actions, triggers, and expressions with zero errors.
- **Verification Method**: Run `actionlint .github/workflows/update-readme.yaml`, assert exit code `0`.

#### TC-WF-02: CI Workflow Validation

- **Input**: `.github/workflows/ci.yaml`
- **Authoritative Source**: `PROJECT.md` Feature 33
- **Expected Output**: `actionlint` verifies workflow passes linting with exit code `0`.
- **Verification Method**: Run `actionlint .github/workflows/ci.yaml`, assert exit code `0`.

#### TC-WF-03: Cargo CRAP Workflow Validation

- **Input**: `.github/workflows/cargo-crap.yaml`
- **Authoritative Source**: `PROJECT.md` Feature 34
- **Expected Output**: `actionlint` verifies workflow passes linting with exit code `0`.
- **Verification Method**: Run `actionlint .github/workflows/cargo-crap.yaml`, assert exit code `0`.

#### TC-WF-04: CodeQL Workflow Validation

- **Input**: `.github/workflows/sec-codeql.yaml`
- **Authoritative Source**: `PROJECT.md` Feature 35
- **Expected Output**: `actionlint` verifies workflow passes linting with exit code `0`.
- **Verification Method**: Run `actionlint .github/workflows/sec-codeql.yaml`, assert exit code `0`.

#### TC-WF-05: Trivy Workflow Validation

- **Input**: `.github/workflows/sec-trivy.yaml`
- **Authoritative Source**: `PROJECT.md` Feature 36
- **Expected Output**: `actionlint` verifies workflow passes linting with exit code `0`.
- **Verification Method**: Run `actionlint .github/workflows/sec-trivy.yaml`, assert exit code `0`.

---

## 4. Tier 2: Boundary and Corner Cases

Tier 2 probes boundary extremes, abnormal inputs, stress thresholds, and error recovery for every feature. Each feature contains at least five distinct boundary test cases.

### Tier 2 Feature 1: CLI Arguments and Options (`F-CLI`)

#### TC-CLI-B01: Missing Required Option Argument

- **Input**: `slop --template` (missing path value)
- **Expected Output**: Clap parser catches missing argument, emits descriptive error to stderr, exits with non-zero code.
- **Verification Method**: Execute command, verify non-zero exit code, verify stderr contains error message.

#### TC-CLI-B02: Unrecognized CLI Option

- **Input**: `slop --unknown-custom-flag`
- **Expected Output**: Clap parser rejects unexpected flag, displays error and available options, exits with non-zero code.
- **Verification Method**: Execute command, verify non-zero exit code, verify stderr mentions unexpected flag.

#### TC-CLI-B03: Non-Existent Template File Path

- **Input**: `slop --template /path/to/missing_template.md --dry-run`
- **Expected Output**: CLI catches file-not-found error, emits human-readable error to stderr, exits with code `1`.
- **Verification Method**: Execute command with non-existent path, verify exit code `1`, verify error in stderr.

#### TC-CLI-B04: Read-Only or Unwritable Output Destination

- **Input**: `slop --offline --output /proc/readonly_file.md`
- **Expected Output**: Atomic file writer catches permission or filesystem write failure, logs error, exits with code `1`.
- **Verification Method**: Run with unwritable destination path, verify exit code `1`, confirm no panic stack trace.

#### TC-CLI-B05: Pathnames Containing Spaces and Special Characters

- **Input**: `slop --template "my templates/custom template.md" --dry-run`
- **Expected Output**: CLI safely handles quoted pathnames with whitespace without splitting arguments.
- **Verification Method**: Create temporary directory with spaces, write template, invoke CLI, verify clean execution.

---

### Tier 2 Feature 2: WakaTime ASCII Compute HUD (`F-HUD`)

#### TC-HUD-B01: Zero Total Compute Time (0.0 Hours)

- **Input**: WakaTime stats dataset where all language durations are 0 hours, 0 minutes (0.0 seconds)
- **Expected Output**: Progress bar renders 20 empty blocks `[░░░░░░░░░░░░░░░░░░░░] 0.0%` without division by zero.
- **Verification Method**: Feed zero-duration dataset into renderer, verify clean rendering with empty blocks and zero panics.

#### TC-HUD-B02: 100% Compute Time Single Language

- **Input**: Single language accounting for 100.0% of duration
- **Expected Output**: Progress bar renders 20 filled blocks `[████████████████████] 100.0%`.
- **Verification Method**: Feed 100% dataset into renderer, assert exactly 20 filled glyphs `█`.

#### TC-HUD-B03: Long Language Name Exceeding Column Width

- **Input**: Language name with 24 characters (e.g. `Protocol Buffers / gRPC`)
- **Expected Output**: Renderer truncates or formats cleanly without expanding total line length beyond 56 characters.
- **Verification Method**: Inspect rendered line length with long language name, assert line width <= 56 characters.

#### TC-HUD-B04: Percentage Clamping on Upstream Anomaly (>100% or <0%)

- **Input**: Upstream API reporting anomalous percentage (e.g. 145.0% or -10.0%)
- **Expected Output**: Renderer clamps bar blocks strictly within range `[0, 20]` without array index out-of-bounds.
- **Verification Method**: Pass out-of-range percentage, verify rendered bar contains between 0 and 20 blocks without crashing.

#### TC-HUD-B05: Empty Languages List

- **Input**: WakaTime dataset with empty languages array `[]`
- **Expected Output**: Renders valid HUD frame with header, dividers, empty state message, and footer without panicking.
- **Verification Method**: Pass empty language slice, verify HUD frame intact and exit code `0`.

---

### Tier 2 Feature 3: Gemini Standup Generator and Model Configuration (`F-STANDUP`)

#### TC-STAND-B01: Unconfigured API Key Graceful Recovery

- **Input**: Invocations where `GEMINI_API_KEY` is empty string (`GEMINI_API_KEY=""`)
- **Expected Output**: System treats empty key as unconfigured, bypasses network, loads deterministic fallback, exits `0`.
- **Verification Method**: Run with `GEMINI_API_KEY=""`, verify fallback standup bullets present, exit code `0`.

#### TC-STAND-B02: HTTP 429 Rate Limit Recovery

- **Input**: Gemini API endpoint responds with HTTP 429 Too Many Requests
- **Expected Output**: CLI logs warning to stderr, falls back immediately to deterministic standup, exits code `0`.
- **Verification Method**: Simulate HTTP 429 via mock or unit test, assert graceful degradation to fallback bullets.

#### TC-STAND-B03: HTTP 500 / 503 Server Error Recovery

- **Input**: Gemini API responds with HTTP 503 Service Unavailable
- **Expected Output**: CLI logs warning to stderr, falls back cleanly to deterministic standup, exits code `0`.
- **Verification Method**: Simulate HTTP 503 response, assert clean exit with fallback standup bullets.

#### TC-STAND-B04: Malformed or Empty JSON Response Body

- **Input**: Gemini API returns HTTP 200 with empty body or empty candidate list `{}`
- **Expected Output**: Parser catches empty candidate structure, degrades to deterministic standup without panicking.
- **Verification Method**: Feed empty JSON payload into Gemini parser, verify fallback response returned.

#### TC-STAND-B05: Response Sanitation of Rogue Em-Dashes

- **Input**: Mock Gemini model response containing Unicode em-dash (code point U+2014)
- **Expected Output**: Sanitation layer replaces em-dash with hyphen `-` or colon `:` before rendering.
- **Verification Method**: Pass string containing code point `\u2014` to standup formatter; assert output contains 0 em-dashes.

---

### Tier 2 Feature 4: Deterministic Offline Fallbacks (`F-FALLBACK`)

#### TC-FALL-B01: Whitespace-Only Environment Variables

- **Input**: `GEMINI_API_KEY="   " WAKATIME_API_KEY="   "`
- **Expected Output**: System trims strings, detects invalid credentials, activates fallback mode without making network calls.
- **Verification Method**: Run binary with whitespace-padded keys, verify fallback activated without socket connections.

#### TC-FALL-B02: DNS Resolution Failure Simulation

- **Input**: Network environment where external hostnames cannot be resolved
- **Expected Output**: HTTP client catches connect error, degrades to fallback telemetry, exits code `0`.
- **Verification Method**: Run in environment with unresolvable DNS, verify clean fallback exit.

#### TC-FALL-B03: Partial Upstream Availability (WakaTime OK, Gemini Down)

- **Input**: WakaTime API returns valid stats, Gemini API returns error
- **Expected Output**: Renders live WakaTime HUD alongside deterministic fallback standup without failing entire process.
- **Verification Method**: Mock partial provider failure, verify hybrid output with live HUD and fallback standup.

#### TC-FALL-B04: Partial Upstream Availability (Gemini OK, WakaTime Down)

- **Input**: Gemini API returns valid standup, WakaTime API returns error
- **Expected Output**: Renders deterministic fallback HUD alongside live Gemini standup without failing entire process.
- **Verification Method**: Mock partial provider failure, verify hybrid output with fallback HUD and live standup.

#### TC-FALL-B05: High-Frequency Consecutive Invocations

- **Input**: 10 rapid sequential executions of `slop --offline --dry-run`
- **Expected Output**: Every run succeeds with exit code `0`, identical outputs, zero memory leaks, zero hanging file descriptors.
- **Verification Method**: Loop 10 runs in bash, assert all return exit code `0`.

---

### Tier 2 Feature 5: Tera Template Engine and Dynamic Context Injection (`F-TERA`)

#### TC-TERA-B01: Template Syntax Error Detection

- **Input**: Template file with unclosed tag: `{{ wakatime_hud`
- **Expected Output**: Tera engine catches parse error, CLI logs error with context to stderr, exits with code `1`.
- **Verification Method**: Create malformed template, execute CLI, verify exit code `1`, assert descriptive error in stderr.

#### TC-TERA-B02: Missing Context Variable in Template

- **Input**: Template referencing undefined variable: `{{ non_existent_variable }}`
- **Expected Output**: Tera engine flags undefined variable, exits with code `1` or renders empty string based on strict mode.
- **Verification Method**: Execute CLI against template with undefined variable, assert controlled failure without panic.

#### TC-TERA-B03: Zero-Byte Empty Template File

- **Input**: Empty file `empty_template.md` (0 bytes)
- **Expected Output**: CLI compiles empty template into empty output file without panicking, exits with code `0`.
- **Verification Method**: Run CLI with empty template, verify output file created with 0 bytes and exit code `0`.

#### TC-TERA-B04: Preservation of Complex Raw HTML Blocks

- **Input**: Template containing nested HTML `<details>`, `<summary>`, and `<img>` tags
- **Expected Output**: All HTML tags preserved exactly verbatim in compiled markdown without modification.
- **Verification Method**: Compare HTML blocks in source template and compiled markdown; assert exact identity.

#### TC-TERA-B05: Atomic File Replacement Under Concurrent Read

- **Input**: CLI writes to `README.md` while another process holds an open read descriptor
- **Expected Output**: Atomic rename via temporary file ensures file is never seen in half-written state.
- **Verification Method**: Verify write implementation writes to `.tmp` file and performs atomic rename/replace.

---

### Tier 2 Feature 6: Zero Em-Dash Enforcement (`F-EMDASH`)

#### TC-DASH-B01: Markdown Header With ASCII Hyphen Integrity

- **Input**: Template containing standard hyphens: `# Project - Automated Profile`
- **Expected Output**: Standard ASCII hyphen `-` (`\x2d`) is preserved intact and NOT mistakenly removed.
- **Verification Method**: Check compiled markdown header; assert hyphen `-` is present and intact.

#### TC-DASH-B02: En-Dash Versus Em-Dash Discrimination

- **Input**: Input containing typographical dash characters (code point `\u2013` or `\u2014`)
- **Expected Output**: Sanitation replaces typographical dashes with standard ASCII hyphen `-`.
- **Verification Method**: Pass text containing both dash types through sanitizer; assert zero occurrences of either Unicode dash.

#### TC-DASH-B03: Multi-Byte UTF-8 Byte Stream Boundary Integrity

- **Input**: Injected context with multibyte characters (emojis `⚡`, box characters `━`, block glyphs `█`, `░`)
- **Expected Output**: Unicode multibyte sequences are parsed correctly without corrupting byte streams.
- **Verification Method**: Run byte-level validator on rendered output; assert valid UTF-8 encoding across all lines.

#### TC-DASH-B04: Zero Em-Dashes Across All Workflow YAML Files

- **Input**: `.github/workflows/*.yaml`
- **Expected Output**: Zero em-dash characters across all workflow step names, comments, and echo statements.
- **Verification Method**: Run grep search for `\u2014` across `.github/workflows/`; assert 0 occurrences.

#### TC-DASH-B05: Zero Em-Dashes Across Test Harness and Outputs

- **Input**: `tests/` test runner and test logs
- **Expected Output**: All test runner scripts, log outputs, and summary tables strictly contain zero em-dashes.
- **Verification Method**: Run em-dash scan across all files in `tests/`; assert 0 occurrences.

---

### Tier 2 Feature 7: GitHub Actions Workflows and Actionlint (`F-WORKFLOWS`)

#### TC-WF-B01: Five-Field Cron Expression Syntax

- **Input**: `schedule.cron` in `update-readme.yaml`
- **Expected Output**: Schedule matches standard 5-field cron syntax (`0 0 * * *`) validated by actionlint.
- **Verification Method**: Verify actionlint checks cron schedule string validity; assert 0 warnings/errors.

#### TC-WF-B02: Manual Trigger Workflow Dispatch Block

- **Input**: `on.workflow_dispatch` in `update-readme.yaml`
- **Expected Output**: Empty map or parameter schema under `workflow_dispatch` accepted by actionlint.
- **Verification Method**: Verify actionlint accepts manual trigger configuration.

#### TC-WF-B03: Pinned Action SHA Integrity Validation

- **Input**: Shared TARS workflows (`ci.yaml`, `cargo-crap.yaml`, `sec-codeql.yaml`, `sec-trivy.yaml`)
- **Expected Output**: Actions are pinned to full 40-character commit SHA `@822009e983b73316cbf67a3606842502c7be4599 # v3.3.0`.
- **Verification Method**: Check action references in workflow files; verify SHA matches pinned version.

#### TC-WF-B04: Modern Runner Action Versioning (Deprecation Prevention)

- **Input**: All `uses:` directives across workflows
- **Expected Output**: No deprecated action runners (e.g. `actions/checkout@v2` is rejected; `actions/checkout@v4` is required).
- **Verification Method**: Actionlint validates action versions; assert zero deprecation errors.

#### TC-WF-B05: Repository Secrets References Syntax

- **Input**: Environment mapping in `update-readme.yaml`
- **Expected Output**: Secrets referenced via `${{ secrets.SECRET_NAME }}` syntax conforming to GitHub Actions expression grammar.
- **Verification Method**: Actionlint validates expression syntax in secret parameter mappings.

---

## 5. Tier 3: Cross-Feature Combinations (Pairwise Matrix)

Tier 3 exercises interactions between multiple features executing concurrently to prevent integration bugs and state pollution.

| Combination ID | Interacting Features                 | Test Configuration                                      | Expected Combined Behavior                                                                  |
| -------------- | ------------------------------------ | ------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| `TC-XF-01`     | `F-CLI` + `F-FALLBACK`               | `--offline --dry-run`                                   | Executes offline fallback logic and emits rendered markdown to stdout without touching disk |
| `TC-XF-02`     | `F-CLI` + `F-TERA`                   | `--template custom.md --output out.md --offline`        | Reads custom template, injects fallback variables, writes atomically to custom output path  |
| `TC-XF-03`     | `F-CLI` + `F-STANDUP`                | `--model gemini-3.8-flash --dry-run`                    | Passes model override to standup generator during preview run without altering disk         |
| `TC-XF-04`     | `F-FALLBACK` + `F-HUD` + `F-STANDUP` | Unset API keys + full profile build                     | Injects both deterministic fallback HUD and fallback AI standup into template cleanly       |
| `TC-XF-05`     | `F-TERA` + `F-HUD`                   | Template with `<details>` wrapping `{{ wakatime_hud }}` | Preserves HTML details block while rendering formatted monospaced HUD block inside          |
| `TC-XF-06`     | `F-CLI` + `F-EMDASH`                 | `--dry-run --verbose`                                   | Emits diagnostic logs to stderr and markdown to stdout; both streams contain zero em-dashes |
| `TC-XF-07`     | `F-FALLBACK` + `F-EMDASH`            | Complete offline execution                              | All fallback strings (HUD footer and standup bullets) pass zero em-dash verification        |
| `TC-XF-08`     | `F-CLI` + `F-TERA`                   | Missing template + custom output path                   | Fails at template read step before touching output path, leaving filesystem unmodified      |
| `TC-XF-09`     | `F-WORKFLOWS` + `F-CLI`              | Workflow step invokes `cargo run -p slop-cli`           | Command flags invoked in CI workflow match binary flags supported by CLI parser             |
| `TC-XF-10`     | `F-CLI` + `F-FALLBACK`               | Populated env keys + explicit `--offline` flag          | `--offline` takes strict precedence over populated environment credentials                  |

---

## 6. Tier 4: Real-World Application Scenarios

Tier 4 exercises full end-to-end user journeys simulating operational deployments, developer workflows, and CI automation.

### Scenario 1: Complete End-to-End Profile Compilation (`TC-E2E-01`)

- **Objective**: Verify full compilation pipeline from `README.template.md` to `README.md`.
- **Preconditions**: `README.template.md` exists and contains valid template layout.
- **Execution Steps**:
  1. Invoke `slop --offline --template README.template.md --output README.md`.
  2. Verify process exits with code `0`.
  3. Verify destination file `README.md` exists and is non-empty.
  4. Inspect `README.md` for presence of:
     - Cloud & AI Systems Engineer hero banner
     - Link to `https://skills.mahdtech.com`
     - Tech stack badges via `skillicons.dev`
     - 56-character ASCII compute cycles HUD
     - 3 to 4 bullet AI telemetry standup
     - Collapsible `<details>` accordions for Credly certifications and 3D skyline
  5. Validate entire rendered `README.md` for zero em-dash compliance.

### Scenario 2: Developer Dry-Run Preview (`TC-E2E-02`)

- **Objective**: Verify local developer preview workflow.
- **Preconditions**: Repository workspace checked out.
- **Execution Steps**:
  1. Record current modification timestamp and SHA-256 hash of `README.md`.
  2. Execute `slop --dry-run`.
  3. Verify process exits with code `0`.
  4. Capture stdout and verify it contains fully rendered markdown profile.
  5. Verify `README.md` on disk is completely unmodified (timestamp and hash unchanged).

### Scenario 3: Zero-Credential Automated CI Fallback (`TC-E2E-03`)

- **Objective**: Verify automated build execution in unconfigured CI runner or PR fork where repository secrets are unavailable.
- **Preconditions**: Subshell environment with all tokens cleared (`unset GEMINI_API_KEY WAKATIME_API_KEY MAHDTECH_GITHUB_TOKEN GITHUB_TOKEN`).
- **Execution Steps**:
  1. Execute `slop --output README.md`.
  2. Verify process completes in under 3 seconds with exit code `0`.
  3. Verify rendered `README.md` contains valid fallback HUD and standup.
  4. Verify no panics or unhandled exceptions in stderr.

### Scenario 4: GitHub Actions Workflow Validation Suite (`TC-E2E-04`)

- **Objective**: Verify all repository workflow files satisfy GitHub Actions schema and security policies.
- **Preconditions**: Workflow files exist in `.github/workflows/`.
- **Execution Steps**:
  1. Locate `actionlint` binary in environment.
  2. Run `actionlint` across all workflow YAML files in `.github/workflows/`.
  3. Assert exit code `0` with zero syntax, runner, or expression errors.
  4. Verify shared TARS action invocations pin commit SHAs (`@822009e983b73316cbf67a3606842502c7be4599`).

### Scenario 5: Repository-Wide Zero Em-Dash Enforcement Audit (`TC-E2E-05`)

- **Objective**: Guarantee zero typographical em-dash regressions across all repository assets.
- **Preconditions**: Full repository workspace.
- **Execution Steps**:
  1. Perform recursive Unicode scan across all markdown files (`*.md`), workflow files (`.github/workflows/*.yaml`), templates (`*.template.md`), and test scripts.
  2. Test specifically for byte pattern `\xe2\x80\x94` (Unicode `U+2014`).
  3. Assert zero occurrences across entire tracked codebase.

---

## 7. Test Execution and Harness Architecture

The test harness is implemented in `tests/e2e_runner.sh` and provides structured execution for all four tiers.

### Harness Commands

```bash
# Execute entire test suite across all four tiers
tests/e2e_runner.sh --all

# Execute specific testing tier
tests/e2e_runner.sh --tier 1
tests/e2e_runner.sh --tier 2
tests/e2e_runner.sh --tier 3
tests/e2e_runner.sh --tier 4

# Execute specific test case by ID
tests/e2e_runner.sh --case TC-CLI-01
```

### Progressive Testability Strategy

During development before the `slop` binary is fully compiled:

1. The test runner checks for the compiled binary at `target/release/slop` or `target/debug/slop`, or via `cargo run -p slop-cli --`.
2. If the binary is not yet compiled, the runner compiles it on-demand or executes validation against the standalone specification harness.
3. Once the crate is built, the runner executes all opaque-box CLI assertions against the real binary.
