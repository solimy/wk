# wk — a tiny terminal time tracker

Track time for named tasks from your terminal. Add tasks, start/stop a single active timer, and get quick day/week/month/year summaries — all stored locally in a lightweight SQLite database.

---

## Features

* Start/stop a single active task timer
* Add, list, and remove tasks
* `info` summaries for **day / week / month / year** (ranked by total time)
* Zero-config storage (SQLite file in your home directory)
* Simple, dependency-light Rust binary

---

## Quick start

```bash
# Build & run (debug)
cargo run -- --help

# Build a release binary
cargo build --release
./target/release/wk --help

# Or install with cargo (adds to $PATH)
cargo install --path . --bins
```

## Usage

```text
wk <COMMAND> [OPTIONS]
```

### Commands

| Command         | What it does                               | Examples                                                   |
| --------------- | ------------------------------------------ | ---------------------------------------------------------- |
| `add <name>`    | Create a new task                          | `wk add "writing"`                                         |
| `list`          | List all tasks (id and name)               | `wk list`                                                  |
| `start <name>`  | Stop any running task, then start `<name>` | `wk start writing`                                         |
| `stop`          | Stop the currently running task (if any)   | `wk stop`                                                  |
| `info [period]` | Show totals for a period; default is `day` | `wk info`, `wk info week`, `wk info month`, `wk info year` |
| `remove <name>` | Delete a task by name                      | `wk remove writing`                                        |

> `period` is one of: `day`, `week`, `month`, `year`. If omitted, `day` is used.

### Examples

Add a couple tasks:

```bash
wk add "coding"
wk add "reading"
wk list
# 1: coding
# 2: reading
```

Track time:

```bash
wk start coding
# ... time passes ...
wk stop
```

Switch tasks (auto-stops the previous one):

```bash
wk start reading
```

See what you did today (default):

```bash
wk info
# Current day:
#     1. coding: 0d 2h 15m 10s
#     2. reading: 0d 0h 45m 05s
```

Look at the week:

```bash
wk info week
```

Remove a task:

```bash
wk remove reading
```

---


## Notes & caveats

* **One active run at a time.** By design, `start` always stops any running task first.

---

## Development tips

* Run with verbose help:

  ```bash
  wk --help
  wk info --help
  ```

* Inspect the DB during development:

  ```bash
  # Debug DB
  sqlite3 dev.sqlite ".tables"
  sqlite3 dev.sqlite "SELECT * FROM runs ORDER BY id DESC LIMIT 5;"
  ```
