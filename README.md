# dep-why

"Why is this dependency in my project?" Finally, a straight answer.

## Why This Exists

You're staring at a security advisory. Or a 47MB node_modules folder. Or a mysterious compile error from a package you've never heard of. And you're asking: "Where did THIS come from?"

npm why exists, sure. But it shows you one path when there might be twelve. cargo tree is great until you're grepping through 500 lines of output. You need to know ALL the paths, clearly, immediately.

dep-why traces every route from your direct dependencies to any transitive dependency. One command. All paths. Done.

## Features

- Multi-ecosystem: npm, cargo, pip (Pipfile.lock and poetry.lock)
- All paths or just the shortest - your choice
- Circular dependency detection with suggested break points
- Multiple output formats: colored tree, JSON, Mermaid diagrams
- Fast - parses lock files, not installed packages
- Depth limiting for those terrifying dependency trees
- Dev dependency filtering - see production deps only by default

## Installation

```bash
cargo install dep-why
```

Or build from source:

```bash
git clone https://github.com/sudokatie/dep-why
cd dep-why
cargo build --release
```

## Quick Start

```bash
# Find why lodash is in your project
dep-why lodash

# Show ALL paths (not just shortest 5)
dep-why lodash --all

# Limit search depth
dep-why lodash -d 3

# JSON output (for scripting)
dep-why lodash -f json

# Mermaid diagram (paste into GitHub markdown)
dep-why lodash -f mermaid

# Include dev dependencies in search
dep-why jest --include-dev

# Check a specific version
dep-why lodash -v 4.17.21

# Detect circular dependencies
dep-why --cycles

# Cycle detection with Mermaid output (for docs)
dep-why --cycles -f mermaid

# Quiet mode for scripts (exit 0 if found)
dep-why lodash -q && echo "found"

# Use a specific lock file
dep-why lodash -l /path/to/package-lock.json

# Force a specific ecosystem
dep-why serde -e cargo
```

## CLI Reference

```
dep-why [OPTIONS] [PACKAGE]

Arguments:
  [PACKAGE]  Package name to search for (not required with --cycles)

Options:
      --cycles             Detect circular dependencies
  -a, --all                Show all paths (default: up to 5 shortest)
  -d, --depth <N>          Maximum depth to search [default: unlimited]
  -f, --format <FORMAT>    Output format [tree, json, mermaid] [default: tree]
  -e, --ecosystem <ECO>    Force ecosystem [npm, cargo, pip]
  -l, --lock-file <PATH>   Path to lock file
      --include-dev        Include dev dependencies in search
  -v, --version-match <V>  Only match specific version
  -q, --quiet              Minimal output (exit 0 if found, for scripts)
      --dir <DIR>          Project directory [default: current]
  -h, --help               Print help
  -V, --version            Print version
```

## Output Formats

### Tree (default)

```
lodash@4.17.21
├── Found via: express@4.18.2
│   └── accepts@1.3.8
│       └── lodash@4.17.21
└── Found via: webpack@5.88.0
    └── lodash@4.17.21

Summary: 2 paths found (shortest: 2, longest: 3)
Direct dependents: express, webpack
```

### JSON

```json
{
  "target": {
    "name": "lodash",
    "version": "4.17.21"
  },
  "paths": [
    {
      "chain": ["express@4.18.2", "accepts@1.3.8", "lodash@4.17.21"],
      "depth": 3,
      "is_dev": false
    }
  ],
  "summary": {
    "total_paths": 2,
    "shortest_depth": 2,
    "longest_depth": 3,
    "direct_dependents": ["express", "webpack"]
  }
}
```

### Mermaid

```
graph TD
    ROOT[my-project]
    ROOT --> express
    express --> accepts
    accepts --> lodash
    lodash["lodash@4.17.21 - TARGET"]
    style lodash fill:#f96
```

Paste this into any GitHub markdown file to get a rendered diagram.

## Circular Dependency Detection

Circular dependencies cause build issues, slow compilation, and can lead to subtle runtime bugs. Use `--cycles` to find them:

```bash
dep-why --cycles
```

Output:
```
Found 1 circular dependency chain(s) involving 3 packages:

Cycle 1: 3 packages
  module-a -> module-b -> module-c -> module-a (cycle)
  Suggested break point: module-c (fewest in-cycle dependencies)
```

The suggested break point is the package with the fewest dependencies within the cycle - typically the easiest place to refactor.

For CI integration, combine with `--quiet` to exit with code 1 if cycles are found:

```bash
dep-why --cycles --quiet || echo "Circular dependencies detected!"
```

## Configuration

Create `.dep-why.toml` in your project root or `~/.config/dep-why/config.toml`:

```toml
# Default output format
format = "tree"

# Default max paths (0 = unlimited with --all)
max_paths = 5

# Include dev dependencies by default
include_dev = false
```

Environment variables:
- `DEP_WHY_FORMAT` - Default output format
- `DEP_WHY_MAX_PATHS` - Default max paths
- `DEP_WHY_COLOR` - Force color output (auto, always, never)

## Supported Lock Files

| Manager | Lock File | Notes |
|---------|-----------|-------|
| npm | package-lock.json | v2/v3 format only |
| Cargo | Cargo.lock | Full transitive support |
| pip | Pipfile.lock | JSON format |
| pip | poetry.lock | TOML format |

Note: requirements.txt is not supported (no dependency graph info).

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (including "not found" - that's a valid answer) |
| 1 | Error (couldn't complete the query) |

## Performance

- Parses lock files directly (no network, no package installation)
- Typical projects: <100ms
- Large monorepos: <2s
- Memory scales with lock file size

## License

MIT

## Author

Katie

---

*Because "where did this come from" shouldn't require archaeology.*
