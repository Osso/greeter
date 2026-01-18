# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Minimal greetd greeter with an iced GUI. Authenticates users via the greetd IPC protocol and starts desktop sessions.

## Build Commands

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo clippy             # Lint
cargo fmt                # Format
```

## Testing

```bash
cargo test                       # Run all tests
cargo test sessions              # Run session parsing tests
cargo test --bin greeter tests:: # Run Greeter state tests
```

Integration testing requires greetd environment (`GREETD_SOCK` env var). Use `greetd-stub` for mock testing.

## Configuration

Config file: `/etc/greeter.toml`

```toml
default_session = "niri"
default_user = "osso"
```

- `default_session` - Pre-selects session by name (case-insensitive match)
- `default_user` - Pre-fills username, focuses password field on startup

## Architecture

**main.rs** - Iced application with Elm-style architecture (Greeter struct, Message enum, update/view functions). Entry point initializes tracing and runs the GUI.

**config.rs** - Loads TOML config from `/etc/greeter.toml` with defaults for missing file/fields.

**greetd.rs** - Async client for greetd IPC protocol over Unix socket. `authenticate()` handles the create-session → post-auth flow. `start_session()` launches the selected session command.

**sessions.rs** - Discovers available sessions by parsing `.desktop` files from `/usr/share/wayland-sessions` and `/usr/share/xsessions`. Distinguishes Wayland vs X11 sessions by directory.

**users.rs** - Enumerates system users from `/etc/passwd` with UID 1000-60000 (regular users).

## Key Dependencies

- `iced` (0.14) - GUI framework with tokio integration
- `greetd_ipc` - Protocol types and tokio codec for greetd communication
- `pwd` - Safe passwd file iteration
