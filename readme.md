# win-diff-patch

[Русский](#русский) | [English](#english)

---

## Русский

**win-diff-patch** — это полнофункциональная, высокопроизводительная консольная утилита «3-в-1» для операционных систем Windows 10/11 (AMD64). Она объединяет в себе возможности классических Linux-утилит `diff` и `patch`, а также специализированный модуль `split` для параллельной высокоскоростной склейки бинарных файлов и чанков данных.

Проект глубоко оптимизирован под архитектуру AMD64, активно использует многопоточность на уровне процессора (`rayon`) и технологию отображения файлов в память (`memmap2`), что позволяет утилизировать физический предел скорости NVMe/SSD накопителей без накладных расходов.

### Основные особенности
*   **Два режима работы**: Поддерживает как классическое построчное текстовое сравнение (алгоритм Майерса), так и мгновенный побайтовый бинарный анализ (алгоритм LCP) исполняемых файлов и прошивок.
*   **Встроенное сжатие LZ4**: При работе с бинарными данными дельта изменений автоматически упаковывается во встроенный высокопроизводительный контейнер LZ4, работающий на скорости до нескольких гигабайт в секунду на ядро.
*   **Полная автономность (Standalone)**: Скомпилированный бинарник статически линкует C-Runtime (UCRT) и базовые библиотеки GCC/MinGW. Ему не нужны сторонние `.dll` или установленная среда MSYS2 — он работает на любой чистой Windows 10/11.
*   **Вшитые метаданные**: В итоговый `.exe` файл автоматически интегрируются иконка приложения и метаданные версии (CompanyName: `buba5473`, копирайты, описание), доступные через свойства файла в Проводнике Windows.
*   **Автоматическая локализация**: Справка утилиты (`--help`) для всех подкоманд автоматически переключается на русский или английский язык в зависимости от системной локали текущего пользователя Windows.
*   **Потокобезопасный прогресс-бар**: При рекурсивном обходе папок в реальном времени отображается индикатор выполнения. Он выводится строго в поток `stderr`, что полностью исключает засорение выходного файла патча при перенаправлении вывода (`>`).

### Структура проекта
```text
win-diff-patch/
├── .cargo/
│   └── config.toml        # Конфигурация статической линковки GCC/MinGW
├── src/
│   ├── main.rs            # Полный исходный код утилиты
│   ├── longhelp_ru.txt    # Расширенная справка на русском языке
│   └── longhelp_en.txt    # Расширенная справка на английском языке
├── build.rs               # Автоматическая генерация метаданных Windows (автор: buba5473)
├── build.sh               # Скрипт автоматизации сборки (Запускать в терминале UCRT64)
└── Cargo.toml             # Манифест пакета, зависимости и профиль оптимизации
```

### Инструкция по сборке в MSYS2
Для компиляции проекта вам понадобится установленная среда разработки **MSYS2**.

1. Откройте терминал **MSYS2 UCRT64**.
2. Перейдите в корневую директорию проекта.
3. Разрешите исполнение скрипта сборки и запустите его:
   ```bash
   chmod +x build.sh
   ./build.sh
   ```
*Скрипт автоматически проверит наличие необходимых компиляторов (`rust`, `gcc`, `windres`), установит их через pacman в случае отсутствия, сконфигурирует окружение и соберет монолитный `win-diff-patch.exe` по пути `target/x86_64-pc-windows-gnu/release/`.*

### Использование и параметры командной строки

#### Основной параметр: `diff`
Используется для построчного сравнения файлов или рекурсивного сравнения каталогов. По умолчанию (если не указаны другие форматы) выводит стандартный **Unified Diff** (с метаданными хуков `@@`).
*   `win-diff-patch diff <ФАЙЛ_1> <ФАЙЛ_2>` — базовое сравнение (формат Unified Diff по умолчанию).
*   `-B`, `--binary` — переключить утилиту в бинарный режим. Выполняет мгновенный LCP-анализ исполняемых файлов или прошивок и сжимает дельту в формат LZ4.
*   `-u`, `--unified` — явное указание вывода изменений в унифицированном формате контекста.
*   `-c`, `--context` — вывод изменений в классическом контекстном формате GNU.
*   `-y`, `--side-by-side` — вывод результатов сопоставления в две параллельные колонки.
*   `-e`, `--ed` — форматирование вывода в виде скрипта команд для редактора `ed`.
*   `-q`, `--brief` — выводить только краткий отчет (различаются файлы или нет).
*   `-i`, `--ignore-case` — регистронезависимое сравнение контента.
*   `-w`, `--ignore-all-space` — полностью игнорировать любые пробельные символы.
*   `-b`, `--ignore-space-change` — игнорировать изменение количества пробелов.
*   `-B`, `--ignore-blank-lines` — игнорировать вставку или удаление пустых строк.
*   `-r`, `--recursive` — параллельное рекурсивное сравнение вложенных каталогов на всех ядрах CPU.
*   `-X <РЕГЕКС>`, `--exclude` — маска исключения файлов из обхода по регулярному выражению.
*   `--strip-trailing-cr` — отрезать символы `\r` на концах строк (защита от CRLF конфликтов).
*   `-L <МЕТКА>`, `--label` — подменить имя файла в заголовках патча на кастомный текст.
*   `--tabsize <NUM>` — задать кастомную ширину колонок для side-by-side режима (по умолчанию 35).
*   `-I <РЕГЕКС>`, `--ignore-matching-lines` — пропустить блоки кода, если все измененные строки подходят под регулярное выражение.
*   `--speed-large-files` — активация внутренних эвристик для ускорения обработки гигантских файлов.

#### Основной параметр: `patch`
Используется для применения файлов различий (как текстовых, так и сжатых бинарных LZ4) к оригинальным структурам данных.
*   `win-diff-patch patch -i <ПАТЧ_ФАЙЛ> [ОРИГИНАЛ]` — базовое наложение патча.
*   `-B`, `--binary` — включить бинарный режим декомпрессии. Декодирует патч WIN_BIN с помощью потокового LZ4-декодера в памяти и собирает прошивку/исполняемый файл.
*   `-o <ФАЙЛ>`, `--output` — записать результат в отдельный файл, не модифицируя оригинал.
*   `-p <NUM>`, `--strip` — отсечь NUM верхних уровней вложенности папок из путей внутри патча.
*   `-F <NUM>`, `--fuzz` — задать максимальный сдвиг строк (размытие контекста) для поиска совпадений в тексте.
*   `-R`, `--reverse` — инвертировать патч (выполнить чистый откат изменений назад).
*   `-b`, `--backup` — создать резервную копию оригинального файла перед мутацией.
*   `-z <СУФФИКС>`, `--suffix` — задать кастомное расширение бэкапа (по умолчанию `.orig`).
*   `-r <ФАЙЛ>`, `--reject-file` — объединить все недошедшие куски (failed hunks) в один указанный файл отклонений вместо создания локальных `.rej`.
*   `--dry-run` — безопасная симуляция обработки в памяти без физической записи на диск.

#### Основной параметр: `split`
Используется для высокоскоростного многопоточного последовательного сшивания чанков данных.
*   `win-diff-patch split <ЧАНК_1> <ЧАНК_2> ... <ЧАНК_N> -o <ВЫХОДНОЙ_ФАЙЛ>`

#### Дополнительный параметр: `longhelp`
Выводит глубокую детализированную документацию по всем не основным (дополнительным) параметрам и техническим краевым случаям (LCP, LZ4, Fuzzing).
*   `win-diff-patch longhelp`

---

## English

**win-diff-patch** is a fully featured, high-performance "3-in-1" console utility optimized for Windows 10/11 (AMD64) systems. It seamlessly combines the core capabilities of traditional Linux `diff` and `patch` utilities, alongside a specialized multi-threaded `split` module designed for maximum-speed sequential binary chunks merging.

The project is heavily optimized for the AMD64 architecture, featuring CPU-level parallelization via pool-threading (`rayon`) and Memory-Mapped I/O (`memmap2`), maximizing storage controller physical capabilities (NVMe/SSD) without overhead.

### Key Features
*   **Dual Processing Engines**: Supports both traditional line-by-line text comparison (Myers algorithm) and instant byte-by-byte binary analysis (LCP algorithm) for executables and firmware.
*   **Embedded LZ4 Compression**: When operating in binary mode, delta payloads are automatically compressed into an embedded high-performance LZ4 frame wrapper, delivering multi-gigabyte per second decompression speeds per CPU core.
*   **100% Standalone**: The compiled binary statically links the C-Runtime (UCRT) and core GCC/MinGW libraries. No external `.dll` files or active MSYS2 environments are needed on target Windows production machines.
*   **Embedded Metadata**: Application icon and version resource attributes (CompanyName: `buba5473`, legal copyrights, description metadata) are compiled via `windres` and embedded directly into the `.exe` file structure.
*   **Automated Localization**: Interactive CLI help manuals (`--help`) for all subcommands dynamically switch between Russian and English based on the host Windows user account system locale.
*   **Thread-Safe Progress Bar**: Displays an interactive progress indicator during directory traversals. It writes exclusively to `stderr`, completely preventing the pollution of patch files when redirecting output (`>`).

### Directory Structure
```text
win-diff-patch/
├── .cargo/
│   └── config.toml        # Static linking configuration for GCC/MinGW
├── src/
│   ├── main.rs            # Application entry point & core logic
│   ├── longhelp_ru.txt    # Comprehensive Russian documentation manual
│   └── longhelp_en.txt    # Comprehensive English documentation manual
├── build.rs               # Windows metadata resource generation script (buba5473)
├── build.sh               # Automated MSYS2 UCRT64 build script
└── Cargo.toml             # Project manifest, dependencies and release profile
```

### Build Instructions
To build the utility, you will need a pre-installed **MSYS2** environment.

1. Open your **MSYS2 UCRT64** terminal.
2. Navigate to the root directory of the project.
3. Make the build script executable and run it:
   ```bash
   chmod +x build.sh
   ./build.sh
   ```
*The script automatically validates required system packages (`rust`, `gcc`, `windres`), prompts for automated installation via pacman if anything is missing, sets static toolchain parameters, and compiles the optimized `win-diff-patch.exe` file inside `target/x86_64-pc-windows-gnu/release/`.*

### Usage and Command Line Options

#### Core Parameter: `diff`
Compares files line-by-line or directories recursively. By default, it generates standard **Unified Diff** formatting containing `@@` hunk metadata.
*   `win-diff-patch diff <FILE_1> <FILE_2>` — Basic comparison (Unified Diff layout by default).
*   `-B`, `--binary` — Enables high-speed binary mode. Triggers a byte-by-byte LCP analysis on executables or firmware images and packs the delta into an LZ4 container.
*   `-u`, `--unified` — Explicitly requests output in unified context layout.
*   `-c`, `--context` — Generates output in classical GNU context layout.
*   `-y`, `--side-by-side` — Outputs differences side-by-side in two columns.
*   `-e`, `--ed` — Formats the output as a script of commands for the `ed` editor.
*   `-q`, `--brief` — Report only whether files differ (brief mode).
*   `-i`, `--ignore-case` — Ignores case transformations.
*   `-w`, `--ignore-all-space` — Disregards all whitespace changes entirely.
*   `-b`, `--ignore-space-change` — Ignores changes in the amount of white space.
*   `-B`, `--ignore-blank-lines` — Discards modifications adding or removing empty lines.
*   `-r`, `--recursive` — Compares directories recursively using parallel operations across all CPU cores.
*   `-X <REGEX>`, `--exclude` — Excludes files from traversal matching a regular expression pattern.
*   `--strip-trailing-cr` — Trims carriage return (`\r`) characters to resolve CRLF mismatches.
*   `-L <LABEL>`, `--label` — Replaces filename in patch headers with custom text.
*   `--tabsize <NUM>` — Sets custom column width for side-by-side layout rendering (default 35).
*   `-I <REGEX>`, `--ignore-matching-lines` — Ignores hunks where all lines match a specified regex.
*   `--speed-large-files` — Forces internal heuristic engine upgrades to accelerate massive files parsing.

#### Core Parameter: `patch`
Applies difference files (both unified text and compressed LZ4 binary files) to original source structures.
*   `win-diff-patch patch -i <PATCH_FILE> [ORIGINAL]` — Basic patching implementation.
*   `-B`, `--binary` — Switches the applier to binary decompression mode. Parses the WIN_BIN container, triggers the streaming LZ4 frame decoder in memory, and rebuilds the final firmware/executable.
*   `-o <FILE>`, `--output` — Writes output to an alternate file instead of mutating the original.
*   `-p <NUM>`, `--strip` — Strips NUM leading components from path directories inside the patch.
*   `-F <NUM>`, `--fuzz` — Sets max fuzz factor lines to match surrounding shifted textual content.
*   `-R`, `--reverse` — Reverses the patch context (rolls back modifications).
*   `-b`, `--backup` — Creates backups of original files before overwriting.
*   `-z <SUFFIX>`, `--suffix` — Replaces default backup extension (default is `.orig`).
*   `-r <FILE>`, `--reject-file` — Routes all failed hunks into a single unified fallback file.
*   `--dry-run` — Simulates execution in memory without hard drive mutations.

#### Core Parameter: `split`
Fuses split data blocks sequentially using parallel operations.
*   `win-diff-patch split <CHUNK_1> <CHUNK_2> ... <CHUNK_N> -o <OUTPUT_FILE>`

#### Core Parameter: `longhelp`
Displays highly detailed bilingual documentation for all non-core parameters and technical edge cases (LCP, LZ4, Fuzzing details).
*   `win-diff-patch longhelp`
