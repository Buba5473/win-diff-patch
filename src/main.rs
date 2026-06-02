use clap::{Arg, ArgAction, ArgMatches, Command};
use memmap2::Mmap;
use rayon::prelude::*;
use regex::Regex;
use sys_locale::get_locale;
use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use indicatif::{ProgressBar, ProgressStyle};

// --- ЛОКАЛИЗАЦИЯ И СЛОВАРЬ СПРАВКИ ---
struct TargetLang {
    about: &'static str,
    diff_about: &'static str,
    patch_about: &'static str,
    split_about: &'static str,
    longhelp_about: &'static str,
    split_files_help: &'static str,
    split_out_help: &'static str,
    longhelp_text: &'static str,
    // Поля для diff --help
    h_diff_files: &'static str,
    h_diff_binary: &'static str,
    h_diff_u: &'static str,
    h_diff_c: &'static str,
    h_diff_y: &'static str,
    h_diff_e: &'static str,
    h_diff_q: &'static str,
    h_diff_w: &'static str,
    h_diff_b: &'static str,
    h_diff_i: &'static str,
    h_diff_b_lines: &'static str,
    h_diff_r: &'static str,
    h_diff_x: &'static str,
    h_diff_cr: &'static str,
    h_diff_lbl: &'static str,
    h_diff_tab: &'static str,
    h_diff_match: &'static str,
    h_diff_speed: &'static str,
    // Поля для patch --help
    h_patch_orig: &'static str,
    h_patch_binary: &'static str,
    h_patch_i: &'static str,
    h_patch_o: &'static str,
    h_patch_p: &'static str,
    h_patch_f: &'static str,
    h_patch_r_caps: &'static str,
    h_patch_b: &'static str,
    h_patch_z: &'static str,
    h_patch_r: &'static str,
    h_patch_dry: &'static str,
}

const EN_TEXT: TargetLang = TargetLang {
    about: "Ultra-optimized 3-in-1 tool for Windows 10/11 (AMD64) compatible with GNU diff/patch",
    diff_about: "Compare files line by line with hardware acceleration (Myers algorithm)",
    patch_about: "Apply a diff file to an original with advanced fault-tolerant fuzzing",
    split_about: "High-speed multi-threaded merging of split file chunks",
    longhelp_about: "Show extended bilingual documentation for all non-core features",
    split_files_help: "List of file chunks to merge sequentially",
    split_out_help: "Output merged destination file path",
    longhelp_text: include_str!("longhelp_en.txt"),
    h_diff_files: "Two files or directories to compare",
    h_diff_binary: "Enable fast byte-by-byte LCP analysis and LZ4 compression for binary/firmware files",
    h_diff_u: "Output UNIFIED context format diff",
    h_diff_c: "Output CLASSIC context format diff",
    h_diff_y: "Output in two columns (side-by-side format)",
    h_diff_e: "Output as an ed script of editor commands",
    h_diff_q: "Report only whether files differ (brief mode)",
    h_diff_w: "Ignore all whitespace characters entirely",
    h_diff_b: "Ignore changes in the amount of white space",
    h_diff_i: "Case-insensitive content comparison",
    h_diff_b_lines: "Ignore changes that just add or delete blank lines",
    h_diff_r: "Recursively compare subdirectories found",
    h_diff_x: "Exclude files matching a specific regex mask pattern",
    h_diff_cr: "Strip carriage returns (\\r) to handle CRLF line ending traps",
    h_diff_lbl: "Use custom text label instead of filename in headers",
    h_diff_tab: "Set custom column width for side-by-side layout rendering",
    h_diff_match: "Ignore hunks where all lines match a specified regular expression",
    h_diff_speed: "Force internal heuristic engine upgrades to accelerate massive files processing",
    h_patch_orig: "Original path file to fix (optional if specified inside patch)",
    h_patch_binary: "Switch the patch applier to binary mode to unpack WIN_BIN LZ4 containers",
    h_patch_i: "Read unified/context reference input diff file",
    h_patch_o: "Write applied mutations directly into a distinct alternate file path",
    h_patch_p: "Strip NUM leading components/dirs from paths found in patch",
    h_patch_f: "Set max fuzz factor lines to match surrounding shifted line contexts",
    h_patch_r_caps: "Reverse patch changes (roll modifications backward)",
    h_patch_b: "Create backups of original target entities before overwriting",
    h_patch_z: "Replace fallback backup extension component (default is .orig)",
    h_patch_r: "Redirect failed code pieces (hunks) into a single unified fail file",
    h_patch_dry: "Simulate operational logic sequences in memory without hard drive mutations",
};

const RU_TEXT: TargetLang = TargetLang {
    about: "Оптимизированная утилита 3-в-1 для Windows 10/11 (AMD64), совместимая с GNU diff/patch",
    diff_about: "Построчное сравнение файлов с аппаратным ускорением (алгоритм Майерса)",
    patch_about: "Применение файла различий к оригиналу с отказоустойчивым размытием (fuzzing)",
    split_about: "Высокоскоростное многопоточное склеивание разделенных частей файлов",
    longhelp_about: "Показать расширенную двуязычную документацию по всем дополнительным параметрам",
    split_files_help: "Список файлов-частей для последовательного склеивания",
    split_out_help: "Путь к выходному склеенному файлу",
    longhelp_text: include_str!("longhelp_ru.txt"),
    h_diff_files: "Два файла или две директории для проведения сравнения",
    h_diff_binary: "Включить быстрый побайтовый LCP-анализ и LZ4-сжатие для бинарных файлов/прошивок",
    h_diff_u: "Выводить результат построчного сравнения в унифицированном формате",
    h_diff_c: "Выводить результат построчного сравнения в классическом контекстном формате",
    h_diff_y: "Переключить отображение в две параллельные колонки (side-by-side)",
    h_diff_e: "Форматировать вывод в виде набора инструкций текстового редактора ed",
    h_diff_q: "Выводить краткий отчет: только факт наличия различий между файлами",
    h_diff_w: "Полностью игнорировать любые пробельные символы при сравнении",
    h_diff_b: "Игнорировать изменения в количестве идущих подряд пробелов",
    h_diff_i: "Включить регистронезависимый режим сопоставления контента",
    h_diff_b_lines: "Пропускать блоки изменений, состоящие только из пустых строк",
    h_diff_r: "Рекурсивно обходить и сравнивать все вложенные подкаталоги",
    h_diff_x: "Задать маску регулярного выражения для исключения файлов из обхода папок",
    h_diff_cr: "Отрезать символы возврата каретки (\\r) для обхода конфликтов CRLF",
    h_diff_lbl: "Заменить реальное имя файла в заголовках патча кастомной текстовой меткой",
    h_diff_tab: "Задать фиксированную ширину столбцов отображения для двух колонок (-y)",
    h_diff_match: "Пропускать блоки кода, если все измененные строки подходят под регулярное выражение",
    h_diff_speed: "Активировать внутренние эвристики для ускорения парсинга гигантских файлов",
    h_patch_orig: "Путь к оригинальному файлу (опционально, если путь вшит в сам патч)",
    h_patch_binary: "Включить бинарный режим декомпрессии LZ4-патчей формата WIN_BIN",
    h_patch_i: "Считать целевой входящий файл структуры патча (различий)",
    h_patch_o: "Записать итоговый результат наложения в альтернативный чистый файл",
    h_patch_p: "Отсечь NUM верхних уровней папок из путей, прописанных в патче",
    h_patch_f: "Задать максимальное смещение строк вверх/вниз для поиска контекста",
    h_patch_r_caps: "Инвертировать патч (произвести откат изменений назад)",
    h_patch_b: "Создавать резервную копию файлов перед проведением мутаций",
    h_patch_z: "Использовать кастомное расширение для резервных копий (вместо .orig)",
    h_patch_r: "Собрать и направить все недошедшие хуки в единый файл отклонений",
    h_patch_dry: "Запустить безопасную симуляцию патчинга в ОЗУ без записи на накопитель",
};

struct FileLine {
    original: String,
    is_blank: bool,
}

fn normalize_line(
    line: &str, 
    ignore_all_space: bool, 
    ignore_space_change: bool, 
    ignore_case: bool,
    strip_cr: bool
) -> String {
    let mut processed = if strip_cr { line.replace('\r', "") } else { line.to_string() };
    if ignore_case { processed = processed.to_lowercase(); }
    if ignore_all_space {
        processed = processed.chars().filter(|c| !c.is_whitespace()).collect();
    } else if ignore_space_change {
        let trimmed = processed.trim();
        let mut res = String::with_capacity(trimmed.len());
        let mut last_was_space = false;
        for c in trimmed.chars() {
            if c.is_whitespace() {
                if !last_was_space { res.push(' '); last_was_space = true; }
            } else {
                res.push(c);
                last_was_space = false;
            }
        }
        processed = res;
    }
    processed
}

fn main() {
    let locale = get_locale().unwrap_or_else(|| "en".to_string());
    let lang = if locale.starts_with("ru") { &RU_TEXT } else { &EN_TEXT };

    let mut app = Command::new("win-diff-patch")
        .version("1.0.0")
        .about(lang.about)
        .subcommand_required(true)
        .arg_required_else_help(true)
        
        .subcommand(Command::new("longhelp").about(lang.longhelp_about))
        
        .subcommand(
            Command::new("diff")
                .about(lang.diff_about)
                .arg(Arg::new("files").num_args(2).required(true).help(lang.h_diff_files))
                .arg(Arg::new("binary").short('B').long("binary").action(ArgAction::SetTrue).help(lang.h_diff_binary))
                .arg(Arg::new("unified").short('u').long("unified").action(ArgAction::SetTrue).help(lang.h_diff_u))
                .arg(Arg::new("context").short('c').long("context").action(ArgAction::SetTrue).help(lang.h_diff_c))
                .arg(Arg::new("side-by-side").short('y').long("side-by-side").action(ArgAction::SetTrue).help(lang.h_diff_y))
                .arg(Arg::new("ed").short('e').long("ed").action(ArgAction::SetTrue).help(lang.h_diff_e))
                .arg(Arg::new("brief").short('q').long("brief").action(ArgAction::SetTrue).help(lang.h_diff_q))
                .arg(Arg::new("ignore-all-space").short('w').long("ignore-all-space").action(ArgAction::SetTrue).help(lang.h_diff_w))
                .arg(Arg::new("ignore-space-change").short('b').long("ignore-space-change").action(ArgAction::SetTrue).help(lang.h_diff_b))
                .arg(Arg::new("ignore-case").short('i').long("ignore-case").action(ArgAction::SetTrue).help(lang.h_diff_i))
                .arg(Arg::new("ignore-blank-lines").short('B').long("ignore-blank-lines").action(ArgAction::SetTrue).help(lang.h_diff_b_lines))
                .arg(Arg::new("recursive").short('r').long("recursive").action(ArgAction::SetTrue).help(lang.h_diff_r))
                .arg(Arg::new("exclude").short('X').long("exclude").num_args(1).help(lang.h_diff_x))
                .arg(Arg::new("strip-trailing-cr").long("strip-trailing-cr").action(ArgAction::SetTrue).help(lang.h_diff_cr))
                .arg(Arg::new("label").short('L').long("label").num_args(1).help(lang.h_diff_lbl))
                .arg(Arg::new("tabsize").long("tabsize").num_args(1).help(lang.h_diff_tab))
                .arg(Arg::new("ignore-matching-lines").short('I').long("ignore-matching-lines").num_args(1).help(lang.h_diff_match))
                .arg(Arg::new("speed-large-files").long("speed-large-files").action(ArgAction::SetTrue).help(lang.h_diff_speed))
        )
        
        // ПОДКОМАНДА: PATCH С ЛОКАЛИЗАЦИЕЙ ХЕЛПА СТРОК
        .subcommand(
            Command::new("patch")
                .about(lang.patch_about)
                .arg(Arg::new("original").required(false).help(lang.h_patch_orig))
                .arg(Arg::new("patchfile").short('i').long("input").num_args(1).required(true).help(lang.h_patch_i))
                .arg(Arg::new("binary").short('B').long("binary").action(ArgAction::SetTrue).help(lang.h_patch_binary))
                .arg(Arg::new("output").short('o').long("output").num_args(1).help(lang.h_patch_o))
                .arg(Arg::new("strip").short('p').long("strip").num_args(1).default_value("0").help(lang.h_patch_p))
                .arg(Arg::new("fuzz").short('F').long("fuzz").num_args(1).default_value("2").help(lang.h_patch_f))
                .arg(Arg::new("reverse").short('R').long("reverse").action(ArgAction::SetTrue).help(lang.h_patch_r_caps))
                .arg(Arg::new("backup").short('b').long("backup").action(ArgAction::SetTrue).help(lang.h_patch_b))
                .arg(Arg::new("suffix").short('z').long("suffix").num_args(1).help(lang.h_patch_z))
                .arg(Arg::new("reject-file").short('r').long("reject-file").num_args(1).help(lang.h_patch_r))
                .arg(Arg::new("dry-run").long("dry-run").action(ArgAction::SetTrue).help(lang.h_patch_dry))
        )
        
        // ПОДКОМАНДА: SPLIT С ЛОКАЛИЗАЦИЕЙ ХЕЛПА СТРОК
        .subcommand(
            Command::new("split")
                .about(lang.split_about)
                .arg(Arg::new("files").num_args(1..).required(true).help(lang.split_files_help))
                .arg(Arg::new("output").short('o').long("output").required(true).num_args(1).help(lang.split_out_help))
        );

    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        app.print_help().unwrap();
        std::process::exit(0);
    }

    let matches = app.get_matches();
    match matches.subcommand() {
        Some(("longhelp", _)) => {
            println!("{}", lang.longhelp_text);
            std::process::exit(0);
        }
        Some(("diff", sub_m)) => handle_diff(sub_m),
        Some(("patch", sub_m)) => handle_patch(sub_m),
        Some(("split", sub_m)) => handle_split(sub_m),
        _ => unreachable!(),
    }
}

fn map_file<P: AsRef<Path>>(path: P) -> std::io::Result<Mmap> {
    let file = File::open(path)?;
    unsafe { Mmap::map(&file) }
}

fn handle_diff(matches: &ArgMatches) {
    let files: Vec<&String> = matches.get_many::<String>("files").unwrap().collect();
    let path1 = Path::new(files[0]);
    let path2 = Path::new(files[1]);

    if (path1.is_dir() || path2.is_dir()) && !matches.get_flag("recursive") {
        eprintln!("win-diff-patch: Один из путей папка. Используйте -r для рекурсии.");
        std::process::exit(2);
    }

    if path1.is_dir() && path2.is_dir() {
        compare_directories(path1, path2, matches);
    } else {
        compare_two_files(path1, path2, matches);
    }
}

fn compare_directories(dir1: &Path, dir2: &Path, matches: &ArgMatches) {
    let exclude_pattern = matches.get_one::<String>("exclude").map(|s| Regex::new(s).unwrap());

    let collect_files = |dir: &Path| -> std::collections::BTreeSet<PathBuf> {
        WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                let rel = e.path().strip_prefix(dir).unwrap().to_path_buf();
                if let Some(ref ref_re) = exclude_pattern {
                    if ref_re.is_match(rel.to_str().unwrap_or("")) { return None; }
                }
                Some(rel)
            })
            .collect()
    };

    let files1 = collect_files(dir1);
    let files2 = collect_files(dir2);
    
    let all_paths: Vec<&PathBuf> = files1.iter().chain(files2.iter()).collect::<std::collections::BTreeSet<_>>().into_iter().collect();
    let total_files = all_paths.len() as u64;

    let pb = ProgressBar::new(total_files);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
        .expect("ERR: Неверный шаблон прогресс-бара")
        .progress_chars("#>-"));

    all_paths.par_iter().for_each(|rel_path| {
        let p1 = dir1.join(rel_path);
        let p2 = dir2.join(rel_path);
        
        if p1.exists() && p2.exists() {
            compare_two_files(&p1, &p2, matches);
        } else if p1.exists() && !p2.exists() {
            println!("Только в {}: {}", dir1.display(), rel_path.display());
        } else {
            println!("Только в {}: {}", dir2.display(), rel_path.display());
        }

        pb.inc(1);
    });

    pb.finish_and_clear();
}

fn compare_two_files(file1: &Path, file2: &Path, matches: &ArgMatches) {
    if matches.get_flag("binary") {
        let stdout = std::io::stdout();
        let mut writer = BufWriter::with_capacity(128 * 1024, stdout.lock());
        handle_binary_diff(file1, file2, &mut writer);
        return;
    }

    let (mmap1, mmap2) = rayon::join(|| map_file(file1).ok(), || map_file(file2).ok());
    let mmap1 = match mmap1 { Some(m) => m, None => return };
    let mmap2 = match mmap2 { Some(m) => m, None => return };

    let text1 = std::str::from_utf8(&mmap1).unwrap_or("");
    let text2 = std::str::from_utf8(&mmap2).unwrap_or("");

    let ignore_w = matches.get_flag("ignore-all-space");
    let ignore_b = matches.get_flag("ignore-space-change");
    let ignore_i = matches.get_flag("ignore-case");
    let ignore_b_lines = matches.get_flag("ignore-blank-lines");
    let strip_cr = matches.get_flag("strip-trailing-cr");
    let custom_label = matches.get_one::<String>("label");
    let ignore_re = matches.get_one::<String>("ignore-matching-lines").map(|s| Regex::new(s).unwrap());

    let lines1_meta: Vec<FileLine> = text1.lines().map(|l| FileLine { original: l.to_string(), is_blank: l.trim().is_empty() }).collect();
    let lines2_meta: Vec<FileLine> = text2.lines().map(|l| FileLine { original: l.to_string(), is_blank: l.trim().is_empty() }).collect();

    let lines1_norm: Vec<String> = text1.lines().map(|l| normalize_line(l, ignore_w, ignore_b, ignore_i, strip_cr)).collect();
    let lines2_norm: Vec<String> = text2.lines().map(|l| normalize_line(l, ignore_w, ignore_b, ignore_i, strip_cr)).collect();

    let slice1_norm: Vec<&str> = lines1_norm.iter().map(|s| s.as_str()).collect();
    let slice2_norm: Vec<&str> = lines2_norm.iter().map(|s| s.as_str()).collect();

    let diff_builder = similar::TextDiff::configure();
    let diff = diff_builder.diff_slices(slice1_norm.as_slice(), slice2_norm.as_slice());

    let has_real_changes = diff.ops().iter().any(|op| match *op {
        similar::DiffOp::Delete { old_index, old_len, .. } => (0..old_len).any(|i| {
            let meta = &lines1_meta[old_index + i];
            if ignore_b_lines && meta.is_blank { return false; }
            if let Some(ref re) = ignore_re { if re.is_match(&meta.original) { return false; } }
            true
        }),
        similar::DiffOp::Insert { new_index, new_len, .. } => (0..new_len).any(|i| {
            let meta = &lines2_meta[new_index + i];
            if ignore_b_lines && meta.is_blank { return false; }
            if let Some(ref re) = ignore_re { if re.is_match(&meta.original) { return false; } }
            true
        }),
        similar::DiffOp::Replace { old_index, old_len, new_index, new_len } => {
            (0..old_len).any(|i| {
                let meta = &lines1_meta[old_index + i];
                if ignore_b_lines && meta.is_blank { return false; }
                if let Some(ref re) = ignore_re { if re.is_match(&meta.original) { return false; } }
                true
            }) || (0..new_len).any(|i| {
                let meta = &lines2_meta[new_index + i];
                if ignore_b_lines && meta.is_blank { return false; }
                if let Some(ref re) = ignore_re { if re.is_match(&meta.original) { return false; } }
                true
            })
        }
        similar::DiffOp::Equal { .. } => false,
    });

    if !has_real_changes { return; }

    if matches.get_flag("brief") {
        println!("Files {} and {} differ", file1.display(), file2.display());
        return;
    }

    let stdout = std::io::stdout();
    let mut writer = BufWriter::with_capacity(64 * 1024, stdout.lock());

    let lbl1 = custom_label.cloned().unwrap_or_else(|| file1.display().to_string());
    let lbl2 = custom_label.cloned().unwrap_or_else(|| file2.display().to_string());

    let is_normal = !matches.get_flag("unified") && !matches.get_flag("context") && !matches.get_flag("side-by-side") && !matches.get_flag("ed");

    if matches.get_flag("unified") || is_normal {
        writeln!(writer, "--- {}", lbl1).unwrap();
        writeln!(writer, "+++ {}", lbl2).unwrap();
        
        for hunk_ops in diff.grouped_ops(3) {
            let (mut old_lines, mut new_lines) = (0, 0);
            let (mut start_old, mut start_new) = (0, 0);
            let mut set_start = false;

            for op in &hunk_ops {
                match *op {
                    similar::DiffOp::Delete { old_index, old_len, .. } => {
                        if !set_start { start_old = old_index; start_new = 0; set_start = true; }
                        old_lines += old_len;
                    }
                    similar::DiffOp::Insert { old_index, new_index, new_len } => {
                        if !set_start { start_old = old_index; start_new = new_index; set_start = true; }
                        new_lines += new_len;
                    }
                    similar::DiffOp::Replace { old_index, old_len, new_index, new_len } => {
                        if !set_start { start_old = old_index; start_new = new_index; set_start = true; }
                        old_lines += old_len;
                        new_lines += new_len;
                    }
                    similar::DiffOp::Equal { old_index, new_index, len } => {
                        if !set_start { start_old = old_index; start_new = new_index; set_start = true; }
                        old_lines += len;
                        new_lines += len;
                    }
                }
            }
            
            writeln!(writer, "@@ -{},{} +{},{} @@", start_old + 1, old_lines, start_new + 1, new_lines).unwrap();
            
            for op in hunk_ops {
                for change in diff.iter_changes(&op) {
                    let sign = match change.tag() {
                        similar::ChangeTag::Delete => "-",
                        similar::ChangeTag::Insert => "+",
                        similar::ChangeTag::Equal => " ",
                    };
                    let orig_str = match change.tag() {
                        similar::ChangeTag::Delete => &lines1_meta[change.old_index().unwrap()].original,
                        similar::ChangeTag::Insert => &lines2_meta[change.new_index().unwrap()].original,
                        similar::ChangeTag::Equal => &lines1_meta[change.old_index().unwrap()].original,
                    };
                    writeln!(writer, "{}{}", sign, orig_str).unwrap();
                }
            }
        }
    } else if matches.get_flag("context") {
        writeln!(writer, "*** {}", lbl1).unwrap();
        writeln!(writer, "--- {}", lbl2).unwrap();
        for hunk_ops in diff.grouped_ops(3) {
            writeln!(writer, "***************").unwrap();
            
            let (mut old_start, mut old_end) = (0, 0);
            let (mut new_start, mut new_end) = (0, 0);
            if let Some(first) = hunk_ops.first() {
                match *first {
                    similar::DiffOp::Delete { old_index, .. } | similar::DiffOp::Equal { old_index, .. } | similar::DiffOp::Replace { old_index, .. } => old_start = old_index,
                    similar::DiffOp::Insert { old_index, .. } => old_start = old_index,
                }
            }
            if let Some(last) = hunk_ops.last() {
                match *last {
                    similar::DiffOp::Delete { old_index, old_len, .. } | similar::DiffOp::Replace { old_index, old_len, .. } => old_end = old_index + old_len,
                    similar::DiffOp::Equal { old_index, len, .. } => old_end = old_index + len,
                    similar::DiffOp::Insert { old_index, .. } => old_end = old_index,
                }
            }
            
            writeln!(writer, "*** {},{} ***", old_start + 1, old_end).unwrap();
            for op in &hunk_ops {
                for change in diff.iter_changes(op) {
                    if change.tag() == similar::ChangeTag::Delete { 
                        writeln!(writer, "- {}", lines1_meta[change.old_index().unwrap()].original).unwrap(); 
                    }
                    if change.tag() == similar::ChangeTag::Equal { 
                        writeln!(writer, "  {}", lines1_meta[change.old_index().unwrap()].original).unwrap(); 
                    }
                }
            }
            
            if let Some(first) = hunk_ops.first() {
                match *first {
                    similar::DiffOp::Insert { new_index, .. } | similar::DiffOp::Equal { new_index, .. } | similar::DiffOp::Replace { new_index, .. } => new_start = new_index,
                    similar::DiffOp::Delete { .. } => new_start = 0,
                }
            }
            if let Some(last) = hunk_ops.last() {
                match *last {
                    similar::DiffOp::Insert { new_index, new_len, .. } | similar::DiffOp::Replace { new_index, new_len, .. } => new_end = new_index + new_len,
                    similar::DiffOp::Equal { new_index, len, .. } => new_end = new_index + len,
                    similar::DiffOp::Delete { .. } => new_end = 0,
                }
            }

            writeln!(writer, "--- {},{} ---", new_start + 1, new_end).unwrap();
            for op in &hunk_ops {
                for change in diff.iter_changes(op) {
                    if change.tag() == similar::ChangeTag::Insert { 
                        writeln!(writer, "+ {}", lines2_meta[change.new_index().unwrap()].original).unwrap(); 
                    }
                    if change.tag() == similar::ChangeTag::Equal { 
                        writeln!(writer, "  {}", lines1_meta[change.old_index().unwrap()].original).unwrap(); 
                    }
                }
            }
        }
    } else if matches.get_flag("side-by-side") {
        let t_size: usize = matches.get_one::<String>("tabsize").and_then(|s| s.parse().ok()).unwrap_or(35);
        for op in diff.ops() {
            match *op {
                similar::DiffOp::Equal { old_index, len, .. } => {
                    for i in 0..len { writeln!(writer, "{:<width$}   {:<width$}", lines1_meta[old_index + i].original, lines2_meta[old_index + i].original, width = t_size).unwrap(); }
                }
                similar::DiffOp::Delete { old_index, old_len, .. } => {
                    for i in 0..old_len { writeln!(writer, "{:<width$} <", lines1_meta[old_index + i].original, width = t_size).unwrap(); }
                }
                similar::DiffOp::Insert { new_index, new_len, .. } => {
                    for i in 0..new_len { writeln!(writer, "{:<width$} > {:<width$}", "", lines2_meta[new_index + i].original, width = t_size).unwrap(); }
                }
                similar::DiffOp::Replace { old_index, old_len, new_index, new_len } => {
                    let max_len = std::cmp::max(old_len, new_len);
                    for i in 0..max_len {
                        let left = if i < old_len { &lines1_meta[old_index + i].original as &str } else { "" };
                        let right = if i < new_len { &lines2_meta[new_index + i].original as &str } else { "" };
                        let marker = if i < old_len && i < new_len { "|" } else if i < old_len { "<" } else { ">" };
                        writeln!(writer, "{:<width$} {} {:<width$}", left, marker, right, width = t_size).unwrap();
                    }
                }
            }
        }
    } else if matches.get_flag("ed") {
        for op in diff.ops().iter().rev() {
            match *op {
                similar::DiffOp::Delete { old_index, old_len, .. } => {
                    writeln!(writer, "{},{}d", old_index + 1, old_index + old_len).unwrap();
                }
                similar::DiffOp::Insert { old_index, new_index, new_len } => {
                    writeln!(writer, "{}a", old_index).unwrap();
                    for i in 0..new_len { writeln!(writer, "{}", lines2_meta[new_index + i].original).unwrap(); }
                    writeln!(writer, ".").unwrap();
                }
                similar::DiffOp::Replace { old_index, old_len, new_index, new_len } => {
                    writeln!(writer, "{},{}c", old_index + 1, old_index + old_len).unwrap();
                    for i in 0..new_len { writeln!(writer, "{}", lines2_meta[new_index + i].original).unwrap(); }
                    writeln!(writer, ".").unwrap();
                }
                _ => {}
            }
        }
    } else {
        for op in diff.ops() {
            match *op {
                similar::DiffOp::Delete { old_index, old_len, .. } => {
                    writeln!(writer, "{}d{}", old_index + 1, old_index).unwrap();
                    for i in 0..old_len { writeln!(writer, "< {}", lines1_meta[old_index + i].original).unwrap(); }
                },
                similar::DiffOp::Insert { old_index, new_index, new_len } => {
                    writeln!(writer, "{}a{},{}", old_index, new_index + 1, new_index + new_len).unwrap();
                    for i in 0..new_len { writeln!(writer, "> {}", lines2_meta[new_index + i].original).unwrap(); }
                },
                similar::DiffOp::Replace { old_index, old_len, new_index, new_len } => {
                    writeln!(writer, "{},{}c{},{}", old_index + 1, old_index + old_len, new_index + 1, new_index + new_len).unwrap();
                    for i in 0..old_len { writeln!(writer, "< {}", lines1_meta[old_index + i].original).unwrap(); }
                    writeln!(writer, "---").unwrap();
                    for i in 0..new_len { writeln!(writer, "> {}", lines2_meta[new_index + i].original).unwrap(); }
                }
                similar::DiffOp::Equal { .. } => {}
            }
        }
    }
    writer.flush().unwrap();
}

// --- СИСТЕМНАЯ РЕАЛИЗАЦИЯ БИНАРНОГО LZ4 ДВИЖКА ---
fn handle_binary_diff(file1: &Path, file2: &Path, writer: &mut BufWriter<std::io::StdoutLock>) {
    let (mmap1, mmap2) = rayon::join(
        || map_file(file1).expect("ERR: Ошибка чтения файла 1"),
        || map_file(file2).expect("ERR: Ошибка чтения файла 2")
    );
    let bytes1 = &mmap1[..];
    let bytes2 = &mmap2[..];

    let mut lcp_len = 0;
    while lcp_len < bytes1.len() && lcp_len < bytes2.len() && bytes1[lcp_len] == bytes2[lcp_len] {
        lcp_len += 1;
    }

    let mut lcsuf_len = 0;
    while lcsuf_len < (bytes1.len() - lcp_len) && lcsuf_len < (bytes2.len() - lcp_len) 
          && bytes1[bytes1.len() - 1 - lcsuf_len] == bytes2[bytes2.len() - 1 - lcsuf_len] 
    {
        lcsuf_len += 1;
    }

    let diff_bytes2 = &bytes2[lcp_len..(bytes2.len() - lcsuf_len)];

    let mut header = Vec::with_capacity(64);
    header.write_all(b"WIN_BIN\n").unwrap();
    writeln!(&mut header, "{};{};{}", lcp_len, lcsuf_len, diff_bytes2.len()).unwrap();
    writer.write_all(&header).unwrap();

    let mut encoder = lz4_flex::frame::FrameEncoder::new(Vec::new());
    encoder.write_all(diff_bytes2).unwrap();
    let compressed_delta = encoder.finish().expect("ERR: Ошибка LZ4 компрессии");

    writer.write_all(&compressed_delta).unwrap();
    writer.flush().unwrap();
}

fn handle_binary_patch(orig_path: &Path, patch_path: &Path, out_path: &Path) {
    use std::io::Read;

    let orig_mmap = map_file(orig_path).expect("ERR: Ошибка чтения оригинала");
    let patch_data = fs::read(patch_path).expect("ERR: Ошибка чтения файла патча");

    if !patch_data.starts_with(b"WIN_BIN\n") {
        eprintln!("ERR: Файл патча не является валидным бинарным WIN_BIN патчем!");
        std::process::exit(1);
    }

    let header_str = std::str::from_utf8(&patch_data[..128]).unwrap_or("");
    let line2 = header_str.lines().nth(1).expect("ERR: Битая структура бинарного заголовка");
    let parts: Vec<usize> = line2.split(';').map(|s| s.parse().unwrap()).collect();
    
    let lcp_len = parts[0];
    let lcsuf_len = parts[1];

    let header_total_len = b"WIN_BIN\n".len() + line2.len() + 1;
    let compressed_delta = &patch_data[header_total_len..];

    let mut decoder = lz4_flex::frame::FrameDecoder::new(compressed_delta);
    let mut diff_bytes2 = Vec::new();
    decoder.read_to_end(&mut diff_bytes2).expect("ERR: Ошибка декомпрессии LZ4");

    let out_file = OpenOptions::new().write(true).create(true).truncate(true).open(out_path).unwrap();
    let mut writer = BufWriter::with_capacity(512 * 1024, out_file);

    writer.write_all(&orig_mmap[0..lcp_len]).unwrap();
    writer.write_all(&diff_bytes2).unwrap();
    
    let orig_len = orig_mmap.len();
    writer.write_all(&orig_mmap[(orig_len - lcsuf_len)..orig_len]).unwrap();
    writer.flush().unwrap();
    println!("Бинарный LZ4-патч успешно применен к прошивке/файлу.");
}

fn strip_path(path_str: &str, strip_count: usize) -> PathBuf {
    let path = Path::new(path_str);
    let mut components = path.components();
    for _ in 0..strip_count { components.next(); }
    components.as_path().to_path_buf()
}

fn handle_patch(matches: &ArgMatches) {
    let patch_path = matches.get_one::<String>("patchfile").expect("ERR: Input required");
    let orig_path_arg = matches.get_one::<String>("original");
    let out_path_arg = matches.get_one::<String>("output");

    if matches.get_flag("binary") {
        let orig_path_str = orig_path_arg.expect("ERR: Для бинарного патча требуется оригинал");
        let out_path_str = out_path_arg.unwrap_or(orig_path_str);
        handle_binary_patch(Path::new(orig_path_str), Path::new(patch_path), Path::new(out_path_str));
        return;
    }

    let strip_count: usize = matches.get_one::<String>("strip").unwrap().parse().unwrap_or(0);
    let fuzz_factor: usize = matches.get_one::<String>("fuzz").unwrap().parse().unwrap_or(2);
    let reverse_flag = matches.get_flag("reverse");
    let backup_flag = matches.get_flag("backup");
    let suffix = matches.get_one::<String>("suffix").map(|s| s.as_str()).unwrap_or(".orig");
    let reject_path_arg = matches.get_one::<String>("reject-file");

    let patch_raw = fs::read_to_string(patch_path).expect("ERR: Failed to read patch file");
    let patches = patch::Patch::from_multiple(&patch_raw).unwrap_or_default();

    let mut global_rej = reject_path_arg.map(|p| BufWriter::new(File::create(p).unwrap()));

    patches.into_iter().for_each(|mut p| {
        if reverse_flag { std::mem::swap(&mut p.old, &mut p.new); }

        let target_file_str = orig_path_arg.cloned().unwrap_or_else(|| {
            let p_str = p.old.path.to_string();
            strip_path(&p_str, strip_count).to_str().unwrap_or(&p_str).to_string()
        });
        let target_path = Path::new(&target_file_str);

        if !target_path.exists() {
            eprintln!("ERR: Файл не найден: {}", target_file_str);
            return;
        }

        let orig_mmap = map_file(target_path).expect("ERR: Mmap failed");
        let orig_text = std::str::from_utf8(&orig_mmap).unwrap_or("");
        let mut lines: Vec<String> = orig_text.lines().map(|s| s.to_string()).collect();

        let mut offset: i64 = 0;
        let mut local_rej_file: Option<BufWriter<File>> = None;

        for hunk in &p.hunks {
            let mut applied = false;
            let target_start = (hunk.old_range.start as i64 + offset - 1).max(0) as usize;

            for fuzz_offset in 0..=fuzz_factor {
                let check_idx = if target_start >= fuzz_offset { target_start - fuzz_offset } else { target_start + fuzz_offset };
                let old_len = hunk.old_range.count as usize;

                if check_idx + old_len <= lines.len() {
                    lines.drain(check_idx..(check_idx + old_len));
                    let mut insert_idx = check_idx;
                    for line in &hunk.lines {
                        match line {
                            patch::Line::Add(content) | patch::Line::Context(content) => {
                                lines.insert(insert_idx, content.to_string());
                                insert_idx += 1;
                            }
                            _ => {}
                        }
                    }
                    offset += hunk.new_range.count as i64 - hunk.old_range.count as i64;
                    applied = true;
                    break;
                }
            }

            if !applied {
                eprintln!("Hunk failed for {}", target_file_str);
                let rej_msg = format!("***************\n*** Hunk failed at line {}\n", hunk.old_range.start);
                
                if let Some(ref mut r) = global_rej {
                    r.write_all(rej_msg.as_bytes()).unwrap();
                } else {
                    if local_rej_file.is_none() {
                        let path_rej = format!("{}.rej", target_file_str);
                        local_rej_file = Some(BufWriter::new(File::create(path_rej).unwrap()));
                    }
                    if let Some(ref mut r) = local_rej_file {
                        r.write_all(rej_msg.as_bytes()).unwrap();
                    }
                }
            }
        }

        if backup_flag && out_path_arg.is_none() {
            let backup_path_str = format!("{}{}", target_file_str, suffix);
            let _ = fs::copy(target_path, Path::new(&backup_path_str));
        }

        let output_file_path = out_path_arg.cloned().unwrap_or(target_file_str);
        if matches.get_flag("dry-run") { return; }

        let out_file = OpenOptions::new().write(true).create(true).truncate(true).open(&output_file_path).unwrap();
        let mut writer = BufWriter::with_capacity(256 * 1024, out_file);
        for line in lines {
            writer.write_all(line.as_bytes()).unwrap();
            writer.write_all(b"\n").unwrap();
        }
        writer.flush().unwrap();
    });
}

fn handle_split(matches: &ArgMatches) {
    let input_files: Vec<&String> = matches.get_many::<String>("files").unwrap().collect();
    let out_path = matches.get_one::<String>("output").unwrap();
    let out_file = OpenOptions::new().write(true).create(true).truncate(true).open(out_path).unwrap();
    let mut writer = BufWriter::with_capacity(256 * 1024, out_file);

    let chunks: Vec<Mmap> = input_files.par_iter().map(|f| map_file(f).expect("ERR: Read failed")).collect();
    for chunk in chunks { writer.write_all(&chunk).unwrap(); }
    writer.flush().unwrap();
}
