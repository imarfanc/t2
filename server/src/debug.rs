use colored::Colorize;
use std::env;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_CONNECTION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy)]
pub(crate) struct Diagnostics {
    enabled: bool,
}

impl Diagnostics {
    pub(crate) fn from_env() -> Self {
        Self {
            enabled: env::var_os("REPO_SERVER_DEBUG").is_some(),
        }
    }

    pub(crate) fn startup(self, root: &Path, host: &str, port: u16, request_logging: bool) {
        if self.enabled {
            eprintln!(
                "[debug startup] pid={} root={} bind={host}:{port} request_logging={request_logging}",
                std::process::id(),
                root.display()
            );
        }
    }

    pub(crate) fn next_connection_id(self) -> u64 {
        NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed)
    }

    pub(crate) fn log(self, connection_id: u64, request_id: u64, message: &str) {
        if !self.enabled {
            return;
        }

        let mut out = std::io::stdout();
        let _ = write!(
            out,
            "  {} conn={connection_id} req={request_id} {message}\r\n",
            "debug".bright_black()
        );
        let _ = out.flush();
    }
}
