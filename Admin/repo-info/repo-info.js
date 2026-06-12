function fmtBytes(n) {
  if (n < 1024) return n + " B";
  const u = ["KB", "MB", "GB", "TB"];
  let i = -1;
  do { n /= 1024; i++; } while (n >= 1024 && i < u.length - 1);
  return n.toFixed(n >= 100 ? 0 : 1) + " " + u[i];
}

// Tiny parser for the known repo-info.yaml shape (scalars + lists of maps).
function parseYaml(text) {
  const doc = {};
  let list = null, cur = null;
  for (const raw of text.split("\n")) {
    if (!raw.trim() || raw.trim().startsWith("#")) continue;
    let m;
    if ((m = raw.match(/^(\w+):\s*(.*)$/))) {
      if (m[2] === "") { list = doc[m[1]] = []; cur = null; }
      else { doc[m[1]] = unquote(m[2]); list = null; }
    } else if (list && (m = raw.match(/^  - (\w+):\s*(.*)$/))) {
      cur = { [m[1]]: unquote(m[2]) };
      list.push(cur);
    } else if (cur && (m = raw.match(/^    (\w+):\s*(.*)$/))) {
      cur[m[1]] = unquote(m[2]);
    }
  }
  doc.entries = doc.entries || [];
  doc.ignored = Array.isArray(doc.ignored) ? doc.ignored : [];
  doc.history = doc.history || [];
  return doc;
}
function unquote(v) {
  v = v.trim();
  if (v.startsWith('"') && v.endsWith('"'))
    v = v.slice(1, -1).replace(/\\n/g, "\n").replace(/\\"/g, '"').replace(/\\\\/g, "\\");
  if (v === "true") return true;
  if (v === "false") return false;
  if (/^\d+$/.test(v)) return Number(v);
  return v;
}

function card(label, value) {
  return `<div class="card"><div class="label">${label}</div><div class="value">${value}</div></div>`;
}

const esc = s => String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;");

function isTableSeparator(line) {
  const t = line.trim();
  if (!t.includes("|") || !t.includes("-")) return false;
  return /^\|?(?:[\s|:-])+?\|?$/.test(t);
}

function isTableRow(line) {
  const t = line.trim();
  return t.includes("|") && !isTableSeparator(line);
}

function parseTableRow(line) {
  let inner = line.trim();
  if (inner.startsWith("|")) inner = inner.slice(1);
  if (inner.endsWith("|")) inner = inner.slice(0, -1);
  return inner.split("|").map(cell => cell.trim());
}

function splitCommitBody(body) {
  const lines = body.split("\n");
  const blocks = [];
  let i = 0;
  let textBuf = [];
  const flushText = () => {
    if (textBuf.length) {
      blocks.push({ type: "text", lines: textBuf.slice() });
      textBuf = [];
    }
  };
  while (i < lines.length) {
    if (isTableRow(lines[i]) && i + 1 < lines.length && isTableSeparator(lines[i + 1])) {
      flushText();
      const tableLines = [lines[i], lines[i + 1]];
      i += 2;
      while (i < lines.length && isTableRow(lines[i])) {
        tableLines.push(lines[i]);
        i += 1;
      }
      blocks.push({ type: "table", lines: tableLines });
    } else {
      textBuf.push(lines[i]);
      i += 1;
    }
  }
  flushText();
  return blocks;
}

function normalizeAsciiIndent(lines) {
  if (lines.length < 2) return lines;
  const rest = lines.slice(1).filter(l => l.trim());
  if (!rest.length) return lines;
  const minRestIndent = Math.min(...rest.map(l => (l.match(/^ */) || [""])[0].length));
  const firstIndent = (lines[0].match(/^ */) || [""])[0].length;
  if (firstIndent < minRestIndent) {
    const pad = " ".repeat(minRestIndent - firstIndent);
    return [pad + lines[0], ...lines.slice(1)];
  }
  return lines;
}

function renderTextBlock(lines, compactFirst8) {
  if (!lines.length) return "";
  let html = `<div class="commit-body-text">`;
  if (compactFirst8) {
    const compactLines = normalizeAsciiIndent(lines.slice(0, 8));
    const restLines = lines.slice(8);
    if (compactLines.length) {
      html += `<pre class="commit-body-compact">${esc(compactLines.join("\n"))}</pre>`;
    }
    if (restLines.length) {
      html += `<pre class="commit-body-pre">${esc(restLines.join("\n"))}</pre>`;
    }
  } else {
    html += `<pre class="commit-body-pre">${esc(lines.join("\n"))}</pre>`;
  }
  return html + `</div>`;
}

function renderMarkdownTable(lines) {
  const header = parseTableRow(lines[0]);
  const rows = lines.slice(2).map(parseTableRow);
  let html = `<div class="commit-body-table-wrap"><table class="commit-body-table"><thead><tr>`;
  for (const cell of header) html += `<th>${esc(cell)}</th>`;
  html += `</tr></thead><tbody>`;
  for (const row of rows) {
    html += `<tr>`;
    for (const cell of row) html += `<td>${esc(cell)}</td>`;
    html += `</tr>`;
  }
  return html + `</tbody></table></div>`;
}

function renderCommitBody(body) {
  const blocks = splitCommitBody(body.trim());
  if (!blocks.length) return "";
  let html = `<div class="commit-body">`;
  let firstText = true;
  for (const block of blocks) {
    if (block.type === "text") {
      html += renderTextBlock(block.lines, firstText);
      firstText = false;
    } else {
      html += renderMarkdownTable(block.lines);
    }
  }
  return html + `</div>`;
}

async function load() {
  let doc;
  try {
    const r = await fetch("repo-info.yaml", { cache: "no-cache" });
    if (!r.ok) throw new Error("HTTP " + r.status);
    doc = parseYaml(await r.text());
  } catch (e) {
    const el = document.getElementById("error");
    el.style.display = "block";
    el.textContent = "Could not load repo-info.yaml (" + e.message + "). Start the repo server — it regenerates the snapshot on startup.";
    document.getElementById("sub").textContent = "";
    return;
  }

  const repoName = (doc.repo_path || "").split("/").filter(Boolean).pop() || "repo";
  document.title = repoName + " — Repo Info";
  document.getElementById("title").textContent = repoName;

  const when = doc.generated_unix ? new Date(doc.generated_unix * 1000).toLocaleString() : "?";
  document.getElementById("sub").innerHTML =
    [doc.git_remote && `<code>${doc.git_remote}</code>`,
     doc.git_branch && `branch <code>${doc.git_branch}</code>`,
     doc.git_commit && `latest: <code>${doc.git_commit}</code>`]
      .filter(Boolean).join(" · ");

  const dirs = doc.entries.filter(e => e.is_dir);
  const files = doc.entries.filter(e => !e.is_dir);
  const biggest = dirs[0];

  document.getElementById("cards").innerHTML =
    card("Total size", fmtBytes(doc.total_bytes)) +
    card("Files", Number(doc.total_files).toLocaleString()) +
    card("Root folders", dirs.length) +
    (biggest ? card("Biggest folder", `${biggest.name}/ · ${fmtBytes(biggest.bytes)}`) : "");

  const max = Math.max(...dirs.map(e => e.bytes), 1);
  document.getElementById("dirs").insertAdjacentHTML("beforeend",
    dirs.map(e => `<tr>
      <td>${e.name}/</td>
      <td class="size">${fmtBytes(e.bytes)}</td>
      <td><div class="bar"><i style="width:${Math.max(1, 100 * e.bytes / max).toFixed(1)}%"></i></div></td>
      <td class="num">${Number(e.files).toLocaleString()}</td>
    </tr>`).join(""));

  document.getElementById("files").insertAdjacentHTML("beforeend",
    files.map(e => `<tr><td>${e.name}</td><td class="size">${fmtBytes(e.bytes)}</td></tr>`).join(""));

  const ignDirs = doc.ignored.filter(e => e.is_dir);
  const ignFiles = doc.ignored.filter(e => !e.is_dir);
  document.getElementById("ign-dirs").insertAdjacentHTML("beforeend",
    ignDirs.length
      ? ignDirs.map(e => `<tr><td class="muted">${e.name}/</td><td class="size">${fmtBytes(e.bytes)}</td><td class="num">${Number(e.files).toLocaleString()}</td></tr>`).join("")
      : `<tr><td class="muted" colspan="3">none</td></tr>`);
  document.getElementById("ign-files").insertAdjacentHTML("beforeend",
    ignFiles.length
      ? ignFiles.map(e => `<tr><td class="muted">${e.name}</td><td class="size">${fmtBytes(e.bytes)}</td></tr>`).join("")
      : `<tr><td class="muted" colspan="2">none</td></tr>`);

  const bodiesOpen = doc.history_expanded === true;
  document.getElementById("history").insertAdjacentHTML("beforeend",
    doc.history.map(c => {
      const body = String(c.body || "").trim();
      if (!body) {
        return `<tr class="history-head">
          <td><code>${esc(c.hash)}</code></td>
          <td class="size">${esc(c.date)}</td>
          <td class="muted">${esc(c.author)}</td>
          <td>${esc(c.subject)}</td>
        </tr>`;
      }
      const chev = bodiesOpen ? "▾" : "▸";
      return `<tr class="history-head">
          <td><code>${esc(c.hash)}</code></td>
          <td class="size">${esc(c.date)}</td>
          <td class="muted">${esc(c.author)}</td>
          <td>
            <button type="button" class="history-toggle" aria-expanded="${bodiesOpen}">
              ${esc(c.subject)} <span class="chev">${chev}</span>
            </button>
          </td>
        </tr>
        <tr class="history-detail"${bodiesOpen ? "" : " hidden"}>
          <td colspan="4">${renderCommitBody(body)}</td>
        </tr>`;
    }).join(""));

  const detailRows = [...document.querySelectorAll("#history tr.history-detail")];
  const toggleBtn = document.getElementById("toggle-bodies");
  if (detailRows.length) {
    toggleBtn.hidden = false;
    const setExpanded = (row, btn, open) => {
      row.hidden = !open;
      if (btn) {
        btn.setAttribute("aria-expanded", open);
        const chev = btn.querySelector(".chev");
        if (chev) chev.textContent = open ? "▾" : "▸";
      }
    };
    const pairs = detailRows.map(row => ({
      row,
      btn: row.previousElementSibling?.querySelector(".history-toggle"),
    }));
    pairs.forEach(({ row, btn }) => {
      if (btn) btn.onclick = () => setExpanded(row, btn, row.hidden);
    });
    const syncBtn = () => {
      toggleBtn.textContent = detailRows.every(r => !r.hidden) ? "Collapse all" : "Expand all";
    };
    toggleBtn.onclick = () => {
      const expand = detailRows.some(r => r.hidden);
      pairs.forEach(({ row, btn }) => setExpanded(row, btn, expand));
      syncBtn();
    };
    syncBtn();
  }

  const fmtIgnore = v => Array.isArray(v) ? v.join(", ") : String(v || "").replace(/[\[\]"]/g, "");
  const ignFolderNote = fmtIgnore(doc.ignore_folders);
  const ignFileNote = fmtIgnore(doc.ignore_files);
  const ignoreNote = [
    ignFolderNote && "folders: " + ignFolderNote,
    ignFileNote && "files: " + ignFileNote,
  ].filter(Boolean).join(" · ");
  document.getElementById("note").textContent =
    "Snapshot generated " + when + " — regenerated each time the repo server starts." +
    (ignoreNote ? " Ignore " + ignoreNote + " (edit Admin/repo-info/config.yaml)." : "");
}

load();
