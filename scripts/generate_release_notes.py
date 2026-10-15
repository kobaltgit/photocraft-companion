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
* **🚀 One-Click PhotoCraft Launch & Automated Onboarding (BUG-058)**: Launch PhotoCraft directly from Companion's waiting modal or manager settings with automatic 64-character crypto token generation (`control.token`), multi-source executable discovery, and silent process startup (`CREATE_NO_WINDOW`).
* **🛡️ Tokio UI Thread Safety & Zero-Crash Settings (BUG-059)**: Eliminated application crash upon clicking "Save" in Plugins Manager by establishing ambient Tokio runtime guard and thread-safe async dispatching via runtime handle.
* **💾 Fail-Safe Configuration Mirroring & Path Persistence (BUG-059)**: Automatic configuration migration from legacy locations, portable executable-adjacent settings mirroring, and robust `photocraft_path` recovery on every start.
* **📚 First-Launch Onboarding Guide**: Added comprehensive visual and algorithmic steps in documentation for first-time Companion users.

### 🇷🇺 Русский
* **🚀 Запуск PhotoCraft в один клик и автоматический онбординг (BUG-058)**: Запуск PhotoCraft прямо из окна ожидания Компаньона или вкладки настроек с автоматической криптографической генерацией 64-значного токена (`control.token`), мульти-источниковым поиском экзешника и скрытым стартом без черных окон терминала (`CREATE_NO_WINDOW`).
* **🛡️ Изоляция рантайма Tokio и устранение падения при сохранении (BUG-059)**: Устранена паника ядра и краш программы при клике на «Сохранить» в менеджере плагинов благодаря постоянной гарде рантайма на главном потоке и безопасной диспетчеризации задач через дескриптор `rt_handle`.
* **💾 Зеркалирование настроек и надежное сохранение пути (BUG-059)**: Автоматическая миграция старых конфигов, зеркалирование файла настроек рядом с бинарником в портативном режиме и гарантированное восстановление `photocraft_path` при каждом старте.
* **📚 Пошаговая инструкция первого запуска в README**: Наглядное руководство пользователя по быстрому старту связки PhotoCraft + Companion.

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
