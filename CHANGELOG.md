# Changelog

## 0.1.0 (2026-02-24)

- Initial release
- Core submission via `send_feedback()`
- Payload builder via `payload_from_args()`
- JSON schema via `tool_input_schema()`
- Retry logic with exponential backoff
- Connection pooling via LazyLock reqwest::Client
- Structured logging for failed submissions
