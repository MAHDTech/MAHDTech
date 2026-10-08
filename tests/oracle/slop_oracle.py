#!/usr/bin/env python3
"""Authoritative Reference Oracle for slop CLI.

Implements the CLI interface contract and formatting specification defined in
PROJECT.md, ORIGINAL_REQUEST.md, and GOAL.md.
Used for differential testing and progressive testability verification.
"""

import argparse
import datetime
import os
import re
import sys


def generate_wakatime_hud():
    divider = "\u2501" * 56
    lines = [
        "\u26a1 WEEKLY COMPUTE CYCLES (via WakaTime API)",
        divider,
        "Rust          28 hrs 40 mins  [\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2588\u2591\u2591\u2591\u2591\u2591\u2591] 58.2%",
        "Python         8 hrs 15 mins  [\u2588\u2588\u2588\u2588\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591] 16.7%",
        "YAML / K8s     5 hrs 10 mins  [\u2588\u2588\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591] 10.5%",
        "Nix            3 hrs 45 mins  [\u2588\u2588\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591\u2591]  7.6%",
        divider,
        "OS: Linux (NixOS) | Primary Editor: Neovim / Helix",
    ]
    return "\n".join(lines)


def generate_ai_standup():
    bullets = [
        "- Telemetry: Active compute cycles concentrated on low-latency Rust systems and autonomous agent frameworks.",
        "- Platform: Advancing Kubernetes platform deployments and infrastructure as code automation.",
        "- Momentum: Continuous integration verification and automated profile compiler pipeline operational.",
        "- Focus: Scaling multi-agent coordination tooling and resilient cloud-native workflows.",
    ]
    return "\n".join(bullets)


def render_template(template_content, context):
    # Check for unclosed tag syntax error
    # Count opening vs closing braces
    open_count = len(re.findall(r"\{\{", template_content))
    close_count = len(re.findall(r"\}\}", template_content))
    if open_count != close_count:
        raise ValueError("Syntax error: unclosed Tera tag detected in template")

    rendered = template_content
    for key, value in context.items():
        pattern = r"\{\{\s*" + re.escape(key) + r"\s*\}\}"
        rendered = re.sub(pattern, value, rendered)
    return rendered


def main():
    parser = argparse.ArgumentParser(
        prog="slop",
        description="Automated high-impact GitHub profile compiler",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview rendered output to stdout without modifying target files",
    )
    parser.add_argument(
        "--template",
        "-t",
        default="README.template.md",
        help="Path to the source profile template file",
    )
    parser.add_argument(
        "--output",
        "-o",
        default="README.md",
        help="Destination path for the compiled output file",
    )
    parser.add_argument(
        "--offline",
        action="store_true",
        help="Bypass external network calls and force deterministic offline mock data",
    )
    parser.add_argument(
        "--model",
        default=os.environ.get("GEMINI_MODEL", "gemini-3.8-flash"),
        help="Override the Gemini model name",
    )
    parser.add_argument(
        "--verbose",
        "-v",
        action="store_true",
        help="Enable verbose logging to stderr",
    )

    args = parser.parse_args()

    if args.verbose:
        sys.stderr.write(f"[slop] Active model: {args.model}\n")
        sys.stderr.write(f"[slop] Offline mode: {args.offline}\n")
        sys.stderr.write(f"[slop] Template path: {args.template}\n")
        sys.stderr.write(f"[slop] Output destination: {args.output}\n")

    if not os.path.exists(args.template):
        sys.stderr.write(f"Error: Template file not found: {args.template}\n")
        sys.exit(1)

    try:
        with open(args.template, "r", encoding="utf-8") as f:
            template_content = f.read()
    except Exception as e:
        sys.stderr.write(f"Error reading template {args.template}: {e}\n")
        sys.exit(1)

    now_utc = datetime.datetime.now(datetime.timezone.utc).strftime(
        "%Y-%m-%d %H:%M UTC"
    )

    context = {
        "wakatime_hud": generate_wakatime_hud(),
        "ai_standup": generate_ai_standup(),
        "last_updated": now_utc,
        "cli_version": "0.1.0",
    }

    try:
        rendered = render_template(template_content, context)
    except Exception as e:
        sys.stderr.write(f"Template rendering error: {e}\n")
        sys.exit(1)

    # Strictly assert zero em-dashes in rendered text
    if "\u2014" in rendered:
        sys.stderr.write("Fatal: Rogue em-dash detected in output stream\n")
        sys.exit(1)

    if args.dry_run:
        sys.stdout.write(rendered)
        sys.exit(0)

    output_dir = os.path.dirname(args.output)
    if output_dir and not os.path.exists(output_dir):
        try:
            os.makedirs(output_dir, exist_ok=True)
        except Exception as e:
            sys.stderr.write(f"Error creating output directory {output_dir}: {e}\n")
            sys.exit(1)

    # Atomic write simulation via tmp file
    tmp_path = f"{args.output}.tmp"
    try:
        with open(tmp_path, "w", encoding="utf-8") as f:
            f.write(rendered)
        os.replace(tmp_path, args.output)
    except Exception as e:
        sys.stderr.write(f"Error writing output to {args.output}: {e}\n")
        if os.path.exists(tmp_path):
            os.remove(tmp_path)
        sys.exit(1)

    sys.exit(0)


if __name__ == "__main__":
    main()
