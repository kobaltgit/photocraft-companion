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
    parser.add_argument("--version", required=True, help="Release version string, e.g. 1.0.3")
    parser.add_argument("--output", default="release_notes.md", help="Output Markdown file path")
    return parser.parse_args()

def extract_latest_chronicle_chapter(chronicle_path):
    if not os.path.exists(chronicle_path):
        return ""
    with open(chronicle_path, "r", encoding="utf-8") as f:
        content = f.read()

    # Find chapters starting with ## [YYYY-MM-DD
    chapters = re.split(r'\n(?=## \[\d{4}-\d{2}-\d{2})', content)
    if len(chapters) > 1:
        # Latest chapter is usually at the bottom in chronicle
        return chapters[-1].strip()
    return ""

def generate_release_notes(version):
    return f"""# PhotoCraft Companion v{version}

[![GitHub Release](https://img.shields.io/badge/Release-v{version}-007acc.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v{version})
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)
[![PhotoCraft](https://img.shields.io/badge/Companion%20For-PhotoCraft-orange.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion)

---

## 🌟 What's New in v{version} / Что нового в версии {version}

### 🇬🇧 English
* **🎨 Interactive UI Plugin System (`interactive_ui`)**: Introduced a 4th major plugin architecture class allowing community developers to create rich, standalone floating tools and windows. Layouts are configured declaratively in `plugin.json` via `WidgetNode` (`color_slot`, `slider`, `dropdown`, `toggle`, `button`, `hue_wheel`, `banner`, `label`, `row`, `separator`) with native bilingual support (`label_ru`, `title_ru`, `options_ru`).
* **⚡ Embedded Rhai Script Engine (`logic.rhai`)**: Pure-Rust embedded scripting engine providing zero-overhead, sandbox-safe plugin interactivity. Supports user interaction hooks `on_state_changed`, system socket events `on_session_colors`, native IPC commands `execute_command`, brush setting `set_colors`, internationalization `t(en, ru)`, format conversions `rgb_to_hex` / `hex_to_rgb`, and perceptual color routines.
* **🔄 Bidirectional Real-Time Color Synchronization (BUG-072 / TASK-014)**: Live two-way communication between PhotoCraft editor and companion tools via declarative socket subscriptions (`ipc_subscriptions: ["session.colors"]`). Automatic sync when using Eyedropper (`I`), default colors (`D`), or color swap (`X`). Features two-tier **Echo Guard** suppression preventing infinite network feedback loops, coupled with an adaptive idle-aware socket poller.
* **👁️ Full W3C WCAG 2.1 & Brettel LMS Color Vision Standards (BUG-071)**: Implemented mathematically rigorous color contrast calculations adhering to W3C WCAG 2.1 relative luminance specifications with strict AA (≥ 3.0:1) and AAA (≥ 4.5:1) pass criteria. Integrates the Brettel/Viénot/Mollon LMS simulation algorithm for deuteranopia, protanopia, tritanopia, and achromatopsia.
* **🧩 ChromaSense A11y Plugin (`ui.photocraft.plugins.chromasense`)**: New official interactive companion tool providing real-time color blindness diagnostics, interactive Hue Wheel, RGB channel sliders, contrast validation, color swap, and instant one-click export into the active PhotoCraft brush.
* **🪟 Multi-Viewport Architecture & Window Independence (BUG-066)**: Interactive plugin windows now run as native first-class desktop viewports via `ctx.show_viewport_immediate`, enabling independent movement, dragging, and minimizing outside the main toolstrip bounds.
* **📖 Plugin Development Guide v2.0**: Exhaustive bilingual documentation (`PLUGIN_DEVELOPMENT_GUIDE.md` and `.en.md`) and updated DevKit skills covering all 4 plugin architectures, manifest specifications, Rhai hooks, and `plugin_validator` workflow.

### 🇷🇺 Русский
* **🎨 Декларативная система интерактивных UI-плагинов (`interactive_ui`)**: Внедрен 4-й фундаментальный класс плагинов, позволяющий создавать полноценные плавающие панели инструментов без модификации ядра. Разметка верстается декларативно в манифесте через `WidgetNode` (`color_slot`, `slider`, `dropdown`, `toggle`, `button`, `hue_wheel`, `banner`, `label`, `row`, `separator`) с полной поддержкой мультиязычности (`label_ru`, `title_ru`, `options_ru`).
* **⚡ Встроенный скриптовый движок Rhai (`logic.rhai`)**: Безопасный и сверхбыстрый скриптовый рантайм на чистом Rust. Предоставляет хук действий пользователя `on_state_changed`, хук системных событий сокета `on_session_colors`, системные вызовы команд PhotoCraft `execute_command`, установку цветов кисти `set_colors`, функции локализации `t(en, ru)`, конвертацию `rgb_to_hex` / `hex_to_rgb` и колориметрическую математику.
* **🔄 Декларативная двунаправленная синхронизация цветов в реальном времени (BUG-072 / TASK-014)**: Бесшовный двусторонний обмен цветами между PhotoCraft и инструментами компаньона через подписку `ipc_subscriptions: ["session.colors"]`. Автоматический подхват цветов при использовании Пипетки (`I`), сбросе палитры (`D`) или инверсии (`X`). Оснащен двухэшелонной защитой **Echo Guard**, физически исключающей сетевые эхо-петли, и адаптивным поллером сокета с нулевой нагрузкой при закрытых окнах.
* **👁️ Соответствие международному стандарту W3C WCAG 2.1 и модели Brettel LMS (BUG-071)**: Математически строгий расчет контрастности по формуле относительной яркости W3C WCAG 2.1 с нормативными порогами AA (≥ 3.0:1) и AAA (≥ 4.5:1). Интегрирована колориметрическая модель Бреттеля/LMS для симуляции дейтеранопии, протанопии, тританопии и монохромазии.
* **🧩 Официальный плагин ChromaSense A11y (`ui.photocraft.plugins.chromasense`)**: Новый интерактивный инструмент экосистемы: диагностика доступности для дальтоников, интерактивный цветовой круг Hue Wheel, раздельные RGB-слайдеры, кнопка Swap и экспорт цвета в активную кисть PhotoCraft в 1 клик.
* **🪟 Мульти-вьюпорт архитектура окон инструментов (BUG-066)**: Окна интерактивных плагинов открываются как независимые нативные окна ОС через `ctx.show_viewport_immediate`, свободно перемещаются по экрану и не зависят от положения тулбара.
* **📖 Руководство разработчика плагинов v2.0**: Комплексная двуязычная документация (`PLUGIN_DEVELOPMENT_GUIDE.md` и `.en.md`) с исчерпывающим описанием всех 4 типов плагинов, API скриптов Rhai и цикла дистрибуции через `plugin_validator`.

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
* **ChromaSense A11y** (`chromasense`): Интерактивная палитра, проверка контраста WCAG 2.1 и симуляция дальтонизма (Brettel LMS).
* **Background Removal** (`remove_bg`): Качественное удаление фона на базе нейросети RMBG-1.4.
* **Depth Map** (`depth_map`): Оценка глубины сцены на базе Depth Anything v2.
* **Image Upscale** (`upscale`): 4x супер-разрешение на базе Real-ESRGAN с бесшовным тайлингом.
* **Generative Inpaint** (`inpaint`): Умное заполнение выделенных областей текстурой окружающего фона (LaMa).
* **Vector Tracing** (`vectorize`): Алгоритмическая векторизация силуэтов и штриховых рисунков.
* **Black & White Pro** (`black_and_white`): Высококонтрастная черно-белая конвертация.

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
