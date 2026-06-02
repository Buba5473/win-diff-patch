#!/usr/bin/env bash

# Скрипт сборки win-diff-patch для среды MSYS2 UCRT64
# Завершать выполнение при любой ошибке
set -e

# Цвета для вывода в консоль
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # Сброс цвета

echo -e "${GREEN}=== Проверка окружения MSYS2 UCRT64 ===${NC}"

# 1. Проверка, что скрипт запущен именно в UCRT64
if [ "$MSYSTEM" != "UCRT64" ]; then
    echo -e "${RED}[ОШИБКА] Этот скрипт должен быть запущен только в консоли MSYS2 UCRT64!${NC}"
    echo -e "${YELLOW}Текущее окружение: $MSYSTEM. Переоткройте терминал UCRT64.${NC}"
    exit 1
fi

# 2. Список необходимых пакетов MSYS2
REQUIRED_PACKAGES=(
    "mingw-w64-ucrt-x86_64-rust"
    "mingw-w64-ucrt-x86_64-gcc"
    "mingw-w64-ucrt-x86_64-toolchain"
)

MISSING_PACKAGES=()

# Проверяем каждый пакет через pacman
for pkg in "${REQUIRED_PACKAGES[@]}"; do
    if ! pacman -Q "$pkg" &>/dev/null; then
        MISSING_PACKAGES+=("$pkg")
    fi
done

# 3. Если что-то отсутствует, предлагаем установить
if [ ${#MISSING_PACKAGES[@]} -ne 0 ]; then
    echo -e "${YELLOW}[ВНИМАНИЕ] Обнаружены отсутствующие компоненты для сборки:${NC}"
    for pkg in "${MISSING_PACKAGES[@]}"; do
        echo "  - $pkg"
    done
    
    echo ""
    read -p "Хотите установить недостающие компоненты прямо сейчас? (y/n): " confirm
    if [[ "$confirm" =~ ^[Yy]$ ]]; then
        echo -e "${GREEN}[INFO] Запуск установки через pacman...${NC}"
        # Обновляем базы данных и ставим пакеты
        if ! pacman -S --noconfirm "${MISSING_PACKAGES[@]}"; then
            echo -e "${RED}[ОШИБКА] Не удалось установить пакеты. Проверьте права или подключение к сети.${NC}"
            exit 1
        fi
        echo -e "${GREEN}[ОК] Все компоненты успешно установлены.${NC}"
    else
        echo -e "${RED}[ОТМЕНА] Сборка невозможна без необходимых инструментов.${NC}"
        exit 1
    fi
fi

# 4. Автоматическое создание/проверка .cargo/config.toml для статической линковки
echo -e "${GREEN}[INFO] Настройка конфигурации статической сборки...${NC}"
mkdir -p .cargo
cat << 'EOF' > .cargo/config.toml
[target.x86_64-pc-windows-gnu]
rustflags = [
    "-C", "link-args=-static",
    "-C", "link-args=-static-libgcc",
    "-C", "link-args=-static-libstdc++"
]
EOF

# 5. Запуск компиляции с максимальными оптимизациями
echo -e "${GREEN}=== Запуск оптимизированной компиляции релиза ===${NC}"
echo -e "${YELLOW}[TARGET] x86_64-pc-windows-gnu (MSYS2 UCRT64)${NC}"
echo -e "${YELLOW}[CPU OPT] native (Векторизация, AVX2/FMA инструкции процессора)${NC}"

# Переменные окружения для Rust:
# target-cpu=native — задействует все аппаратные фичи текущего процессора AMD64
# +crt-static — вшивает C-Runtime (UCRT) статически
export RUSTFLAGS="-C target-cpu=native -C target-feature=+crt-static"

# Запуск Cargo сборки под целевую платформу GNU/MinGW
cargo build --release --target x86_64-pc-windows-gnu

# 6. Проверка результата и вывод информации
TARGET_BIN="target/x86_64-pc-windows-gnu/release/win-diff-patch.exe"

if [ -f "$TARGET_BIN" ]; then
    echo -e "\n${GREEN}====================================================${NC}"
    echo -e "${GREEN}[УСПЕХ] Сборка проекта полностью завершена!${NC}"
    echo -e "${GREEN}Файл: $TARGET_BIN${NC}"
    
    # Расчет и вывод размера файла
    FILE_SIZE=$(wc -c < "$TARGET_BIN")
    FILE_SIZE_MB=$(echo "scale=2; $FILE_SIZE / 1048576" | bc)
    echo -e "${GREEN}Итоговый размер бинарника: $FILE_SIZE_MB МБ ($FILE_SIZE байт)${NC}"
    echo -e "${YELLOW}[INFO] EXE не имеет внешних зависимостей и готов к работе в чистой Windows 10/11.${NC}"
    echo -e "${GREEN}====================================================${NC}"
else
    echo -e "${RED}[ОШИБКА] Компиляция завершилась, но исполняемый файл не найден.${NC}"
    exit 1
fi
