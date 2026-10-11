# Банк идей и бэклог (Ideas & Backlog)

## 1. Реестр архитектурных и продуктовых идей

| ID | Фича / Концепция | Описание и ценность | Приоритет | Статус |
|---|---|---|---|---|
| IDEA-001 | In-Place Remove BG | Бесшовное удаление фона активного слоя без открытия файлов через буфер обмена | P0 | DONE |
| IDEA-002 | Smart Layer Naming | Сохранение пользовательского имени слоя и добавление суффикса `(No BG)` / `(Без фона)` с защитой от дублирования | P0 | ACCEPTED |
| IDEA-003 | Multi-Language on the Fly (i18n) | Синхронизация с языком PhotoCraft (`auto`, `en`, `ru`, `de`, `fr`, `es`, `it`, `ja`, `zh`, `ko`, `pt`) из `preferences.json` | P0 | ACCEPTED |
| IDEA-004 | Universal Hardware Auto-Select | DirectML (DirectX 12) для дискретных и встроенных видеокарт (NVIDIA/AMD/Intel) с безопасным откатом на CPU | P0 | ACCEPTED |
| IDEA-005 | Plugin Manager GUI | Графическое окно управления плагинами: тумблеры вкл/выкл, настройка хоткеев, порядок кнопок, статус связи | P1 | ACCEPTED |
| IDEA-006 | Open Plugin SDK (`photocraft_plugin_sdk`) | Публичный трейт `Plugin` и контекст `PluginContext` (доступ к 851 команде PhotoCraft, маскам, пикселям, UI и ИИ) | P1 | ACCEPTED |
| IDEA-007 | No-Code Declarative Plugins | Поддержка простых плагинов через `plugins/*/plugin.json` для вызова внешних утилит, скриптов и макросов | P1 | ACCEPTED |
| IDEA-008 | Docked Toolbar (Overlay UI) | Плашка кнопок на `egui` / `eframe` в тёмной теме PhotoCraft (`#1e1e1e`), прилипающая к окну редактора | P1 | IN_PROGRESS |
| IDEA-009 | Hotkey Dispatcher & Safe Resolver | Глобальные и контекстные хоткеи с защитой от конфликтов со встроенными 851 командами PhotoCraft | P1 | ACCEPTED |
| IDEA-010 | Connection & Token Settings | Поле ручного ввода токена и порта в GUI + автообнаружение из `control.token`, хранение в `companion_config.json` | P1 | ACCEPTED |
| IDEA-011 | Auto-Hide Source Layer | Автоматическое скрытие исходного слоя после вырезания для мгновенного просмотра прозрачного фона | P2 | ACCEPTED |
| IDEA-012 | Smart Upscale (Real-ESRGAN) | Нейросетевое увеличение разрешения слоя в 2x/4x с сохранением текстур | P2 | BACKLOG |
| IDEA-013 | Quick Inpainting Eraser | Удаление нежелательных объектов по маске выделения пользователя | P2 | BACKLOG |
| IDEA-014 | Plugin Marketplace / Catalog | Загрузка и обновление плагинов сообщества из GitHub прямо через интерфейс менеджера | P3 | IDEA |
| IDEA-015 | Package Format `.pcpl` | Стандартный формат пакета плагинов (zip-архив с манифестом, иконками и моделями) и установка в 1 клик | P0 | ACCEPTED |
| IDEA-016 | Window Tracker & Auto-Snap | Автоопределение координат окна PhotoCraft и привязка плавающей панели в верхнюю зону над холстом | P0 | DONE |
| IDEA-017 | Embedded Script Engine (Rhai) | Встроенный безопасный скриптинг на чистом Rust для легковесной автоматизации без компиляции (`plugin.rhai`) | P1 | SPECIFIED |
| IDEA-018 | External CLI Runner | Запуск внешних консольных утилит (Python, ImageMagick, Go, C++) с передачей параметров документа и слоев | P1 | SPECIFIED |
| IDEA-019 | Dynamic C-ABI DynLib Runner | Поддержка сторонних динамических библиотек (`.dll` / `.so` / `.dylib`) через `libloading` со стабильным C-ABI | P2 | SPECIFIED |


---

## 2. Детализация ключевых концепций

### Концепция: Universal Plugin Core (Rhai, CLI, DynLib)
*Детальная спецификация:* [`docs/UNIVERSAL_PLUGIN_CORE_SPECIFICATION.md`](file:///d:/Projects/active/photocraft-plugins/docs/UNIVERSAL_PLUGIN_CORE_SPECIFICATION.md)
* **Embedded Rhai Scripts (`plugin_type: "script"`):** Песочница на чистом Rust без C-зависимостей. Скрипт в `main.rhai` получает безопасный доступ к `get_document()`, вызову 851 команды `execute_command(id, params)` и манипуляциям слоями.
* **External CLI Processes (`plugin_type: "external_cli"`):** Асинхронный запуск внешних программ (`entrypoint: "tool.exe"`) через `tokio::process::Command` с подстановкой шаблонов аргументов (`{doc_name}`, `{width}`, `{height}`) и режимами передачи растра (`JsonArgs`, `StdinStdout`, `TempFileInPlace`).
* **C-ABI Dynamic Libraries (`plugin_type: "dynamic_lib"`):** Загрузка динамических библиотек (`entrypoint: "plugin.dll"`) через `libloading` со стабильной C-структурой `PhotoCraftContextC` и точкой входа `photocraft_plugin_execute`. Гарантирует совместимость с любыми компиляторами (Rust, C++, Zig, Go, C# AOT).

### Концепция: Open Plugin SDK
Сторонние разработчики получают готовый интерфейс:
* `ctx.copy_layer()` / `ctx.paste_layer()` — обмен пикселями в памяти без файлов.
* `ctx.get_selection_mask()` — чтение маски текущего выделения пользователя.
* `ctx.execute_command(id, params)` — доступ к 851 команде PhotoCraft (слои, маски, фильтры, история).
* `ctx.ai().load_onnx_model(path)` / `ctx.ai().run_model(inputs)` — готовый аппаратный инференс на GPU/CPU.
* `ctx.ui().set_progress(percent, msg)` / `ctx.ui().show_notification(...)` — обратная связь в интерфейсе.

### Концепция: Plugin Manager GUI
Отдельное окно настроек компаньона:
* Список установленных плагинов с тумблерами включения/выключения.
* Визуальный переназначатель горячих клавиш (нажал сочетание — зафиксировалось).
* Перетаскивание порядка кнопок на плавающей плашке.
* Выбор языка (`Auto`, `English`, `Русский`, ...) и выбор устройства инференса (`Auto GPU`, `DirectML`, `CPU`).

