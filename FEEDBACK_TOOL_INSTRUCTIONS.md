# Feedback Tool: Server Instructions Guide

**You must add instructions to your MCP server telling agents to use the feedback tool.** This is the single most important step when integrating PatchworkMCP. Without explicit server instructions, agents will not call the feedback tool, and you will receive no feedback data.

## Why server instructions are critical

The MCP protocol allows servers to return an `instructions` field during initialization. This text is injected into the agent's system prompt and treated as authoritative guidance. It is the primary mechanism for telling agents how to behave with your server's tools.

Without server instructions:

- Agents discover the `feedback` tool in their tool list but treat it as optional.
- Most agents will not call a tool unless they are explicitly told to, even when the tool's own description says "call this whenever…".
- You end up with zero feedback, and the integration appears broken even though everything is wired up correctly.

With server instructions:

- Agents treat calling the `feedback` tool as a required step whenever they hit a limitation.
- You receive actionable data about missing tools, incomplete results, and workarounds.
- Your server improves over time based on real agent behavior.

## Recommended instructions text

Use the following text as your server's `instructions` field. Adapt it to your server's domain if needed, but keep the directive strong and unambiguous.

```text
IMPORTANT: If you encounter a limitation — a missing tool, incomplete data,
wrong format, or any gap that prevents you from fully completing the user's
request — you MUST call the `feedback` tool BEFORE responding to the user.
Be specific about what you needed, what you tried, and what would have helped.
Do not skip this step. Reporting limitations is how this server improves.
```

### Key principles for effective instructions

1. **Use strong, direct language.** Words like "MUST" and "IMPORTANT" signal to the agent that this is not optional. Weak phrasing like "consider calling" or "you may want to" will be ignored.

2. **Specify timing.** Tell the agent to call the feedback tool BEFORE responding to the user. If you don't, agents will respond first and never circle back.

3. **Tell the agent what to include.** Ask for specifics: what they needed, what they tried, and what would have helped. Vague feedback is not actionable.

4. **Explain the purpose.** A short sentence like "Reporting limitations is how this server improves" gives the agent context for why the action matters, which increases compliance.

## How to set server instructions

When your MCP server handles the `initialize` request, include the instructions in your response. The exact mechanism depends on your MCP framework, but the pattern is always the same: return an `instructions` string in your `InitializeResult`.

### Example (Rust pseudo-code)

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
// The exact API depends on your MCP server framework.
```

### Example (JSON initialize response)

```json
{
  "protocolVersion": "2024-11-05",
  "capabilities": {
    "tools": {}
  },
  "serverInfo": {
    "name": "my-server",
    "version": "1.0.0"
  },
  "instructions": "IMPORTANT: If you encounter a limitation — a missing tool, incomplete data, wrong format, or any gap that prevents you from fully completing the user's request — you MUST call the `feedback` tool BEFORE responding to the user. Be specific about what you needed, what you tried, and what would have helped. Do not skip this step. Reporting limitations is how this server improves."
}
```

## Integration checklist

1. **Register the feedback tool.** Use `TOOL_NAME`, `TOOL_DESCRIPTION`, and `tool_input_schema()` from this library to register the tool with your MCP server.

2. **Set server instructions.** Add the instructions text above (or your own variant) to your server's `InitializeResult`.

3. **Verify the integration.** Ask your agent to perform a task that your server cannot fully handle. The agent should call the `feedback` tool before responding to you. If it does not, your instructions are missing or too weak.

4. **Monitor feedback.** Log in to [patchworkmcp.com](https://patchworkmcp.com) and confirm that feedback entries are appearing for your server.

## Common mistakes

| Mistake | Result | Fix |
|---|---|---|
| No server instructions set | Agent never calls the feedback tool | Add the `instructions` field to your initialize response |
| Weak language ("consider", "optionally") | Agent treats the tool as optional and skips it | Use "MUST" and "IMPORTANT" |
| Instructions say "after responding" | Agent responds first and never calls the tool | Say "BEFORE responding to the user" |
| Tool registered but instructions missing | Tool appears in the list but is never invoked | Always pair tool registration with server instructions |
| Instructions too generic | Agent does not know what details to include | Be specific: "what you needed, what you tried, what would have helped" |
