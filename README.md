# Q

**Микро-команды для AI.** Одна буква — одно действие. Минимум токенов, максимум скорости.

## Установка

```bash
cargo install --path .
```

Или напрямую:

```bash
cargo build --release
cp target/release/q ~/.local/bin/
```

Требуется: Rust (MSRV 1.70), `curl` (для `q h`), `diff` (для `q d`), `git` (для `q g`).

## Команды

| Команда | Алиас | Описание |
|---|---|---|
| `q find <glob>` | `q f` | Поиск файлов по glob-шаблону |
| `q search <regex>` | `q s` | Поиск содержимого (grep) |
| `q print <file>` | `q p` | Печать файла или диапазона |
| `q replace <file> <old> <new>` | `q r` | Замена текста (plain или regex) |
| `q write <file> [content]` | `q w` | Запись файла (из args или stdin) |
| `q info <path>` | `q i` | Информация о файле/папке |
| `q tree [path]` | `q t` | Дерево директории |
| `q diff <a> <b>` | `q d` | Diff двух файлов |
| `q http <url>` | `q h` | HTTP GET (curl) |
| `q git <action>` | `q g` | Git (status/log/diff/show/staged) |

## Фишки

- **`--json`** — JSON-вывод для всех команд (AI-friendly)
- **`--type rs,py,ts`** — фильтр по языку для `q f`/`q s`
- **`q p file:10-20`** — диапазон строк в аргументе
- **`q r --regex`** — замена по регулярному выражению
- **`q r --dry-run`** — показать diff без записи
- **`q r --undo`** — откат последней замены (из `.q.bak`)
- **`--no-trunc`** — отключить автообрезание (200 строк)

## Примеры

```bash
q f "**/*.rs"                     # найти все .rs файлы
q f "**/*.ts" --type ts           # только TypeScript
q s "fn main"                     # grep по проекту
q s "TODO" --type py,ts --json    # TODO только в Python/TS, JSON
q p src/main.rs:50-60             # строки 50-60
q p src/main.rs:50:+5             # 5 строк начиная с 50
q i src/main.rs                   # статистика файла
q r test.txt "foo" "bar" --dry-run  # проверить замену
q r test.txt "foo" "bar" --regex  # regex-замена
q r test.txt --undo               # откат
q t . -d 2                        # дерево глубиной 2
q g s                             # git status
q g l -n 5                        # git log (5 коммитов)
q h https://example.com           # HTTP GET
```

## Лицензия

MIT
