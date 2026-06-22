//! `/api/run` — execute commands for the docs guide's "Run" buttons.
//!
//! Two request shapes, both returning the same JSON
//! (`command`, `exit_code`, `stdout`, `stderr`):
//!
//!   { "id": "go_version" }   → run a whitelisted fixed command (always allowed)
//!   { "script": "brew ..." } → run an arbitrary script via zsh (same-origin only)
//!
//! The whitelist path can never run anything the client invents. The script
//! path is gated on [`origin_ok`] so only pages served by this server (or
//! non-browser clients like curl) can use it — a random website in the same
//! browser sends a foreign `Origin` and is rejected.

use serde_json::json;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Fixed commands runnable by id. The client sends only the key, so it can
/// never inject arguments or a different program.
const RUN_WHITELIST: &[(&str, &str, &[&str])] = &[
    ("sw_vers", "sw_vers", &[]),
    ("uname", "uname", &["-a"]),
    ("whoami", "whoami", &[]),
    ("date", "date", &[]),
    ("uptime", "uptime", &[]),
    // macOS Reset — page "1" (2-second). Non-destructive only: these just
    // open a System Settings pane, they don't change anything.
    (
        "open_about",
        "open",
        &["x-apple.systempreferences:com.apple.SystemProfiler.AboutExtension"],
    ),
    (
        "open_users",
        "open",
        &["x-apple.systempreferences:com.apple.preferences.users"],
    ),
    // Verify steps — print a version, change nothing.
    ("rustc_version", "rustc", &["--version"]),
    ("cargo_version", "cargo", &["--version"]),
    ("brew_version", "brew", &["--version"]),
    ("go_version", "go", &["version"]),
    ("uv_version", "uv", &["--version"]),
    ("deno_version", "deno", &["--version"]),
];

/// Kill a runaway command (e.g. one waiting on a sudo password) after this long.
const RUN_TIMEOUT: Duration = Duration::from_secs(120);

/// True if a script request may run: no `Origin` header (curl, same-origin
/// navigations) or an `Origin` whose host is loopback. Foreign sites are
/// rejected so they can't drive the local machine.
pub fn origin_ok(origin: Option<&str>) -> bool {
    let Some(origin) = origin else {
        return true; // non-browser client (curl, etc.); no CSRF surface
    };
    // origin is "scheme://host[:port]"; isolate the bare host.
    let after_scheme = origin.split_once("://").map(|(_, r)| r).unwrap_or(origin);
    let host = after_scheme.split('/').next().unwrap_or(after_scheme); // drop any path
    let host = host.rsplit_once(':').map(|(h, _)| h).unwrap_or(host); // drop any port
    matches!(host, "localhost" | "127.0.0.1" | "[::1]" | "::1")
}

/// Handle a POST /api/run body. `allow_script` comes from [`origin_ok`].
pub fn api_run(body: &[u8], allow_script: bool) -> (u16, String) {
    let parsed: serde_json::Value = match serde_json::from_slice(body) {
        Ok(v) => v,
        Err(_) => return (400, r#"{"error":"invalid JSON body"}"#.into()),
    };

    // Arbitrary script branch.
    if let Some(script) = parsed.get("script").and_then(|v| v.as_str()) {
        if !allow_script {
            return (
                403,
                json!({ "error": "script execution is only allowed from this server's own pages" })
                    .to_string(),
            );
        }
        if script.trim().is_empty() {
            return (400, json!({ "error": "empty script" }).to_string());
        }
        let label = script
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .unwrap_or("(script)");
        let mut cmd = Command::new("zsh");
        cmd.arg("-lc").arg(script);
        return finish(&format!("zsh: {label}"), spawn(cmd));
    }

    // Whitelisted id branch.
    let id = parsed.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let Some((_, prog, args)) = RUN_WHITELIST.iter().find(|(k, _, _)| *k == id) else {
        return (
            400,
            json!({ "error": format!("unknown command id: {id}") }).to_string(),
        );
    };
    let mut cmd = Command::new(prog);
    cmd.args(*args);
    finish(&format!("{prog} {}", args.join(" ")).trim().to_string(), spawn(cmd))
}

/// Captured result of a finished (or killed) child.
struct Output {
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Spawn with piped output and wait up to [`RUN_TIMEOUT`], killing on timeout.
/// stdout/stderr are drained on their own threads so a chatty command can't
/// deadlock by filling a pipe buffer before it exits.
fn spawn(mut cmd: Command) -> std::io::Result<Output> {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn()?;

    let mut out_pipe = child.stdout.take().expect("piped stdout");
    let mut err_pipe = child.stderr.take().expect("piped stderr");
    let out_reader = thread::spawn(move || {
        let mut s = String::new();
        let _ = out_pipe.read_to_string(&mut s);
        s
    });
    let err_reader = thread::spawn(move || {
        let mut s = String::new();
        let _ = err_pipe.read_to_string(&mut s);
        s
    });

    let deadline = Instant::now() + RUN_TIMEOUT;
    let mut timed_out = false;
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            timed_out = true;
            break;
        }
        thread::sleep(Duration::from_millis(40));
    }

    let status = child.wait()?; // reap (already exited, or just killed)
    let stdout = out_reader.join().unwrap_or_default();
    let mut stderr = err_reader.join().unwrap_or_default();
    let exit_code = if timed_out { None } else { status.code() };
    if timed_out {
        stderr.push_str(&format!(
            "\n[killed: exceeded {}s timeout — interactive commands (sudo, prompts) can't run here]",
            RUN_TIMEOUT.as_secs()
        ));
    }

    Ok(Output {
        exit_code,
        stdout,
        stderr,
    })
}

/// Turn a spawn result into the JSON HTTP response.
fn finish(command: &str, result: std::io::Result<Output>) -> (u16, String) {
    match result {
        Ok(out) => (
            200,
            json!({
                "command": command,
                "exit_code": out.exit_code,
                "stdout": out.stdout,
                "stderr": out.stderr,
            })
            .to_string(),
        ),
        Err(e) => (500, json!({ "error": format!("failed to run: {e}") }).to_string()),
    }
}
