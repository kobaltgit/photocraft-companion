# PhotoCraft Plugins & Companion

<div align="center">

[![CI & Quality Gate](https://github.com/kobaltgit/photocraft-plugins/actions/workflows/ci.yml/badge.svg)](https://github.com/kobaltgit/photocraft-plugins/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust: 2021 Edition](https://img.shields.io/badge/Rust-2021_Edition-orange.svg)](https://www.rust-lang.org/)
[![GUI: egui 0.36](https://img.shields.io/badge/GUI-egui_0.36-blueviolet.svg)](https://github.com/emilk/egui)
[![AI Backend: DirectML GPU](https://img.shields.io/badge/AI_Backend-DirectML_GPU-brightgreen.svg)](https://github.com/microsoft/DirectML)
[![Tests: 62 Passing](https://img.shields.io/badge/Tests-62_Passing-success.svg)](crates/)

**Мощная экосистема плагинов, локальных нейросетей и интерактивных оверлей-панелей для графического редактора PhotoCraft.**

[Возможности](#-основные-возможности) • [Быстрый старт](#-быстрый-старт) • [Архитектура](#-архитектура-монорепозитория) • [Пакеты .pcpl](#-пакетный-формат-pcpl) • [Документация](#-документация-проекта) • [Лицензия](#-лицензия)

</div>

---

## 💡 О проекте

**PhotoCraft Plugins & Companion** решает ключевую задачу расширения графического редактора **PhotoCraft** (open-source reimplementation of Photoshop на Rust):
- **Бесшовное исполнение (In-Place Execution):** все операции (вырезание фона нейросетью, фильтры, макросы) выполняются прямо на активном слое в открытом холсте редактора через протокол JSON-RPC (порт 18492) и буфер обмена Windows без ручного сохранения/открытия файлов.
- **В обход Wasm-песочницы:** редактор PhotoCraft запускает плагины в изолированной песочнице без прямого доступа к локальному GPU и диску. Компаньон снимает это ограничение, предоставляя нативную мощь DirectML и ONNX Runtime.

---

## ✨ Основные возможности

* ✂️ **Локальное нейросетевое удаление фона (AI In-Place Remove BG):**
  * Встроенная модель Bria AI **RMBG-1.4** (~168 МБ).
  * Аппаратное ускорение **DirectML (DirectX 12 GPU)** для видеокарт NVIDIA, AMD и Intel с безопасным откатом на CPU.
  * Сохранение геометрии на холстах любого разрешения (проверено вплоть до 8K: 8256×4608).
  * Интеллектуальное именование: автоматическое добавление мультиязычного суффикса `(No BG)` / `(Без фона)` без дублирования.

* 🎛️ **Плавающая панель инструментов (Two-Tier Floating Toolstrip):**
  * Компактная двухъярусная палитра на `egui` / `eframe` с родной эстетикой PhotoCraft (радиус скругления 2.0 px).
  * Автопозиционирование над холстом PhotoCraft справа через Win32 API (`EnumWindows`, `GetWindowRect`).
  * Переключение между горизонтальной (высота 78 px) и вертикальной (ширина 50 px) ориентацией без обрезания кнопок при любом числе плагинов $N$.
  * Парящий режим: панель не засоряет Панель задач Windows (`.with_taskbar(false)`).
  * Всплывающие независимые тултипы поверх рабочего стола.

* ⚙️ **Менеджер плагинов в стиле *PeekIt Preferences*:**
  * Верхние горизонтальные вкладки: `[ General ]`, `[ Plugins ]`, `[ Hotkeys ]`, `[ About ]`.
  * Индикатор связи: пульсирующий статус сокета PhotoCraft (`● Connected` / `○ Offline`).
  * Мультиязычность (i18n): поддержка всех 11 языков редактора с авто-определением из настроек.
  * Темы оформления: идеальная синхронизация тёмной и чистой светлой тем.
  * Интерактивный рекордер горячих клавиш: захват сложных комбинаций (`Ctrl+Shift+...`) с защитой от залипания модификаторов.

* 📦 **100% автономность и формат пакетов `.pcpl`:**
  * Все веса нейросетей хранятся локально в папке плагина (`plugins/<id>/models/`).
  * Установка расширений в один клик из файлов `.pcpl` (PhotoCraft Plugin Package) через стандартный диалог проводника.
  * Автопроверка целостности моделей и скачивание отсутствующих весов прямо из GUI.

* ⏳ **Режим ожидания редактора (Offline Mode):**
  * Компаньон не завершает работу с ошибкой, если PhotoCraft закрыт, а выводит графитовое окно ожидания.
  * Фоновый опрос сокета каждые 1.5 секунды: как только редактор запускается, тулбар бесшовно появляется на холсте.

* 🔔 **Реактивный системный трей (0 ms):**
  * Фоновое прослушивание хоткеев и управление окнами через иконку в трее Windows.
  * Мгновенный отклик меню без задержек и нативная тёмная тема контекстного меню Windows.

---

## 🚀 Быстрый старт

### Требования
* **ОС:** Windows 10 / 11 (x64)
* **Rust:** Toolchain 1.80+ (версия 2021)
* **GPU (опционально):** Любая видеокарта с поддержкой DirectX 12 (NVIDIA RTX / AMD Radeon / Intel Arc) для DirectML-ускорения.

### 1. Клонирование репозитория
```bash
git clone https://github.com/kobaltgit/photocraft-plugins.git
cd photocraft-plugins
```

### 2. Загрузка весов нейросетей (для разработчиков)
Модели нейросетей исключены из репозитория (`.gitignore`) для сохранения чистоты и легковесности Git. Для загрузки весов RMBG-1.4 выполните скрипт:
```powershell
.\scripts\fetch_models.ps1
```
*(Либо веса можно скачать в один клик через кнопку `[ ⬇ Скачать модель ]` в Менеджере плагинов при первом запуске).*

### 3. Сборка и запуск
```bash
# Запуск тестов воркспейса (62 теста)
cargo test --workspace

# Запуск приложения компаньона
cargo run --package companion_app
```

---

## 🏗 Архитектура монорепозитория

Проект организован как модульный Cargo Workspace с чистым разделением обязанностей:

```text
photocraft-plugins/
├── Cargo.toml                      # Корневой манифест Rust Cargo Workspace
├── crates/                         # Независимые пакеты
│   ├── core_contracts/            # Трейты Plugin, DTO, Config, i18n, пакетный установщик .pcpl
│   ├── photocraft_bridge/         # Асинхронный клиент сокета PhotoCraft (JSON-RPC 18492)
│   ├── ai_engine/                 # DirectML GPU инференс (ort, ONNX Runtime, RMBG-1.4)
│   ├── overlay_ui/                # Плавающий тулбар, Менеджер плагинов и оверлеи (egui 0.36)
│   ├── hotkey_service/            # Глобальные горячие клавиши ОС (global-hotkey)
│   └── companion_app/             # Главный бинарник (системный трей, DI, жизненный цикл)
├── plugins/                       # Каталог установленных плагинов
│   ├── remove_bg/                 # Автономный плагин нейросетевого удаления фона
│   └── black_and_white/          # Декларативный No-Code макрос
├── scripts/
│   └── fetch_models.ps1           # Скрипт подгрузки нейросетевых моделей
└── docs/                          # Центр документации и спецификаций
```

---

## 📦 Пакетный формат `.pcpl`

Любой плагин может быть упакован в переносимый ZIP-контейнер `.pcpl`:
```text
my_plugin.pcpl
├── plugin.json              # Декларативный манифест
├── icon.svg                 # Векторная иконка для панели
└── models/                  # (опционально) Локальные веса ONNX
    └── rmbg-1.4.onnx
```
Установка выполняется в один клик через кнопку `[ + Установить плагин (.pcpl) ]` в окне Менеджера плагинов.

---

## 📚 Документация проекта

Подробные инженерные спецификации и журнал разработки находятся в папке [`docs/`](docs/):
- 📘 [`GEMINI.md`](GEMINI.md) — Главный архитектурный манифест проекта.
- 🎯 [`docs/PLAN.md`](docs/PLAN.md) — План реализации плавающей панели и автономных плагинов.
- 🗺️ [`docs/ROADMAP.md`](docs/ROADMAP.md) — Дорожная карта этапов разработки.
- ✅ [`docs/CHECKLIST.md`](docs/CHECKLIST.md) — Интерактивный чек-лист статуса всех 16 фаз.
- 🐛 [`docs/BUGS_AND_ISSUES.md`](docs/BUGS_AND_ISSUES.md) — Журнал решенных дефектов (BUG-001 – BUG-026).
- 📜 [`docs/CHRONICLE.md`](docs/CHRONICLE.md) / [`docs/CHRONICLE.en.md`](docs/CHRONICLE.en.md) — Авторская летопись разработки.
- 📐 [`docs/UNIVERSAL_PLUGIN_CORE_SPECIFICATION.md`](docs/UNIVERSAL_PLUGIN_CORE_SPECIFICATION.md) — Спецификация универсального ядра (Rhai, CLI, C-ABI DynLib).

---

## 👤 Автор

**kobaltGIT** ([GitHub](https://github.com/kobaltgit))

---

## 📄 Лицензия

Проект распространяется под открытой лицензией [MIT License](LICENSE).
