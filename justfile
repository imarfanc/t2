# macOS reset
# Requires: https://github.com/casey/just — `brew install just`
# Run `just` to list recipes.

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Default: show available recipes
_default:
    @just --choose

# --- Root server ---

# Serve with startup and connection diagnostics enabled.
serve-debug:
    @echo "repo-server preflight"
    @echo "  time:  $(date '+%Y-%m-%d %H:%M:%S %Z')"
    @echo "  cwd:   {{justfile_directory()}}"
    @echo "  rustc: $(rustc --version)"
    @echo "  cargo: $(cargo --version)"
    @echo "  open files limit: $(ulimit -n)"
    @if command -v lsof >/dev/null 2>&1; then \
        listeners="$(lsof -nP -iTCP:8008 -sTCP:LISTEN 2>/dev/null || true)"; \
        if [ -n "$listeners" ]; then \
            echo "  port 8008 is already in use:"; \
            echo "$listeners" | sed 's/^/    /'; \
        else \
            echo "  port 8008: available"; \
        fi; \
    fi
    REPO_SERVER_DEBUG=1 cargo run --release --manifest-path server/Cargo.toml -- "{{justfile_directory()}}"

# Serve the repo at http://localhost:8008.
serve:
    cargo run --release --manifest-path server/Cargo.toml -- "{{justfile_directory()}}"

# --- TUI demo ---

# Build and open the single-file Rust Markdown TUI.
test-tui:
    scripts/test-tui/test.rs
