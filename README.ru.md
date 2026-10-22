# PhotoCraft Plugins & Companion

<p align="center">
  <img src="assets/github-social-preview.svg" alt="PhotoCraft Companion Social Preview" width="100%">
</p>

<div align="center">

[English](README.md) • **[Русский](README.ru.md)**

---

[![CI & Quality Gate](https://github.com/kobaltgit/photocraft-companion/actions/workflows/ci.yml/badge.svg)](https://github.com/kobaltgit/photocraft-companion/actions/workflows/ci.yml)
[![Discord](https://img.shields.io/badge/Discord-Companion_Сервер-5865F2?logo=discord&logoColor=white)](https://discord.gg/2gzn4Un4Bw)
[![Release: v1.2.2](https://img.shields.io/github/v/release/kobaltgit/photocraft-companion?color=blue)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v1.2.2)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust: 2021 Edition](https://img.shields.io/badge/Rust-2021_Edition-orange.svg)](https://www.rust-lang.org/)
[![GUI: egui 0.36](https://img.shields.io/badge/GUI-egui_0.36-blueviolet.svg)](https://github.com/emilk/egui)
[![AI Backend: DirectML GPU](https://img.shields.io/badge/AI_Backend-DirectML_GPU-brightgreen.svg)](https://github.com/microsoft/DirectML)
[![Architecture: Zero-Core](https://img.shields.io/badge/Architecture-Zero--Core-brightgreen.svg)](GEMINI.md)
[![Code Style: rustfmt](https://img.shields.io/badge/Code_Style-rustfmt-blue.svg)](https://github.com/rust-lang/rustfmt)
[![Tests: 159 Passing](https://img.shields.io/badge/Tests-159_Passing-success.svg)](crates/)

**Экосистема плагинов, локальных нейросетей и оверлей-панелей для графического редактора PhotoCraft.**

[Почему Companion?](#-почему-companion-преимущества-перед-встроенной-системой-плагинов-photocraft) • [Возможности](#-основные-возможности) • [Установка и дистрибутивы](#-установка-и-дистрибутивы-installers--portable) • [Первый запуск](#-инструкция-первый-запуск-и-быстрое-подключение) • [Магазин плагинов](#-витрина-и-каталог-плагинов-community-marketplace) • [Быстрый старт](#-быстрый-старт-для-разработчиков) • [Архитектура](#-архитектура-монорепозитория) • [Пакеты .pcpl](#-пакетный-формат-pcpl) • [Документация](#-документация-проекта) • [Discord](#-сообщество-и-discord) • [Лицензия](#-лицензия)


</div>

> [!TIP]
> 💬 **Присоединяйтесь к Discord-серверу PhotoCraft Companion:** Есть вопросы по плагинам, идеи новых инструментов или нужна помощь по разработке? **[Вступайте в наш Discord-сервер](https://discord.gg/2gzn4Un4Bw)** для живого общения с автором и разработчиками расширений!

---

## 💡 О проекте

**PhotoCraft Plugins & Companion** решает задачу расширения графического редактора **PhotoCraft** (open-source reimplementation of Photoshop на Rust):

- **Бесшовное исполнение (In-Place Execution):** все операции (вырезание фона нейросетью, фильтры, макросы) выполняются прямо на активном слое в открытом холсте редактора через протокол JSON-RPC (порт 18492) и системный буфер обмена без ручного сохранения/открытия файлов.
- **В обход Wasm-песочницы:** редактор PhotoCraft запускает плагины в изолированной песочнице без прямого доступа к локальному GPU и диску. Компаньон предоставляет нативный доступ к DirectML и ONNX Runtime.

---

## ⚖️ Почему Companion? Преимущества перед встроенной системой плагинов PhotoCraft

В редактор PhotoCraft заложен собственный внутренний механизм расширений на базе WebAssembly (Wasm). Несмотря на безопасность песочницы, строгая изоляция Wasm делает невозможными тяжелые нейросетевые вычисления и современные плавающие интерфейсы.

**PhotoCraft Companion** выводит возможности редактора на принципиально иной уровень:

| Критерий | Встроенные Wasm-плагины PhotoCraft | PhotoCraft Companion & Экосистема |
| :--- | :--- | :--- |
| **Локальный ИИ и нейросети** | ❌ **Нет доступа к GPU** (изолированы внутри Wasm; невозможно задействовать тензорные ядра DirectML / Metal) | ✅ **Полное DirectML GPU ускорение** (DirectX 12 / Metal через ONNX Runtime; инференс за ~2 секунды) |
| **Хранение моделей и автономность** | ❌ Лимит виртуальной памяти песочницы; невозможно упаковать веса в сотни мегабайт | ✅ **100% автономия:** Модели лежат в `plugins/<id>/models/`, автопроверка SHA-256 и загрузка в 1 клик |
| **Создание No-Code макросов** | ❌ Требуется компилятор Rust/C и сборка в `.wasm` через wasm-pack | ✅ **Zero-Code JSON:** Достаточно перечислить нужные команды PhotoCraft в `plugin.json` — готово за 1 минуту |
| **Плавающая панель (Toolstrip)** | ❌ Заперты в статичном верхнем меню (`Фильтры › Плагины`); нет своего окна | ✅ **Двухъярусный Toolstrip:** Компактная палитра на `egui` поверх холста с переключением ориентации и треем |
| **Встроенный магазин плагинов** | ❌ Ручное копирование файлов по системным каталогам | ✅ **1-Click Marketplace:** Каталог онлайн-плагинов, автозагрузка и горячая перезагрузка прямо из UI |
| **In-Place пиксельный конвейер** | ⚠️ Привязаны к внутренним буферам Wasm ABI | ✅ **Субпиксельный In-Place конвейер:** Прямой высокоскоростной обмен пикселями через JSON-RPC и память буфера |

---

### ✨ Основные возможности

- ✂️ **Локальные нейросети и компьютерное зрение (Zero-Core Architecture):**
  - **AI Remove Background:** Модель Bria AI **RMBG-1.4** (~168 МБ) для фотореалистичного отделения фона с сохранением геометрии вплоть до 8K.
  - **AI Generative Fill (LaMa):** Контекстно-зависимая генеративная заливка без промптов и удаление объектов по маске лассо.
  - **AI Super-Resolution (2x):** Real-ESRGAN Compact с бесшовным тайлингом и сшиванием швов для холстов любого масштаба без переполнения VRAM.
  - **Отказоустойчивый рантайм:** Аппаратное ускорение **DirectML (DirectX 12 GPU)** с автопереключением на CPU при сбоях.
  - **Сверхбыстрый тайлинг с кэшированием сессий:** Сессия GPU DirectML инициализируется 1 раз на весь холст, ускоряя обработку больших изображений до 10–25 раз.
  - **Стандарт Zero-Core:** Ноль захардкоженного кода под конкретные плагины в ядре — все рабочие процессы декларируются в `plugin.json`.

- 📐 **Суб-инструменты в стиле Photoshop (Sub-Actions & Flyouts):**
  - Плагины могут декларировать вариации действий (например, вырезка на новый слой vs неразрушающая маска слоя).
  - На кнопке инструмента рисуется векторный шеврон; клик по шеврону или долгое удержание (>350 мс) открывает отдельное плавающее меню (`egui::ViewportBuilder`) с запоминанием выбора в конфиге.

- ⚡ **Декларативный движок Workflow Engine и контекстные переменные:**
  - Цепочки команд с подстановкой переменных (`$source_layer`, `$result_layer`, `$active_layer`, `$doc_width`, `$doc_height`, `$has_selection`, `$fg_color`, `$bg_color`, интерполяция `{{...}}`).
  - Условное ветвление (`when: "$has_selection == true"`), сохранение слоев (`save_as`), обработка ошибок (`ignore_errors`) и встроенные скрипты Rhai.

- 🔄 **Двусторонняя синхронизация цветов с PhotoCraft (Session Colors) и Echo Guard:**
  - Мгновенное обновление цветов при работе пипеткой (`I`), сбросе (`D`) или смене мест (`X`).
  - Zero-waste опрос (180 мс только при открытых окнах) и подавление обратного эха (`echo_guard`).

- 🎛️ **Интерактивные окна Multi-Viewport:**
  - Отдельные плавающие окна инструментов на базе декларативных виджетов (слайдеры, плашки цвета, цветовой круг, WCAG-диагностика, чекбоксы) с реактивной логикой на Rhai.

- 🏪 **Встроенный магазин плагинов (In-App Marketplace):**
  - Интегрированная 4-я вкладка «Магазин» в Менеджере плагинов для поиска и установки расширений сообщества из GitHub Releases.
  - Установка легковесных пакетов `.pcpl` (< 1 КБ) в один клик с мгновенным появлением кнопок на тулбаре без перезапуска.
  - Векторные авторские SVG-иконки с автоподгонкой контраста под темную и светлую темы.
  - Локальное дисковое кэширование каталога для стабильного оффлайн-доступа.

- 🎛️ **Плавающая панель инструментов (Two-Tier Floating Toolstrip):**
  - Компактная двухъярусная палитра на `egui` / `eframe` с родной эстетикой PhotoCraft и векторными SVG-иконками.
  - Переключение между горизонтальной (высота 78 px) и вертикальной (ширина 50 px) ориентацией без обрезания кнопок при любом числе плагинов $N$.
  - Не засоряет панель задач ОС (`.with_taskbar(false)`), запоминает координаты на экране при перетаскивании.
  - Плавающий мини-HUD со статусом операции, живым таймером и блокировкой от повторных кликов.

- ⚙️ **Менеджер плагинов (Plugin Manager):**
  - Горизонтальные вкладки: `[ Общие ]`, `[ Плагины ]`, `[ Магазин ]`, `[ О программе ]`.
  - Трехпозиционный индикатор связи с PhotoCraft (`● Подключено` / `● Требуется токен` / `○ Офлайн`) с диалогом выбора файла токена (`rfd`).
  - Темы оформления: переключение между тёмной и чистой светлой темами.
  - Встроенный автоапдейтер программы: проверка релизов на GitHub, сравнение SemVer и скачивание обновления в 1 клик.

- 📦 **100% автономность и формат пакетов `.pcpl`:**
  - Все веса нейросетей хранятся локально в папке плагина (`plugins/<id>/models/`).
  - Прогресс-бар скачивания моделей с отображением текущей скорости (МБ/с), процентов и мегабайт.

- ⏳ **Режим ожидания редактора (Offline Mode):**
  - Компаньон не падает, если PhotoCraft закрыт, а выводит графитовое окно ожидания.
  - Фоновый опрос сокета каждые 1.5 секунды: как только редактор запускается, тулбар бесшовно появляется на холсте.

- 🔔 **Реактивный системный трей (0 ms):**
  - Иконка в системном трее Windows, Linux и macOS с мгновенным откликом без задержек и темной темой контекстного меню.

---

## 📦 Установка и дистрибутивы (Installers & Portable)

Готовые бинарные сборки доступны для всех популярных платформ в виде **установщиков** и **портативных архивов**:

| Платформа                         | Установщик (Setup)                            | Портативная версия (Portable)                              | Примечания                                                         |
| :-------------------------------- | :-------------------------------------------- | :--------------------------------------------------------- | :----------------------------------------------------------------- |
| **Windows (x64)**                 | `PhotoCraft-Companion-Setup-Windows-x64.exe`  | `PhotoCraft-Companion-v1.2.2-windows-x64-portable.zip`     | Inno Setup с автозапуском в трее / Чистый архив без записи в реестр |
| **macOS (Apple Silicon & Intel)** | `PhotoCraft-Companion-macos.dmg`              | `PhotoCraft-Companion-v1.2.2-macos-portable.tar.gz`        | DMG-образ для установки в `/Applications` / Автономный бандл `.app` |
| **Linux (x86_64)**                | `PhotoCraft-Companion-v1.2.2-linux-amd64.deb` | `PhotoCraft-Companion-v1.2.2-linux-x86_64-portable.tar.gz` | Debian/Ubuntu пакет с ярлыком меню / Автономный tar.gz             |

> [!TIP]
> Все релизы и пакеты публикуются на странице **[GitHub Releases](https://github.com/kobaltgit/photocraft-companion/releases)**.

> [!NOTE]
> **Для пользователей macOS:** Если при запуске система сообщает, что *"Программа повреждена"* (политика Gatekeeper для неподписанного открытого ПО), откройте Терминал и снимите атрибут карантина:
> ```bash
> xattr -cr "/Applications/PhotoCraft Companion.app"
> ```
> Или перейдите в **Системные настройки › Защита и безопасность** и нажмите **Подтвердить вход**.

---

## 🚦 Инструкция: Первый запуск и быстрое подключение

Связь между PhotoCraft Companion и редактором PhotoCraft осуществляется через локальный сокет управления (порт 18492) с авторизацией по токену (`control.token`). Начиная с версии **v1.0.7**, запуск и подключение полностью автоматизированы в 1 клик!

### Вариант 1: Запуск PhotoCraft в 1 клик из Компаньона (Рекомендуемый)
1. **Запустите PhotoCraft Companion**:
   - Если PhotoCraft еще не запущен, Компаньон откроет компактное окно ожидания: *«Ожидание запуска PhotoCraft...»*.
2. **Запустите редактор кнопкой**:
   - Нажмите кнопку **`[ > Запустить PhotoCraft ]`** прямо в окне ожидания (или в меню **Настройки › Общие**).
   - Компаньон сам найдет `photocraft.exe` по стандартным путям (или предложит выбрать файл через проводник `[ Обзор... ]`), сгенерирует криптографически стойкий 64-символьный токен `control.token`, запустит редактор с ключами `--control 18492 --control-token-file` и мгновенно установит связь.
   - Выбранный путь надежно сохраняется в конфигурации пользователя.
3. **Готово к работе**:
   - Окно ожидания плавно сменится компактным плавающим тулбаром поверх вашего рабочего холста.

### Вариант 2: Ручной запуск PhotoCraft
Если вы предпочитаете запускать PhotoCraft собственным ярлыком или из консоли:
1. Запустите PhotoCraft с аргументами сокета управления:
   ```powershell
   photocraft.exe --control 18492 --control-token-file "PhotoCraftData/control.token"
   ```
2. Запустите `photocraft-companion.exe`. Он автоматически обнаружит запущенный редактор, прочитает `control.token`, и индикатор сменится на **● Подключено**.

---

### Сборка релизных пакетов локально

- **Для Windows:**
  ```powershell
  .\scripts\package_release.ps1
  ```
  *(Соберет портативный архив в `dist/` и установщик Inno Setup при наличии ISCC).*

- **Для Linux / macOS:**
  ```bash
  chmod +x ./scripts/package_release.sh
  ./scripts/package_release.sh
  ```

---

## 🚀 Быстрый старт (для разработчиков)

### Требования к окружению

- **ОС:** Windows 10 / 11 (x64), Linux (X11 / Wayland) или macOS
- **Rust:** Тулчейн 1.80+ (редакция 2021)
- **Видеокарта (опционально):** Любой GPU с поддержкой DirectX 12 (NVIDIA, AMD, Intel) для DirectML.

### 1. Клонирование репозитория

```bash
git clone https://github.com/kobaltgit/photocraft-companion.git
cd photocraft-companion
```

### 2. Загрузка моделей нейросетей

Веса моделей исключены из Git-репозитория (`.gitignore`) для легковесности. Запустите скрипт загрузки модели RMBG-1.4:

```powershell
.\scripts\fetch_models.ps1
```

*(Или нажмите кнопку `[ ⬇ Скачать модель ]` в карточке плагина в окне Менеджера при первом запуске).*

### 3. Сборка и запуск

```bash
# Запуск приложения компаньона
cargo run --package companion_app
```

### 4. Контроль качества и запуск тестов

Для гарантии надежности и защиты архитектуры Zero-Core от хардкода используйте автоматические барьеры качества:

> [!NOTE]
> **Правило Resource Guard:** Все команды `cargo` запускаются строго с флагом `-j 1` для предотвращения зависания ПК разработчика из-за перегрузки CPU и RAM.

```bash
# 1. Проверка стиля кода (нулевая толерантность к расхождениям)
cargo fmt --all -- --check

# 2. Полный прогон тестов воркспейса (152 модульных и интеграционных тестов)
cargo test --workspace -j 1

# 3. Линтер архитектурной чистоты (запрет хардкода плагинов и keyword-sniffing)
cargo test -p ai_engine --test architecture_integrity_test -j 1

# 4. Линтер регламентов разработки (синхронизация версий в 5 файлах, Resource Guard, хроника)
cargo test -p ai_engine --test dev_regulations_lint_test -j 1

# 5. Линтеры интерфейса (чистота локализации и запрет невоспроизводимых эмодзи)
cargo test -p overlay_ui --test i18n_lint_test -j 1
cargo test -p overlay_ui --test vector_first_lint_test -j 1

# 6. Валидатор плагинов и генератор реестра (проверка манифестов, иконок SVG и ссылок на веса)
cargo run -p plugin_validator -j 1 -- --validate plugins
```

---

## 🏗 Архитектура монорепозитория

Проект организован по принципу независимых пакетов внутри единого Cargo Workspace:

```text
photocraft-plugins/
├── Cargo.toml                      # Корневой манифест Cargo Workspace
├── crates/                         # Независимые крейты
│   ├── core_contracts/            # Трейты плагинов, DTO, конфиг, i18n, установщик .pcpl
│   ├── photocraft_bridge/         # Асинхронный клиент к сокету PhotoCraft (JSON-RPC 18492)
│   ├── ai_engine/                 # DirectML GPU ONNX рантайм (RMBG, Real-ESRGAN, LaMa)
│   ├── overlay_ui/                # Плавающий Toolstrip, встроенный магазин, менеджер плагинов (egui 0.36)
│   ├── plugin_validator/          # CLI валидации плагинов, упаковки и генерации реестра
│   └── companion_app/             # Главный бинарник (трей, жизненный цикл, DI, workflow engine)
├── plugins/                       # Каталог установленных плагинов
│   ├── black_and_white/          # Декларативный No-Code макрос
│   ├── chromasense/              # Интерактивная панель доступности цвета (Rhai, WCAG 2.1, CVD)
│   ├── depth_map/                 # Автономный плагин 3D карты глубины (Depth Anything V2)
│   ├── inpaint/                   # Автономный плагин умного заполнения (LaMa)
│   ├── remove_bg/                 # Автономный плагин удаления фона (RMBG-1.4, Sub-Actions)
│   ├── upscale/                   # Автономный плагин 2x супер-разрешения (Real-ESRGAN)
│   └── vectorize/                 # Автономный плагин умной трассировки в вектор SVG
├── scripts/
│   ├── fetch_models.ps1           # PowerShell скрипт скачивания весов моделей
│   ├── package_release.ps1        # Скрипт сборки инсталлятора Inno Setup и zip
│   └── package_release.sh         # Скрипт упаковки DEB для Linux и DMG для macOS
└── docs/                          # Инженерная документация и летописи разработки
```

---

## 📦 Пакетный формат `.pcpl`

Любой плагин может распространяться в виде портативного архива `.pcpl` (формат ZIP):

```text
my_plugin.pcpl
├── plugin.json              # Декларативный манифест (метаданные, команды, хоткеи)
├── icon.svg                 # Векторная иконка для панели
└── models/                  # (Опционально) Локальные веса моделей ONNX
    └── rmbg-1.4.onnx
```

Установка выполняется в один клик через кнопку `[ + Установить плагин (.pcpl) ]` в окне Менеджера плагинов.

---

## 🏪 Витрина и каталог плагинов (Plugin Marketplace)

Готовые пакеты `.pcpl` можно скачать в 1 клик и сразу установить через **Менеджер плагинов** Компаньона:

<!-- PLUGINS_CATALOG_START -->

| Иконка | Название и плагин | Тип | Версия | Автор | Пакет (.pcpl) |
| :---: | :--- | :---: | :---: | :--- | :---: |
| <img src="plugins/black_and_white/icon.svg" width="28" height="28"> | **B&W Film Pop (No-Code)**<br>_Duplicates active layer, converts to grayscale with auto-contrast adjustment._ | `⚡ Macro` | `1.0.0` | kobaltGIT | [📥 Скачать .pcpl](https://github.com/kobaltgit/photocraft-companion/releases/download/plugins/black_and_white.pcpl) |
| <img src="plugins/chromasense/icon.svg" width="28" height="28"> | **ChromaSense**<br>_Accessibility Color Assistant for artists with color vision deficiencies_ | `🎨 UI Panel` | `1.0.0` | kobaltGIT | [📥 Скачать .pcpl](https://github.com/kobaltgit/photocraft-companion/releases/download/plugins/chromasense.pcpl) |
| <img src="plugins/depth_map/icon.svg" width="28" height="28"> | **AI Depth Map (Depth Anything V2)**<br>_State-of-the-art monocular 3D depth estimation for Lens Blur, bokeh and atmospheric lighting via Depth Anything V2._ | `🤖 AI (ONNX)` | `1.0.0` | kobaltGIT | [📥 Скачать .pcpl](https://github.com/kobaltgit/photocraft-companion/releases/download/plugins/depth_map.pcpl) |
| <img src="plugins/frequency_separation/icon.svg" width="28" height="28"> | **Frequency Separation (Retouch)**<br>_Splits portrait into high-frequency texture and low-frequency color layers for professional beauty retouching._ | `⚡ Macro` | `1.0.0` | kobaltGIT | [📥 Скачать .pcpl](https://github.com/kobaltgit/photocraft-companion/releases/download/plugins/frequency_separation.pcpl) |
| <img src="plugins/inpaint/icon.svg" width="28" height="28"> | **AI Generative Fill (LaMa)**<br>_Zero-prompt neural generative fill and object eraser for PhotoCraft using LaMa model via DirectML GPU._ | `🤖 AI (ONNX)` | `1.0.0` | kobaltGIT | [📥 Скачать .pcpl](https://github.com/kobaltgit/photocraft-companion/releases/download/plugins/inpaint.pcpl) |
| <img src="plugins/remove_bg/icon.svg" width="28" height="28"> | **AI Remove Background (RMBG-1.4)**<br>_High-fidelity neural background removal for PhotoCraft using Bria AI RMBG-1.4 model via local ONNX Runtime._ | `🤖 AI (ONNX)` | `1.1.0` | kobaltGIT | [📥 Скачать .pcpl](https://github.com/kobaltgit/photocraft-companion/releases/download/plugins/remove_bg.pcpl) |
| <img src="plugins/upscale/icon.svg" width="28" height="28"> | **AI Super-Resolution (2x)**<br>_2x neural super-resolution upscale for PhotoCraft active layer via local ONNX Runtime & DirectML GPU._ | `🤖 AI (ONNX)` | `1.0.0` | kobaltGIT | [📥 Скачать .pcpl](https://github.com/kobaltgit/photocraft-companion/releases/download/plugins/upscale.pcpl) |
| <img src="plugins/vectorize/icon.svg" width="28" height="28"> | **Smart Vectorizer / SVG Trace**<br>_High-fidelity vectorization and SVG tracing of raster layers with Bezier curve fitting._ | `📐 Algorithm` | `1.0.0` | kobaltGIT | [📥 Скачать .pcpl](https://github.com/kobaltgit/photocraft-companion/releases/download/plugins/vectorize.pcpl) |

<!-- PLUGINS_CATALOG_END -->

> [!TIP]
> **Хотите опубликовать свой плагин?** Изучите [Руководство разработчика плагинов](docs/PLUGIN_DEVELOPMENT_GUIDE.md), создайте директорию в `plugins/<id>/` и отправьте Pull Request! Наш автоматический CI проверит манифест, векторную иконку и веса моделей перед публикацией.

---

## 📚 Документация проекта

Подробные инженерные спецификации и журнал разработки находятся в папке [`docs/`](docs/):

- 📘 [`GEMINI.md`](GEMINI.md) — Главный архитектурный манифест проекта.
- 📋 [`docs/AGENTS_MASTER_GUIDE.md`](docs/AGENTS_MASTER_GUIDE.md) — Генеральный регламент разработки и правила для субагентов.
- 🎯 [`docs/PLAN.md`](docs/PLAN.md) — План реализации плавающей панели и автономных плагинов.
- 🗺️ [`docs/ROADMAP.md`](docs/ROADMAP.md) — Дорожная карта этапов разработки.
- ✅ [`docs/CHECKLIST.md`](docs/CHECKLIST.md) — Интерактивный чек-лист статуса всех фаз.
- 🐛 [`docs/BUGS_AND_ISSUES.md`](docs/BUGS_AND_ISSUES.md) — Журнал решенных дефектов (BUG-001 – BUG-075).
- 📜 [`docs/CHRONICLE.md`](docs/CHRONICLE.md) / [`docs/CHRONICLE.en.md`](docs/CHRONICLE.en.md) — Авторская летопись разработки.
- 🛠️ [`docs/PLUGIN_DEVELOPMENT_GUIDE.md`](docs/PLUGIN_DEVELOPMENT_GUIDE.md) / [`EN`](docs/PLUGIN_DEVELOPMENT_GUIDE.en.md) — Руководство разработчика плагинов (No-Code макросы, нейросети, создание расширений).
- 📖 [`docs/PHOTOCRAFT_COMMANDS_REFERENCE.md`](docs/PHOTOCRAFT_COMMANDS_REFERENCE.md) / [`EN`](docs/PHOTOCRAFT_COMMANDS_REFERENCE.en.md) — Полный каталог всех **851** команд ядра PhotoCraft.
- 📐 [`docs/UNIVERSAL_PLUGIN_CORE_SPECIFICATION.md`](docs/UNIVERSAL_PLUGIN_CORE_SPECIFICATION.md) — Спецификация универсального ядра (Rhai, CLI, C-ABI DynLib).
- 🚀 [`docs/FIRST_RUN_GUIDE.md`](docs/FIRST_RUN_GUIDE.md) — Руководство по установке и первому запуску (Windows, macOS, Linux).
- 🤝 [`CONTRIBUTING.md`](CONTRIBUTING.md) — Руководство по участию в разработке, ключевые правила и чек-лист PR.

---

## 💬 Сообщество и Discord

Есть вопросы, идеи новых плагинов или хотите поделиться своими разработками?
Присоединяйтесь к официальному Discord-серверу **PhotoCraft Companion**:

<div align="center">

[![Присоединиться к Discord](https://img.shields.io/badge/Discord-PhotoCraft%20Companion-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.gg/2gzn4Un4Bw)

**👉 [https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)**

</div>

---

## 👤 Автор

**kobaltGIT** ([GitHub](https://github.com/kobaltgit))

---

## 📄 Лицензия

Проект распространяется под открытой лицензией [MIT License](LICENSE).