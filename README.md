# Stealth Assistant

A transparent, click-through AI assistant overlay built with Rust + egui. It floats above your other windows — hidden from screen recorders — so you can ask questions without leaving your current workspace.

## Features

- **Transparent overlay window** with adjustable opacity and click-through mode
- **Stealth mode**: hidden from screen recorders/capture on Windows (`WDA_EXCLUDEFROMCAPTURE`) and set as a notification-type window on X11
- **Multi-provider support**: OpenAI, DeepSeek, Gemini, and Claude
- **Streaming responses** — text appears token-by-token as the model generates it
- **Custom system prompt** editable in the Settings tab
- **Global hotkey support** and audio capture deps wired in for future features

## Supported Platforms

| Platform | Status |
| --- | --- |
| Linux (X11) | ✅ Primary |
| Windows | ✅ Implemented |
| macOS | ⚠️ Partially implemented |

## Building

### Prerequisites

- Rust stable (edition 2024), e.g. via [rustup](https://rustup.rs)
- Linux only: X11 dev headers (`libx11-dev`, `libxcb-shape0-dev`) and a working OpenGL for egui

```bash
cargo build --release
./target/release/stealth-assistant
```

## Configuration

Configuration is stored in `stealth_config.json` in the OS-standard config directory (auto-created with defaults on first run):

| Platform | Location |
| --- | --- |
| Windows | `%APPDATA%\stealth-assistant\stealth_config.json` |
| macOS | `~/Library/Application Support/stealth-assistant/stealth_config.json` |
| Linux | `$XDG_CONFIG_HOME/stealth-assistant/stealth_config.json` or `~/.config/stealth-assistant/stealth_config.json` |

On first run after upgrading, a pre-existing `stealth_config.json` in the working directory is migrated to the new location automatically.

Options:

| Key | Type | Default | Description |
| --- | --- | --- | --- |
| `selected_provider` | `string` | `"Gemini"` | Active provider: `Gemini`, `OpenAI`, `DeepSeek`, or `Claude` |
| `openai_key` | `string` | `""` | OpenAI API key |
| `openai_model` | `string` | `"gpt-4o"` | OpenAI model id |
| `deepseek_key` | `string` | `""` | DeepSeek API key |
| `deepseek_model` | `string` | `"deepseek-chat"` | DeepSeek model id |
| `gemini_key` | `string` | `""` | Google Gemini API key |
| `gemini_model` | `string` | `"gemini-1.5-flash"` | Gemini model id |
| `claude_key` | `string` | `""` | Anthropic Claude API key |
| `claude_model` | `string` | `"claude-3-5-sonnet-20240620"` | Claude model id |
| `system_prompt` | `string` | *(see below)* | System context sent with every request |
| `opacity` | `float` | `0.90` | Window opacity (0.0–1.0) |
| `is_click_through` | `bool` | `false` | Start with click-through enabled |
| `enable_stealth_on_launch` | `bool` | `true` | Hide from screen recorders on startup |

All of these can also be edited in the **Settings** tab (select the provider to show its key/model fields). Click **Save Settings** to persist.

### Default system prompt

```
You are an expert AI assistant. Keep responses as short as possible while still being complete and informative.
No filler, no fluff, no unnecessary greetings. Always give technically correct, precise, and genuinely intelligent
answers, not generic or surface-level responses. Think before answering. Never use bullet points, markdown, or any
formatting. Respond only in plain conversational text, as if speaking directly to someone in an interview.
```

## Usage

1. Type your question in the input box at the bottom (or press **Ctrl+Enter**).
2. Click **Send** (or press **Ctrl+Enter**).
3. The response streams into the chat area.
4. **Clear** empties the response.

The top bar includes:

- **💬 Assistant / ⚙ Settings** — switch tabs
- **Click-Through** checkbox — when enabled, mouse clicks pass through the window (except over the UI controls themselves), letting you interact with what's underneath
- Provider name — the currently active provider

## Stealth & Click-Through Notes

- **Click-through** uses the shape/input extension on X11 (`shape::SK::INPUT`) and `WS_EX_TRANSPARENT` on Windows. It toggles all-or-nothing window-wide; per-widget exceptions (interacting with your other apps except through the chat input) are on the roadmap.
- **Screen-recorder hiding** on Windows uses `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`. On Linux the window is marked `_NET_WM_WINDOW_TYPE_NOTIFICATION`. None of these protect against OS-level or secure-desktop capture.

## Debugging

If a request appears stuck on **"Thinking..."**, common causes:

- The API key is invalid or the model id doesn't exist for the selected provider (the app currently surfaces these errors silently on empty responses).
- No network connection to the provider endpoint.
- A TLS/certificate issue in `reqwest`.

## Cross-Compiling for Windows (from Linux)

For quick Windows testing without CI:

```bash
rustup target add x86_64-pc-windows-gnu
sudo apt install gcc-mingw-w64-x86-64

# .cargo/config.toml
# [target.x86_64-pc-windows-gnu]
# linker = "x86_64-w64-mingw32-gcc"

cargo build --target x86_64-pc-windows-gnu
```

> Note: `whisper-rs` (bundles C/C++ whisper.cpp) and `cpal` commonly fail under mingw cross-compilation. For testing, a native Linux build under Wine, or a Windows VM, is often more reliable.

## Project Layout

```
src/
├── main.rs        # Entry point, tokio runtime, eframe setup
├── app.rs         # UI: chat, settings, theming, request orchestration
├── config.rs      # AppConfig load/save from stealth_config.json
├── llm/           # Provider clients: openai.rs, gemini.rs, claude.rs
├── platform/      # Per-OS stealth + click-through (linux, windows, macos)
└── audio/         # Audio capture scaffolding (reserved)
```