# macOS reset
# Requires: https://github.com/casey/just — `brew install just`
# Run `just` to list recipes.

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# Default: show available recipes
default:
    @just --choose

# --- Root server ---
