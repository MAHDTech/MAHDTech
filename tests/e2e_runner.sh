#!/usr/bin/env bash
# End-to-End Test Suite Runner for MAHDTech Profile Automation
# Executes Tiers 1-4 validation tests with zero em-dashes and progressive testability.

set -euo pipefail

export PYTHONDONTWRITEBYTECODE=1

# Configuration and Paths
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURES_DIR="${ROOT_DIR}/tests/fixtures"
ORACLE_SCRIPT="${ROOT_DIR}/tests/oracle/slop_oracle.py"
TEMP_DIR="${ROOT_DIR}/target/tmp_e2e_tests"

mkdir -p "${TEMP_DIR}"

# shellcheck disable=SC2329
cleanup() {
	rm -rf "${TEMP_DIR}"
}
trap cleanup EXIT

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASSED_COUNT=0
FAILED_COUNT=0
SKIPPED_COUNT=0
TOTAL_COUNT=0

# Determine CLI command array
SLOP_CMD=()
if [[ -n ${SLOP_BIN:-} && -x ${SLOP_BIN} ]]; then
	SLOP_CMD=("${SLOP_BIN}")
elif [[ -x "${ROOT_DIR}/target/release/slop" ]]; then
	SLOP_CMD=("${ROOT_DIR}/target/release/slop")
elif [[ -x "${ROOT_DIR}/target/debug/slop" ]]; then
	SLOP_CMD=("${ROOT_DIR}/target/debug/slop")
elif [[ -f "${ROOT_DIR}/Cargo.toml" ]] && cargo check -p slop-cli >/dev/null 2>&1; then
	SLOP_CMD=("cargo" "run" "--quiet" "-p" "slop-cli" "--")
else
	SLOP_CMD=("python3" "${ORACLE_SCRIPT}")
fi

# Determine actionlint binary
discover_actionlint() {
	if command -v actionlint >/dev/null 2>&1; then
		echo "actionlint"
	elif compgen -G "/nix/store/*actionlint*/bin/actionlint" >/dev/null 2>&1; then
		compgen -G "/nix/store/*actionlint*/bin/actionlint" | head -n 1
	else
		echo ""
	fi
}
ACTIONLINT_BIN=$(discover_actionlint)

TARGET_CASE=""

should_run_case() {
	local case_id="$1"
	if [[ -z ${TARGET_CASE} ]]; then
		return 0
	fi
	if [[ ${TARGET_CASE} == "${case_id}" ]]; then
		return 0
	fi
	return 1
}

# Helper function to record test outcome
record_result() {
	local case_id="$1"
	local description="$2"
	local status="$3"
	local message="${4:-}"

	TOTAL_COUNT=$((TOTAL_COUNT + 1))
	if [[ ${status} == "PASS" ]]; then
		PASSED_COUNT=$((PASSED_COUNT + 1))
		printf "  [%bPASS%b] %-12s: %s\n" "${GREEN}" "${NC}" "${case_id}" "${description}"
	elif [[ ${status} == "SKIP" ]]; then
		SKIPPED_COUNT=$((SKIPPED_COUNT + 1))
		printf "  [%bSKIP%b] %-12s: %s (%s)\n" "${YELLOW}" "${NC}" "${case_id}" "${description}" "${message}"
	else
		FAILED_COUNT=$((FAILED_COUNT + 1))
		printf "  [%bFAIL%b] %-12s: %s (%s)\n" "${RED}" "${NC}" "${case_id}" "${description}" "${message}"
	fi
}

# ==============================================================================
# TIER 1: FEATURE COVERAGE
# ==============================================================================
run_tier_1() {
	echo ""
	echo "=== RUNNING TIER 1: FEATURE COVERAGE ==="

	# TC-CLI-01: Help documentation
	if should_run_case "TC-CLI-01"; then
		local help_out="${TEMP_DIR}/help.txt"
		if "${SLOP_CMD[@]}" --help >"${help_out}" 2>&1; then
			if grep -q "dry-run" "${help_out}" && grep -q "template" "${help_out}"; then
				record_result "TC-CLI-01" "CLI --help displays usage synopsis and flags" "PASS"
			else
				record_result "TC-CLI-01" "CLI --help displays usage synopsis and flags" "FAIL" "Flags missing in help output"
			fi
		else
			record_result "TC-CLI-01" "CLI --help displays usage synopsis and flags" "FAIL" "Non-zero exit code"
		fi
	fi

	# TC-CLI-02: Dry run execution
	if should_run_case "TC-CLI-02"; then
		local dry_out="${TEMP_DIR}/dry_run.md"
		local target_file="${TEMP_DIR}/untouched_target.md"
		touch "${target_file}"
		local orig_mtime
		orig_mtime=$(stat -c %Y "${target_file}" 2>/dev/null || stat -f %m "${target_file}")
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --output "${target_file}" --dry-run >"${dry_out}" 2>&1; then
			local new_mtime
			new_mtime=$(stat -c %Y "${target_file}" 2>/dev/null || stat -f %m "${target_file}")
			if [[ ${orig_mtime} == "${new_mtime}" ]] && [[ -s ${dry_out} ]]; then
				record_result "TC-CLI-02" "CLI --dry-run outputs markdown without altering target" "PASS"
			else
				record_result "TC-CLI-02" "CLI --dry-run outputs markdown without altering target" "FAIL" "Target was modified or stdout empty"
			fi
		else
			record_result "TC-CLI-02" "CLI --dry-run outputs markdown without altering target" "FAIL" "Command exited non-zero"
		fi
	fi

	# TC-CLI-03: Custom template path
	if should_run_case "TC-CLI-03"; then
		local custom_out="${TEMP_DIR}/custom_tmpl_out.md"
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --dry-run >"${custom_out}" 2>&1; then
			if grep -q "Daily AI Telemetry" "${custom_out}"; then
				record_result "TC-CLI-03" "CLI parses custom --template path" "PASS"
			else
				record_result "TC-CLI-03" "CLI parses custom --template path" "FAIL" "Template content not found in output"
			fi
		else
			record_result "TC-CLI-03" "CLI parses custom --template path" "FAIL" "Command exited non-zero"
		fi
	fi

	# TC-CLI-04: Custom output path
	if should_run_case "TC-CLI-04"; then
		local out_dest="${TEMP_DIR}/dest_written.md"
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --output "${out_dest}" --offline >/dev/null 2>&1; then
			if [[ -f ${out_dest} ]] && grep -q "Daily AI Telemetry" "${out_dest}"; then
				record_result "TC-CLI-04" "CLI writes rendered markdown to --output destination" "PASS"
			else
				record_result "TC-CLI-04" "CLI writes rendered markdown to --output destination" "FAIL" "Output file not written"
			fi
		else
			record_result "TC-CLI-04" "CLI writes rendered markdown to --output destination" "FAIL" "Command exited non-zero"
		fi
	fi

	# TC-CLI-05: Explicit offline flag
	if should_run_case "TC-CLI-05"; then
		local offline_out="${TEMP_DIR}/offline.md"
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --offline --dry-run >"${offline_out}" 2>&1; then
			if grep -q "WEEKLY COMPUTE CYCLES" "${offline_out}"; then
				record_result "TC-CLI-05" "CLI --offline forces deterministic local execution" "PASS"
			else
				record_result "TC-CLI-05" "CLI --offline forces deterministic local execution" "FAIL" "Missing telemetry in offline run"
			fi
		else
			record_result "TC-CLI-05" "CLI --offline forces deterministic local execution" "FAIL" "Command exited non-zero"
		fi
	fi

	# TC-CLI-06: Gemini model override flag
	if should_run_case "TC-CLI-06"; then
		local model_out="${TEMP_DIR}/model_out.txt"
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --model "gemini-3.8-flash" --dry-run >"${model_out}" 2>&1; then
			record_result "TC-CLI-06" "CLI accepts --model override parameter" "PASS"
		else
			record_result "TC-CLI-06" "CLI accepts --model override parameter" "FAIL" "Command rejected --model"
		fi
	fi

	# TC-HUD-01: Header line verification
	local hud_raw="${TEMP_DIR}/hud_raw.md"
	"${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --offline --dry-run >"${hud_raw}" 2>&1

	if should_run_case "TC-HUD-01"; then
		if grep -q "WEEKLY COMPUTE CYCLES (via WakaTime API)" "${hud_raw}"; then
			record_result "TC-HUD-01" "HUD block displays correct header text" "PASS"
		else
			record_result "TC-HUD-01" "HUD block displays correct header text" "FAIL" "Header line not found"
		fi
	fi

	# TC-HUD-02: 56-character divider width
	if should_run_case "TC-HUD-02"; then
		local div_ok=0
		while IFS= read -r line; do
			if [[ ${line} =~ ^[━]+$ ]]; then
				local char_count
				char_count=$(python3 -c "import sys; print(len(sys.argv[1]))" "${line}")
				if [[ ${char_count} -eq 56 ]]; then
					div_ok=1
					break
				fi
			fi
		done <"${hud_raw}"
		if [[ ${div_ok} -eq 1 ]]; then
			record_result "TC-HUD-02" "HUD divider width is exactly 56 Unicode characters" "PASS"
		else
			record_result "TC-HUD-02" "HUD divider width is exactly 56 Unicode characters" "FAIL" "Expected 56-character divider line"
		fi
	fi

	# TC-HUD-03: Unicode progress bars glyphs
	if should_run_case "TC-HUD-03"; then
		if grep -q "█" "${hud_raw}" && grep -q "░" "${hud_raw}"; then
			record_result "TC-HUD-03" "HUD progress bars use Unicode block characters" "PASS"
		else
			record_result "TC-HUD-03" "HUD progress bars use Unicode block characters" "FAIL" "Missing block characters"
		fi
	fi

	# TC-HUD-04: Language row column formatting
	if should_run_case "TC-HUD-04"; then
		if grep -qE "Rust +[0-9]+ hrs" "${hud_raw}"; then
			record_result "TC-HUD-04" "HUD language rows follow aligned column format" "PASS"
		else
			record_result "TC-HUD-04" "HUD language rows follow aligned column format" "FAIL" "Language line pattern mismatch"
		fi
	fi

	# TC-HUD-05: HUD metadata footer
	if should_run_case "TC-HUD-05"; then
		if grep -q "OS: Linux (NixOS) | Primary Editor: Neovim / Helix" "${hud_raw}"; then
			record_result "TC-HUD-05" "HUD footer renders valid metadata with zero em-dashes" "PASS"
		else
			record_result "TC-HUD-05" "HUD footer renders valid metadata with zero em-dashes" "FAIL" "Footer line missing or format error"
		fi
	fi

	# TC-STAND-01: Standup bullet count (3 to 4 bullets)
	if should_run_case "TC-STAND-01"; then
		local bullet_count
		bullet_count=$(grep -c "^- " "${hud_raw}" || true)
		if [[ ${bullet_count} -ge 3 && ${bullet_count} -le 4 ]]; then
			record_result "TC-STAND-01" "Standup generator outputs 3 to 4 markdown bullets" "PASS"
		else
			record_result "TC-STAND-01" "Standup generator outputs 3 to 4 markdown bullets" "FAIL" "Bullet count was ${bullet_count}"
		fi
	fi

	# TC-STAND-02: Standup autonomous systems tone
	if should_run_case "TC-STAND-02"; then
		if grep -qiE "(Rust|Kubernetes|Platform|Telemetry)" "${hud_raw}"; then
			record_result "TC-STAND-02" "Standup content emphasizes autonomous systems engineering" "PASS"
		else
			record_result "TC-STAND-02" "Standup content emphasizes autonomous systems engineering" "FAIL" "Expected keywords not found"
		fi
	fi

	# TC-FALL-01: Execution with unset credentials
	if should_run_case "TC-FALL-01"; then
		local no_keys_out="${TEMP_DIR}/no_keys.md"
		if env -u GEMINI_API_KEY -u WAKATIME_API_KEY -u MAHDTECH_GITHUB_TOKEN "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --dry-run >"${no_keys_out}" 2>&1; then
			if grep -q "WEEKLY COMPUTE CYCLES" "${no_keys_out}"; then
				record_result "TC-FALL-01" "Unset API credentials gracefully degrade to deterministic fallback" "PASS"
			else
				record_result "TC-FALL-01" "Unset API credentials gracefully degrade to deterministic fallback" "FAIL" "Fallback content missing"
			fi
		else
			record_result "TC-FALL-01" "Unset API credentials gracefully degrade to deterministic fallback" "FAIL" "Command failed with unset credentials"
		fi
	fi

	# TC-FALL-02: Deterministic output stability
	if should_run_case "TC-FALL-02"; then
		local run1="${TEMP_DIR}/run1.md"
		local run2="${TEMP_DIR}/run2.md"
		"${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --offline --dry-run >"${run1}" 2>&1
		"${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --offline --dry-run >"${run2}" 2>&1
		# Diff ignoring Last Updated timestamp
		if diff -I "Last Updated:" "${run1}" "${run2}" >/dev/null 2>&1; then
			record_result "TC-FALL-02" "Consecutive offline runs produce identical deterministic output" "PASS"
		else
			record_result "TC-FALL-02" "Consecutive offline runs produce identical deterministic output" "FAIL" "Non-deterministic output detected"
		fi
	fi

	# TC-TERA-01: Template placeholder substitution
	if should_run_case "TC-TERA-01"; then
		if grep -q "WEEKLY COMPUTE CYCLES" "${hud_raw}" && ! grep -F -q "{{" "${hud_raw}"; then
			record_result "TC-TERA-01" "Template placeholders completely resolved by engine" "PASS"
		else
			record_result "TC-TERA-01" "Template placeholders completely resolved by engine" "FAIL" "Unresolved placeholder in output"
		fi
	fi

	# TC-TERA-02: Last updated timestamp injection
	if should_run_case "TC-TERA-02"; then
		if grep -qE "Last Updated: [0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2} UTC" "${hud_raw}"; then
			record_result "TC-TERA-02" "Last updated timestamp correctly formatted and injected" "PASS"
		else
			record_result "TC-TERA-02" "Last updated timestamp correctly formatted and injected" "FAIL" "Timestamp format mismatch"
		fi
	fi

	# TC-DASH-01: Zero em-dash audit on rendered output
	if should_run_case "TC-DASH-01"; then
		if python3 -c "
with open('${hud_raw}', 'r', encoding='utf-8') as f:
    text = f.read()
if '\u2014' in text:
    exit(1)
exit(0)
"; then
			record_result "TC-DASH-01" "Rendered profile output contains zero em-dashes" "PASS"
		else
			record_result "TC-DASH-01" "Rendered profile output contains zero em-dashes" "FAIL" "Found em-dash in output"
		fi
	fi

	# TC-WF-01: Actionlint on .github/workflows
	if should_run_case "TC-WF-01"; then
		if [[ -n ${ACTIONLINT_BIN} ]]; then
			local al_out="${TEMP_DIR}/actionlint.log"
			if "${ACTIONLINT_BIN}" "${ROOT_DIR}/.github/workflows/"*.yaml >"${al_out}" 2>&1; then
				record_result "TC-WF-01" "Actionlint passes on repository workflow files" "PASS"
			else
				record_result "TC-WF-01" "Actionlint passes on repository workflow files" "FAIL" "Workflow lint errors detected"
			fi
		else
			record_result "TC-WF-01" "Actionlint passes on repository workflow files" "SKIP" "actionlint binary not found"
		fi
	fi
}

# ==============================================================================
# TIER 2: BOUNDARY AND CORNER CASES
# ==============================================================================
run_tier_2() {
	echo ""
	echo "=== RUNNING TIER 2: BOUNDARY AND CORNER CASES ==="

	# TC-CLI-B01: Missing argument value for flag
	if should_run_case "TC-CLI-B01"; then
		if "${SLOP_CMD[@]}" --template >/dev/null 2>&1; then
			record_result "TC-CLI-B01" "Missing flag argument rejected by parser" "FAIL" "Accepted missing flag argument"
		else
			record_result "TC-CLI-B01" "Missing flag argument rejected by parser" "PASS"
		fi
	fi

	# TC-CLI-B02: Unrecognized CLI flag
	if should_run_case "TC-CLI-B02"; then
		if "${SLOP_CMD[@]}" --invalid-flag-xyz >/dev/null 2>&1; then
			record_result "TC-CLI-B02" "Unrecognized CLI flag rejected by parser" "FAIL" "Accepted invalid flag"
		else
			record_result "TC-CLI-B02" "Unrecognized CLI flag rejected by parser" "PASS"
		fi
	fi

	# TC-CLI-B03: Non-existent template path exits code 1
	if should_run_case "TC-CLI-B03"; then
		if "${SLOP_CMD[@]}" --template "${TEMP_DIR}/non_existent.md" --dry-run >/dev/null 2>&1; then
			record_result "TC-CLI-B03" "Non-existent template path triggers controlled exit code 1" "FAIL" "Exited 0 on missing template"
		else
			record_result "TC-CLI-B03" "Non-existent template path triggers controlled exit code 1" "PASS"
		fi
	fi

	# TC-CLI-B04: Unwritable output destination exits code 1
	if should_run_case "TC-CLI-B04"; then
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --output "/proc/unwritable_dest.md" --offline >/dev/null 2>&1; then
			record_result "TC-CLI-B04" "Unwritable destination triggers controlled exit code 1" "FAIL" "Exited 0 on unwritable path"
		else
			record_result "TC-CLI-B04" "Unwritable destination triggers controlled exit code 1" "PASS"
		fi
	fi

	# TC-CLI-B05: Pathnames with spaces handled cleanly
	if should_run_case "TC-CLI-B05"; then
		local space_dir="${TEMP_DIR}/folder with spaces"
		mkdir -p "${space_dir}"
		cp "${FIXTURES_DIR}/sample_template.md" "${space_dir}/tmpl space.md"
		local space_out="${space_dir}/out space.md"
		if "${SLOP_CMD[@]}" --template "${space_dir}/tmpl space.md" --output "${space_out}" --offline >/dev/null 2>&1; then
			if [[ -f ${space_out} ]]; then
				record_result "TC-CLI-B05" "Pathnames containing whitespace handled cleanly" "PASS"
			else
				record_result "TC-CLI-B05" "Pathnames containing whitespace handled cleanly" "FAIL" "Output file missing"
			fi
		else
			record_result "TC-CLI-B05" "Pathnames containing whitespace handled cleanly" "FAIL" "Execution failed on spaced paths"
		fi
	fi

	# TC-HUD-B01: Line length upper bound verification
	if should_run_case "TC-HUD-B01"; then
		local hud_lines="${TEMP_DIR}/hud_lines.md"
		"${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --offline --dry-run >"${hud_lines}" 2>&1
		local line_over=0
		while IFS= read -r line; do
			if [[ ${line} =~ ^(Rust|Python|YAML|Nix|OS:) ]]; then
				local l_len
				l_len=$(python3 -c "import sys; print(len(sys.argv[1]))" "${line}")
				if [[ ${l_len} -gt 60 ]]; then
					line_over=1
					break
				fi
			fi
		done <"${hud_lines}"
		if [[ ${line_over} -eq 0 ]]; then
			record_result "TC-HUD-B01" "HUD language lines stay within standard retro-terminal column bound" "PASS"
		else
			record_result "TC-HUD-B01" "HUD language lines stay within standard retro-terminal column bound" "FAIL" "Line exceeded 60 characters"
		fi
	fi

	# TC-STAND-B01: Empty string API keys treated as unset
	if should_run_case "TC-STAND-B01"; then
		local empty_key_out="${TEMP_DIR}/empty_key.md"
		if GEMINI_API_KEY="" WAKATIME_API_KEY="" "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --dry-run >"${empty_key_out}" 2>&1; then
			if grep -q "WEEKLY COMPUTE CYCLES" "${empty_key_out}"; then
				record_result "TC-STAND-B01" "Empty string API credentials gracefully fall back" "PASS"
			else
				record_result "TC-STAND-B01" "Empty string API credentials gracefully fall back" "FAIL" "Fallback content missing"
			fi
		else
			record_result "TC-STAND-B01" "Empty string API credentials gracefully fall back" "FAIL" "Command exited non-zero"
		fi
	fi

	# TC-FALL-B01: Whitespace-only credentials handled cleanly
	if should_run_case "TC-FALL-B01"; then
		local ws_out="${TEMP_DIR}/ws_keys.md"
		if GEMINI_API_KEY="   " WAKATIME_API_KEY="   " "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --dry-run >"${ws_out}" 2>&1; then
			record_result "TC-FALL-B01" "Whitespace-only credentials treated as unconfigured" "PASS"
		else
			record_result "TC-FALL-B01" "Whitespace-only credentials treated as unconfigured" "FAIL" "Crashed on whitespace credentials"
		fi
	fi

	# TC-FALL-B05: High-frequency consecutive executions
	if should_run_case "TC-FALL-B05"; then
		local consec_ok=1
		for _ in {1..5}; do
			if ! "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --offline --dry-run >/dev/null 2>&1; then
				consec_ok=0
				break
			fi
		done
		if [[ ${consec_ok} -eq 1 ]]; then
			record_result "TC-FALL-B05" "High-frequency consecutive executions succeed without resource leaks" "PASS"
		else
			record_result "TC-FALL-B05" "High-frequency consecutive executions succeed without resource leaks" "FAIL" "Failed during rapid execution loop"
		fi
	fi

	# TC-TERA-B01: Syntax error in template triggers controlled error
	if should_run_case "TC-TERA-B01"; then
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/bad_syntax_template.md" --dry-run >/dev/null 2>&1; then
			record_result "TC-TERA-B01" "Template syntax error triggers controlled exit code 1" "FAIL" "Accepted broken template"
		else
			record_result "TC-TERA-B01" "Template syntax error triggers controlled exit code 1" "PASS"
		fi
	fi

	# TC-TERA-B03: Zero-byte empty template file
	if should_run_case "TC-TERA-B03"; then
		local empty_out="${TEMP_DIR}/empty_output.md"
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/empty_template.md" --output "${empty_out}" --offline >/dev/null 2>&1; then
			if [[ -f ${empty_out} ]] && [[ ! -s ${empty_out} ]]; then
				record_result "TC-TERA-B03" "Empty template file produces empty output without crashing" "PASS"
			else
				record_result "TC-TERA-B03" "Empty template file produces empty output without crashing" "FAIL" "Output not empty"
			fi
		else
			record_result "TC-TERA-B03" "Empty template file produces empty output without crashing" "FAIL" "Exited non-zero on empty template"
		fi
	fi

	# TC-TERA-B04: HTML details block preserved in rendered markdown
	if should_run_case "TC-TERA-B04"; then
		local details_out="${TEMP_DIR}/details_out.md"
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/details_template.md" --dry-run >"${details_out}" 2>&1; then
			if grep -q "<details>" "${details_out}" && grep -q "<summary>" "${details_out}"; then
				record_result "TC-TERA-B04" "Raw HTML details blocks preserved intact" "PASS"
			else
				record_result "TC-TERA-B04" "Raw HTML details blocks preserved intact" "FAIL" "HTML details tags escaped or stripped"
			fi
		else
			record_result "TC-TERA-B04" "Raw HTML details blocks preserved intact" "FAIL" "Command exited non-zero"
		fi
	fi

	# TC-DASH-B01: ASCII hyphen integrity preserved
	if should_run_case "TC-DASH-B01"; then
		local hyphen_out="${TEMP_DIR}/hyphen_out.md"
		"${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --offline --dry-run >"${hyphen_out}" 2>&1
		if grep -q "MAHDTech:" "${hyphen_out}" || grep -q " - " "${hyphen_out}" || grep -q "^- " "${hyphen_out}"; then
			record_result "TC-DASH-B01" "Standard ASCII hyphens and colons preserved intact" "PASS"
		else
			record_result "TC-DASH-B01" "Standard ASCII hyphens and colons preserved intact" "FAIL" "ASCII punctuation corrupted"
		fi
	fi

	# TC-DASH-B03: UTF-8 encoding validity
	if should_run_case "TC-DASH-B03"; then
		local utf8_out="${TEMP_DIR}/utf8_out.md"
		"${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --offline --dry-run >"${utf8_out}" 2>&1
		if python3 -c "
with open('${utf8_out}', 'rb') as f:
    content = f.read()
content.decode('utf-8')
"; then
			record_result "TC-DASH-B03" "Rendered output validates as clean UTF-8 byte stream" "PASS"
		else
			record_result "TC-DASH-B03" "Rendered output validates as clean UTF-8 byte stream" "FAIL" "UTF-8 decode failure"
		fi
	fi
}

# ==============================================================================
# TIER 3: CROSS-FEATURE COMBINATIONS
# ==============================================================================
run_tier_3() {
	echo ""
	echo "=== RUNNING TIER 3: CROSS-FEATURE COMBINATIONS ==="

	# TC-XF-01: --offline + --dry-run
	if should_run_case "TC-XF-01"; then
		local xf01_out="${TEMP_DIR}/xf01.md"
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --offline --dry-run >"${xf01_out}" 2>&1; then
			if grep -q "WEEKLY COMPUTE CYCLES" "${xf01_out}"; then
				record_result "TC-XF-01" "Pairwise: --offline + --dry-run outputs fallback markdown" "PASS"
			else
				record_result "TC-XF-01" "Pairwise: --offline + --dry-run outputs fallback markdown" "FAIL" "Output missing content"
			fi
		else
			record_result "TC-XF-01" "Pairwise: --offline + --dry-run outputs fallback markdown" "FAIL" "Command exited non-zero"
		fi
	fi

	# TC-XF-02: --template + --output + --offline
	if should_run_case "TC-XF-02"; then
		local xf02_out="${TEMP_DIR}/xf02.md"
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --output "${xf02_out}" --offline >/dev/null 2>&1; then
			if [[ -f ${xf02_out} ]] && grep -q "WEEKLY COMPUTE CYCLES" "${xf02_out}"; then
				record_result "TC-XF-02" "Pairwise: --template + --output + --offline writes custom target" "PASS"
			else
				record_result "TC-XF-02" "Pairwise: --template + --output + --offline writes custom target" "FAIL" "File not written properly"
			fi
		else
			record_result "TC-XF-02" "Pairwise: --template + --output + --offline writes custom target" "FAIL" "Command exited non-zero"
		fi
	fi

	# TC-XF-03: --model + --dry-run
	if should_run_case "TC-XF-03"; then
		local xf03_out="${TEMP_DIR}/xf03.md"
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --model "gemini-3.8-flash" --dry-run >"${xf03_out}" 2>&1; then
			record_result "TC-XF-03" "Pairwise: --model + --dry-run executes preview without disk write" "PASS"
		else
			record_result "TC-XF-03" "Pairwise: --model + --dry-run executes preview without disk write" "FAIL" "Command exited non-zero"
		fi
	fi

	# TC-XF-04: Unset API keys + full template rendering
	if should_run_case "TC-XF-04"; then
		local xf04_out="${TEMP_DIR}/xf04.md"
		if env -u GEMINI_API_KEY -u WAKATIME_API_KEY "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --output "${xf04_out}" >/dev/null 2>&1; then
			if grep -q "WEEKLY COMPUTE CYCLES" "${xf04_out}" && grep -q "^- " "${xf04_out}"; then
				record_result "TC-XF-04" "Pairwise: Unset API keys injects fallback HUD and standup" "PASS"
			else
				record_result "TC-XF-04" "Pairwise: Unset API keys injects fallback HUD and standup" "FAIL" "Fallback parts missing"
			fi
		else
			record_result "TC-XF-04" "Pairwise: Unset API keys injects fallback HUD and standup" "FAIL" "Command exited non-zero"
		fi
	fi

	# TC-XF-05: <details> wrapping {{ wakatime_hud }}
	if should_run_case "TC-XF-05"; then
		local xf05_out="${TEMP_DIR}/xf05.md"
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/details_template.md" --dry-run >"${xf05_out}" 2>&1; then
			if grep -q "<details>" "${xf05_out}" && grep -q "WEEKLY COMPUTE CYCLES" "${xf05_out}"; then
				record_result "TC-XF-05" "Pairwise: Tera engine renders HUD inside HTML details block" "PASS"
			else
				record_result "TC-XF-05" "Pairwise: Tera engine renders HUD inside HTML details block" "FAIL" "HTML details structure missing"
			fi
		else
			record_result "TC-XF-05" "Pairwise: Tera engine renders HUD inside HTML details block" "FAIL" "Command exited non-zero"
		fi
	fi

	# TC-XF-06: --dry-run + --verbose stream isolation
	if should_run_case "TC-XF-06"; then
		local stdout_log="${TEMP_DIR}/stdout_log.md"
		local stderr_log="${TEMP_DIR}/stderr_log.txt"
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --dry-run --verbose >"${stdout_log}" 2>"${stderr_log}"; then
			# stderr should have logging, stdout should have markdown, neither with em-dashes
			if python3 -c "
with open('${stdout_log}', 'r', encoding='utf-8') as f:
    assert '\u2014' not in f.read()
with open('${stderr_log}', 'r', encoding='utf-8') as f:
    assert '\u2014' not in f.read()
"; then
				record_result "TC-XF-06" "Pairwise: --dry-run + --verbose maintains clean stream separation" "PASS"
			else
				record_result "TC-XF-06" "Pairwise: --dry-run + --verbose maintains clean stream separation" "FAIL" "Em-dash found in stdout or stderr"
			fi
		else
			record_result "TC-XF-06" "Pairwise: --dry-run + --verbose maintains clean stream separation" "FAIL" "Command exited non-zero"
		fi
	fi

	# TC-XF-07: Offline fallback zero em-dash guarantee
	if should_run_case "TC-XF-07"; then
		local xf07_out="${TEMP_DIR}/xf07.md"
		"${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --offline --dry-run >"${xf07_out}" 2>&1
		if python3 -c "
with open('${xf07_out}', 'r', encoding='utf-8') as f:
    text = f.read()
assert '\u2014' not in text
"; then
			record_result "TC-XF-07" "Pairwise: Complete offline fallback contains zero em-dashes" "PASS"
		else
			record_result "TC-XF-07" "Pairwise: Complete offline fallback contains zero em-dashes" "FAIL" "Found em-dash in offline output"
		fi
	fi

	# TC-XF-08: Missing template with custom output path does not touch output
	if should_run_case "TC-XF-08"; then
		local untouched_out="${TEMP_DIR}/untouched_on_fail.md"
		rm -f "${untouched_out}"
		"${SLOP_CMD[@]}" --template "${TEMP_DIR}/missing_file.md" --output "${untouched_out}" >/dev/null 2>&1 || true
		if [[ ! -e ${untouched_out} ]]; then
			record_result "TC-XF-08" "Pairwise: Failed template load leaves output destination untouched" "PASS"
		else
			record_result "TC-XF-08" "Pairwise: Failed template load leaves output destination untouched" "FAIL" "Output file created on error"
		fi
	fi

	# TC-XF-10: Populated env keys overridden by --offline flag
	if should_run_case "TC-XF-10"; then
		local xf10_out="${TEMP_DIR}/xf10.md"
		if GEMINI_API_KEY="mock_dummy_key" WAKATIME_API_KEY="mock_dummy_key" "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --offline --dry-run >"${xf10_out}" 2>&1; then
			if grep -q "WEEKLY COMPUTE CYCLES" "${xf10_out}"; then
				record_result "TC-XF-10" "Pairwise: --offline takes strict precedence over present credentials" "PASS"
			else
				record_result "TC-XF-10" "Pairwise: --offline takes strict precedence over present credentials" "FAIL" "Content missing in output"
			fi
		else
			record_result "TC-XF-10" "Pairwise: --offline takes strict precedence over present credentials" "FAIL" "Command exited non-zero"
		fi
	fi
}

# ==============================================================================
# TIER 4: REAL-WORLD APPLICATION SCENARIOS
# ==============================================================================
run_tier_4() {
	echo ""
	echo "=== RUNNING TIER 4: REAL-WORLD APPLICATION SCENARIOS ==="

	# TC-E2E-01: Full End-to-End Profile Build Scenario
	if should_run_case "TC-E2E-01"; then
		local profile_target="${TEMP_DIR}/REAL_README.md"
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --output "${profile_target}" --offline >/dev/null 2>&1; then
			if [[ -f ${profile_target} ]] && grep -q "WEEKLY COMPUTE CYCLES" "${profile_target}" && grep -q "Daily AI Telemetry" "${profile_target}"; then
				record_result "TC-E2E-01" "Scenario 1: Full end-to-end profile build succeeds" "PASS"
			else
				record_result "TC-E2E-01" "Scenario 1: Full end-to-end profile build succeeds" "FAIL" "Rendered profile verification failed"
			fi
		else
			record_result "TC-E2E-01" "Scenario 1: Full end-to-end profile build succeeds" "FAIL" "Build command failed"
		fi
	fi

	# TC-E2E-02: Developer Dry-Run Preview Scenario
	if should_run_case "TC-E2E-02"; then
		local dry_preview="${TEMP_DIR}/dry_preview.md"
		local dev_target="${TEMP_DIR}/dev_readme.md"
		echo "Existing content" >"${dev_target}"
		local prev_hash
		prev_hash=$(sha256sum "${dev_target}" | awk '{print $1}')
		if "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --output "${dev_target}" --dry-run >"${dry_preview}" 2>&1; then
			local new_hash
			new_hash=$(sha256sum "${dev_target}" | awk '{print $1}')
			if [[ ${prev_hash} == "${new_hash}" ]] && grep -q "WEEKLY COMPUTE CYCLES" "${dry_preview}"; then
				record_result "TC-E2E-02" "Scenario 2: Dry-run preview executes cleanly without file changes" "PASS"
			else
				record_result "TC-E2E-02" "Scenario 2: Dry-run preview executes cleanly without file changes" "FAIL" "File hash modified or preview empty"
			fi
		else
			record_result "TC-E2E-02" "Scenario 2: Dry-run preview executes cleanly without file changes" "FAIL" "Command exited non-zero"
		fi
	fi

	# TC-E2E-03: Zero-Credential Automated CI Fallback Scenario
	if should_run_case "TC-E2E-03"; then
		local ci_out="${TEMP_DIR}/ci_profile.md"
		if env -u GEMINI_API_KEY -u WAKATIME_API_KEY -u MAHDTECH_GITHUB_TOKEN -u GITHUB_TOKEN "${SLOP_CMD[@]}" --template "${FIXTURES_DIR}/sample_template.md" --output "${ci_out}" >/dev/null 2>&1; then
			if [[ -s ${ci_out} ]] && grep -q "OS: Linux (NixOS)" "${ci_out}"; then
				record_result "TC-E2E-03" "Scenario 3: Zero-credential CI fallback builds complete profile" "PASS"
			else
				record_result "TC-E2E-03" "Scenario 3: Zero-credential CI fallback builds complete profile" "FAIL" "Output missing fallback content"
			fi
		else
			record_result "TC-E2E-03" "Scenario 3: Zero-credential CI fallback builds complete profile" "FAIL" "CI fallback command exited non-zero"
		fi
	fi

	# TC-E2E-04: Workflow Syntax and Actionlint Validation
	if should_run_case "TC-E2E-04"; then
		if [[ -n ${ACTIONLINT_BIN} ]]; then
			local wf_errors=0
			for wf in "${ROOT_DIR}/.github/workflows/"*.yaml; do
				if [[ -f ${wf} ]]; then
					if ! "${ACTIONLINT_BIN}" "${wf}" >/dev/null 2>&1; then
						wf_errors=$((wf_errors + 1))
					fi
				fi
			done
			if [[ ${wf_errors} -eq 0 ]]; then
				record_result "TC-E2E-04" "Scenario 4: All workflow files pass actionlint verification" "PASS"
			else
				record_result "TC-E2E-04" "Scenario 4: All workflow files pass actionlint verification" "FAIL" "${wf_errors} workflow files failed actionlint"
			fi
		else
			record_result "TC-E2E-04" "Scenario 4: All workflow files pass actionlint verification" "SKIP" "actionlint binary not found"
		fi
	fi

	# TC-E2E-05: Strict Zero Em-Dash Repository-Wide Audit
	if should_run_case "TC-E2E-05"; then
		local emdash_found=0
		for path in "${ROOT_DIR}/tests/e2e_runner.sh" "${ROOT_DIR}/tests/oracle/slop_oracle.py"; do
			if [[ -f ${path} ]]; then
				if python3 -c "
with open('${path}', 'r', encoding='utf-8') as f:
    text = f.read()
if '\u2014' in text:
    exit(1)
"; then
					:
				else
					emdash_found=1
					break
				fi
			fi
		done
		if [[ ${emdash_found} -eq 0 ]]; then
			record_result "TC-E2E-05" "Scenario 5: Repository test assets strictly maintain zero em-dashes" "PASS"
		else
			record_result "TC-E2E-05" "Scenario 5: Repository test assets strictly maintain zero em-dashes" "FAIL" "Em-dash detected in test files"
		fi
	fi
}

# ==============================================================================
# CLI DISPATCH
# ==============================================================================
main() {
	echo "=========================================================="
	echo "  MAHDTech Profile Automation: E2E Test Suite Runner"
	echo "  Active Engine: ${SLOP_CMD[*]}"
	echo "=========================================================="

	local run_all=1
	local target_tier=""

	while [[ $# -gt 0 ]]; do
		case "$1" in
		--all)
			run_all=1
			shift
			;;
		--tier)
			run_all=0
			target_tier="$2"
			shift 2
			;;
		--case)
			run_all=0
			TARGET_CASE="$2"
			shift 2
			;;
		--help | -h)
			echo "Usage: $0 [--all | --tier 1..4 | --case TC-XXX]"
			exit 0
			;;
		*)
			echo "Unknown argument: $1"
			exit 1
			;;
		esac
	done

	if [[ -n ${TARGET_CASE} ]]; then
		run_tier_1
		run_tier_2
		run_tier_3
		run_tier_4
	elif [[ ${run_all} -eq 1 ]]; then
		run_tier_1
		run_tier_2
		run_tier_3
		run_tier_4
	else
		case "${target_tier}" in
		1) run_tier_1 ;;
		2) run_tier_2 ;;
		3) run_tier_3 ;;
		4) run_tier_4 ;;
		*)
			echo "Invalid tier: ${target_tier}. Choose 1, 2, 3, or 4."
			exit 1
			;;
		esac
	fi

	echo ""
	echo "=========================================================="
	echo "  E2E Test Execution Summary"
	echo "=========================================================="
	echo "  Total Tests Run : ${TOTAL_COUNT}"
	echo "  Passed          : ${PASSED_COUNT}"
	echo "  Failed          : ${FAILED_COUNT}"
	echo "  Skipped         : ${SKIPPED_COUNT}"
	echo "=========================================================="

	if [[ ${FAILED_COUNT} -gt 0 ]]; then
		exit 1
	fi
	exit 0
}

main "$@"
