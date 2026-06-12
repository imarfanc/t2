const tabs = [
  ["summary", "Summary"],
  ["system", "System"],
  ["root", "Root"],
  ["top-files", "Top Files"],
  ["top-dirs", "Top Dirs"],
  ["top-dirs-by-files", "Top by Files"],
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

function setStatus(message, kind = "") {
  statusEl.textContent = message;
  statusEl.className = kind;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
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
          <div class="panel-head">${label}</div>
          <div class="panel-body" id="body-${id}"></div>
        </section>`
    )
    .join("");

  tabBar.addEventListener("click", (event) => {
    const button = event.target.closest(".tab");
    if (!button) return;
    showTab(button.dataset.tab);
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

function renderSummary(data) {
  const s = data.summary;
  return `
    <dl class="kv-grid">
      <dt>root</dt><dd>${escapeHtml(data.root_path)}</dd>
      <dt>total</dt><dd>${escapeHtml(s.root_total_size_human)}</dd>
      <dt>depth</dt><dd>${escapeHtml(s.depth)}</dd>
      <dt>dirs</dt><dd>${s.dir_count}</dd>
      <dt>files</dt><dd>${s.file_count}</dd>
      <dt>scanned</dt><dd>${escapeHtml(s.scanned_file_size)}</dd>
      <dt>root items</dt><dd>${s.root_item_count}</dd>
    </dl>`;
}

function renderTable(headers, rows, mapRow) {
  if (!rows.length) return `<p class="empty">No rows.</p>`;
  return `
    <table>
      <thead><tr>${headers.map((h) => `<th>${escapeHtml(h)}</th>`).join("")}</tr></thead>
      <tbody>
        ${rows.map((row) => `<tr>${mapRow(row).map((cell) => `<td>${escapeHtml(cell)}</td>`).join("")}</tr>`).join("")}
      </tbody>
    </table>`;
}

function renderSystem(rows) {
  if (!rows?.length) return `<p class="empty">Loading system info…</p>`;
  return `
    <dl class="kv-grid">
      ${rows.map(({ key, value }) => `<dt>${escapeHtml(key)}</dt><dd>${escapeHtml(value)}</dd>`).join("")}
    </dl>`;
}

function renderAll(data) {
  document.getElementById("body-summary").innerHTML = renderSummary(data);
  document.getElementById("body-system").innerHTML = renderSystem(systemInfo);
  document.getElementById("body-root").innerHTML = renderTable(
    ["#", "Type", "Name", "Size"],
    data.root_entries,
    (row) => [row.index, row.kind, row.name, row.size]
  );
  document.getElementById("body-top-files").innerHTML = renderTable(
    ["#", "File", "Path", "Size"],
    data.top_files,
    (row) => [row.index, row.name, row.path, row.size]
  );
  document.getElementById("body-top-dirs").innerHTML = renderTable(
    ["#", "Directory", "Path", "Size"],
    data.top_dirs,
    (row) => [row.index, row.name, row.path, row.size]
  );
  document.getElementById("body-top-dirs-by-files").innerHTML = renderTable(
    ["#", "Directory", "Path", "Files"],
    data.top_dirs_by_files,
    (row) => [row.index, row.name, row.path, row.file_count]
  );
  document.getElementById("body-tree").innerHTML = `
    <pre class="tree">◆ ${escapeHtml(data.root_path)}  ◌ ${escapeHtml(data.summary.root_total_size_human)}\n\n${escapeHtml(data.tree_lines.join("\n"))}</pre>`;
}

async function loadConfig() {
  const response = await fetch("/api/config");
  const config = await response.json();
  ignoreDirsInput.value = config.ignore_dirs;
  ignoreFilesInput.value = config.ignore_files;
}

async function loadSystem() {
  const response = await fetch("/api/system");
  systemInfo = await response.json();
  if (currentScan) renderAll(currentScan);
}

async function browseFolder() {
  browseBtn.disabled = true;
  setStatus("Choose a folder…");
  try {
    const response = await fetch("/api/browse", { method: "POST" });
    const payload = await response.json();
    if (payload.path) {
      rootInput.value = payload.path;
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
  setStatus("Scanning…");
  try {
    const response = await fetch(`/api/scan?${scanQueryParams()}`);
    const payload = await response.json();
    if (!response.ok) throw new Error(payload.error || "Scan failed");
    currentScan = payload;
    renderAll(currentScan);
    saveBtn.disabled = false;
    setStatus(`Scanned ${payload.summary.file_count} files in ${payload.summary.dir_count} directories.`, "ok");
  } catch (error) {
    currentScan = null;
    setStatus(error.message, "error");
  } finally {
    scanBtn.disabled = false;
  }
}

async function saveReport() {
  saveBtn.disabled = true;
  setStatus("Saving report…");
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

buildTabs();
Promise.all([
  loadConfig().catch(() => {}),
  loadSystem().catch(() => setStatus("Could not load system info.", "error")),
]);
browseBtn.addEventListener("click", browseFolder);
scanBtn.addEventListener("click", runScan);
saveBtn.addEventListener("click", saveReport);
rootInput.addEventListener("keydown", (event) => {
  if (event.key === "Enter") runScan();
});
