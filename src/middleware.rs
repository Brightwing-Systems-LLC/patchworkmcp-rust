//! PatchworkMCP middleware for Rust MCP servers.
//!
//! Provides heartbeat monitoring. Call `start_middleware()` after your server
//! starts. It sends periodic heartbeat pings to PatchworkMCP so you can
//! monitor server uptime and tool availability.
//!
//! Configuration via environment (or MiddlewareOptions):
//!   PATCHWORKMCP_URL          - default: https://patchworkmcp.com
//!   PATCHWORKMCP_API_KEY      - required API key
//!   PATCHWORKMCP_SERVER_SLUG  - required server identifier

use serde::Serialize;
use std::env;
use std::time::Duration;
use tokio::task::JoinHandle;

use crate::CLIENT;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug, Serialize)]
struct HeartbeatPayload {
    server_slug: String,
    tool_count: usize,
    tool_names: Vec<String>,
}

/// Configuration for the PatchworkMCP middleware.
pub struct MiddlewareOptions {
    /// Override PATCHWORKMCP_URL.
    pub patchwork_url: Option<String>,
    /// Override PATCHWORKMCP_API_KEY.
    pub api_key: Option<String>,
    /// Override PATCHWORKMCP_SERVER_SLUG.
    pub server_slug: Option<String>,
    /// List of tool names your server exposes.
    pub tool_names: Vec<String>,
}

/// Handle to the running middleware. Call `stop()` to cancel the heartbeat loop.
pub struct PatchworkMiddleware {
    cancel: Option<tokio::sync::oneshot::Sender<()>>,
    handle: Option<JoinHandle<()>>,
}

impl PatchworkMiddleware {
    /// Stop the heartbeat loop.
    pub fn stop(self) {
        if let Some(tx) = self.cancel {
            let _ = tx.send(());
        }
        if let Some(handle) = self.handle {
            handle.abort();
        }
    }
}

fn resolve_mw_url(opts: &MiddlewareOptions) -> String {
    opts.patchwork_url
        .clone()
        .unwrap_or_else(|| {
            env::var("PATCHWORKMCP_URL")
                .unwrap_or_else(|_| "https://patchworkmcp.com".to_string())
        })
        .trim_end_matches('/')
        .to_string()
}

fn resolve_mw_key(opts: &MiddlewareOptions) -> Option<String> {
    opts.api_key
        .clone()
        .or_else(|| env::var("PATCHWORKMCP_API_KEY").ok())
        .filter(|k| !k.is_empty())
}

fn resolve_mw_slug(opts: &MiddlewareOptions) -> String {
    opts.server_slug
        .clone()
        .unwrap_or_else(|| {
            env::var("PATCHWORKMCP_SERVER_SLUG").unwrap_or_else(|_| "unknown".to_string())
        })
}

/// Start the PatchworkMCP heartbeat middleware.
///
/// Spawns a background tokio task that sends heartbeats every 60 seconds.
/// Returns a handle that can be used to stop the middleware.
///
/// ```rust,no_run
/// let mw = patchworkmcp::start_middleware(patchworkmcp::MiddlewareOptions {
///     patchwork_url: None,
///     api_key: None,
///     server_slug: None,
///     tool_names: vec!["my_tool".to_string()],
/// });
/// // ... later ...
/// mw.stop();
/// ```
pub fn start_middleware(opts: MiddlewareOptions) -> PatchworkMiddleware {
    let api_url = resolve_mw_url(&opts);
    let api_key = resolve_mw_key(&opts);
    let server_slug = resolve_mw_slug(&opts);

    if api_key.is_none() || server_slug == "unknown" || server_slug.is_empty() {
        eprintln!("PatchworkMCP middleware not started: missing API_KEY or SERVER_SLUG");
        return PatchworkMiddleware {
            cancel: None,
            handle: None,
        };
    }

    let tool_names = opts.tool_names;
    let slug = server_slug.clone();
    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

    eprintln!("PatchworkMCP middleware started for {server_slug}");

    let handle = tokio::spawn(async move {
        // Send an initial heartbeat immediately.
        send_heartbeat(&api_url, api_key.as_deref(), &server_slug, &tool_names).await;

        let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);
        interval.tick().await; // consume the first (immediate) tick

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    send_heartbeat(&api_url, api_key.as_deref(), &server_slug, &tool_names).await;
                }
                _ = &mut rx => {
                    break;
                }
            }
        }
    });

    PatchworkMiddleware {
        cancel: Some(tx),
        handle: Some(handle),
    }
}

async fn send_heartbeat(
    api_url: &str,
    api_key: Option<&str>,
    server_slug: &str,
    tool_names: &[String],
) {
    let payload = HeartbeatPayload {
        server_slug: server_slug.to_string(),
        tool_count: tool_names.len(),
        tool_names: tool_names.to_vec(),
    };

    let endpoint = format!("{api_url}/api/v1/heartbeat/");
    let mut req = CLIENT.post(&endpoint).json(&payload);
    if let Some(key) = api_key {
        req = req.header("Authorization", format!("Bearer {key}"));
    }

    match req.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            if status >= 400 {
                eprintln!("PatchworkMCP heartbeat returned {status}");
            }
        }
        Err(e) => {
            eprintln!("PatchworkMCP heartbeat failed: {e}");
        }
    }
}
