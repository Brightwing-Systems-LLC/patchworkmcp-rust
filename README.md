# PatchworkMCP - Rust SDK

PatchworkMCP helps MCP server authors understand how agents actually use their servers — what works, what's missing, and what to build next. This SDK adds three things to your server:

1. **A feedback tool** that agents call when they hit a limitation (missing tool, incomplete data, wrong format). This creates a structured stream of real-world gap reports visible on your PatchworkMCP dashboard.
2. **Server instructions** that tell agents when and how to use the feedback tool. Without these, agents see the tool but don't know when to call it.
3. **A heartbeat monitor** that pings PatchworkMCP every 60 seconds so you can track uptime and see which tools your server exposes.

None of these change your server's existing behavior. They're additive — your existing tools, resources, and prompts stay exactly as they are.

> **Note:** The Rust MCP ecosystem is still maturing. This SDK provides the feedback payload, HTTP submission, schema constants, and heartbeat middleware. Wire the feedback tool into your MCP framework's registration system as needed.

## Quick Start

### 1. Create a PatchworkMCP account

Go to [patchworkmcp.com](https://patchworkmcp.com), create a team, register your server, and generate an API key. You'll need:
- Your **API key**
- Your **server slug** (the identifier for your server on PatchworkMCP)

### 2. Install the SDK

Add to your `Cargo.toml`:

```toml
[dependencies]
patchworkmcp = "0.1"
```

Or copy the source files into your project. Dependencies: `reqwest`, `serde`, `serde_json`, `tokio`.

### 3. Set environment variables

| Variable | Description | Required |
|---|---|---|
| `PATCHWORKMCP_API_KEY` | Your API key from patchworkmcp.com | Yes |
| `PATCHWORKMCP_SERVER_SLUG` | Your server's slug from patchworkmcp.com | Yes |
| `PATCHWORKMCP_URL` | API endpoint (default: `https://patchworkmcp.com`) | No |

### 4. Add to your server

```rust
use patchworkmcp::{
    TOOL_NAME, TOOL_DESCRIPTION, tool_input_schema,
    payload_from_args, send_feedback,
    start_middleware, MiddlewareOptions,
};

// -- Server instructions --
// This is what tells agents to use the feedback tool. Without it,
// agents may see the tool but won't know when to call it.
let instructions = "If you encounter a limitation — a missing tool, incomplete data, \
    wrong format, or any gap that prevents you from fully completing the user's \
    request — call the `feedback` tool BEFORE responding to the user. Be specific \
    about what you needed and what would have helped.";

// -- Feedback tool --
// Register with your MCP framework using these constants:
let schema = tool_input_schema();
// TOOL_NAME = "feedback"
// TOOL_DESCRIPTION = "Report when you cannot find what you need..."

// In your tool handler:
// let payload = payload_from_args(&args, None);
// let message = send_feedback(&payload, None).await;

// -- Heartbeat monitor --
// Sends a ping every 60s with your server slug and tool list.
let mw = start_middleware(MiddlewareOptions {
    patchwork_url: None,
    api_key: None,
    server_slug: None,
    tool_names: vec!["my_tool_1".to_string(), "my_tool_2".to_string()],
});

// To stop later: mw.stop()
```

## What Each Piece Does

### Feedback Tool

When an agent can't find the right tool, gets incomplete results, or has to work around a limitation, it calls the `feedback` tool with structured data:

- **what_i_needed** — the capability or data it was looking for
- **what_i_tried** — which tools it tried and what happened
- **gap_type** — category: `missing_tool`, `incomplete_results`, `missing_parameter`, `wrong_format`, `other`
- **suggestion** — the agent's idea for what would help

These reports appear on your PatchworkMCP dashboard, giving you a prioritized list of what to build next based on real agent usage.

### Server Instructions (Critical)

**You must add instructions to your MCP server telling the agent to use the feedback tool.** This is the single most important step in the integration. Without explicit instructions, agents will silently ignore the feedback tool — even though it appears in their tool list.

The `instructions` field on your MCP server is what makes the feedback tool useful. It tells agents: "if you hit a wall, report it before responding." Key principles:

1. **Tell the agent it is required.** Agents treat server instructions as authoritative. If you don't say "you must call the feedback tool," they won't.
2. **Specify when to call it.** List the concrete scenarios: missing tool, incomplete results, wrong format, about to say "not possible."
3. **Say to call it BEFORE responding.** If the agent responds first, it rarely circles back to submit feedback.
4. **Ask for specifics.** Generic feedback like "something was missing" is not actionable.

Without instructions, PatchworkMCP receives zero signal about what your server is missing. **No instructions = no feedback.**

For the full guide on writing effective agent instructions, see [FEEDBACK_TOOL_INSTRUCTIONS.md](FEEDBACK_TOOL_INSTRUCTIONS.md).

### Heartbeat Monitor

The middleware sends a heartbeat to PatchworkMCP every 60 seconds containing:
- Your server slug
- How many tools your server exposes
- The list of tool names

This powers uptime monitoring on your dashboard and lets PatchworkMCP track which tools are live.

## Alternative Integration Patterns

### Override configuration

```rust
use patchworkmcp::{Options, MiddlewareOptions, payload_from_args, send_feedback, start_middleware};

let opts = Options {
    patchwork_url: Some("https://custom.example.com".to_string()),
    api_key: Some("my-key".to_string()),
    server_slug: Some("my-server".to_string()),
};

let payload = payload_from_args(&args, Some(&opts));
let message = send_feedback(&payload, Some(&opts)).await;

let mw = start_middleware(MiddlewareOptions {
    patchwork_url: Some("https://custom.example.com".to_string()),
    api_key: Some("my-key".to_string()),
    server_slug: Some("my-server".to_string()),
    tool_names: vec!["my_tool_1".to_string()],
});
```

## Reliability

- Feedback submissions retry up to 2 times with exponential backoff (500ms, 1000ms)
- Retries on 429 (rate limit) and 5xx (server error) status codes
- Uses a static `reqwest::Client` for connection pooling and TLS session reuse
- On failure, logs the full payload with `PATCHWORKMCP_UNSENT_FEEDBACK` prefix for later replay
- Never panics — always returns a user-facing message
- Heartbeats are fire-and-forget; failures are logged but don't affect your server

## License

MIT
