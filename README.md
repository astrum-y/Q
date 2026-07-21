# Q

**Микро-команды для AI.** Одна буква — одно действие. Минимум токенов, максимум скорости.

**Micro-commands for AI.** One letter, one action. Minimum tokens, maximum speed.

---

## 🤖 AI setup prompt

Copy this to any AI chat:

```
Please install Q from https://github.com/astrum-y/Q and follow
the setup guide in the repository README to configure your
environment. After installation, use `q` CLI instead of standard
bash commands for all file and git operations.
```

---

## Установка / Install

### Linux / macOS

```bash
cargo install --git https://github.com/astrum-y/Q
```

Или из исходников / Or from source:
```bash
git clone https://github.com/astrum-y/Q && cd Q && cargo install --path .
```

Бинарник в `~/.cargo/bin/q`. / Binary at `~/.cargo/bin/q`.

### macOS (Homebrew)

```bash
# Скоро / Coming soon
```

Пока — через `cargo install` выше. / For now — `cargo install` above.

### Windows

```powershell
cargo install --git https://github.com/astrum-y/Q
```

Бинарник в `%USERPROFILE%\.cargo\bin\q.exe`. / Binary at `%USERPROFILE%\.cargo\bin\q.exe`.

### Проверка / Verify

```bash
q --version
# → q 0.1.0
```

### Требования / Requirements

| Tool  | Linux | macOS | Windows |
|-------|-------|-------|---------|
| Rust 1.70+ | `apt install rustc cargo` | `brew install rust` | [rustup.rs](https://rustup.rs) |
| `curl` | built-in | built-in | built-in (Win10+) |
| `diff` | built-in | built-in | `diffutils` via scoop/choco |
| `git`  | `apt install git` | `brew install git` | [git-scm.com](https://git-scm.com) |

---

## Подключение к AI / Connect to AI

### opencode

Добавь в `~/.config/opencode/opencode.jsonc`:

```json
"instructions": ["~/.config/opencode/AGENTS.md"]
```

И создай `~/.config/opencode/AGENTS.md` (содержимое ниже / content below).

### Claude Code

```bash
mkdir -p ~/.claude
# скопируй содержимое AGENTS.md в ~/.claude/instructions.md
```

### Cursor

```bash
# скопируй содержимое AGENTS.md в ~/.cursorrules
```

### Windsurf

```bash
# скопируй содержимое AGENTS.md в ~/.windsurfrules
```

### Aider

```bash
echo 'read: AGENTS.md' >> ~/.aider.conf.yml
```

После настройки — **выйди и заново запусти** клиент, чтобы конфиг применился.

---

## Команды / Commands

### Файловые операции / File operations

| Команда / Command | Алиас | Описание / Description | Вместо / Instead of |
|---|---|---|---|
| `q find <glob> [path]` | `q f` | Поиск файлов по glob | `find /path -name` |
| `q search <regex> [path]` | `q s` | Поиск содержимого (grep) | `grep -rn /path` |
| `q print <file>` | `q p` | Печать файла или диапазона | `cat` |
| `q replace <file> <old> <new>` | `q r` | Замена текста (plain / regex) | `sed -i` |
| `q write <file> [content]` | `q w` | Запись файла (args / stdin) | `echo > file` |
| `q info <path>` | `q i` | Информация о файле/папке | `stat`, `wc -l` |
| `q tree [path]` | `q t` | Дерево директории | `tree` |
| `q ls [path]` | `q l` | Список директории | `ls` |
| `q mkdir <path>` | `q md` | Создать директорию | `mkdir -p` |
| `q mv <from> <to>` | — | Переименовать / переместить | `mv` |
| `q cp <from> <to>` | — | Копировать | `cp` |
| `q diff <a> <b>` | `q d` | Diff двух файлов | `diff -u` |
| `q http <url>` | `q h` | HTTP GET | `curl -sL` |

### Git

| Команда / Command | Описание / Description | Вместо / Instead of |
|---|---|---|
| `q g s` | git status --short | `git status --short` |
| `q g d` | git diff | `git diff` |
| `q g l` | git log --oneline -10 | `git log --oneline -10` |
| `q g st` | git diff --cached | `git diff --cached` |
| `q g sh <rev>` | git show | `git show HEAD` |
| `q g cm <msg>` | git add -A && git commit -m | `git add -A && git commit -m` |
| `q g b` | git branch | `git branch` |
| `q g ch <branch>` | git checkout | `git checkout main` |

---

## Фишки / Features

- **`--json`** — JSON-вывод для всех команд (AI-friendly, экономит парсинг)
- **`--type rs,py,ts`** — фильтр по языку для `q f`/`q s`
- **`q p file:10-20`** — диапазон строк прямо в аргументе
- **`q r --regex`** — замена по регулярному выражению
- **`q r --dry-run`** — показать diff без записи
- **`q r --undo`** — откат последней замены (из `.q.bak`)
- **`--no-trunc`** — отключить автообрезание (по умолчанию 200 строк)

---

## Примеры / Examples

```bash
q f "**/*.rs"                          # найти все .rs файлы
q f "**/*.ts" --type ts                # только TypeScript
q f "*.cfg" /etc                       # найти .cfg во всей системе
q s "fn main"                          # grep по проекту
q s "TODO" --type py,ts --json         # TODO только в Python/TS, JSON
q s "error" /var/log                   # поиск по абсолютному пути
q p src/main.rs:50-60                  # строки 50-60
q p src/main.rs:50:+5                  # 5 строк начиная с 50
q i src/main.rs                        # статистика файла
q r test.txt "foo" "bar" --dry-run     # проверить замену
q r test.txt "foo" "bar" --regex       # regex-замена
q r test.txt --undo                    # откат
q t . -d 2                             # дерево глубиной 2
q l -a                                 # ls со скрытыми
q l --long                             # ls подробно
q md src/components                    # mkdir -p
q mv old.txt new.txt                   # mv
q cp a.txt b.txt                       # cp
q g s                                  # git status
q g d                                  # git diff
q g l -n 5                             # git log (5 коммитов)
q g cm "fix: typo"                     # git commit
q g b                                  # git branch
q g ch main                            # git checkout
q h https://example.com                # HTTP GET
```

---

## Экономия токенов / Token savings

| Операция / Operation | bash | q | Экономия / Savings |
|---|---|---|---|
| Найти / find .rs файлы | `find . -name "*.rs"` | `q f "**/*.rs"` | ~3× |
| Grep по проекту | `grep -rn "foo" --include="*.rs"` | `q s "foo" --type rust` | ~4× |
| Замена / replace | `sed -i 's/old/new/g' file` | `q r file old new -a` | ~3× |
| Git status | `git status --short` | `q g s` | ~3× |
| Git log | `git log --oneline -5` | `q g l -n 5` | ~2× |
| Чтение файла / read | `cat file` (весь файл) | `q p file:1-50` (только 50 строк) | ~10×+ |

---

## Лицензия / License

MIT
