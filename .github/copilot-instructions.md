# GitHub Copilot Instructions

## Testing and Quality Checks

**Always verify changes by running:**

1. **Run tests**: `cargo test`
2. **Run clippy**: `cargo clippy -- -D warnings`

These checks must be run after making any code changes to ensure quality and prevent regressions.

## Documentation

**Do not create markdown files for summaries, notes, or tracking.** Work in memory instead. Only create markdown files when explicitly requested by name or path.
