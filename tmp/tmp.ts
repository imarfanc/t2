import * as path from "jsr:@std/path";

const ROOT = ".";
const SHOW_HIDDEN = true;
const FOLLOW_SYMLINKS = false;
const SCAN_EVERYTHING = false; // true = scan everything (cycle-safe), false = respect MAX_DEPTH
const MAX_DEPTH = 3;

const TOP_N_FILES = 25;
const TOP_N_DIRS = 50;

const HOME = Deno.env.get("HOME") || "";
const OUTPUT_DIR = path.join(HOME, "Developer/macos-reset"); // avoid hardcoded username
const SAVE_OUTPUT = true;
const OUTPUT_BASENAME = "tree_output";
const OUTPUT_EXT = ".txt";
const APPEND_TIMESTAMP_SUFFIX = true; // tree-output-yy.m.d_h.mmam.txt

const SHOW_SPINNER = true;
const USE_ANSI = true;

const EXCLUDE_DIR_NAMES = new Set([
    ".git",
    "node_modules",
    ".venv",
    "__pycache__",
    ".DS_Store",
]);

const ICON = {
    section: "▾",
    dir: "□",
    file: "▪",
    size: "◌",
    root: "◆",
    live: "●",
    ok: "■",
};

type EntryInfo = {
    path: string;
    name: string;
    isDir: boolean;
    size: number;
};

// -----------------------------------------------------------------------------
// ANSI
// -----------------------------------------------------------------------------

const ansi = {
    reset: "\x1b[0m",
    bold: "\x1b[1m",
    dim: "\x1b[2m",
    cyan: "\x1b[36m",
    magenta: "\x1b[35m",
    green: "\x1b[32m",
    yellow: "\x1b[33m",
    white: "\x1b[37m",
    blue: "\x1b[34m",
    gray: "\x1b[90m",
    red: "\x1b[31m",
};

function color(text: string, ...codes: string[]) {
    if (!USE_ANSI) return text;
    return `${codes.join("")}${text}${ansi.reset}`;
}

function stripAnsi(s: string) {
    return s.replace(/\x1b\[[0-9;]*m/g, "");
}

// -----------------------------------------------------------------------------
// Helpers
// -----------------------------------------------------------------------------

function padRight(s: string, n: number): string {
    const len = stripAnsi(s).length;
    return len >= n ? s : s + " ".repeat(n - len);
}

function padLeft(s: string, n: number): string {
    const len = stripAnsi(s).length;
    return len >= n ? s : " ".repeat(n - len) + s;
}

function humanSize(bytes: number): string {
    const units = ["B", "KB", "MB", "GB", "TB", "PB"];
    let size = bytes;
    let i = 0;
    while (size >= 1024 && i < units.length - 1) {
        size /= 1024;
        i++;
    }
    return i === 0
        ? `${Math.floor(size)} ${units[i]}`
        : `${size.toFixed(1)} ${units[i]}`;
}

function isHiddenName(name: string): boolean {
    return name.startsWith(".");
}

function formatTimestampSuffix(date = new Date()): string {
    const yy = String(date.getFullYear()).slice(-2);
    const m = String(date.getMonth() + 1);
    const d = String(date.getDate());

    let hour = date.getHours();
    const minute = String(date.getMinutes()).padStart(2, "0");
    const ampm = hour >= 12 ? "pm" : "am";

    hour = hour % 12;
    if (hour === 0) hour = 12;

    return `${yy}.${m}.${d}_${hour}.${minute}${ampm}`;
}

function sortEntries(a: EntryInfo, b: EntryInfo): number {
    if (a.isDir !== b.isDir) return a.isDir ? -1 : 1;
    return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
}

function relFromRoot(rootPath: string, targetPath: string): string {
    const rel = path.relative(rootPath, targetPath);
    return rel === "" ? "." : rel;
}

function makeOutputPath(): string {
    const suffix = APPEND_TIMESTAMP_SUFFIX ? `-${formatTimestampSuffix()}` : "";
    return path.join(OUTPUT_DIR, `${OUTPUT_BASENAME}${suffix}${OUTPUT_EXT}`);
}

// -----------------------------------------------------------------------------
// Filesystem helpers
// -----------------------------------------------------------------------------

async function realRoot(p: string): Promise<string | null> {
    try {
        const st = await Deno.stat(p);
        if (!st.isDirectory) return null;
        return await Deno.realPath(p);
    } catch {
        return null;
    }
}

async function getEntryInfo(
    fullPath: string,
    name: string,
): Promise<EntryInfo | null> {
    try {
        const st = FOLLOW_SYMLINKS
            ? await Deno.stat(fullPath)
            : await Deno.lstat(fullPath);
        return {
            path: fullPath,
            name,
            isDir: st.isDirectory,
            size: st.isFile ? st.size : 0,
        };
    } catch {
        return null;
    }
}

async function listDir(dir: string): Promise<EntryInfo[]> {
    const out: EntryInfo[] = [];

    try {
        for await (const entry of Deno.readDir(dir)) {
            if (!SHOW_HIDDEN && isHiddenName(entry.name)) continue;

            const fullPath = path.join(dir, entry.name);
            const info = await getEntryInfo(fullPath, entry.name);
            if (!info) continue;
            if (info.isDir && EXCLUDE_DIR_NAMES.has(info.name)) continue;

            out.push(info);
        }
    } catch {
        return [];
    }

    out.sort(sortEntries);
    return out;
}

// -----------------------------------------------------------------------------
// Spinner
// -----------------------------------------------------------------------------

const spinnerFrames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
let spinnerIndex = 0;
let spinnerTimer: number | null = null;

function startSpinner() {
    if (!SHOW_SPINNER) return;
    spinnerTimer = setInterval(() => {
        const frame = spinnerFrames[spinnerIndex++ % spinnerFrames.length];
        const elapsed = ((performance.now() - startTime) / 1000).toFixed(1);
        
        // Fancier stats with icons and dim separators
        const dirStat = `${color(ICON.dir, ansi.cyan)} ${color(dirCount.toString(), ansi.bold, ansi.white)}`;
        const fileStat = `${color(ICON.file, ansi.white)} ${color(fileCount.toString(), ansi.bold, ansi.white)}`;
        const sizeStat = `${color(ICON.size, ansi.yellow)} ${color(humanSize(scannedFileBytes), ansi.bold, ansi.white)}`;
        const sep = color(" ┊ ", ansi.gray);
        
        const statusLine = `\r${color(frame, ansi.green)} ${color("SCANNING", ansi.bold, ansi.white)} ${sep}${dirStat}${sep}${fileStat}${sep}${sizeStat}${sep}${color(elapsed + "s", ansi.dim, ansi.white)} `;
        
        Deno.stdout.writeSync(new TextEncoder().encode(statusLine));
    }, 80);
}

function stopSpinner(finalText = "") {
    if (spinnerTimer !== null) clearInterval(spinnerTimer);
    spinnerTimer = null;
    Deno.stdout.writeSync(new TextEncoder().encode(`\r${" ".repeat(160)}\r`));
    if (finalText) {
        Deno.stdout.writeSync(new TextEncoder().encode(`${finalText}\n`));
    }
}

// -----------------------------------------------------------------------------
// Table renderer
// -----------------------------------------------------------------------------

function makeTable(title: string, headers: string[], rows: string[][]): string {
    const widths = headers.map((h, i) =>
        Math.max(
            stripAnsi(h).length,
            ...rows.map((r) => stripAnsi(r[i] ?? "").length),
        )
    );

    const top = "┌" + widths.map((w) => "─".repeat(w + 2)).join("┬") + "┐";
    const mid = "├" + widths.map((w) => "─".repeat(w + 2)).join("┼") + "┤";
    const bot = "└" + widths.map((w) => "─".repeat(w + 2)).join("┴") + "┘";

    const fmtRow = (cols: string[]) =>
        "│ " + cols.map((c, i) => padRight(c ?? "", widths[i])).join(" │ ") +
        " │";

    return [
        color(title, ansi.bold, ansi.magenta),
        color(top, ansi.gray),
        color(fmtRow(headers), ansi.bold, ansi.cyan),
        color(mid, ansi.gray),
        ...rows.map((r) => fmtRow(r)),
        color(bot, ansi.gray),
    ].join("\n");
}

// -----------------------------------------------------------------------------
// Scan
// -----------------------------------------------------------------------------

const rootPath = await realRoot(ROOT);
if (!rootPath) {
    console.error(
        color(
            `Error: '${ROOT}' is not a valid directory.`,
            ansi.bold,
            ansi.red,
        ),
    );
    Deno.exit(1);
}

let fileCount = 0;
let dirCount = 0;
let scannedFileBytes = 0;
let maxDepthReached = 0;

const largestFiles: Array<{ path: string; size: number }> = [];
const dirSizes = new Map<string, number>();
const visitedDirs = new Set<string>();

async function scanDirSize(dir: string): Promise<number> {
    const real = await Deno.realPath(dir).catch(() => dir);
    if (visitedDirs.has(real)) return 0;
    visitedDirs.add(real);

    let total = 0;
    const entries = await listDir(dir);

    for (const entry of entries) {
        if (entry.isDir) {
            total += await scanDirSize(entry.path);
        } else {
            total += entry.size;
        }
    }

    dirSizes.set(dir, total);
    return total;
}

const HARD_DEPTH_LIMIT = 200; // safety cap even when IGNORE_MAX_DEPTH is true

async function buildTreeLines(
    dir: string,
    depth: number,
    prefix = "",
    visited = new Set<string>(),
): Promise<string[]> {
    if (depth >= HARD_DEPTH_LIMIT) return [];
    if (!SCAN_EVERYTHING && depth >= MAX_DEPTH) return [];
    if (depth > maxDepthReached) maxDepthReached = depth;

    const lines: string[] = [];
    const entries = await listDir(dir);

    for (let i = 0; i < entries.length; i++) {
        const entry = entries[i];
        const isLast = i === entries.length - 1;
        const connector = isLast ? "└── " : "├── ";
        const nextPrefix = prefix + (isLast ? "    " : "│   ");

        if (entry.isDir) {
            const real = await Deno.realPath(entry.path).catch(() =>
                entry.path
            );
            if (visited.has(real)) continue;
            visited.add(real);

            dirCount++;
            const size = dirSizes.get(entry.path) ?? 0;

            lines.push(
                prefix +
                    color(connector, ansi.gray) +
                    color(`${ICON.dir} ${entry.name}/`, ansi.bold, ansi.cyan) +
                    "  " +
                    color(
                        `${ICON.size} ${humanSize(size)}`,
                        ansi.dim,
                        ansi.white,
                    ),
            );

            const childLines = await buildTreeLines(
                entry.path,
                depth + 1,
                nextPrefix,
                visited,
            );
            for (const line of childLines) lines.push(line);
        } else {
            fileCount++;
            scannedFileBytes += entry.size;
            largestFiles.push({ path: entry.path, size: entry.size });

            lines.push(
                prefix +
                    color(connector, ansi.gray) +
                    color(`${ICON.file} ${entry.name}`, ansi.white) +
                    "  " +
                    color(
                        `${ICON.size} ${humanSize(entry.size)}`,
                        ansi.dim,
                        ansi.white,
                    ),
            );
        }
    }

    return lines;
}

// -----------------------------------------------------------------------------
// Startup banner
// -----------------------------------------------------------------------------

console.log("");
console.log(color(`  ${ICON.root} dir scanner`, ansi.bold, ansi.magenta));
console.log(color(`  ${"─".repeat(30)}`, ansi.gray));
console.log(
    color(`  ${ICON.dir} root: `, ansi.dim) +
        color(rootPath, ansi.bold, ansi.white),
);
console.log(
    color(`  ${ICON.size} mode: `, ansi.dim) +
        color(
            SCAN_EVERYTHING ? "scan everything" : `max depth ${MAX_DEPTH}`,
            ansi.green,
        ),
);
console.log(color(`  ${"─".repeat(30)}`, ansi.gray));
console.log("");

let startTime = performance.now();
startSpinner();

const rootTotalSize = await scanDirSize(rootPath);
const rootEntries = await listDir(rootPath);
const treeLines = await buildTreeLines(rootPath, 0);

const durationMs = performance.now() - startTime;
const durationSec = (durationMs / 1000).toFixed(2);

// Final fancy status line before stopping spinner
const dirStat = `${color(ICON.dir, ansi.cyan)} ${color(dirCount.toString(), ansi.bold, ansi.white)}`;
const fileStat = `${color(ICON.file, ansi.white)} ${color(fileCount.toString(), ansi.bold, ansi.white)}`;
const sizeStat = `${color(ICON.size, ansi.yellow)} ${color(humanSize(scannedFileBytes), ansi.bold, ansi.white)}`;
const sep = color(" ┊ ", ansi.gray);
const finalStatus = `${color(ICON.ok, ansi.green)} ${color("COMPLETED", ansi.bold, ansi.white)} ${sep}${dirStat}${sep}${fileStat}${sep}${sizeStat}${sep}${color(durationSec + "s", ansi.dim, ansi.white)}`;

stopSpinner();
// ... scan logic ends ...
// (We'll print finalStatus later at the very end)

// -----------------------------------------------------------------------------
// Rankings
// -----------------------------------------------------------------------------

largestFiles.sort((a, b) => b.size - a.size);
const topFiles = largestFiles.slice(0, TOP_N_FILES);

const topDirs = [...dirSizes.entries()]
    .filter(([p]) => p !== rootPath)
    .sort((a, b) => b[1] - a[1])
    .slice(0, TOP_N_DIRS);

// -----------------------------------------------------------------------------
// Tables
// -----------------------------------------------------------------------------

const outputPath = SAVE_OUTPUT ? makeOutputPath() : "";

const summaryTable = makeTable(
    `${ICON.section} Summary`,
    ["Field", "Value"],
    [
        ["Root", color(rootPath, ansi.white)],
        [
            "Depth",
            SCAN_EVERYTHING
                ? `${maxDepthReached} (actual)`
                : `${MAX_DEPTH} (max)`,
        ],
        ["Directories", color(String(dirCount), ansi.cyan)],
        ["Files", color(String(fileCount), ansi.white)],
        ["Scanned File Size", color(humanSize(scannedFileBytes), ansi.green)],
        ["Root Total Size", color(humanSize(rootTotalSize), ansi.green)],
        ["Root Items", String(rootEntries.length)],
        ["Show Hidden", SHOW_HIDDEN ? "Yes" : "No"],
        ["Follow Symlinks", FOLLOW_SYMLINKS ? "Yes" : "No"],
        ["Top Files Count", String(TOP_N_FILES)],
        ["Top Dirs Count", String(TOP_N_DIRS)],
        ["Save Output", SAVE_OUTPUT ? "Yes" : "No"],
        ["Output Dir", OUTPUT_DIR],
        ["Output File", SAVE_OUTPUT ? outputPath : "—"],
        ["Duration", `${durationSec}s`],
        ["Generated", new Date().toLocaleString()],
    ],
);

// -----------------------------------------------------------------------------
// System info
// -----------------------------------------------------------------------------

function run(cmd: string[]): string {
    try {
        const result = new Deno.Command(cmd[0], {
            args: cmd.slice(1),
            stdout: "piped",
        }).outputSync();
        return new TextDecoder().decode(result.stdout).trim();
    } catch {
        return "—";
    }
}

const buildVer = typeof Deno.build.os !== "undefined"
    ? `${Deno.build.os} ${Deno.build.arch}`
    : "—";

const systemInfoRows: string[][] = [
    ["User", color(run(["whoami"]), ansi.white)],
    ["Hostname", color(run(["hostname"]), ansi.white)],
    ["OS", color(run(["sw_vers", "-productName"]), ansi.white)],
    ["OS Version", color(run(["sw_vers", "-productVersion"]), ansi.white)],
    ["Build", color(run(["sw_vers", "-buildVersion"]), ansi.white)],
    ["Platform", color(buildVer, ansi.white)],
    ["User Lang", color(Deno.env.get("LANG") ?? "—", ansi.dim, ansi.white)],
    [
        "Processor",
        color(run(["sysctl", "-n", "machdep.cpu.brand_string"]), ansi.white),
    ],
    ["Cores", color(run(["sysctl", "-n", "hw.ncpu"]), ansi.green)],
    [
        "Memory",
        color(
            humanSize(parseInt(run(["sysctl", "-n", "hw.memsize"])) || 0),
            ansi.green,
        ),
    ],
    [
        "Uptime",
        color(run(["uptime"]).replace(/^\s*/, ""), ansi.dim, ansi.white),
    ],
    ["Date", color(new Date().toLocaleString(), ansi.dim, ansi.white)],
    ["Home", color(Deno.env.get("HOME") ?? "—", ansi.white)],
    ["Shell", color(Deno.env.get("SHELL") ?? "—", ansi.white)],
    ["Term", color(Deno.env.get("TERM") ?? "—", ansi.dim, ansi.white)],
    ["Deno", color(run(["deno", "-V"]), ansi.cyan)],
];

const systemInfoTable = makeTable(
    `${ICON.section} System Info`,
    ["Field", "Value"],
    systemInfoRows,
);

const rootTable = makeTable(
    `${ICON.section} Root Contents`,
    ["#", "Type", "Name", "Size"],
    rootEntries.map((e, i) => [
        padLeft(String(i + 1), 2),
        e.isDir
            ? color(`${ICON.dir} dir`, ansi.cyan)
            : color(`${ICON.file} file`, ansi.white),
        e.isDir ? color(`${e.name}/`, ansi.bold, ansi.cyan) : e.name,
        color(
            e.isDir ? humanSize(dirSizes.get(e.path) ?? 0) : humanSize(e.size),
            ansi.green,
        ),
    ]),
);

const topFilesTable = makeTable(
    `${ICON.section} Top ${TOP_N_FILES} Largest Files`,
    ["#", "File", "Relative Path", "Size"],
    topFiles.map((f, i) => {
        const rel = relFromRoot(rootPath, f.path);
        return [
            padLeft(String(i + 1), 2),
            color(`${ICON.file} ${path.basename(f.path)}`, ansi.white),
            truncatePath(rel),
            color(humanSize(f.size), ansi.green),
        ];
    }),
);

function truncatePath(p: string, maxLength = 40): string {
    if (p.length <= maxLength) return p;
    const half = Math.floor((maxLength - 3) / 2);
    return p.slice(0, half) + "..." + p.slice(-half);
}

const topDirsTable = makeTable(
    `${ICON.section} Top ${TOP_N_DIRS} Largest Directories`,
    ["#", "Directory", "Relative Path", "Total Size"],
    topDirs.map(([p, size], i) => {
        const rel = relFromRoot(rootPath, p);
        return [
            padLeft(String(i + 1), 2),
            color(`${ICON.dir} ${path.basename(p)}/`, ansi.bold, ansi.cyan),
            truncatePath(rel),
            color(humanSize(size), ansi.green),
        ];
    }),
);

// -----------------------------------------------------------------------------
// Final output
// -----------------------------------------------------------------------------

const finalOutput = [
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
    color(`${ICON.section} Full Tree`, ansi.bold, ansi.magenta),
    color(`${ICON.root} ${rootPath}`, ansi.bold, ansi.magenta) +
    "  " +
    color(`${ICON.size} ${humanSize(rootTotalSize)}`, ansi.dim, ansi.white),
    ...treeLines,
].join("\n");

console.log(finalOutput);

// -----------------------------------------------------------------------------
// Save output
// -----------------------------------------------------------------------------

if (SAVE_OUTPUT) {
    await Deno.mkdir(path.dirname(outputPath), { recursive: true });
    await Deno.writeTextFile(outputPath, stripAnsi(finalOutput));
    console.log("");
    console.log(finalStatus);
    console.log(color(`Saved: ${outputPath}`, ansi.bold, ansi.green));
}
