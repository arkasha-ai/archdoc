# Changelog

All notable changes to ArchDoc are documented in this file.

Format follows [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased] — feature/improvements-v2

### Added
- **Config validation** (`Config::validate()`) — checks project root, language, scan includes, src_roots, cache age, and file size formats with helpful error messages
- **Duration & file size parsers** — `parse_duration()` (s/m/h/d/w) and `parse_file_size()` (B/KB/MB/GB) utility functions
- **Dependency cycle detection** (`cycle_detector.rs`) — DFS-based algorithm to find circular module dependencies
- **Cycle detection in renderer** — Critical points section now shows detected dependency cycles
- **Full pipeline integration tests** — Tests for config validation, scanning, cycle detection, and rendering
- **Stats command** — `archdoc stats` displays project-level statistics (files, modules, symbols, edges)
- **Check command** — `archdoc check` verifies documentation consistency with code
- **Colored CLI output** — Progress bars and colored status messages
- **Comprehensive README** — Badges, configuration reference table, command documentation, architecture overview

### Changed
- **CLI architecture** — Decomposed into separate command modules (generate, check, stats, init)
- **Error handling** — Improved error messages with `thiserror` and `anyhow`
- **Clippy compliance** — All warnings resolved

### Fixed
- Various clippy warnings and code style issues
