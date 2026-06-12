# repo-server stall debugging notes

*2026-06-12 — investigating slow/hung navigation (especially back-nav) in `server/src/main.rs`*

## Symptom

- Initial page load fine. Clicking a link fine. Going **back** sometimes stalls for a long time.
- Pre-fix logs showed odd slow responses: 214.7 ms, 202.6 ms, 703.9 ms on a local static server that normally responds in <1 ms.
- After the first round of fixes, navigation to `/docs/install-rust/` hung entirely in a fresh Chrome tab.

## Original code problems (first round of fixes, already applied)

1. **`Connection: close` on HTTP/1.1, one request per connection.**
   Browsers preconnect sockets and expect HTTP/1.1 keep-alive by default. The server killed every socket after one request (or silently after a 5s read timeout). On back-nav, Chrome can grab a dead pooled socket → RST → retry, which looks like a random multi-hundred-ms (or worse) stall.
2. **No `TCP_NODELAY`, headers and body written separately.**
   Nagle's algorithm + delayed ACK can add ~40–200 ms per response. This matches the 202/214 ms log lines.
3. **No `Cache-Control` headers.** Every back-nav refetched all assets.

### Changes made

- `handle()` now loops, serving multiple requests per connection (keep-alive), honoring `Connection: close` from the client; idle read timeout raised 5s → 120s.
- `set_nodelay(true)`; response header + body built into one buffer and sent with a single `write_all`.
- `Cache-Control: no-cache` for HTML, `max-age=60` for other assets.
- `respond()` gained a `keep_alive` parameter and emits `Connection: keep-alive`/`close` accordingly.

## New observation after the fix (NOT yet resolved)

Reproduced live with the rebuilt server:

- Navigated Chrome (chrome3Air, same machine) to `http://localhost:8008/` — loaded fine.
- Navigated to `/docs/install-rust/` — **tab hung indefinitely** (extension couldn't even access page contents).
- Server terminal (cmux) showed a **runaway request flood**: alternating
  `200 GET /` and `200 GET /docs/install-rust/index.html`, each <1–10 ms, repeating dozens+ of times per second, nonstop.
- Meanwhile a cmux built-in browser pane showed URL `/docs/install-rust/` but **rendered the content of `/`** (the repo index) — i.e. the client appears to receive the *wrong body* for a request.

### What the flood implies

- The server is responding fast (all 200s, sub-ms), so the stall is not server-side slowness.
- Wrong-body-for-URL on a keep-alive connection is the classic signature of **broken response framing / response-to-request misalignment**: if a client receives a response it can't trust (length mismatch, leftover bytes, or response intended for a different request), it discards and retries → retry loop → "hang" in the visible tab.
- Notably, requests for `/docs/install-rust/index.html` (explicit `index.html`) never appeared in pre-keep-alive logs. Something — likely a client retry/normalization path or the page itself — started requesting it directly.

### Suspects to check next

1. **Response framing on the keep-alive path.** Verify byte-exact responses, e.g.:
   `printf 'GET / HTTP/1.1\r\nHost: x\r\n\r\nGET /docs/install-rust/ HTTP/1.1\r\nHost: x\r\n\r\n' | nc localhost 8008 | head -c 2000`
   Check both responses are well-formed and in order, with correct `Content-Length`.
2. **HEAD handling on keep-alive**: `respond()` sends `Content-Length: N` with no body for HEAD — correct per spec, but verify no client desync.
3. **Multiple clients involved**: the flood may have been my Chrome MCP tab retrying, the cmux webview, or both. Isolate by testing with a single `curl` / `nc` client and watching the log.
4. **What's in the served HTML**: check whether `/docs/install-rust/index.html` or shared JS does any fetch/redirect to `/` that could explain the alternation.
5. If framing checks out, consider reverting to `Connection: close` but keeping `TCP_NODELAY` + single-write + cache headers, and see if stalls disappear — that isolates keep-alive as the culprit.

## Environment notes

- Machine: m5Air, 16 GB RAM, heavily loaded at the time (14 GB used, 5.6 GB swap, memory pressure yellow) — could exaggerate stalls but doesn't explain the request flood or wrong-body rendering.
- Many idle `zsh` processes in Activity Monitor — unrelated.
- Server is run via `just serve` → `cargo run --release --manifest-path server/Cargo.toml`.

## Status

- Round-1 fixes (nodelay, single write, cache headers, keep-alive) are in `server/src/main.rs`.
- Added `just serve-debug` preflight output plus per-connection/request diagnostics
  enabled by `REPO_SERVER_DEBUG=1` in `just serve`.
- The pipelining test returned correctly framed, ordered responses with matching
  `Content-Length` values.
- **Root cause found:** every regex in `docs/_shared/code-highlight.js` lacked
  the global `g` flag, while `highlight()` used `RegExp.exec()` in a `while`
  loop. The first successful match therefore repeated forever, exhausting the
  renderer until the in-app browser reported "This page crashed." All
  highlighter regexes now use `g`.
