# rustin

A fast, modern, cross-platform to-do list manager written in Rust.

Rustin keeps your tasks on a small board with three states:

- `todo`
- `in-progress`
- `done`

## Quick Start

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
- `init` - Initialize a new board or set its title
- `stat` - Show task activity statistics and time summaries
- `help` - Print command help

## Options

- `-v, --verbose...` Increase logging verbosity
- `-q, --quiet...` Decrease logging verbosity
- `-h, --help` Print help
- `-V, --version` Print version

## Typical Workflow

```bash
# 1) Create or rename your board
rustin init

# 2) Add tasks
rustin add "Write README"
rustin add "Review pull request"

# 3) Show current board
rustin list

# 3b) Show activity stats
rustin stat

# 4) Move tasks between columns
rustin inprogress 1
rustin done 1

# 5) Remove a task
rustin remove 2
```

## Get Help For Any Command

Use command-specific help when you need argument details:

```bash
rustin help add
rustin help list
rustin help done
```

## Project Info

- Version: `1.0.0-rc.1`
- Author: Gabor Szajko <szajkogabor@gmail.com>
- License: MIT
