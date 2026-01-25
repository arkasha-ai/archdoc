# ArchDoc

ArchDoc is a tool for automatically generating architecture documentation for Python projects. It analyzes your Python codebase and creates comprehensive documentation that helps developers understand the structure, dependencies, and key components of the project.

## Features

- **Automatic Documentation Generation**: Automatically generates architecture documentation from Python source code
- **AST-Based Analysis**: Uses Python AST to extract imports, definitions, and function calls
- **Diff-Aware Updates**: Preserves manual content while updating generated sections
- **Caching**: Caches analysis results for faster subsequent runs
- **Configurable**: Highly configurable through `archdoc.toml`
- **Template-Based Rendering**: Uses Handlebars templates for customizable output

## Installation

To install ArchDoc, you'll need Rust installed on your system. Then run:

```bash
cargo install --path archdoc-cli
```

## Usage

### Initialize Configuration

First, initialize the configuration in your project:

```bash
archdoc init
```

This creates an `archdoc.toml` file with default settings.

### Generate Documentation

Generate architecture documentation for your project:

```bash
archdoc generate
```

This will create documentation files in the configured output directory.

### Check Documentation Consistency

Verify that your documentation is consistent with the code:

```bash
archdoc check
```

## Configuration

ArchDoc is configured through an `archdoc.toml` file. Here's an example configuration:

```toml
[project]
root = "."
out_dir = "docs/architecture"
entry_file = "ARCHITECTURE.md"
language = "python"

[scan]
include = ["src"]
exclude = [".venv", "venv", "__pycache__", ".git", "dist", "build"]

[python]
src_roots = ["src"]
include_tests = true
parse_docstrings = true

[analysis]
resolve_calls = true
detect_integrations = true

[output]
single_file = false
per_file_docs = true
create_directories = true

[caching]
enabled = true
cache_dir = ".archdoc/cache"
max_cache_age = "24h"
```

## How It Works

1. **Scanning**: ArchDoc scans your project directory for Python files based on the configuration
2. **Parsing**: It parses each Python file using AST to extract structure and relationships
3. **Analysis**: It analyzes the code to identify imports, definitions, and function calls
4. **Documentation Generation**: It generates documentation using templates
5. **Output**: It writes the documentation to files, preserving manual content

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.