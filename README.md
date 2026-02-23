# PatchworkMCP - Rust

Drop-in feedback tool for Rust MCP servers. Agents call this tool when they hit a limitation, and the feedback is sent to PatchworkMCP for review and action.

## Setup

1. Go to [patchworkmcp.com](https://patchworkmcp.com) and create an account
2. Create a team and generate an API key
3. Configure your server (you'll need the server slug and API key)

## Install

Add to your `Cargo.toml`:

```toml
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
```

Then copy `src/lib.rs` into your project (or add as a dependency).

## Configure

Set these environment variables (or pass them via Options):

| Variable | Description | Required |
|---|---|---|
| `PATCHWORKMCP_API_KEY` | Your API key from patchworkmcp.com | Yes |
| `PATCHWORKMCP_SERVER_SLUG` | Your server's slug from patchworkmcp.com | Yes |
| `PATCHWORKMCP_URL` | API endpoint (default: `https://patchworkmcp.com`) | No |

## Usage

```rust
use patchworkmcp::{payload_from_args, send_feedback};

// Build payload from MCP call_tool arguments:
let payload = payload_from_args(&args, None);

// Send feedback:
let message = send_feedback(&payload, None).await;
```

### With options

```rust
use patchworkmcp::{Options, payload_from_args, send_feedback};

let opts = Options {
    patchwork_url: Some("https://custom.example.com".to_string()),
    api_key: Some("my-key".to_string()),
    server_slug: Some("my-server".to_string()),
};

let payload = payload_from_args(&args, Some(&opts));
let message = send_feedback(&payload, Some(&opts)).await;
```

### Tool registration

```rust
use patchworkmcp::{TOOL_NAME, TOOL_DESCRIPTION, tool_input_schema};

// Register with your MCP framework:
let schema = tool_input_schema();
```

### Server Instructions

When initializing your MCP server, set the `instructions` field to tell agents when to use the feedback tool:

> If you encounter a limitation — a missing tool, incomplete data, wrong format, or any gap that prevents you from fully completing the user's request — call the `feedback` tool BEFORE responding to the user. Be specific about what you needed and what would have helped.

Without server instructions, agents may see the tool but not know when to call it. Adapt the text to your server's domain if needed.

## How It Works

- Retries up to 2 times with exponential backoff (500ms, 1000ms)
- Retries on 429 (rate limit) and 5xx (server error) status codes
- Uses a static `reqwest::Client` for connection pooling and TLS session reuse
- On failure, logs the full payload with `PATCHWORKMCP_UNSENT_FEEDBACK` prefix for later replay
- Never panics — always returns a user-facing message

## License

MIT
