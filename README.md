# ArchDoc

![Rust](https://img.shields.io/badge/Rust-1.85%2B-orange?logo=rust)
![License](https://img.shields.io/badge/License-MIT-blue)
![Tests](https://img.shields.io/badge/Tests-50%20passing-brightgreen)

**Automatic architecture documentation generator for Python projects.**

ArchDoc analyzes your Python codebase using AST parsing and generates comprehensive Markdown documentation covering module structure, dependencies, integration points, and critical hotspots.

## Features

- **AST-Based Analysis** — Full Python AST traversal for imports, classes, functions, calls, and docstrings
- **Dependency Graph** — Module-level and file-level dependency tracking with cycle detection
- **Integration Detection** — Automatically identifies HTTP, database, and message queue integrations
- **Diff-Aware Updates** — Preserves manually written sections while regenerating docs
- **Caching** — Content-hash based caching for fast incremental regeneration
- **Config Validation** — Comprehensive validation of `archdoc.toml` with helpful error messages
- **Statistics** — Project-level stats: file counts, symbol counts, fan-in/fan-out metrics
- **Consistency Checks** — Verify documentation stays in sync with code changes

## Installation

Requires Rust 1.85+:

```bash
cargo install --path archdoc-cli
```

## Quick Start

```bash
# Initialize config in your Python project
archdoc init

# Generate architecture docs
archdoc generate

# View project statistics
archdoc stats

# Check docs are up-to-date
archdoc check
```

## Commands

### `archdoc generate`

Scans the project, analyzes Python files, and generates documentation:

```
$ archdoc generate
🔍 Scanning project...
📂 Found 24 Python files in 6 modules
🔬 Analyzing dependencies...
📝 Generating documentation...
✅ Generated docs/architecture/ARCHITECTURE.md
✅ Generated 6 module docs
```

Output includes:
- **ARCHITECTURE.md** — Top-level overview with module index, dependency graph, and critical points
- **Per-module docs** — Detailed documentation for each module with symbols, imports, and metrics
- **Integration map** — HTTP, database, and queue integration points
- **Critical points** — High fan-in/fan-out symbols and dependency cycles

### `archdoc stats`

Displays project statistics without generating docs:

```
$ archdoc stats
📊 Project Statistics
  Files:    24
  Modules:  6
  Classes:  12
  Functions: 47
  Imports:  89
  Edges:    134
```

### `archdoc check`

Verifies documentation consistency with the current codebase:

```
$ archdoc check
✅ Documentation is up-to-date
```

Returns non-zero exit code if docs are stale — useful in CI pipelines.

### `archdoc init`

Creates a default `archdoc.toml` configuration file:

```
$ archdoc init
✅ Created archdoc.toml with default settings
```

## Configuration Reference

ArchDoc is configured via `archdoc.toml`:

| Section | Key | Default | Description |
|---------|-----|---------|-------------|
| `project` | `root` | `"."` | Project root directory |
| `project` | `out_dir` | `"docs/architecture"` | Output directory for generated docs |
| `project` | `entry_file` | `"ARCHITECTURE.md"` | Main documentation file name |
| `project` | `language` | `"python"` | Project language (only `python` supported) |
| `scan` | `include` | `["src", "app", "tests"]` | Directories to scan |
| `scan` | `exclude` | `[".venv", "__pycache__", ...]` | Directories to skip |
| `scan` | `max_file_size` | `"10MB"` | Skip files larger than this (supports KB, MB, GB) |
| `scan` | `follow_symlinks` | `false` | Whether to follow symbolic links |
| `python` | `src_roots` | `["src", "."]` | Python source roots for import resolution |
| `python` | `include_tests` | `true` | Include test files in analysis |
| `python` | `parse_docstrings` | `true` | Extract docstrings from symbols |
| `python` | `max_parse_errors` | `10` | Max parse errors before aborting |
| `analysis` | `resolve_calls` | `true` | Resolve function call targets |
| `analysis` | `detect_integrations` | `true` | Detect HTTP/DB/queue integrations |
| `output` | `single_file` | `false` | Generate everything in one file |
| `output` | `per_file_docs` | `true` | Generate per-module documentation |
| `thresholds` | `critical_fan_in` | `20` | Fan-in threshold for critical symbols |
| `thresholds` | `critical_fan_out` | `20` | Fan-out threshold for critical symbols |
| `caching` | `enabled` | `true` | Enable analysis caching |
| `caching` | `cache_dir` | `".archdoc/cache"` | Cache directory |
| `caching` | `max_cache_age` | `"24h"` | Cache TTL (supports s, m, h, d, w) |

### Example Configuration

```toml
[project]
root = "."
out_dir = "docs/architecture"
language = "python"

[scan]
include = ["src", "app"]
exclude = [".venv", "__pycache__", ".git"]
max_file_size = "10MB"

[python]
src_roots = ["src"]
include_tests = true
parse_docstrings = true

[analysis]
resolve_calls = true
detect_integrations = true
integration_patterns = [
    { type = "http", patterns = ["requests", "httpx", "aiohttp"] },
    { type = "db", patterns = ["sqlalchemy", "psycopg", "sqlite3"] },
    { type = "queue", patterns = ["celery", "kafka", "redis"] }
]

[caching]
enabled = true
max_cache_age = "24h"
```

## How It Works

1. **Scan** — Walks the project tree, filtering by include/exclude patterns
2. **Parse** — Parses each Python file with a full AST traversal (via `rustpython-parser`)
3. **Analyze** — Builds a project model with modules, symbols, edges, and metrics
4. **Detect** — Identifies integration points (HTTP, DB, queues) and dependency cycles
5. **Render** — Generates Markdown using Handlebars templates
6. **Write** — Outputs files with diff-aware updates preserving manual sections

## Architecture

```
archdoc/
├── archdoc-cli/          # CLI binary (commands, output formatting)
│   └── src/
│       ├── main.rs
│       └── commands/     # generate, check, stats, init
├── archdoc-core/         # Core library
│   └── src/
│       ├── config.rs         # Config loading & validation
│       ├── scanner.rs        # File discovery
│       ├── python_analyzer.rs # AST analysis
│       ├── model.rs          # Project IR (modules, symbols, edges)
│       ├── cycle_detector.rs # Dependency cycle detection
│       ├── renderer.rs       # Markdown generation
│       ├── writer.rs         # File output with diff awareness
│       └── cache.rs          # Analysis caching
└── test-project/         # Example Python project for testing
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License — see the LICENSE file for details.
