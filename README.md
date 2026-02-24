# dep-why

"Why is this dependency in my project?" Finally, a straight answer.

## Why This Exists

You're staring at a security advisory. Or a 47MB node_modules folder. Or a mysterious compile error from a package you've never heard of. And you're asking: "Where did THIS come from?"

npm why exists, sure. But it shows you one path when there might be twelve. cargo tree is great until you're grepping through 500 lines of output. You need to know ALL the paths, clearly, immediately.

dep-why traces every route from your direct dependencies to any transitive dependency. One command. All paths. Done.

## Features

- Multi-ecosystem: npm, cargo, pip (Pipfile.lock and poetry.lock)
- All paths or just the shortest - your choice
- Multiple output formats: colored tree, JSON, Mermaid diagrams
- Fast - parses lock files, not installed packages
- Depth limiting for those terrifying dependency trees

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

# Show ALL paths (not just shortest)
dep-why lodash --all

# With version numbers
dep-why lodash -v

# JSON output (for scripting)
dep-why lodash -f json

# Mermaid diagram (paste into GitHub markdown)
dep-why lodash -f mermaid

# Check a different directory
dep-why lodash -d /path/to/project

# Force a specific package manager
dep-why serde --manager cargo
```

## CLI Reference

```
dep-why [OPTIONS] <PACKAGE>

Arguments:
  <PACKAGE>  Package name to trace

Options:
  -a, --all              Show all paths (default: only shortest)
  -d, --dir <DIR>        Project directory (default: current)
  --manager <MANAGER>    Force package manager [npm, cargo, pip]
  -f, --format <FORMAT>  Output format [tree, json, mermaid]
  --max-depth <N>        Maximum search depth [default: 20]
  -v, --versions         Show dependency versions
  -h, --help             Print help
  -V, --version          Print version
```

## Output Formats

### Tree (default)

```
Found 2 path(s):

Path 1:
my-app
  └─ express
    └─ accepts
      └─ lodash

Path 2:
my-app
  └─ webpack
    └─ lodash
```

### JSON

```json
{
  "total_paths": 2,
  "paths": [
    {
      "packages": [
        {"name": "my-app", "version": "1.0.0"},
        {"name": "express", "version": "4.18.2"},
        {"name": "lodash", "version": "4.17.21"}
      ],
      "length": 3
    }
  ]
}
```

### Mermaid

```
graph TD
    my_app["my-app"]
    express["express"]
    lodash["lodash"]
    my_app --> express
    express --> lodash
```

Paste this into any GitHub markdown file to get a rendered diagram.

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
| 0 | Success |
| 1 | Package not found |
| 2 | No lock file / invalid path |
| 3 | Parse error |
| 4 | IO error |

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
