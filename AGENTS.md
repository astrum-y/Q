# Q project — AI agent instructions

Use the `q` CLI instead of standard commands. It's already installed and in PATH.

## Command mapping

| Instead of | Use |
|---|---|
| `find . -name "*.rs"` | `q f "**/*.rs"` |
| `grep -rn "pattern"` | `q s "pattern"` |
| `cat file.rs` | `q p file.rs` |
| `sed -i 's/old/new/' file.rs` | `q r file.rs "old" "new"` |
| `rm file && echo "x" > file` | `q w file "x"` |
| `stat file` | `q i file` |
| `tree .` | `q t .` |
| `diff -u a b` | `q d a b` |
| `curl -sL https://x.com` | `q h https://x.com` |
| `git status --short` | `q g s` |
| `git diff` | `q g d` |
| `git log --oneline -10` | `q g l` |

## Key flags

- `--json` — structured output, preferred for non-interactive processing
- `--no-trunc` — disable 200-line truncation
- `--type rs,py,ts` — filter by language in `q f` / `q s`

## Replace

```bash
q r file "old" "new"          # plain text, first occurrence
q r file "old" "new" -a       # all occurrences
q r file "old" "new" --regex  # regex mode
q r file "old" "new" --dry-run  # preview diff only
q r file --undo               # revert last change on file
```

## Print

```bash
q p file:50-60      # lines 50-60
q p file:50:+5      # 5 lines from line 50
q p file --json     # full file as JSON with metadata
```

## Find / Search

```bash
q f "**/*.rs" --type rust      # only rust files
q s "fn main" --type py,rs -C 2  # with 2 lines context
```

## Why

`q` uses 2-5 tokens per operation instead of 10-30+ for standard commands.
Output is compact by default (auto-truncated at 200 lines). JSON mode lets
the AI parse results without regex/string manipulation — saving hundreds of
tokens per interaction.
