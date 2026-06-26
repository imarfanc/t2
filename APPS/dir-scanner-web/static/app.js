const tabs = [
  ["summary", "Summary"],
  ["system", "System"],
  ["root", "Root"],
  ["top-files", "Top Files"],
  ["top-dirs", "Top Dirs"],
  ["top-dirs-by-files", "Top by Files"],
  ["ignored", "Ignored"],
  ["tree", "Tree"],
];

const rootInput = document.getElementById("root");
const browseBtn = document.getElementById("browse");
const scanBtn = document.getElementById("scan");
const saveBtn = document.getElementById("save");
const ignoreDirsInput = document.getElementById("ignore-dirs");
const ignoreFilesInput = document.getElementById("ignore-files");
const statusEl = document.getElementById("status");
const tabBar = document.getElementById("tabs");
const panels = document.getElementById("panels");

let currentScan = null;
let systemInfo = null;
const sortState = {}; // tableId -> { key, dir }
const filterState = {}; // tableId -> string
let scanTimerId = null;
let scanStartedAt = 0;
let scanProgress = null;

// ---------- helpers ----------

function setStatus(message, kind = "") {
  statusEl.innerHTML = "";
  if (kind === "busy") {
    const spin = document.createElement("span");
    spin.className = "spinner";
    statusEl.appendChild(spin);
    kind = "";
  }
  statusEl.appendChild(document.createTextNode(message));
  statusEl.className = kind;
}

function formatElapsedSeconds(startedAt) {
  return ((performance.now() - startedAt) / 1000).toFixed(1);
}

function formatScanProgressMessage(progress, startedAt) {
  const secs = formatElapsedSeconds(startedAt);
  if (!progress) return `Scanning… ${secs}s`;

  if (progress.phase === "sizing" && progress.dir_count > 0) {
    return `Sizing… ${progress.dir_count.toLocaleString()} directories — ${secs}s`;
  }

  const parts = [];
  if (progress.file_count > 0) {
    parts.push(`${progress.file_count.toLocaleString()} files`);
  }
  if (progress.dir_count > 0) {
    parts.push(`${progress.dir_count.toLocaleString()} directories`);
  }
  if (progress.scanned_bytes > 0) {
    parts.push(humanSize(progress.scanned_bytes));
  }

  if (parts.length) return `Scanning… ${parts.join(", ")} — ${secs}s`;
  return `Scanning… ${secs}s`;
}

function startScanStatusTimer() {
  stopScanStatusTimer();
  scanStartedAt = performance.now();
  scanProgress = null;
  setStatus(formatScanProgressMessage(null, scanStartedAt), "busy");
  scanTimerId = setInterval(() => {
    setStatus(formatScanProgressMessage(scanProgress, scanStartedAt), "busy");
  }, 200);
}

function stopScanStatusTimer() {
  if (scanTimerId) {
    clearInterval(scanTimerId);
    scanTimerId = null;
  }
  scanProgress = null;
}

function humanSize(bytes) {
  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  let size = bytes;
  let unit = 0;
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024;
    unit += 1;
  }
  return unit === 0 ? `${bytes} B` : `${size.toFixed(1)} ${units[unit]}`;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function persistSettings() {
  try {
    localStorage.setItem(
      "dir-scanner-web",
      JSON.stringify({
        root: rootInput.value,
        ignoreDirs: ignoreDirsInput.value,
        ignoreFiles: ignoreFilesInput.value,
      })
    );
  } catch {}
}

function restoreSettings() {
  try {
    const saved = JSON.parse(localStorage.getItem("dir-scanner-web") || "null");
    if (!saved) return false;
    if (saved.root) rootInput.value = saved.root;
    if (saved.ignoreDirs != null) ignoreDirsInput.value = saved.ignoreDirs;
    if (saved.ignoreFiles != null) ignoreFilesInput.value = saved.ignoreFiles;
    return true;
  } catch {
    return false;
  }
}

function scanQueryParams() {
  const params = new URLSearchParams();
  const root = rootInput.value.trim() || ".";
  params.set("root", root);
  const ignoreDirs = ignoreDirsInput.value.trim();
  const ignoreFiles = ignoreFilesInput.value.trim();
  if (ignoreDirs) params.set("ignore_dirs", ignoreDirs);
  if (ignoreFiles) params.set("ignore_files", ignoreFiles);
  return params;
}

// ---------- tabs ----------

function buildTabs() {
  tabBar.innerHTML = tabs
    .map(
      ([id, label], index) =>
        `<button class="tab${index === 0 ? " active" : ""}" data-tab="${id}">${index + 1}. ${label}</button>`
    )
    .join("");

  panels.innerHTML = tabs
    .map(
      ([id, label], index) => `
        <section class="panel${index === 0 ? " active" : ""}" id="panel-${id}">
          <div class="panel-head" id="head-${id}">${label}</div>
          <div class="panel-body" id="body-${id}"></div>
        </section>`
    )
    .join("");

  tabBar.addEventListener("click", (event) => {
    const button = event.target.closest(".tab");
    if (!button) return;
    showTab(button.dataset.tab);
  });

  // keyboard shortcuts 1-7 (when not typing in an input)
  document.addEventListener("keydown", (event) => {
    if (event.metaKey || event.ctrlKey || event.altKey) return;
    if (/^(INPUT|TEXTAREA|SELECT)$/.test(document.activeElement?.tagName)) return;
    const n = Number(event.key);
    if (n >= 1 && n <= tabs.length) showTab(tabs[n - 1][0]);
  });
}

function showTab(id) {
  document.querySelectorAll(".tab").forEach((tab) => {
    tab.classList.toggle("active", tab.dataset.tab === id);
  });
  document.querySelectorAll(".panel").forEach((panel) => {
    panel.classList.toggle("active", panel.id === `panel-${id}`);
  });
}

// ---------- rendering ----------

function statCard(label, value, sub = "") {
  return `
    <div class="stat-card">
      <div class="stat-label">${escapeHtml(label)}</div>
      <div class="stat-value">${escapeHtml(value)}</div>
      ${sub ? `<div class="stat-sub">${escapeHtml(sub)}</div>` : ""}
    </div>`;
}

function sizeBar(bytes, maxBytes, label) {
  const pct = maxBytes > 0 ? Math.max(0.5, (bytes / maxBytes) * 100) : 0;
  return `
    <div class="size-cell">
      <span class="size-label">${escapeHtml(label)}</span>
      <span class="size-bar" style="--w:${pct.toFixed(2)}%"></span>
    </div>`;
}

function renderSummary(data) {
  const s = data.summary;
  const cards =
    statCard("Total size", s.root_total_size_human) +
    statCard("Files", s.file_count.toLocaleString()) +
    statCard("Directories", s.dir_count.toLocaleString()) +
    statCard("Depth", s.depth) +
    statCard("Root items", String(s.root_item_count)) +
    statCard("Scanned", s.scanned_file_size);

  // horizontal bar breakdown of root entries by size
  const entries = [...data.root_entries].sort((a, b) => b.size_bytes - a.size_bytes);
  const max = entries[0]?.size_bytes || 0;
  const total = s.root_total_size || 1;
  const breakdown = entries
    .slice(0, 12)
    .map((e) => {
      const pct = ((e.size_bytes / total) * 100).toFixed(1);
      const w = max > 0 ? Math.max(0.5, (e.size_bytes / max) * 100) : 0;
      return `
        <div class="bd-row" title="${escapeHtml(e.name)} — ${escapeHtml(e.size)} (${pct}%)">
          <span class="bd-name${e.kind === "dir" ? " is-dir" : ""}">${escapeHtml(e.name)}</span>
          <span class="bd-track"><span class="bd-fill" style="--w:${w.toFixed(2)}%"></span></span>
          <span class="bd-size">${escapeHtml(e.size)}</span>
          <span class="bd-pct">${pct}%</span>
        </div>`;
    })
    .join("");

  return `
    <div class="root-line">${escapeHtml(data.root_path)}</div>
    <div class="stat-grid">${cards}</div>
    <div class="bd-title">Largest root entries</div>
    <div class="breakdown">${breakdown || `<p class="empty">No entries.</p>`}</div>`;
}

function renderSystem(rows) {
  if (!rows?.length) return `<p class="empty">Loading system info…</p>`;
  return `
    <dl class="kv-grid">
      ${rows.map(({ key, value }) => `<dt>${escapeHtml(key)}</dt><dd>${escapeHtml(value)}</dd>`).join("")}
    </dl>`;
}

// Sortable, filterable table.
// columns: [{ key, label, sortKey?, numeric?, render? }]
function renderDataTable(tableId, columns, rows) {
  const filter = (filterState[tableId] || "").toLowerCase();
  const sort = sortState[tableId];

  let view = rows;
  if (filter) {
    view = view.filter((row) =>
      columns.some((col) => String(row[col.key] ?? "").toLowerCase().includes(filter))
    );
  }
  if (sort) {
    const col = columns.find((c) => (c.sortKey || c.key) === sort.key);
    view = [...view].sort((a, b) => {
      const ka = a[sort.key];
      const kb = b[sort.key];
      const cmp = col?.numeric ? ka - kb : String(ka).localeCompare(String(kb));
      return sort.dir === "asc" ? cmp : -cmp;
    });
  }

  const head = columns
    .map((col) => {
      const key = col.sortKey || col.key;
      const active = sort?.key === key;
      const arrow = active ? (sort.dir === "asc" ? " ▲" : " ▼") : "";
      return `<th class="sortable${active ? " sorted" : ""}" data-sort="${key}" data-numeric="${col.numeric ? 1 : 0}">${escapeHtml(col.label)}${arrow}</th>`;
    })
    .join("");

  const body = view.length
    ? view
        .map(
          (row) =>
            `<tr>${columns
              .map((col) => `<td class="col-${col.key}">${col.render ? col.render(row) : escapeHtml(row[col.key])}</td>`)
              .join("")}</tr>`
        )
        .join("")
    : `<tr><td colspan="${columns.length}" class="empty">No matching rows.</td></tr>`;

  return `
    <div class="table-tools">
      <input type="search" class="table-filter" data-table="${tableId}"
             placeholder="Filter…" value="${escapeHtml(filterState[tableId] || "")}" aria-label="Filter rows">
      <span class="table-count">${view.length}/${rows.length}</span>
    </div>
    <div class="table-wrap">
      <table data-table="${tableId}">
        <thead><tr>${head}</tr></thead>
        <tbody>${body}</tbody>
      </table>
    </div>`;
}

function copyableCell(text) {
  return `<span class="copyable" data-copy="${escapeHtml(text)}" title="Click to copy">${escapeHtml(text)}</span>`;
}

function renderAll(data) {
  document.getElementById("body-summary").innerHTML = renderSummary(data);
  document.getElementById("body-system").innerHTML = renderSystem(systemInfo);

  const maxRoot = Math.max(0, ...data.root_entries.map((r) => r.size_bytes));
  document.getElementById("body-root").innerHTML = renderDataTable(
    "root",
    [
      { key: "index", label: "#", numeric: true },
      { key: "kind", label: "Type" },
      { key: "name", label: "Name" },
      {
        key: "size",
        label: "Size",
        sortKey: "size_bytes",
        numeric: true,
        render: (row) => sizeBar(row.size_bytes, maxRoot, row.size),
      },
    ],
    data.root_entries
  );

  const fileCols = (rows) => {
    const max = Math.max(0, ...rows.map((r) => r.size_bytes));
    return [
      { key: "index", label: "#", numeric: true },
      { key: "name", label: "Name" },
      { key: "path", label: "Path", render: (row) => copyableCell(row.path) },
      {
        key: "size",
        label: "Size",
        sortKey: "size_bytes",
        numeric: true,
        render: (row) => sizeBar(row.size_bytes, max, row.size),
      },
    ];
  };

  document.getElementById("body-top-files").innerHTML = renderDataTable(
    "top-files",
    fileCols(data.top_files),
    data.top_files
  );
  document.getElementById("body-top-dirs").innerHTML = renderDataTable(
    "top-dirs",
    fileCols(data.top_dirs),
    data.top_dirs
  );

  const maxCount = Math.max(0, ...data.top_dirs_by_files.map((r) => r.file_count));
  document.getElementById("body-top-dirs-by-files").innerHTML = renderDataTable(
    "top-dirs-by-files",
    [
      { key: "index", label: "#", numeric: true },
      { key: "name", label: "Directory" },
      { key: "path", label: "Path", render: (row) => copyableCell(row.path) },
      {
        key: "file_count",
        label: "Files",
        numeric: true,
        render: (row) => sizeBar(row.file_count, maxCount, row.file_count.toLocaleString()),
      },
    ],
    data.top_dirs_by_files
  );

  const ignoredRows = data.ignored || [];
  const maxIgnored = Math.max(0, ...ignoredRows.map((r) => r.size_bytes));
  const ignoredTotal = ignoredRows.reduce((sum, r) => sum + r.size_bytes, 0);
  document.getElementById("head-ignored").textContent = ignoredRows.length
    ? `Ignored — ${ignoredRows.length} entries, ${humanSize(ignoredTotal)} excluded from scan`
    : "Ignored";
  document.getElementById("body-ignored").innerHTML = ignoredRows.length
    ? renderDataTable(
        "ignored",
        [
          { key: "index", label: "#", numeric: true },
          { key: "kind", label: "Type" },
          { key: "name", label: "Name" },
          { key: "path", label: "Path", render: (row) => copyableCell(row.path) },
          { key: "file_count", label: "Files", numeric: true },
          {
            key: "size",
            label: "Size",
            sortKey: "size_bytes",
            numeric: true,
            render: (row) => sizeBar(row.size_bytes, maxIgnored, row.size),
          },
        ],
        ignoredRows
      )
    : `<p class="empty">Nothing was ignored in this scan.</p>`;

  document.getElementById("body-tree").innerHTML = `
    <div class="table-tools">
      <input type="search" class="tree-filter" placeholder="Filter tree…" aria-label="Filter tree lines">
      <button id="copy-tree" type="button" class="mini">Copy</button>
      <button id="export-tree" type="button" class="mini">Export .txt</button>
    </div>
    <pre class="tree" id="tree-pre">◆ ${escapeHtml(data.root_path)}  ◌ ${escapeHtml(data.summary.root_total_size_human)}\n\n${escapeHtml(data.tree_lines.join("\n"))}</pre>`;

  wireTreeTools(data);
}

function wireTreeTools(data) {
  const pre = document.getElementById("tree-pre");
  const filter = document.querySelector(".tree-filter");
  const header = `◆ ${data.root_path}  ◌ ${data.summary.root_total_size_human}\n\n`;
  filter?.addEventListener("input", () => {
    const q = filter.value.trim().toLowerCase();
    const lines = q
      ? data.tree_lines.filter((line) => line.toLowerCase().includes(q))
      : data.tree_lines;
    pre.textContent = header + lines.join("\n");
  });
  document.getElementById("copy-tree")?.addEventListener("click", async (event) => {
    await navigator.clipboard.writeText(header + data.tree_lines.join("\n"));
    event.target.textContent = "Copied ✓";
    setTimeout(() => (event.target.textContent = "Copy"), 1200);
  });
  document.getElementById("export-tree")?.addEventListener("click", () => {
    const stamp = new Date().toISOString().slice(0, 19).replaceAll(":", "-");
    const rootName =
      data.root_path.split("/").filter(Boolean).pop() || "root";
    const blob = new Blob([header + data.tree_lines.join("\n") + "\n"], {
      type: "text/plain",
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `tree_${rootName}_${stamp}.txt`;
    a.click();
    URL.revokeObjectURL(url);
  });
}

// Delegated events for table sorting, filtering, copy-to-clipboard.
panels.addEventListener("click", async (event) => {
  const th = event.target.closest("th.sortable");
  if (th && currentScan) {
    const table = th.closest("table").dataset.table;
    const key = th.dataset.sort;
    const prev = sortState[table];
    sortState[table] =
      prev?.key === key
        ? { key, dir: prev.dir === "asc" ? "desc" : "asc" }
        : { key, dir: th.dataset.numeric === "1" ? "desc" : "asc" };
    renderAll(currentScan);
    return;
  }
  const copyable = event.target.closest(".copyable");
  if (copyable) {
    await navigator.clipboard.writeText(copyable.dataset.copy);
    copyable.classList.add("copied");
    setTimeout(() => copyable.classList.remove("copied"), 600);
  }
});

panels.addEventListener("input", (event) => {
  const input = event.target.closest(".table-filter");
  if (!input || !currentScan) return;
  filterState[input.dataset.table] = input.value;
  const pos = input.selectionStart;
  renderAll(currentScan);
  const fresh = panels.querySelector(`.table-filter[data-table="${input.dataset.table}"]`);
  fresh?.focus();
  fresh?.setSelectionRange(pos, pos);
});

// ---------- actions ----------

async function loadConfig() {
  const response = await fetch("/api/config");
  const config = await response.json();
  // localStorage wins over config defaults
  if (!restoreSettings()) {
    ignoreDirsInput.value = config.ignore_dirs;
    ignoreFilesInput.value = config.ignore_files;
  }
}

async function loadSystem() {
  const response = await fetch("/api/system");
  systemInfo = await response.json();
  if (currentScan) renderAll(currentScan);
  else document.getElementById("body-system").innerHTML = renderSystem(systemInfo);
}

async function browseFolder() {
  browseBtn.disabled = true;
  setStatus("Choose a folder…", "busy");
  try {
    const response = await fetch("/api/browse", { method: "POST" });
    const payload = await response.json();
    if (payload.path) {
      rootInput.value = payload.path;
      persistSettings();
      setStatus(`Selected ${payload.path}`);
    } else {
      setStatus("Browse cancelled.");
    }
  } catch (error) {
    setStatus(error.message, "error");
  } finally {
    browseBtn.disabled = false;
  }
}

async function runScan() {
  scanBtn.disabled = true;
  saveBtn.disabled = true;
  persistSettings();
  startScanStatusTimer();

  try {
    const payload = await new Promise((resolve, reject) => {
      const source = new EventSource(`/api/scan/stream?${scanQueryParams()}`);
      let settled = false;

      const finish = (handler) => {
        if (settled) return;
        settled = true;
        source.close();
        handler();
      };

      source.onmessage = (event) => {
        let message;
        try {
          message = JSON.parse(event.data);
        } catch (error) {
          finish(() => reject(new Error("Invalid scan progress")));
          return;
        }

        if (message.type === "progress") {
          scanProgress = message;
          setStatus(formatScanProgressMessage(scanProgress, scanStartedAt), "busy");
          return;
        }

        if (message.type === "done") {
          finish(() => resolve(message.data));
          return;
        }

        if (message.type === "error") {
          finish(() => reject(new Error(message.error || "Scan failed")));
        }
      };

      source.onerror = () => {
        finish(() => reject(new Error("Scan failed")));
      };
    });

    currentScan = payload;
    Object.keys(sortState).forEach((k) => delete sortState[k]);
    renderAll(currentScan);
    saveBtn.disabled = false;
    const secs = formatElapsedSeconds(scanStartedAt);
    setStatus(
      `Scanned ${payload.summary.file_count.toLocaleString()} files in ${payload.summary.dir_count.toLocaleString()} directories — ${payload.summary.root_total_size_human} total, ${secs}s.`,
      "ok"
    );
  } catch (error) {
    currentScan = null;
    setStatus(error.message, "error");
  } finally {
    stopScanStatusTimer();
    scanBtn.disabled = false;
  }
}

async function saveReport() {
  saveBtn.disabled = true;
  setStatus("Saving report…", "busy");
  try {
    const response = await fetch(`/api/save?${scanQueryParams()}`, {
      method: "POST",
    });
    const payload = await response.json();
    if (!response.ok) throw new Error(payload.error || "Save failed");
    setStatus(`Saved ${payload.path}`, "ok");
  } catch (error) {
    setStatus(error.message, "error");
  } finally {
    saveBtn.disabled = !currentScan;
  }
}

// ---------- init ----------

buildTabs();
Promise.all([
  loadConfig().catch(() => restoreSettings()),
  loadSystem().catch(() => setStatus("Could not load system info.", "error")),
]);
browseBtn.addEventListener("click", browseFolder);
scanBtn.addEventListener("click", runScan);
saveBtn.addEventListener("click", saveReport);
[rootInput, ignoreDirsInput, ignoreFilesInput].forEach((input) => {
  input.addEventListener("change", persistSettings);
  input.addEventListener("keydown", (event) => {
    if (event.key === "Enter") runScan();
  });
});
