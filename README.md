# Q

**Микро-команды для AI.** Одна буква — одно действие. Минимум токенов, максимум скорости.

## Установка

### Linux / macOS

```bash
# Через cargo (рекомендуется)
cargo install --git https://github.com/astrum-y/Q

# Или из исходников
git clone https://github.com/astrum-y/Q
cd Q
cargo install --path .
```

Бинарник в `~/.cargo/bin/q`. Убедись, что `~/.cargo/bin` в `PATH`.

### macOS (Homebrew)

```bash
# Скоро будет доступен в tap
# brew install q
```

Пока — через `cargo install` (см. выше).

### Windows

```powershell
# Через cargo (требуется Rust)
cargo install --git https://github.com/astrum-y/Q

# Или scoop (если есть)
# scoop install q
```

Бинарник в `%USERPROFILE%\.cargo\bin\q.exe`.
Добавь `%USERPROFILE%\.cargo\bin` в `PATH` если ещё не добавлен.

### Проверка

```bash
q --version
# → q 0.1.0
```

### Требования

| Инструмент | Для | Linux | macOS | Windows |
|---|---|---|---|---|
| Rust 1.70+ | Сборка | `apt install rustc cargo` | `brew install rust` | [rustup.rs](https://rustup.rs) |
| `curl` | `q h` | встроен | встроен | встроен в Win10+ |
| `diff` | `q d` | встроен | встроен | `diffutils` через scoop/choco |
| `git` | `q g` | `apt install git` | `brew install git` | [git-scm.com](https://git-scm.com) |

## Подключение к AI-агентам

Чтобы AI использовал `q` вместо bash-команд, создай файл с инструкциями.
Ниже — пути для каждого клиента.

### opencode

```bash
# Глобальный конфиг
mkdir -p ~/.config/opencode

# Создать AGENTS.md с инструкциями
```

Добавь в `~/.config/opencode/opencode.jsonc`:

```json
"instructions": ["~/.config/opencode/AGENTS.md"]
```

И создай `~/.config/opencode/AGENTS.md`:

```markdown
# Global Q instructions

Use `q` CLI instead of standard bash commands.

## Command mapping

| Instead of | Use |
|---|---|
| `find . -name "*.rs"` | `q f "**/*.rs"` |
| `grep -rn "pattern"` | `q s "pattern"` |
| `cat file.rs` | `q p file.rs` |
| `sed -i 's/old/new/' file.rs` | `q r file.rs "old" "new"` |
| `echo "x" > file` | `q w file "x"` |
| `stat file` | `q i file` |
| `tree .` | `q t .` |
| `ls` | `q l` |
| `mkdir -p path` | `q md path` |
| `mv from to` | `q mv from to` |
| `cp from to` | `q cp from to` |
| `diff -u a b` | `q d a b` |
| `curl -sL https://x.com` | `q h https://x.com` |
| `git status --short` | `q g s` |
| `git diff` | `q g d` |
| `git log --oneline -10` | `q g l` |
| `git diff --cached` | `q g st` |
| `git show HEAD` | `q g sh HEAD` |
| `git add -A && git commit -m "x"` | `q g cm "x"` |
| `git branch` | `q g b` |
| `git checkout main` | `q g ch main` |

## Key flags

- `--json` — structured output, preferred for AI processing
- `--no-trunc` — disable 200-line truncation
- `--type rs,py,ts` — filter by language in `q f` / `q s`

## Why

`q` uses 2-5 tokens per operation instead of 10-30+ for standard commands.
Output is auto-truncated at 200 lines. JSON mode saves hundreds of tokens.
```

После изменения конфига — **полностью выйди и заново запусти** opencode.

### Claude Code

```bash
# Глобально (все проекты):
mkdir -p ~/.claude
# Скопируй AGENTS.md выше в этот файл:
touch ~/.claude/instructions.md

# Или для конкретного проекта:
touch .claude/instructions.md
```

### Cursor

```bash
touch ~/.cursorrules
# Скопируй содержимое AGENTS.md в этот файл
```

### Windsurf

```bash
touch ~/.windsurfrules
# Скопируй содержимое AGENTS.md в этот файл
```

### Aider

```bash
# В ~/.aider.conf.yml:
echo 'read: AGENTS.md' >> ~/.aider.conf.yml
```

### GitHub Copilot / Zed

Скопируй содержимое AGENTS.md в `.github/copilot-instructions.md` (для Copilot)
или в `.zed/instructions.md` (для Zed).

## Команды

### Файловые операции

| Команда | Алиас | Описание | Вместо |
|---|---|---|---|
| `q find <glob> [path]` | `q f` | Поиск файлов по glob | `find /path -name` |
| `q search <regex> [path]` | `q s` | Поиск содержимого (grep) | `grep -rn /path` |
| `q print <file>` | `q p` | Печать файла или диапазона | `cat` |
| `q replace <file> <old> <new>` | `q r` | Замена текста | `sed -i` |
| `q write <file> [content]` | `q w` | Запись файла | `echo > file` |
| `q info <path>` | `q i` | Информация о файле/папке | `stat`, `wc -l` |
| `q tree [path]` | `q t` | Дерево директории | `tree` |
| `q ls [path]` | `q l` | Список директории | `ls` |
| `q mkdir <path>` | `q md` | Создать директорию | `mkdir -p` |
| `q mv <from> <to>` | — | Переименовать/переместить | `mv` |
| `q cp <from> <to>` | — | Копировать | `cp` |
| `q diff <a> <b>` | `q d` | Diff двух файлов | `diff -u` |
| `q http <url>` | `q h` | HTTP GET | `curl -sL` |

### Git

| Команда | Описание | Вместо |
|---|---|---|
| `q git status` | `q g s` | `git status --short` |
| `q git diff` | `q g d` | `git diff` |
| `q git log` | `q g l` | `git log --oneline -10` |
| `q git staged` | `q g st` | `git diff --cached` |
| `q git show <rev>` | `q g sh` | `git show` |
| `q git commit <msg>` | `q g cm` | `git add -A && git commit -m` |
| `q git branch` | `q g b` | `git branch` |
| `q git checkout <branch>` | `q g ch` | `git checkout` |

## Фишки

- **`--json`** — JSON-вывод для всех команд (AI-friendly, экономит парсинг)
- **`--type rs,py,ts`** — фильтр по языку для `q f`/`q s`
- **`q p file:10-20`** — диапазон строк прямо в аргументе
- **`q r --regex`** — замена по регулярному выражению
- **`q r --dry-run`** — показать diff без записи
- **`q r --undo`** — откат последней замены (из `.q.bak`)
- **`--no-trunc`** — отключить автообрезание (по умолчанию 200 строк)

## Примеры

```bash
q f "**/*.rs"                          # найти все .rs файлы
q f "**/*.ts" --type ts                # только TypeScript
q s "fn main"                          # grep по проекту
q s "TODO" --type py,ts --json         # TODO только в Python/TS, JSON
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

## Экономия токенов

| Операция | bash | q | Экономия |
|---|---|---|---|
| Найти .rs файлы | `find . -name "*.rs"` | `q f "**/*.rs"` | ~3x |
| Grep по проекту | `grep -rn "foo" --include="*.rs"` | `q s "foo" --type rust` | ~4x |
| Замена в файле | `sed -i 's/old/new/g' file` | `q r file old new -a` | ~3x |
| Git status | `git status --short` | `q g s` | ~3x |
| Git log | `git log --oneline -5` | `q g l -n 5` | ~2x |
| Чтение файла | `cat file` (весь файл) | `q p file:1-50` (только 50 строк) | ~10x+ |

## Лицензия

MIT
