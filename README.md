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

### Server Instructions (Critical)

**You must add instructions to your MCP server telling agents to use the feedback tool.** Without explicit server instructions, agents will see the feedback tool in their tool list but will almost never call it on their own. The tool description alone is not enough — agents need to be told directly, via the server's `instructions` field, that they are expected to report limitations.

This is the single most important step in integrating PatchworkMCP. If you skip it, you will receive little to no feedback, and the integration will appear broken even though everything is wired up correctly.

#### Why this matters

MCP servers can return an `instructions` field during initialization. This text is injected into the agent's system prompt and treated as authoritative guidance. Without it:

- Agents discover the `feedback` tool but treat it as optional.
- Most agents will not call a tool unless they are told to, even if the tool description says "call this whenever…".
- You end up with zero feedback data, defeating the purpose of the integration.

#### What to set

When your MCP server handles the `initialize` request, include an `instructions` string. Use the following text as a starting point and adapt it to your server's domain:

```text
IMPORTANT: If you encounter a limitation — a missing tool, incomplete data,
wrong format, or any gap that prevents you from fully completing the user's
request — you MUST call the `feedback` tool BEFORE responding to the user.
Be specific about what you needed, what you tried, and what would have helped.
Do not skip this step. Reporting limitations is how this server improves.
```

#### Example (pseudo-code)

```rust
// When building your MCP server's initialize response:
let instructions = concat!(
    "IMPORTANT: If you encounter a limitation — a missing tool, incomplete data, ",
    "wrong format, or any gap that prevents you from fully completing the user's ",
    "request — you MUST call the `feedback` tool BEFORE responding to the user. ",
    "Be specific about what you needed, what you tried, and what would have helped. ",
    "Do not skip this step. Reporting limitations is how this server improves."
);

// Pass `instructions` in your server's InitializeResult.
```

#### Checklist

1. Register the `feedback` tool using `TOOL_NAME`, `TOOL_DESCRIPTION`, and `tool_input_schema()`.
2. Set the server `instructions` field with the text above (or your own variant).
3. Verify by asking your agent to perform a task your server cannot handle — the agent should call `feedback` before responding.

See [FEEDBACK_TOOL_INSTRUCTIONS.md](./FEEDBACK_TOOL_INSTRUCTIONS.md) for a detailed guide you can reference or share with your team.

## How It Works

- Retries up to 2 times with exponential backoff (500ms, 1000ms)
- Retries on 429 (rate limit) and 5xx (server error) status codes
- Uses a static `reqwest::Client` for connection pooling and TLS session reuse
- On failure, logs the full payload with `PATCHWORKMCP_UNSENT_FEEDBACK` prefix for later replay
- Never panics — always returns a user-facing message

## License

MIT
