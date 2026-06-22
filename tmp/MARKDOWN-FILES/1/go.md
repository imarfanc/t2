---
title: "go"
sort: 4
category: "macOS reset"
description: "install go"
date: 2026-5-1
tags:
    - macOS
    - reset
    - go
    - install
    - golang
---

# install go

## using brew

```sh
brew install go
```

## using curl

Your error happened because **interactive zsh treats `#` as a command unless
`interactivecomments` is enabled**. Best fix: paste the whole installer as a
**heredoc** so comments are safe.

On zsh the version URL **must** use single quotes (`'...?m=text'`) or `?`
triggers a glob error — the script below already does that.

Paste this whole thing:

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

### Why this is better

- Keeps downloads and temp files under **`~/Developer/go-tmp`** (created if
  missing; not deleted when the script finishes)
- Works safely when pasted into **zsh**
- Avoids the `zsh: command not found: #` problem
- Detects Apple Silicon vs Intel Mac
- Downloads the latest official Go release
- Installs to `/usr/local/go`
- Cleans old duplicate Go PATH blocks from `~/.zprofile`
- Forces `/usr/local/go/bin` to the front of PATH
- Shows a sanity-check table at the end
- Shows all `go` binaries found with `which -a go` so you can spot conflicts
  with Homebrew or old installs.

## charm stack (optional)

All from [Charm](https://charm.sh) — tools for pretty terminal UIs:

| Tool                                                         | What it is                                                       | When you use it                           |
| ------------------------------------------------------------ | ---------------------------------------------------------------- | ----------------------------------------- |
| **[lipgloss](https://github.com/charmbracelet/lipgloss)**    | Go library — borders, colors, tables (like CSS for the terminal) | Inside Go programs (this scanner uses it) |
| **[Bubble Tea](https://github.com/charmbracelet/bubbletea)** | Go framework — event loop for interactive TUIs                   | Full apps (menus, forms, live updates)    |
| **[Bubbles](https://github.com/charmbracelet/bubbles)**      | Go components for Bubble Tea — spinner, text input, lists, etc.  | Building blocks inside Bubble Tea apps    |

**This doc:** the backup scanner pulls **lipgloss** + **bubbles/spinner** via
`go get` — **brew or curl Go** is all you need.

## backup dir

Same directory scanner as [deno.md](./deno.md) and [uv.md](../1/uv.md); uses
**lipgloss** + **Bubble Tea spinner** (Charm Go libs, like uv uses Rich).
Dependencies are fetched with `go get` — works with brew or curl Go.
`OUTPUT_DIR` is resolved with `os.UserHomeDir()`. The shell snippet writes
`main.go` and the module under **`~/Developer/go-tmp`** (created if missing;
left on disk after the run).

```sh
TMP="$HOME/Developer/go-tmp"
mkdir -p "$TMP"
cat > "$TMP/main.go" <<'GO'
package main

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"regexp"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"time"

	"github.com/charmbracelet/bubbles/spinner"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/charmbracelet/lipgloss/table"
)

const (
	defaultScanRoot       = "."
	outputBasename        = "tree_output"
	outputExt             = ".txt"
	showHidden            = true
	followSymlinks        = false
	scanEverything        = false
	maxDepth              = 50
	topNFiles             = 25
	topNDirs              = 50
	saveOutput            = true
	appendTimestampSuffix = true
	hardDepthLimit        = 200
)

var (
	outputDir = filepath.Join(mustHome(), "Developer", "macos-reset")

	excludeDirNames = map[string]struct{}{
		".git":         {},
		"node_modules": {},
		".venv":        {},
		"__pycache__":  {},
		".DS_Store":    {},
	}

	icon = map[string]string{
		"section": "▾",
		"dir":     "□",
		"file":    "▪",
		"size":    "◌",
		"root":    "◆",
	}

	ansiRe = regexp.MustCompile(`\x1b\[[0-9;]*m`)

	headingStyle = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("212"))
	panelStyle   = lipgloss.NewStyle().
			Border(lipgloss.RoundedBorder()).
			BorderForeground(lipgloss.Color("212")).
			Padding(1, 2).
			Bold(true)
	savedStyle = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("82"))
	errStyle   = lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("1"))
)

type entryInfo struct {
	path  string
	name  string
	isDir bool
	size  int64
}

type fileSize struct {
	path string
	size int64
}

type scanData struct {
	rootPath         string
	rootTotalSize    int64
	rootEntries      []entryInfo
	treeLines        []string
	fileCount        int
	dirCount         int
	scannedFileBytes int64
	maxDepthReached  int
	largestFiles     []fileSize
	dirSizes         map[string]int64
}

func scanRootFromEnv() string {
	if v := strings.TrimSpace(os.Getenv("SCAN_ROOT")); v != "" {
		return v
	}
	return defaultScanRoot
}

func main() {
	rootArg := scanRootFromEnv()
	rootPath, err := realRoot(rootArg)
	if err != nil {
		fmt.Fprintln(os.Stderr, errStyle.Render("Error: '"+rootArg+"' is not a valid directory."))
		os.Exit(1)
	}

	mode := fmt.Sprintf("max depth %d", maxDepth)
	if scanEverything {
		mode = "scan everything"
	}

	fmt.Println()
	fmt.Println(panelStyle.Render(strings.Join([]string{
		fmt.Sprintf("%s dir scanner", icon["root"]),
		fmt.Sprintf("%s root  %s", icon["dir"], rootPath),
		fmt.Sprintf("%s mode  %s", icon["size"], mode),
	}, "\n")))
	fmt.Println()

	var data scanData
	withSpinner("scanning…", func() {
		data = scanTree(rootPath)
	})

	outputPath := ""
	if saveOutput {
		outputPath = makeOutputPath()
	}

	depthLabel := fmt.Sprintf("%d (max)", maxDepth)
	if scanEverything {
		depthLabel = fmt.Sprintf("%d (actual)", data.maxDepthReached)
	}

	memRaw := run([]string{"sysctl", "-n", "hw.memsize"})
	memBytes, _ := strconv.ParseInt(memRaw, 10, 64)

	topFiles := data.largestFiles
	if len(topFiles) > topNFiles {
		topFiles = topFiles[:topNFiles]
	}

	type dirSize struct {
		path string
		size int64
	}
	var topDirs []dirSize
	for p, size := range data.dirSizes {
		if p != data.rootPath {
			topDirs = append(topDirs, dirSize{p, size})
		}
	}
	sort.Slice(topDirs, func(i, j int) bool { return topDirs[i].size > topDirs[j].size })
	if len(topDirs) > topNDirs {
		topDirs = topDirs[:topNDirs]
	}

	systemInfoTable := renderSection(
		fmt.Sprintf("%s System Info", icon["section"]),
		[]string{"Field", "Value"},
		[][]string{
			{"User", run([]string{"whoami"})},
			{"Hostname", run([]string{"hostname"})},
			{"OS", run([]string{"sw_vers", "-productName"})},
			{"OS Version", run([]string{"sw_vers", "-productVersion"})},
			{"Build", run([]string{"sw_vers", "-buildVersion"})},
			{"Platform", fmt.Sprintf("%s %s", runtime.GOOS, runtime.GOARCH)},
			{"User Lang", envOr("LANG", "—")},
			{"Processor", run([]string{"sysctl", "-n", "machdep.cpu.brand_string"})},
			{"Cores", run([]string{"sysctl", "-n", "hw.ncpu"})},
			{"Memory", humanSize(memBytes)},
			{"Uptime", strings.TrimSpace(run([]string{"uptime"}))},
			{"Date", time.Now().Format(time.RFC1123)},
			{"Home", envOr("HOME", "—")},
			{"Shell", envOr("SHELL", "—")},
			{"Term", envOr("TERM", "—")},
			{"Go", run([]string{"go", "version"})},
		},
	)

	summaryTable := renderSection(
		fmt.Sprintf("%s Summary", icon["section"]),
		[]string{"Field", "Value"},
		[][]string{
			{"Root", data.rootPath},
			{"Depth", depthLabel},
			{"Directories", strconv.Itoa(data.dirCount)},
			{"Files", strconv.Itoa(data.fileCount)},
			{"Scanned File Size", humanSize(data.scannedFileBytes)},
			{"Root Total Size", humanSize(data.rootTotalSize)},
			{"Root Items", strconv.Itoa(len(data.rootEntries))},
			{"Show Hidden", yesNo(showHidden)},
			{"Follow Symlinks", yesNo(followSymlinks)},
			{"Top Files Count", strconv.Itoa(topNFiles)},
			{"Top Dirs Count", strconv.Itoa(topNDirs)},
			{"Save Output", yesNo(saveOutput)},
			{"Output Dir", outputDir},
			{"Output File", orDash(outputPath)},
			{"Generated", time.Now().Format(time.RFC1123)},
		},
	)

	rootRows := make([][]string, 0, len(data.rootEntries))
	for i, e := range data.rootEntries {
		typ := fmt.Sprintf("%s file", icon["file"])
		name := e.name
		size := humanSize(e.size)
		if e.isDir {
			typ = fmt.Sprintf("%s dir", icon["dir"])
			name = e.name + "/"
			size = humanSize(data.dirSizes[e.path])
		}
		rootRows = append(rootRows, []string{
			fmt.Sprintf("%d", i+1),
			typ,
			name,
			size,
		})
	}
	rootTable := renderSection(
		fmt.Sprintf("%s Root Contents", icon["section"]),
		[]string{"#", "Type", "Name", "Size"},
		rootRows,
	)

	topFilesRows := make([][]string, 0, len(topFiles))
	for i, f := range topFiles {
		topFilesRows = append(topFilesRows, []string{
			fmt.Sprintf("%d", i+1),
			fmt.Sprintf("%s %s", icon["file"], filepath.Base(f.path)),
			relFromRoot(data.rootPath, f.path),
			humanSize(f.size),
		})
	}
	topFilesTable := renderSection(
		fmt.Sprintf("%s Top %d Largest Files", icon["section"], topNFiles),
		[]string{"#", "File", "Relative Path", "Size"},
		topFilesRows,
	)

	topDirsRows := make([][]string, 0, len(topDirs))
	for i, d := range topDirs {
		topDirsRows = append(topDirsRows, []string{
			fmt.Sprintf("%d", i+1),
			fmt.Sprintf("%s %s/", icon["dir"], filepath.Base(d.path)),
			relFromRoot(data.rootPath, d.path),
			humanSize(d.size),
		})
	}
	topDirsTable := renderSection(
		fmt.Sprintf("%s Top %d Largest Directories", icon["section"], topNDirs),
		[]string{"#", "Directory", "Relative Path", "Total Size"},
		topDirsRows,
	)

	treeHeader := headingStyle.Render(fmt.Sprintf("%s Full Tree", icon["section"])) + "\n" +
		headingStyle.Render(fmt.Sprintf("%s %s  %s %s", icon["root"], data.rootPath, icon["size"], humanSize(data.rootTotalSize)))

	finalOutput := strings.Join([]string{
		systemInfoTable,
		"",
		summaryTable,
		"",
		rootTable,
		"",
		topFilesTable,
		"",
		topDirsTable,
		"",
		treeHeader,
		strings.Join(data.treeLines, "\n"),
	}, "\n")

	fmt.Println(finalOutput)

	if saveOutput && outputPath != "" {
		if err := os.MkdirAll(filepath.Dir(outputPath), 0o755); err != nil {
			fmt.Fprintf(os.Stderr, "%s\n", err)
			os.Exit(1)
		}
		if err := os.WriteFile(outputPath, []byte(stripANSI(finalOutput)), 0o644); err != nil {
			fmt.Fprintf(os.Stderr, "%s\n", err)
			os.Exit(1)
		}
		fmt.Println()
		fmt.Println(savedStyle.Render("Saved: " + outputPath))
	}
}

func scanTree(rootPath string) scanData {
	var (
		fileCount        int
		dirCount         int
		scannedFileBytes int64
		maxDepthReached  int
		largestFiles     []fileSize
		dirSizes         = map[string]int64{}
		visitedDirs      = map[string]struct{}{}
	)

	var scanDirSize func(dir string) int64
	scanDirSize = func(dir string) int64 {
		real, _ := filepath.EvalSymlinks(dir)
		if real == "" {
			real = dir
		}
		if _, ok := visitedDirs[real]; ok {
			return 0
		}
		visitedDirs[real] = struct{}{}

		var total int64
		for _, entry := range listDir(dir) {
			if entry.isDir {
				total += scanDirSize(entry.path)
			} else {
				total += entry.size
			}
		}
		dirSizes[dir] = total
		return total
	}

	rootTotalSize := scanDirSize(rootPath)
	rootEntries := listDir(rootPath)

	var buildTreeLines func(dir string, depth int, prefix string, visited map[string]struct{}) []string
	buildTreeLines = func(dir string, depth int, prefix string, visited map[string]struct{}) []string {
		if depth >= hardDepthLimit {
			return nil
		}
		if !scanEverything && depth >= maxDepth {
			return nil
		}
		if depth > maxDepthReached {
			maxDepthReached = depth
		}

		entries := listDir(dir)
		var lines []string
		for i, entry := range entries {
			isLast := i == len(entries)-1
			connector := "├── "
			nextPrefix := prefix + "│   "
			if isLast {
				connector = "└── "
				nextPrefix = prefix + "    "
			}

			if entry.isDir {
				real, _ := filepath.EvalSymlinks(entry.path)
				if real == "" {
					real = entry.path
				}
				if _, ok := visited[real]; ok {
					continue
				}
				visited[real] = struct{}{}

				dirCount++
				size := dirSizes[entry.path]
				lines = append(lines, prefix+connector+
					fmt.Sprintf("%s %s/", icon["dir"], entry.name)+"  "+
					fmt.Sprintf("%s %s", icon["size"], humanSize(size)))
				lines = append(lines, buildTreeLines(entry.path, depth+1, nextPrefix, visited)...)
			} else {
				fileCount++
				scannedFileBytes += entry.size
				largestFiles = append(largestFiles, fileSize{entry.path, entry.size})
				lines = append(lines, prefix+connector+
					fmt.Sprintf("%s %s", icon["file"], entry.name)+"  "+
					fmt.Sprintf("%s %s", icon["size"], humanSize(entry.size)))
			}
		}
		return lines
	}

	treeLines := buildTreeLines(rootPath, 0, "", map[string]struct{}{})
	sort.Slice(largestFiles, func(i, j int) bool { return largestFiles[i].size > largestFiles[j].size })

	return scanData{
		rootPath:         rootPath,
		rootTotalSize:    rootTotalSize,
		rootEntries:      rootEntries,
		treeLines:        treeLines,
		fileCount:        fileCount,
		dirCount:         dirCount,
		scannedFileBytes: scannedFileBytes,
		maxDepthReached:  maxDepthReached,
		largestFiles:     largestFiles,
		dirSizes:         dirSizes,
	}
}

func mustHome() string {
	home, err := os.UserHomeDir()
	if err != nil {
		panic(err)
	}
	return home
}

func renderTable(headers []string, rows [][]string) string {
	t := table.New().
		Border(lipgloss.RoundedBorder()).
		BorderStyle(lipgloss.NewStyle().Foreground(lipgloss.Color("240"))).
		Headers(headers...).
		StyleFunc(func(row, col int) lipgloss.Style {
			if row == table.HeaderRow {
				return lipgloss.NewStyle().Bold(true).Foreground(lipgloss.Color("86"))
			}
			return lipgloss.NewStyle().Foreground(lipgloss.Color("252"))
		})
	for _, row := range rows {
		cells := make([]string, len(headers))
		for i := range headers {
			if i < len(row) {
				cells[i] = row[i]
			}
		}
		t = t.Row(cells...)
	}
	return t.String()
}

func renderSection(title string, headers []string, rows [][]string) string {
	return headingStyle.Render(title) + "\n" + renderTable(headers, rows)
}

type spinDone struct{}

type spinModel struct {
	spinner spinner.Model
	title   string
	done    <-chan struct{}
}

func (m spinModel) Init() tea.Cmd {
	return tea.Batch(m.spinner.Tick, waitSpinDone(m.done))
}

func waitSpinDone(done <-chan struct{}) tea.Cmd {
	return func() tea.Msg {
		<-done
		return spinDone{}
	}
}

func (m spinModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg.(type) {
	case spinDone:
		return m, tea.Quit
	case tea.KeyMsg:
		return m, tea.Quit
	default:
		var cmd tea.Cmd
		m.spinner, cmd = m.spinner.Update(msg)
		return m, cmd
	}
}

func (m spinModel) View() string {
	return fmt.Sprintf("%s %s", m.spinner.View(), m.title)
}

func withSpinner(title string, fn func()) {
	done := make(chan struct{})
	go func() {
		fn()
		close(done)
	}()
	p := tea.NewProgram(spinModel{
		spinner: spinner.New(spinner.WithSpinner(spinner.Dot)),
		title:   title,
		done:    done,
	})
	_, _ = p.Run()
}

func stripANSI(s string) string {
	return ansiRe.ReplaceAllString(s, "")
}

func humanSize(bytes int64) string {
	units := []string{"B", "KB", "MB", "GB", "TB", "PB"}
	size := float64(bytes)
	i := 0
	for size >= 1024 && i < len(units)-1 {
		size /= 1024
		i++
	}
	if i == 0 {
		return fmt.Sprintf("%d %s", int64(size), units[i])
	}
	return fmt.Sprintf("%.1f %s", size, units[i])
}

func isHiddenName(name string) bool {
	return strings.HasPrefix(name, ".")
}

func formatTimestampSuffix(t time.Time) string {
	yy := strconv.Itoa(t.Year())[2:]
	m := strconv.Itoa(int(t.Month()))
	d := strconv.Itoa(t.Day())
	hour := t.Hour() % 12
	if hour == 0 {
		hour = 12
	}
	ampm := "am"
	if t.Hour() >= 12 {
		ampm = "pm"
	}
	return fmt.Sprintf("%s.%s.%s_%d.%02d%s", yy, m, d, hour, t.Minute(), ampm)
}

func relFromRoot(rootPath, targetPath string) string {
	rel, err := filepath.Rel(rootPath, targetPath)
	if err != nil || rel == "." {
		return "."
	}
	return rel
}

func makeOutputPath() string {
	suffix := ""
	if appendTimestampSuffix {
		suffix = "-" + formatTimestampSuffix(time.Now())
	}
	return filepath.Join(outputDir, outputBasename+suffix+outputExt)
}

func yesNo(v bool) string {
	if v {
		return "Yes"
	}
	return "No"
}

func orDash(s string) string {
	if s == "" {
		return "—"
	}
	return s
}

func envOr(key, fallback string) string {
	if v := os.Getenv(key); v != "" {
		return v
	}
	return fallback
}

func run(cmd []string) string {
	if len(cmd) == 0 {
		return "—"
	}
	out, err := exec.Command(cmd[0], cmd[1:]...).Output()
	if err != nil {
		return "—"
	}
	return strings.TrimSpace(string(out))
}

func realRoot(p string) (string, error) {
	path, err := filepath.Abs(p)
	if err != nil {
		return "", err
	}
	st, err := os.Stat(path)
	if err != nil || !st.IsDir() {
		return "", fmt.Errorf("not a directory")
	}
	resolved, err := filepath.EvalSymlinks(path)
	if err != nil {
		return path, nil
	}
	return resolved, nil
}

func getEntryInfo(fullPath, name string) (*entryInfo, error) {
	var st os.FileInfo
	var err error
	if followSymlinks {
		st, err = os.Stat(fullPath)
	} else {
		st, err = os.Lstat(fullPath)
	}
	if err != nil {
		return nil, err
	}
	size := int64(0)
	if st.Mode().IsRegular() {
		size = st.Size()
	}
	return &entryInfo{
		path:  fullPath,
		name:  name,
		isDir: st.IsDir(),
		size:  size,
	}, nil
}

func listDir(dir string) []entryInfo {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil
	}

	var out []entryInfo
	for _, child := range entries {
		if !showHidden && isHiddenName(child.Name()) {
			continue
		}
		fullPath := filepath.Join(dir, child.Name())
		info, err := getEntryInfo(fullPath, child.Name())
		if err != nil {
			continue
		}
		if info.isDir {
			if _, skip := excludeDirNames[info.name]; skip {
				continue
			}
		}
		out = append(out, *info)
	}

	sort.Slice(out, func(i, j int) bool {
		if out[i].isDir != out[j].isDir {
			return out[i].isDir
		}
		return strings.ToLower(out[i].name) < strings.ToLower(out[j].name)
	})
	return out
}

GO
```

## first run main.go

```sh
go mod init scan >/dev/null 2>&1 && \
go get github.com/charmbracelet/lipgloss@v1.1.0 \
       github.com/charmbracelet/bubbletea@v1.3.4 \
       github.com/charmbracelet/bubbles/spinner@v0.20.0 >/dev/null 2>&1 && \
go run "$HOME/Developer/go-tmp/main.go"
```

## subsequent run main.go

```sh
go run "$HOME/Developer/go-tmp/main.go"
```
