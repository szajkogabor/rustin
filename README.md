# rustin

A fast, modern, cross-platform to-do list manager written in Rust.

Rustin keeps your tasks on a small board with three states:

- `todo`
- `in-progress`
- `done`

## Quick Start

### Install via Homebrew (macOS / Linux)

```bash
brew install szajkogabor/tap/rustin
```

### Install via AUR (Arch Linux)

```bash
yay -S rustin-bin
```

### Install from source

From the project root:

```bash
cargo run -- --help
```

General usage:

```bash
rustin [OPTIONS] [COMMAND]
```

## Commands

Rustin supports the following commands:

- `add` - Add a new task
- `remove` - Remove an existing task
- `list` - List all tasks
- `todo` - Move a task to the `todo` column
- `inprogress` - Move a task to the `in-progress` column
- `done` - Move a task to the `done` column
- `show` - Show all fields of a single task
- `stat` - Show board activity statistics
- `edit` - Edit fields of a task
- `init` - Initialize a new board or set its title
- `tui` - Open the interactive terminal UI
- `archive` - Archive all done tasks (soft-delete)
- `bin` - List deleted tasks
- `undelete` - Restore a deleted task
- `help` - Print command help

### Aliases

- `add` -> `a`
- `remove` -> `r`
- `list` -> `l`
- `todo` -> `t`
- `inprogress` -> `ip`
- `done` -> `d`
- `show` -> `s`
- `stat` -> `st`
- `edit` -> `e`
- `tui` -> `ui`
- `archive` -> `ar`
- `bin` -> `b`
- `undelete` -> `ud`

## Options

- `-v, --verbose...` Increase logging verbosity
- `-q, --quiet...` Decrease logging verbosity
- `-h, --help` Print help
- `-V, --version` Print version

## Typical Workflow

```bash
# 1) Create or rename your board
rustin init
rustin init --gitignore add

# 2) Add tasks
rustin add "Write README"
rustin add "Review pull request"

# 3) Show current board
rustin list

# 3b) Show activity stats
rustin stat

# 3c) Inspect or edit a task
rustin show 1
rustin edit 1 --title "Ship README"

# 4) Move tasks between columns
rustin inprogress 1
rustin done 1

# 4b) Open the terminal UI
rustin tui

# 5) Remove a task
rustin remove 2
```

## Get Help For Any Command

Use command-specific help when you need argument details:

```bash
rustin help add
rustin help list
rustin help done
rustin help edit

## Notes

- Running `rustin` with no command defaults to `rustin list`.
- `rustin init --gitignore ask|add|skip` controls how `.rustin.json` is handled in git repositories.
- `rustin tui` opens the interactive three-column board view.
- `rustin stat` counts only completed `in-progress -> done` cycles.
- Re-entering `in-progress` restarts the active timer for that run.
- Leaving `in-progress` for anything other than `done`, or ending history while still `in-progress`, does not count toward active time.
```

## Project Info

- Version: `1.0.0-rc.1`
- Author: Gabor Szajko <szajkogabor@gmail.com>
- License: MIT
