{
  pkgs,
  config,
  lib,
  ...
}:
let
  devPackages = with pkgs; [
    cargo-audit
    cargo-edit
    cargo-deny
    cargo-llvm-cov
    cargo-nextest
    cargo-watch
    clang
    codeql
    convco
    figlet
    git
    hello
    jq
    mold
    ripgrep
    trivy
  ];
in
{
  name = "MAHDTech";

  env = {
    PROJECT = config.name;
    RUST_BACKTRACE = "1";
    RUST_LOG = "info";
  };

  cachix = {
    enable = true;
    pull = [
      "mahdtech"
    ];
    push = "mahdtech";
  };

  devenv = {
    warnOnNewVersion = true;
  };

  packages = devPackages;

  enterShell = ''
    if [[ "''${CI:-false}" == "true" ]]; then
      echo "devenv running in CI"
    else
      figlet -f slant -w 180 "$(echo "$PROJECT" | tr '[:lower:]-' '[:upper:] ')"

      hello --greeting="Hello ''${USER:-user}, welcome to the $PROJECT project."

      ${lib.optionalString (config.scripts != { }) ''
        echo ""
        echo "#########################"
        echo "#### Helper scripts #####"
        echo "#########################"
        echo "🦾"
        ${lib.concatStrings (
          lib.mapAttrsToList (
            name: value:
            "printf '🦾 %-20s  %s\\n' '${name}' '${
              if value.description != null then value.description else ""
            }'\n"
          ) config.scripts
        )}
        echo "🦾"
        echo "#########################"
      ''}
    fi
    echo -e "\nAGENT_SKILLS_HOME is set to ''${AGENT_SKILLS_HOME:-(not set!)}"
  '';

  languages = {
    nix = {
      enable = true;
    };
    shell = {
      enable = true;
    };
    rust = {
      enable = true;
      toolchainFile = ./rust-toolchain.toml;
      mold.enable = pkgs.stdenv.hostPlatform.isLinux;
    };
  };

  git-hooks = {
    excludes = [
      "^target/"
      "^.agents/"
      "^.tars/"
      ".agents/"
      ".devenv/"
      "^.vscode/"
      "/resources/"
      "^\\.cache/"
    ];
    hooks = {
      action-validator.enable = true;
      actionlint.enable = true;
      cargo-check.enable = true;
      check-json.enable = true;
      check-merge-conflicts.enable = true;
      check-shebang-scripts-are-executable = {
        enable = true;
        excludes = [
          "\\.rs$"
        ];
      };
      check-symlinks.enable = true;
      check-yaml.enable = true;
      clippy.enable = true;
      commitizen.enable = true;
      convco = {
        enable = true;
        settings = {
          configPath = ".versionrc";
        };
      };
      cspell = {
        enable = true;
        excludes = [
          "\\.versionrc$"
        ];
        args = [
          "lint"
          "--no-must-find-files"
        ];
      };
      deadnix.enable = true;
      editorconfig-checker.enable = true;
      lychee = {
        enable = true;
        excludes = [
          "\\.rs$"
        ];
        settings = {
          configPath = "lychee.toml";
          flags = "--exclude 'dist/.*' --exclude 'profile-3d-contrib/.*'";
        };
      };
      markdownlint = {
        enable = true;
        excludes = [
        ];
        settings = {
          configuration = {
            MD013 = false;
            MD025 = false;
            MD036 = false;
            MD041 = false;
            MD051 = false;
            MD033 = {
              allowed_elements = [
                "a"
                "b"
                "br"
                "h3"
                "nobr"
                "pre"
                "sup"
                "summary"
                "details"
                "ParamField"
                "Expandable"
                "Warning"
                "ResponseField"
                "span"
                "Card"
                "Note"
                "Info"
                "Steps"
                "Step"
                "Icon"
                "img"
                "mandate"
                "constraints"
                "instructions"
                "exit_criteria"
              ];
            };
          };
        };
      };
      mixed-line-endings.enable = true;
      nixfmt.enable = true;
      prettier = {
        enable = true;
        excludes = [
          "\\.devcontainer\\.json$"
          "\\.devcontainer/devcontainer\\.json$"
        ];
      };
      ripsecrets.enable = true;
      rustfmt.enable = true;
      shellcheck.enable = true;
      shfmt.enable = true;
      trim-trailing-whitespace = {
        enable = true;
        excludes = [
        ];
      };
      trufflehog = {
        enable = true;
        excludes = [
        ];
      };
      yamllint = {
        enable = true;
        settings = {
          configuration = ''
            extends: relaxed
            rules:
              line-length: disable
              indentation: enable
          '';
        };
      };
    };
  };

  starship = {
    enable = true;
    config.enable = false;
  };

  devcontainer = {
    enable = true;
    settings = {
      customizations = {
        vscode = {
          extensions = [
          ];
        };
      };
    };
  };

  scripts = { };

  enterTest = ''
    if [ -f Cargo.toml ]; then
      cargo test --workspace;
    fi
  '';

  ## Outputs
  # Recipe lives in packages/slop.nix and builds from pkgs alone. This output
  # is the local/CI build handle (devenv build outputs.slop, pushed to Cachix).
  #
  # Downstream projects can import packages/slop.nix directly.
  outputs.slop = import ./packages/slop.nix { inherit pkgs; };
}
