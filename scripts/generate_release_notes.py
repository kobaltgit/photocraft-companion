#!/usr/bin/env bash
#!/usr/bin/env python3
"""
PhotoCraft Companion - Automated Release Notes Generator
Generates clean, bilingual (English & Russian) GitHub Release Notes with download tables and features list.
"""

import argparse
import os
import re
import sys

def parse_args():
    parser = argparse.ArgumentParser(description="Generate bilingual release notes for PhotoCraft Companion")
    parser.add_argument("--version", required=True, help="Release version string, e.g. 1.2.0")
    parser.add_argument("--output", default="release_notes.md", help="Output Markdown file path")
    return parser.parse_args()

def generate_release_notes(version):
    return f"""# PhotoCraft Companion v{version}

[![GitHub Release](https://img.shields.io/badge/Release-v{version}-007acc.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v{version})
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)
[![Companion For](https://img.shields.io/badge/Companion%20For-PhotoCraft-orange.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion)
[![Discord](https://img.shields.io/badge/Discord-Companion_Server-5865F2?logo=discord&logoColor=white)](https://discord.gg/2gzn4Un4Bw)

---

## 🌟 What's New in v{version} / Что нового в версии {version}

### 🇬🇧 English
* **⚡ Universal Declarative Workflow Engine (`workflow_engine`)**: Major architectural advancement enabling multi-step, complex image manipulation pipelines without compiling Rust code. Features dynamic context variable resolution (`$source_layer`, `$result_layer`, `$active_layer`, `$doc_width`, `$doc_height`, `$has_selection`, `$fg_color`, `$bg_color`, and string interpolation `{{...}}`), conditional branching (`when`), variable capture (`save_as`), error tolerance (`ignore_errors`), and sandboxed Rhai scripting.
* **📐 Photoshop-Style Sub-Tools & Flyout Viewports (`sub_actions`)**: Tool buttons in the Toolstrip can now declare multiple functional variations (e.g. Cutout to new layer vs Non-destructive Layer Mask). Buttons render a subtle vector corner chevron; clicking the chevron or long-pressing (>350ms) opens a dedicated floating flyout window (`egui::ViewportBuilder`), persisting user choices in settings.
* **✂️ AI Remove Background v1.1.0 with Sub-Tools**: Upgraded RMBG-1.4 to offer two native operational modes: instant subject isolation onto a clean transparent layer (**Cutout**) and non-destructive native layer mask creation (**Create Layer Mask**) via seamless selection inversion and temporary layer cleanup.
* **🚀 Up to 25x Faster AI Upscaling via Warm DirectML Session Tiling (BUG-075)**: Resolved severe tile inference bottleneck in `run_tiled_rgb_inference`. The DirectML/CPU session is initialized strictly once before tiling begins and reused across all tile iterations, eliminating redundant model reads and HLSL shader recompilations in VRAM. Per-tile inference dropped from 16s down to ~1.9s.
* **🛡️ No-Code Duplicate Layer Execution Guard (BUG-074)**: Fixed an execution dispatch flaw where No-Code macros executed both as pre-commands and plugin body, producing triple identical layers. No-Code pipelines now run strictly once.
* **📖 Plugin Developer Guide v2.1 & DevKit Update**: Comprehensive bilingual documentation and Antigravity DevKit skills updated to cover the Workflow Engine, context variables, and Sub-Actions schema.
* **💬 Dedicated Companion Discord Community**: Prominent community server invitations and badges added for live developer and artist collaboration.

### 🇷🇺 Русский
* **⚡ Универсальный декларативный движок рабочих процессов Workflow Engine (`workflow_engine`)**: Фундаментальное расширение возможностей рантайма, позволяющее строить сложные многошаговые пайплайны обработки без компиляции кода. Включает автоматическую интерполяцию контекстных переменных (`$source_layer`, `$result_layer`, `$active_layer`, `$doc_width`, `$doc_height`, `$has_selection`, `$fg_color`, `$bg_color` и строковый синтаксис `{{...}}`), условное ветвление (`when`), сохранение промежуточных слоев (`save_as`), безопасный пропуск ошибок (`ignore_errors`) и скриптовые шаги Rhai.
* **📐 Суб-инструменты в стиле Photoshop и плавающие меню Flyout (`sub_actions`)**: Кнопки тулбара теперь могут объявлять несколько функциональных режимов работы. Кнопки с суб-действиями снабжены векторным индикатором-шевроном; клик по шеврону или долгое нажатие (>350 мс) открывает отдельное плавающее окно выбора (`egui::ViewportBuilder`), а выбранный режим надёжно сохраняется в конфигурации пользователя.
* **✂️ Обновление плагина AI Remove Background v1.1.0**: В инструмент RMBG-1.4 внедрены два официальных суб-инструмента: быстрая вырезка объекта на чистый прозрачный слой (**Удаление фона / Вырезка**) и создание неразрушающей маски слоя (**Создать маску слоя**) с автоматическим переносом альфа-контура в выделение и очисткой временных слоев.
* **🚀 Ускорение ИИ-апскейла до 25 раз благодаря кэшированию сессий DirectML (BUG-075)**: Устранено узкое горлышко производительности в тайлинге апскейла `run_tiled_rgb_inference`. Сессия DirectML/CPU теперь инициализируется ровно 1 раз на весь холст и остаётся горячей в VRAM GPU на всех тайлах без повторного вычитывания 71 МБ весов с диска. Время обработки тайла снижено с 16с до ~1.9с.
* **🛡️ Защита от дублирования слоев в No-Code макросах (BUG-074)**: Исправлен дефект, при котором No-Code плагины дублировали шаги как pre-команды и тело плагина, создавая тройные слои. Команды макроса теперь выполняются строго один раз.
* **📖 Руководство разработчика плагинов v2.1 и обновление DevKit**: Полная двуязычная документация по созданию плагинов с Workflow Engine, спецификацией шагов и суб-инструментов.
* **💬 Официальный Discord-сервер PhotoCraft Companion**: Прямые ссылки на сервер сообщества для оперативной поддержки и обмена плагинами.

---

## 📦 Packages & Downloads / Файлы дистрибутива

| Platform / ОС | Package Type / Тип пакета | File Name / Имя файла | Description / Назначение |
|---|---|---|---|
| 🪟 **Windows x64** | **Setup Installer** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Рекомендуемый установщик с ярлыками и интеграцией в систему |
| 🪟 **Windows x64** | **Portable Archive** | `PhotoCraft-Companion-v{version}-windows-x64-portable.zip` | Портабельная версия: распакуйте и запускайте без установки |
| 🐧 **Linux x64** | **DEB Package** | `photocraft-companion_{version}_amd64.deb` | Установочный пакет для Ubuntu, Debian, Mint |
| 🐧 **Linux x64** | **Portable Archive** | `photocraft-companion-v{version}-linux-x64-portable.tar.gz` | Портабельный архив для любых Linux-дистрибутивов |
| 🍏 **macOS** | **DMG Installer** | `PhotoCraft-Companion-Setup-macOS.dmg` | Образ диска для macOS (Apple Silicon & Intel) |
| 🍏 **macOS** | **Portable Archive** | `photocraft-companion-v{version}-macos-portable.tar.gz` | Портабельный архив для macOS |

---

## 🧩 Community Plugins Marketplace / Каталог плагинов

Все официальные плагины (`.pcpl`) автоматически доступны для установки в один клик через вкладку **«Магазин плагинов»** в менеджере:
* **Background Removal** (`remove_bg`): Качественное удаление фона на базе нейросети RMBG-1.4 с выбором режимов Cutout и Layer Mask.
* **ChromaSense A11y** (`chromasense`): Интерактивная палитра, проверка контраста WCAG 2.1 и симуляция дальтонизма (Brettel LMS).
* **Depth Map** (`depth_map`): Оценка глубины сцены на базе Depth Anything v2.
* **Image Upscale** (`upscale`): 2x супер-разрешение на базе Real-ESRGAN с ускоренным кэшированным тайлингом.
* **Generative Inpaint** (`inpaint`): Умное заполнение выделенных областей текстурой окружающего фона (LaMa).
* **Vector Tracing** (`vectorize`): Алгоритмическая векторизация силуэтов и штриховых рисунков.
* **Black & White Pro** (`black_and_white`): Высококонтрастная черно-белая конвертация No-Code.

---
"""

def main():
    args = parse_args()
    notes = generate_release_notes(args.version)
    with open(args.output, "w", encoding="utf-8") as f:
        f.write(notes)
    print(f"Generated release notes for v{args.version} at {args.output}")

if __name__ == "__main__":
    main()
