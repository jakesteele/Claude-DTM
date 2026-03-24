# Claude-DTM

A dynamic tiling terminal manager for running multiple [Claude Code](https://docs.anthropic.com/en/docs/claude-code) sessions simultaneously. Each session runs in its own git worktree, displayed in a keyboard-driven tiling layout.

Think **"tmux meets dwm, purpose-built for Claude Code."**

![Claude-DTM running in Ghostty](assets/screenshot.png)

## Features

- **Tiling layouts** — Master-stack, monocle (fullscreen), and grid layouts
- **Git worktree isolation** — Each Claude session gets its own worktree and branch
- **PTY multiplexing** — Real terminal emulation via `vt100` + `portable-pty`
- **Status detection** — Auto-detects if Claude is running, waiting for input, or done
- **Color-coded borders** — Yellow (running), green (waiting), grey (done), blue (paused)
- **Session persistence** — Sessions save on exit and can be resumed on restart
- **Search sessions** — Fuzzy-find sessions by name or branch with `s`
- **Keyboard-only** — No mouse needed, dwm-style keybindings
- **Futuristic TUI** — Built with `ratatui`, includes key legend and statusbar

## Install

### From source (requires Rust)

```bash
git clone https://github.com/jakesteele/Claude-DTM.git
cd Claude-DTM
cargo install --path .
```

This installs `claude-dtm` to `~/.cargo/bin/` (already in your PATH if Rust is set up).

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- Git
- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed and available as `claude` in your PATH

## Usage

```bash
# Run from inside any git repository (uses current directory)
cd ~/your-project
claude-dtm

# Specify a repo and base branch explicitly
claude-dtm --repo ~/your-project --base-branch develop

# Use a custom command instead of claude
claude-dtm --command "claude --dangerously-skip-permissions"
```

### Quick Start

1. `cd` into a git repository
2. Run `claude-dtm`
3. Press `n` to create a new session (name it, set branch, confirm)
4. Press `f` to enter the pane and interact with Claude
5. Press `Esc` to go back to navigation mode
6. Press `n` again to spawn more sessions
7. Press `g` for grid layout to see them all at once

## Keybindings

### Navigation

| Key | Action |
|-----|--------|
| `j` / `k` | Focus next / previous pane |
| `J` / `K` | Swap focused pane with next / previous |
| `Enter` | Zoom: swap focused pane with master |
| `1`–`9` | Jump to pane by number |
| `h` / `l` | Shrink / grow master area |
| `i` / `d` | Increment / decrement master count |

### Layouts

| Key | Action |
|-----|--------|
| `t` | Master-stack layout (one large + stack) |
| `m` | Monocle layout (fullscreen focused pane) |
| `g` | Grid layout (equal-sized tiles) |

### Sessions

| Key | Action |
|-----|--------|
| `n` | New session (creates worktree + spawns Claude) |
| `f` | Enter focused pane (send keystrokes to Claude) |
| `s` | Search sessions by name/branch |
| `z` | Promote focused session to master position |
| `Esc` | Exit pane / close dialog |
| `q` | Kill focused session (removes worktree) |
| `p` | Pause focused session |
| `r` | Resume paused session |
| `Q` | Quit application |

## Layouts

### Master-Stack (`t`)
```
┌──────────────┬────────┐
│              │ stack1 │
│    master    │────────│
│              │ stack2 │
│              │────────│
│              │ stack3 │
└──────────────┴────────┘
```

### Grid (`g`)
```
┌────────┬────────┐
│   1    │   2    │
│────────│────────│
│   3    │   4    │
└────────┴────────┘
```

### Monocle (`m`)
```
┌────────────────────────┐
│                        │
│    fullscreen pane     │
│       [2/5]            │
│                        │
└────────────────────────┘
```

## Configuration

Config is stored at `~/.config/claude-dtm/config.json`. Created automatically with defaults on first run.

```json
{
    "default_repo": ".",
    "default_base_branch": "main",
    "default_command": "claude",
    "master_ratio": 0.55,
    "master_count": 1,
    "default_layout": "master_stack",
    "border_style": "rounded",
    "color_scheme": {
        "running": "yellow",
        "waiting": "green",
        "done": "gray",
        "paused": "blue",
        "focused": "white"
    }
}
```

## How It Works

1. **Worktrees** — Each session creates a git worktree (`git worktree add`) with its own branch, so sessions are fully isolated
2. **PTY** — Claude Code is spawned in a pseudo-terminal via `portable-pty`, giving it a real terminal environment
3. **Terminal parsing** — PTY output is parsed through `vt100` to capture colors, cursor position, and formatting
4. **Rendering** — The parsed terminal state is converted to `ratatui` spans and rendered in tiled panes
5. **Threaded I/O** — Each PTY reader runs on its own thread for responsive updates

## Tech Stack

| Concern | Crate |
|---------|-------|
| TUI framework | `ratatui` + `crossterm` |
| PTY management | `portable-pty` |
| Terminal parsing | `vt100` |
| Git operations | Shell out to `git` |
| Config/state | `serde` + `serde_json` |
| CLI args | `clap` |

## License

MIT
