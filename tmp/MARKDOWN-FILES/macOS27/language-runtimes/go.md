---
title: "go"
sort: 1
category: "language-runtimes"
description: "Go runtime installation via Homebrew or cURL"
date: 2026-5-1
tags:
  - go
  - golang
  - install
---

# Go Installation

For directory scanning utilities implemented in Go, see [go-scanner.md](../tools/directory-scanner/go-scanner.md).

## Using Homebrew

```sh
brew install go
```

## Using cURL Script

Paste this script to download, extract, and configure official Go binaries on macOS (arm64/amd64):

```sh
zsh <<'ZSH'
set -euo pipefail
setopt interactivecomments 2>/dev/null || true

bold=$'\033[1m'
green=$'\033[32m'
yellow=$'\033[33m'
red=$'\033[31m'
blue=$'\033[34m'
reset=$'\033[0m'

ok()   { printf "%s✓%s %s\n" "$green" "$reset" "$*"; }
warn() { printf "%s!%s %s\n" "$yellow" "$reset" "$*"; }
fail() { printf "%s✗%s %s\n" "$red" "$reset" "$*"; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "Missing required command: $1"
}

need_cmd curl
need_cmd tar
need_cmd awk
need_cmd sudo
need_cmd uname

ARCH="$(uname -m)"

case "$ARCH" in
  arm64)
    GOARCH="arm64"
    ;;
  x86_64)
    GOARCH="amd64"
    ;;
  *)
    fail "Unsupported Mac architecture: $ARCH"
    ;;
esac

workdir="$HOME/Developer/go-tmp"
mkdir -p "$workdir"

printf "\n%sGo Official Installer for macOS%s\n" "$bold" "$reset"
printf "%s──────────────────────────────%s\n\n" "$blue" "$reset"

ok "Detected architecture: $ARCH → Go $GOARCH"

VERSION="$(
  curl -fsSL --retry 3 --connect-timeout 15 'https://go.dev/VERSION?m=text' \
    | awk 'NR == 1 { print; exit }'
)"

case "$VERSION" in
  go[0-9]*)
    ok "Latest Go version: $VERSION"
    ;;
  *)
    fail "Could not detect valid Go version. Got: ${VERSION:-empty}"
    ;;
esac

TARBALL_URL="https://go.dev/dl/${VERSION}.darwin-${GOARCH}.tar.gz"
TARBALL="$workdir/go.tar.gz"

printf "\nDownloading:\n%s\n\n" "$TARBALL_URL"

curl -fL --retry 3 --connect-timeout 15 "$TARBALL_URL" -o "$TARBALL"

test -s "$TARBALL" || fail "Download failed or file is empty."

ok "Downloaded tarball: $(du -h "$TARBALL" | awk '{print $1}')"

printf "\nRequesting sudo once...\n"
sudo -v

printf "\nInstalling to /usr/local/go ...\n"

sudo rm -rf /usr/local/go
sudo tar -C /usr/local -xzf "$TARBALL"
sudo chown -R root:wheel /usr/local/go 2>/dev/null || true

test -x /usr/local/go/bin/go || fail "Go binary was not installed correctly."

ok "Installed Go into /usr/local/go"

PROFILE="$HOME/.zprofile"
touch "$PROFILE"

START_MARKER="# >>> official Go path >>>"
END_MARKER="# <<< official Go path <<<"

cleaned_profile="$workdir/zprofile.cleaned"

awk -v start="$START_MARKER" -v end="$END_MARKER" '
  $0 == start { skip = 1; next }
  $0 == end { skip = 0; next }
  skip != 1 { print }
' "$PROFILE" > "$cleaned_profile"

cat "$cleaned_profile" > "$PROFILE"

cat >> "$PROFILE" <<'EOF'

# >>> official Go path >>>
export PATH="/usr/local/go/bin:$PATH"
# <<< official Go path <<<
EOF

export PATH="/usr/local/go/bin:$PATH"
hash -r 2>/dev/null || true
rehash 2>/dev/null || true

GO_BIN="$(command -v go || true)"
GO_VERSION="$(go version 2>/dev/null || true)"
GO_ROOT="$(go env GOROOT 2>/dev/null || true)"
GO_PATH="$(go env GOPATH 2>/dev/null || true)"
GO_ENV_ARCH="$(go env GOARCH 2>/dev/null || true)"
GO_ENV_OS="$(go env GOOS 2>/dev/null || true)"

printf "\n%sVisual Sanity Check%s\n" "$bold" "$reset"
printf "%s────────────────────%s\n" "$blue" "$reset"

printf "%-22s %s\n" "Expected version:" "$VERSION"
printf "%-22s %s\n" "Active go:" "${GO_BIN:-not found}"
printf "%-22s %s\n" "go version:" "${GO_VERSION:-failed}"
printf "%-22s %s\n" "GOROOT:" "${GO_ROOT:-failed}"
printf "%-22s %s\n" "GOPATH:" "${GO_PATH:-failed}"
printf "%-22s %s\n" "GOOS / GOARCH:" "${GO_ENV_OS:-?} / ${GO_ENV_ARCH:-?}"

printf "\n%sPATH priority check%s\n" "$bold" "$reset"
printf "%s──────────────────%s\n" "$blue" "$reset"
which -a go 2>/dev/null | awk '{ printf "%2d. %s\n", NR, $0 }' || true

printf "\n"

if [ "$GO_BIN" = "/usr/local/go/bin/go" ]; then
  ok "Good: /usr/local/go/bin/go is first in PATH."
else
  warn "Go installed, but another go is first in PATH."
  warn "Open a new terminal tab, then run: which go && go version"
fi

if printf "%s" "$GO_VERSION" | grep -q "$VERSION"; then
  ok "Version check passed."
else
  fail "Version mismatch. Expected $VERSION but got: ${GO_VERSION:-nothing}"
fi

printf "\n%sDone.%s Open a new terminal tab and run:\n\n" "$green" "$reset"
printf "  go version\n"
printf "  which go\n\n"
ZSH
```
