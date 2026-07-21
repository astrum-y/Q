use std::io::{self, Read};
use std::path::Path;
use std::process::{self, Command};

use chrono::prelude::*;
use clap::{Parser, Subcommand};
use globset::{Glob, GlobSetBuilder};
use ignore::types::TypesBuilder;
use ignore::WalkBuilder;
use regex::Regex;
use serde::Serialize;
use walkdir::WalkDir;

const DEFAULT_MAX_LINES: usize = 200;

// ── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "q", version, about = "микро-команды для AI")]
struct Cli {
    #[arg(short = 'j', long = "json", global = true, help = "JSON output")]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(alias = "f", about = "Поиск файлов по glob")]
    Find {
        #[arg(help = "glob-шаблон (напр. **/*.rs)")]
        pattern: String,
        #[arg(short = 't', long = "type", help = "тип файла (rs,py,ts,js,c,pp,go,java,md,html,css)")]
        type_names: Option<String>,
    },

    #[command(alias = "s", about = "Поиск содержимого (grep)")]
    Search {
        #[arg(help = "regex-паттерн")]
        pattern: String,
        #[arg(short = 'g', long = "glob", help = "фильтр файлов (напр. *.rs)")]
        glob: Option<String>,
        #[arg(short = 't', long = "type", help = "тип файла (rs,py,ts,js,c,pp,go,java,md,html,css)")]
        type_names: Option<String>,
        #[arg(short = 'C', long = "context", default_value = "0", help = "строк контекста")]
        context: usize,
        #[arg(long = "no-trunc", help = "не обрезать вывод")]
        no_trunc: bool,
    },

    #[command(alias = "p", about = "Печать файла")]
    Print {
        #[arg(help = "путь к файлу [file:start..end или file:start:+count]")]
        file: String,
        #[arg(short = 'o', long = "offset", help = "начальная строка (1-based)")]
        offset: Option<usize>,
        #[arg(short = 'l', long = "limit", help = "макс строк")]
        limit: Option<usize>,
        #[arg(long = "no-trunc", help = "не обрезать вывод")]
        no_trunc: bool,
    },

    #[command(alias = "r", about = "Замена текста в файле")]
    Replace {
        #[arg(help = "путь к файлу")]
        file: String,
        #[arg(help = "старый текст (или regex при --regex) [пропустить при --undo]")]
        old: Option<String>,
        #[arg(help = "новый текст [пропустить при --undo]")]
        new: Option<String>,
        #[arg(short = 'a', long = "all", help = "заменить все вхождения")]
        all: bool,
        #[arg(long = "dry-run", help = "показать diff без записи")]
        dry_run: bool,
        #[arg(long = "regex", help = "использовать regex вместо plain text")]
        regex: bool,
        #[arg(long = "undo", help = "откатить замену из .q.bak")]
        undo: bool,
    },

    #[command(alias = "w", about = "Записать файл (из аргумента или stdin)")]
    Write {
        #[arg(help = "путь к файлу")]
        file: String,
        #[arg(help = "содержимое (если не указано — читает stdin)")]
        content: Option<String>,
    },

    #[command(alias = "i", about = "Информация о файле")]
    Info {
        #[arg(help = "путь к файлу или директории")]
        path: String,
    },

    #[command(alias = "t", about = "Дерево директории")]
    Tree {
        #[arg(help = "путь (по умолчанию .)")]
        path: Option<String>,
        #[arg(short = 'd', long = "depth", default_value = "3", help = "глубина")]
        depth: usize,
        #[arg(short = 'D', long = "dirs-only", help = "только папки")]
        dirs_only: bool,
    },

    #[command(alias = "l", about = "Список директории")]
    Ls {
        #[arg(help = "путь (по умолчанию .)")]
        path: Option<String>,
        #[arg(short = 'l', long = "long", help = "подробный формат")]
        long: bool,
        #[arg(short = 'a', long = "all", help = "показать скрытые")]
        all: bool,
    },

    #[command(alias = "md", about = "Создать директорию (mkdir -p)")]
    Mkdir {
        #[arg(help = "путь")]
        path: String,
    },

    #[command(about = "Переименовать/переместить")]
    Mv {
        #[arg(help = "откуда")]
        from: String,
        #[arg(help = "куда")]
        to: String,
    },

    #[command(about = "Копировать")]
    Cp {
        #[arg(help = "откуда")]
        from: String,
        #[arg(help = "куда")]
        to: String,
    },

    #[command(alias = "d", about = "Diff двух файлов")]
    Diff {
        #[arg(help = "файл A")]
        file_a: String,
        #[arg(help = "файл B")]
        file_b: String,
    },

    #[command(alias = "h", about = "HTTP GET (curl)")]
    Http {
        #[arg(help = "URL")]
        url: String,
    },

    #[command(about = "Git-команды")]
    Git {
        #[command(subcommand)]
        action: GitAction,
    },
}

#[derive(Subcommand)]
enum GitAction {
    #[command(alias = "s", about = "git status")]
    Status,
    #[command(alias = "d", about = "git diff")]
    Diff,
    #[command(alias = "l", about = "git log")]
    Log {
        #[arg(short = 'n', default_value = "10", help = "кол-во коммитов")]
        count: usize,
    },
    #[command(about = "git diff --cached")]
    Staged,
    #[command(about = "Показать содержимое коммита")]
    Show {
        #[arg(help = "ревизия (HEAD, хэш, ...)")]
        rev: String,
    },
    #[command(alias = "cm", about = "git commit -m")]
    Commit {
        #[arg(help = "сообщение коммита")]
        msg: String,
    },
    #[command(alias = "b", about = "git branch")]
    Branch,
    #[command(alias = "ch", about = "git checkout")]
    Checkout {
        #[arg(help = "ветка")]
        branch: String,
    },
}

// ── JSON-модели ──────────────────────────────────────────────────────────────

#[derive(Serialize)]
struct FindResult {
    kind: &'static str,
    files: Vec<String>,
    count: usize,
}

#[derive(Clone, Serialize)]
struct SearchMatch {
    file: String,
    line: usize,
    column: usize,
    text: String,
}

#[derive(Serialize)]
struct SearchResult {
    kind: &'static str,
    pattern: String,
    matches: Vec<SearchMatch>,
    count: usize,
    truncated: bool,
}

#[derive(Serialize)]
struct PrintResult {
    kind: &'static str,
    file: String,
    lines: usize,
    content: String,
    truncated: bool,
}

#[derive(Serialize)]
struct ReplaceResult {
    kind: &'static str,
    file: String,
    replacements: usize,
    diff: Option<String>,
}

#[derive(Serialize)]
struct WriteResult {
    kind: &'static str,
    file: String,
    bytes: usize,
}

#[derive(Serialize)]
struct InfoResult {
    kind: &'static str,
    path: String,
    is_dir: bool,
    size_bytes: Option<u64>,
    lines: Option<usize>,
    modified: Option<String>,
}

#[derive(Serialize)]
struct TreeEntry {
    path: String,
    is_dir: bool,
    depth: usize,
}

#[derive(Serialize)]
struct TreeResult {
    kind: &'static str,
    root: String,
    entries: Vec<TreeEntry>,
}

#[derive(Serialize)]
struct LsEntry {
    name: String,
    is_dir: bool,
    size: Option<u64>,
    modified: Option<String>,
}

#[derive(Serialize)]
struct LsResult {
    kind: &'static str,
    path: String,
    entries: Vec<LsEntry>,
}

#[derive(Serialize)]
struct FsResult {
    kind: &'static str,
    from: String,
    to: String,
}

#[derive(Serialize)]
struct DiffResult {
    kind: &'static str,
    file_a: String,
    file_b: String,
    diff: String,
}

#[derive(Serialize)]
struct HttpResult {
    kind: &'static str,
    url: String,
    status: Option<i32>,
    body: String,
    truncated: bool,
}

#[derive(Serialize)]
struct GitResult {
    kind: &'static str,
    command: String,
    output: String,
}

#[derive(Serialize)]
struct ErrorResult {
    kind: &'static str,
    error: String,
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();
    let json = cli.json;

    let result = match &cli.command {
        Commands::Find { pattern, type_names } => cmd_find(pattern, type_names.as_deref(), json),
        Commands::Search { pattern, glob, type_names, context, no_trunc } => {
            cmd_search(pattern, glob.as_deref(), type_names.as_deref(), *context, *no_trunc, json)
        }
        Commands::Print { file, offset, limit, no_trunc } => cmd_print(file, *offset, *limit, *no_trunc, json),
        Commands::Replace { file, old, new, all, dry_run, regex, undo } => {
            cmd_replace(file, old.as_deref(), new.as_deref(), *all, *dry_run, *regex, *undo, json)
        }
        Commands::Write { file, content } => cmd_write(file, content.as_deref(), json),
        Commands::Info { path } => cmd_info(path, json),
        Commands::Tree { path, depth, dirs_only } => cmd_tree(path.as_deref(), *depth, *dirs_only, json),
        Commands::Ls { path, long, all } => cmd_ls(path.as_deref(), *long, *all, json),
        Commands::Mkdir { path } => cmd_mkdir(path, json),
        Commands::Mv { from, to } => cmd_mv(from, to, json),
        Commands::Cp { from, to } => cmd_cp(from, to, json),
        Commands::Diff { file_a, file_b } => cmd_diff(file_a, file_b, json),
        Commands::Http { url } => cmd_http(url, json),
        Commands::Git { action } => cmd_git(action, json),
    };

    match result {
        Ok(output) => {
            if !output.is_empty() {
                println!("{output}");
            }
        }
        Err(e) => {
            if json {
                let err = ErrorResult { kind: "error", error: e };
                println!("{}", serde_json::to_string(&err).unwrap());
            } else {
                eprintln!("error: {e}");
                process::exit(1);
            }
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn build_type_matcher(type_names: &str) -> Result<Option<ignore::types::Types>, String> {
    if type_names.is_empty() {
        return Ok(None);
    }
    let mut tb = TypesBuilder::new();
    tb.add_defaults();
    for t in type_names.split(',') {
        let t = t.trim();
        if t.is_empty() {
            continue;
        }
        tb.select(t);
    }
    tb.build().map(Some).map_err(|e| format!("type error: {e}"))
}

fn apply_trunc(text: &str, max_lines: usize, no_trunc: bool) -> (String, bool) {
    if no_trunc || max_lines == 0 {
        return (text.to_string(), false);
    }
    let lines: Vec<&str> = text.lines().collect();
    if lines.len() <= max_lines {
        return (text.to_string(), false);
    }
    let truncated: String = lines[..max_lines].join("\n");
    let note = format!("\n... ({}/{}) lines shown, use --no-trunc for full output", max_lines, lines.len());
    (format!("{truncated}{note}"), true)
}

// ── Find ─────────────────────────────────────────────────────────────────────

fn cmd_find(pattern: &str, type_names: Option<&str>, json: bool) -> Result<String, String> {
    let mut builder = GlobSetBuilder::new();
    builder.add(Glob::new(pattern).map_err(|e| format!("invalid glob: {e}"))?);
    let glob_set = builder.build().map_err(|e| format!("glob error: {e}"))?;

    let type_matcher = match type_names {
        Some(t) => build_type_matcher(t)?,
        None => None,
    };

    let mut files = Vec::new();

    let mut wb = WalkBuilder::new(".");
    wb.standard_filters(true);
    wb.follow_links(true);
    if let Some(ref tm) = type_matcher {
        wb.types(tm.clone());
    }
    for entry in wb.build().filter_map(|e| e.ok()) {
        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }
        let path = entry.path().to_string_lossy().to_string();
        if glob_set.is_match(&path) {
            files.push(path);
        }
    }

    files.sort();

    if json {
        let r = FindResult { kind: "find", files: files.clone(), count: files.len() };
        serde_json::to_string(&r).map_err(|e| e.to_string())
    } else {
        if files.is_empty() {
            Ok(String::new())
        } else {
            Ok(files.join("\n"))
        }
    }
}

// ── Search (grep) ────────────────────────────────────────────────────────────

fn cmd_search(
    pattern: &str,
    glob: Option<&str>,
    type_names: Option<&str>,
    context: usize,
    no_trunc: bool,
    json: bool,
) -> Result<String, String> {
    let re = Regex::new(pattern).map_err(|e| format!("invalid regex: {e}"))?;

    let glob_filter: Option<globset::GlobSet> = if let Some(g) = glob {
        let mut builder = GlobSetBuilder::new();
        builder.add(Glob::new(g).map_err(|e| format!("invalid glob: {e}"))?);
        Some(builder.build().map_err(|e| format!("glob error: {e}"))?)
    } else {
        None
    };

    let type_matcher = match type_names {
        Some(t) => build_type_matcher(t)?,
        None => None,
    };

    let mut matches = Vec::new();

    let mut wb = WalkBuilder::new(".");
    wb.standard_filters(true);
    wb.follow_links(true);
    if let Some(ref tm) = type_matcher {
        wb.types(tm.clone());
    }
    for entry in wb.build().filter_map(|e| e.ok()) {
        if !entry.file_type().map_or(false, |ft| ft.is_file()) {
            continue;
        }
        let path_str = entry.path().to_string_lossy();
        if let Some(ref gs) = glob_filter {
            if !gs.is_match(path_str.as_ref()) {
                continue;
            }
        }

        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if content.contains('\0') {
            continue;
        }

        let lines_vec: Vec<&str> = content.lines().collect();
        for (line_idx, line) in lines_vec.iter().enumerate() {
            if let Some(m) = re.find(line) {
                matches.push(SearchMatch {
                    file: path_str.to_string(),
                    line: line_idx + 1,
                    column: m.start() + 1,
                    text: (*line).to_string(),
                });
                if context > 0 {
                    let start = if line_idx >= context { line_idx - context } else { 0 };
                    let end = (line_idx + context + 1).min(lines_vec.len());
                    for ctx_idx in start..end {
                        if ctx_idx != line_idx {
                            matches.push(SearchMatch {
                                file: path_str.to_string(),
                                line: ctx_idx + 1,
                                column: 0,
                                text: lines_vec[ctx_idx].to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    matches.sort_by(|a, b| a.file.cmp(&b.file).then(a.line.cmp(&b.line)));

    let mut out = String::new();
    for m in &matches {
        if m.column == 0 {
            out.push_str(&format!("{}:{}: {}\n", m.file, m.line, m.text));
        } else {
            out.push_str(&format!("{}:{}:{}: {}\n", m.file, m.line, m.column, m.text));
        }
    }

    if json {
        let (_, truncated) = apply_trunc(&out, DEFAULT_MAX_LINES, no_trunc);
        let r = SearchResult {
            kind: "search",
            pattern: pattern.to_string(),
            matches: matches.clone(),
            count: matches.len(),
            truncated,
        };
        serde_json::to_string(&r).map_err(|e| e.to_string())
    } else {
        if out.is_empty() {
            Ok(String::new())
        } else {
            let (trimmed, _) = apply_trunc(out.trim_end(), DEFAULT_MAX_LINES, no_trunc);
            Ok(trimmed)
        }
    }
}

// ── Print ────────────────────────────────────────────────────────────────────

fn parse_range(file_arg: &str) -> (&str, Option<usize>, Option<usize>) {
    if let Some(idx) = file_arg.find(':') {
        let path = &file_arg[..idx];
        let range = &file_arg[idx + 1..];
        if let Some(plus_pos) = range.find(":+") {
            let start: usize = range[..plus_pos].parse().unwrap_or(1);
            let count: usize = range[plus_pos + 2..].parse().unwrap_or(0);
            return (path, Some(start), Some(count));
        }
        if let Some(dash_pos) = range.find('-') {
            let start: usize = range[..dash_pos].parse().unwrap_or(1);
            let end: usize = range[dash_pos + 1..].parse().unwrap_or(0);
            let limit = if end >= start { end - start + 1 } else { 0 };
            return (path, Some(start), Some(limit));
        }
        let start: usize = range.parse().unwrap_or(1);
        return (path, Some(start), None);
    }
    (file_arg, None, None)
}

fn cmd_print(file_arg: &str, offset: Option<usize>, limit: Option<usize>, no_trunc: bool, json: bool) -> Result<String, String> {
    let (file, parsed_offset, parsed_limit) = parse_range(file_arg);
    let offset = offset.or(parsed_offset);
    let limit = limit.or(parsed_limit);

    let content = std::fs::read_to_string(file).map_err(|e| format!("read {file}: {e}"))?;
    let lines: Vec<&str> = content.lines().collect();
    let total = lines.len();

    let start = offset.unwrap_or(1).saturating_sub(1);
    let end = match limit {
        Some(l) => (start + l).min(total),
        None => total,
    };

    let selected: Vec<&str> = lines[start..end].to_vec();
    let out = selected.join("\n");

    if json {
        let (content, truncated) = apply_trunc(&out, DEFAULT_MAX_LINES, no_trunc);
        let r = PrintResult {
            kind: "print",
            file: file.to_string(),
            lines: total,
            content,
            truncated,
        };
        serde_json::to_string(&r).map_err(|e| e.to_string())
    } else {
        if out.is_empty() {
            return Ok(String::new());
        }
        let (trimmed, truncated) = apply_trunc(&out, DEFAULT_MAX_LINES, no_trunc);
        let meta = format!("--- {file} ({}:{}-{} / {} lines) ---", file, start + 1, end, total);
        let mut result = format!("{meta}\n{trimmed}");
        if truncated {
            result.push_str("\n(... truncated, use --no-trunc for full output)");
        }
        Ok(result)
    }
}

// ── Replace ──────────────────────────────────────────────────────────────────

fn cmd_replace(
    file: &str,
    old: Option<&str>,
    new: Option<&str>,
    all: bool,
    dry_run: bool,
    use_regex: bool,
    undo: bool,
    json: bool,
) -> Result<String, String> {
    // ── Undo mode ─────────────────────────────────────────────────────────
    if undo {
        let bak = format!("{file}.q.bak");
        if !Path::new(&bak).exists() {
            return Err(format!("backup not found: {bak}"));
        }
        let original = std::fs::read_to_string(&bak).map_err(|e| format!("read backup: {e}"))?;
        std::fs::write(file, &original).map_err(|e| format!("restore {file}: {e}"))?;
        std::fs::remove_file(&bak).ok();
        if json {
            let r = ReplaceResult {
                kind: "replace",
                file: file.to_string(),
                replacements: 0,
                diff: None,
            };
            return serde_json::to_string(&r).map_err(|e| e.to_string());
        }
        return Ok(format!("{file}: restored from {bak}"));
    }

    let old = old.ok_or("old is required (use --undo without old/new)")?;
    let new = new.ok_or("new is required (use --undo without old/new)")?;

    let content = std::fs::read_to_string(file).map_err(|e| format!("read {file}: {e}"))?;

    let (new_content, count) = if use_regex {
        let re = Regex::new(old).map_err(|e| format!("invalid regex: {e}"))?;
        let new_content = if all {
            re.replace_all(&content, new).to_string()
        } else {
            re.replace(&content, new).to_string()
        };
        let count = if all {
            re.find_iter(&content).count()
        } else {
            usize::from(re.is_match(&content))
        };
        (new_content, count)
    } else if all {
        (content.replace(old, new), content.matches(old).count())
    } else {
        match content.find(old) {
            Some(pos) => {
                let mut s = content.clone();
                s.replace_range(pos..pos + old.len(), new);
                (s, 1)
            }
            None => return Err(format!("'{old}' not found in {file}")),
        }
    };

    if count == 0 {
        return Err(format!("'{old}' not found in {file}"));
    }

    // Compute diff for dry-run or display
    let diff = if dry_run || !json {
        let tmp_old = format!("/tmp/_q_diff_old_{}", std::process::id());
        let tmp_new = format!("/tmp/_q_diff_new_{}", std::process::id());
        let _ = std::fs::write(&tmp_old, &content);
        let _ = std::fs::write(&tmp_new, &new_content);
        let result = Command::new("diff")
            .arg("-u")
            .arg("--label")
            .arg(file)
            .arg("--label")
            .arg(file)
            .arg(&tmp_old)
            .arg(&tmp_new)
            .output()
            .ok();
        let _ = std::fs::remove_file(&tmp_old);
        let _ = std::fs::remove_file(&tmp_new);
        result.and_then(|o| {
            let s = String::from_utf8(o.stdout).ok()?;
            if s.is_empty() { None } else { Some(s) }
        })
    } else {
        None
    };

    if dry_run {
        let diff_text = diff.as_deref().unwrap_or("(diff unavailable)");
        if json {
            let r = ReplaceResult {
                kind: "replace",
                file: file.to_string(),
                replacements: count,
                diff: Some(diff_text.to_string()),
            };
            serde_json::to_string(&r).map_err(|e| e.to_string())
        } else {
            Ok(format!("[dry-run] {file}: {count} replacement(s)\n{diff_text}"))
        }
    } else {
        // Create backup
        let bak = format!("{file}.q.bak");
        std::fs::write(&bak, &content).ok();

        std::fs::write(file, &new_content).map_err(|e| format!("write {file}: {e}"))?;

        if json {
            let r = ReplaceResult {
                kind: "replace",
                file: file.to_string(),
                replacements: count,
                diff,
            };
            serde_json::to_string(&r).map_err(|e| e.to_string())
        } else {
            Ok(format!("{file}: {count} replacement(s) (backup: {bak})"))
        }
    }
}

// ── Write ────────────────────────────────────────────────────────────────────

fn cmd_write(file: &str, content: Option<&str>, json: bool) -> Result<String, String> {
    let content = match content {
        Some(c) => c.to_string(),
        None => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("read stdin: {e}"))?;
            buf
        }
    };

    if let Some(parent) = Path::new(file).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("mkdir: {e}"))?;
    }

    let bytes = content.len();
    std::fs::write(file, &content).map_err(|e| format!("write {file}: {e}"))?;

    if json {
        let r = WriteResult { kind: "write", file: file.to_string(), bytes };
        serde_json::to_string(&r).map_err(|e| e.to_string())
    } else {
        Ok(format!("{file}: {bytes} bytes written"))
    }
}

// ── Info ─────────────────────────────────────────────────────────────────────

fn cmd_info(path: &str, json: bool) -> Result<String, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("stat {path}: {e}"))?;
    let is_dir = meta.is_dir();

    let size = if is_dir { None } else { Some(meta.len()) };
    let lines = if is_dir {
        None
    } else {
        std::fs::read_to_string(path)
            .ok()
            .map(|s| s.lines().count())
    };

    let modified = meta
        .modified()
        .ok()
        .map(|t| -> String {
            let dt: DateTime<Local> = t.into();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        });

    if json {
        let r = InfoResult {
            kind: "info",
            path: path.to_string(),
            is_dir,
            size_bytes: size,
            lines,
            modified,
        };
        serde_json::to_string(&r).map_err(|e| e.to_string())
    } else {
        let mut out = String::new();
        out.push_str(&format!("path:     {path}\n"));
        out.push_str(&format!("type:     {}\n", if is_dir { "directory" } else { "file" }));
        if let Some(s) = size {
            out.push_str(&format!("size:     {} bytes\n", s));
        }
        if let Some(l) = lines {
            out.push_str(&format!("lines:    {l}\n"));
        }
        if let Some(m) = modified {
            out.push_str(&format!("modified: {m}\n"));
        }
        Ok(out.trim_end().to_string())
    }
}

// ── Tree ─────────────────────────────────────────────────────────────────────

fn cmd_tree(path: Option<&str>, depth: usize, dirs_only: bool, json: bool) -> Result<String, String> {
    let root = path.unwrap_or(".");
    let root_path = std::fs::canonicalize(root).map_err(|e| format!("path {root}: {e}"))?;
    let root_str = root_path.to_string_lossy().to_string();

    let mut entries = Vec::new();

    let walker = WalkDir::new(root)
        .max_depth(depth)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            name != ".git" && name != "node_modules" && name != "target"
        });

    for entry in walker {
        let entry = entry.map_err(|e| format!("walk error: {e}"))?;
        let ft = entry.file_type();
        if dirs_only && !ft.is_dir() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(root)
            .unwrap_or(entry.path());
        let path_str = rel.to_string_lossy().to_string();
        let d = entry.depth();
        entries.push(TreeEntry {
            path: path_str,
            is_dir: ft.is_dir(),
            depth: d,
        });
    }

    if json {
        let r = TreeResult { kind: "tree", root: root_str, entries };
        serde_json::to_string(&r).map_err(|e| e.to_string())
    } else {
        if entries.is_empty() {
            return Ok(String::new());
        }
        let mut out = String::new();
        out.push_str(&format!("{}/\n", root_str));
        for e in &entries {
            if e.path.is_empty() {
                continue;
            }
            let indent = "  ".repeat(e.depth.saturating_sub(1));
            let prefix = if e.is_dir { "  " } else { "  " };
            let suffix = if e.is_dir { "/" } else { "" };
            let last = e.path.rsplit('/').next().unwrap_or(&e.path);
            out.push_str(&format!("{indent}{prefix}{last}{suffix}\n"));
        }
        Ok(out.trim_end().to_string())
    }
}

// ── Ls ───────────────────────────────────────────────────────────────────────

fn cmd_ls(path: Option<&str>, long: bool, all: bool, json: bool) -> Result<String, String> {
    let root = path.unwrap_or(".");
    let mut entries = Vec::new();

    let rd = std::fs::read_dir(root).map_err(|e| format!("read_dir {root}: {e}"))?;
    let mut names: Vec<_> = rd.filter_map(|e| e.ok()).collect();
    names.sort_by_key(|e| e.file_name());

    for entry in names {
        let name = entry.file_name().to_string_lossy().to_string();
        if !all && name.starts_with('.') {
            continue;
        }
        let ft = entry.file_type().ok();
        let is_dir = ft.map_or(false, |f| f.is_dir());
        let meta = entry.metadata().ok();
        let size = meta.as_ref().map(|m| m.len());
        let modified = meta.and_then(|m| m.modified().ok()).map(|t| -> String {
            let dt: DateTime<Local> = t.into();
            dt.format("%Y-%m-%d %H:%M:%S").to_string()
        });
        entries.push(LsEntry { name, is_dir, size, modified });
    }

    if json {
        let r = LsResult { kind: "ls", path: root.to_string(), entries };
        serde_json::to_string(&r).map_err(|e| e.to_string())
    } else if long {
        let mut out = String::new();
        for e in &entries {
            let size_str = e.size.map(|s| format!("{:>8}", s)).unwrap_or("       -".into());
            let mod_str = e.modified.as_deref().unwrap_or("-");
            let suffix = if e.is_dir { "/" } else { "" };
            out.push_str(&format!("{size_str} {mod_str} {}{suffix}\n", e.name));
        }
        Ok(out.trim_end().to_string())
    } else {
        let out: Vec<String> = entries.iter().map(|e| {
            if e.is_dir { format!("{}/", e.name) } else { e.name.clone() }
        }).collect();
        Ok(out.join("  "))
    }
}

// ── Mkdir ────────────────────────────────────────────────────────────────────

fn cmd_mkdir(path: &str, json: bool) -> Result<String, String> {
    std::fs::create_dir_all(path).map_err(|e| format!("mkdir {path}: {e}"))?;
    if json {
        let r = serde_json::json!({"kind": "mkdir", "path": path});
        serde_json::to_string(&r).map_err(|e| e.to_string())
    } else {
        Ok(format!("created {path}/"))
    }
}

// ── Mv ───────────────────────────────────────────────────────────────────────

fn cmd_mv(from: &str, to: &str, json: bool) -> Result<String, String> {
    if let Some(parent) = Path::new(to).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }
    std::fs::rename(from, to).map_err(|e| format!("mv {from} {to}: {e}"))?;
    if json {
        let r = FsResult { kind: "mv", from: from.to_string(), to: to.to_string() };
        serde_json::to_string(&r).map_err(|e| e.to_string())
    } else {
        Ok(format!("{from} → {to}"))
    }
}

// ── Cp ───────────────────────────────────────────────────────────────────────

fn cmd_cp(from: &str, to: &str, json: bool) -> Result<String, String> {
    if let Some(parent) = Path::new(to).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).ok();
        }
    }
    std::fs::copy(from, to).map_err(|e| format!("cp {from} {to}: {e}"))?;
    if json {
        let r = FsResult { kind: "cp", from: from.to_string(), to: to.to_string() };
        serde_json::to_string(&r).map_err(|e| e.to_string())
    } else {
        Ok(format!("{from} → {to}"))
    }
}

// ── Diff ─────────────────────────────────────────────────────────────────────

fn cmd_diff(file_a: &str, file_b: &str, json: bool) -> Result<String, String> {
    let output = Command::new("diff")
        .arg("-u")
        .arg(file_a)
        .arg(file_b)
        .output()
        .map_err(|e| format!("diff execution: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let result = if stdout.is_empty() { stderr } else { stdout };

    if json {
        let r = DiffResult {
            kind: "diff",
            file_a: file_a.to_string(),
            file_b: file_b.to_string(),
            diff: result.trim().to_string(),
        };
        serde_json::to_string(&r).map_err(|e| e.to_string())
    } else {
        if result.trim().is_empty() {
            Ok("files are identical".to_string())
        } else {
            Ok(result.trim_end().to_string())
        }
    }
}

// ── HTTP ─────────────────────────────────────────────────────────────────────

fn cmd_http(url: &str, json: bool) -> Result<String, String> {
    let output = Command::new("curl")
        .args(["-sL", "--max-time", "15"])
        .arg(url)
        .output()
        .map_err(|e| format!("curl: {e}"))?;

    let status = output.status.code();
    let body = String::from_utf8_lossy(&output.stdout).to_string();

    if json {
        let (body, truncated) = apply_trunc(&body, DEFAULT_MAX_LINES, false);
        let r = HttpResult {
            kind: "http",
            url: url.to_string(),
            status,
            body,
            truncated,
        };
        serde_json::to_string(&r).map_err(|e| e.to_string())
    } else {
        let (trimmed, _) = apply_trunc(body.trim_end(), DEFAULT_MAX_LINES, false);
        Ok(trimmed)
    }
}

// ── Git ──────────────────────────────────────────────────────────────────────

fn cmd_git(action: &GitAction, json: bool) -> Result<String, String> {
    // Special cases: commit, branch, checkout
    match action {
        GitAction::Commit { msg } => {
            Command::new("git").args(["add", "-A"]).output().ok();
            let out = Command::new("git")
                .args(["commit", "-m", msg])
                .output()
                .map_err(|e| format!("git commit: {e}"))?;
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let result = if stdout.is_empty() { stderr } else { stdout };
            if json {
                let r = GitResult { kind: "git", command: "commit".into(), output: result.trim().into() };
                return serde_json::to_string(&r).map_err(|e| e.to_string());
            }
            return Ok(result.trim_end().to_string());
        }
        GitAction::Branch => {
            let out = Command::new("git").args(["branch"]).output().map_err(|e| format!("git branch: {e}"))?;
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            if json {
                let r = GitResult { kind: "git", command: "branch".into(), output: stdout.trim().into() };
                return serde_json::to_string(&r).map_err(|e| e.to_string());
            }
            return Ok(stdout.trim_end().to_string());
        }
        GitAction::Checkout { branch } => {
            let out = Command::new("git")
                .args(["checkout", branch])
                .output()
                .map_err(|e| format!("git checkout: {e}"))?;
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let result = if stdout.is_empty() { stderr } else { stdout };
            if json {
                let r = GitResult { kind: "git", command: "checkout".into(), output: result.trim().into() };
                return serde_json::to_string(&r).map_err(|e| e.to_string());
            }
            return Ok(result.trim_end().to_string());
        }
        _ => {}
    }

    let (args, desc): (Vec<String>, &str) = match action {
        GitAction::Status => (vec!["status".into(), "--short".into()], "status"),
        GitAction::Diff => (vec!["diff".into()], "diff"),
        GitAction::Log { count } => (vec!["log".into(), "--oneline".into(), format!("-{count}")], "log"),
        GitAction::Staged => (vec!["diff".into(), "--cached".into()], "staged"),
        GitAction::Show { rev } => (vec!["show".into(), "--stat".into(), "--oneline".into(), rev.into()], "show"),
        _ => unreachable!(),
    };

    let output = Command::new("git")
        .args(&args)
        .output()
        .map_err(|e| format!("git {desc}: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let result = if stdout.is_empty() { stderr } else { stdout };

    if json {
        let r = GitResult {
            kind: "git",
            command: desc.to_string(),
            output: result.trim().to_string(),
        };
        serde_json::to_string(&r).map_err(|e| e.to_string())
    } else {
        if result.trim().is_empty() {
            Ok(format!("(empty -- `git {desc}`)"))
        } else {
            Ok(result.trim_end().to_string())
        }
    }
}
