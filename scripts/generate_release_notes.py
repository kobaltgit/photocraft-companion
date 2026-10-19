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
* **🛡️ Zero-Core Architecture & Quality Gates (BUG-062)**: Introduced role-based model routing (`manifest.model_by_role(...)`) in `core_contracts` and established the `architecture_integrity_test` linter to strictly eliminate keyword-sniffing and hardcoded model filenames or plugin IDs in core runtime.
* **🔒 Strict Plugin Model Isolation (BUG-064)**: Blocked Directory Traversal (`..`, `/`, `\\`) in manifest parsing and runtime path resolution. Plugins are completely self-contained and isolated from accessing peer plugin models.
* **📦 Mandatory Model Weights Verification (BUG-065)**: Enforced bilingual model weights documentation (`Model Weights:` / `Ссылки на веса модели:`) with verified Hugging Face download links in `plugin_validator` CI.
* **📂 Plugins Folder Explorer & Multiplatform Reveal**: Added "Open Plugins Folder" button in Plugin Manager header with cross-platform native folder launch (`resolve_reveal_target`) across Windows, macOS, and Linux.
* **🧪 Automated Development Regulations Linter (BUG-063)**: Added 7-point compliance suite (`dev_regulations_lint_test`) ensuring version sync across 5 manifests, crate dependency isolation, Resource Guard enforcement, and bilingual chronicle sync.

### 🇷🇺 Русский
* **🛡️ Архитектурный барьер Zero-Core и Quality Gates (BUG-062)**: Внедрена ролевая маршрутизация моделей (`model_by_role(...)`) в `core_contracts` и статический линтер `architecture_integrity_test`, полностью искоренивший keyword-sniffing и хардкод имен файлов моделей и ID плагинов в ядре.
* **🔒 Железная изоляция моделей плагинов (BUG-064)**: Заблокирован Directory Traversal (`..`, `/`, `\\`) в манифестах и рантайме. Плагины полностью автономны и физически изолированы от использования чужих моделей.
* **📦 Автоматическая валидация ссылок на веса моделей (BUG-065)**: Введено аппаратное требование двуязычных разделов со ссылками на веса моделей и репозитории Hugging Face в валидаторе плагинов.
* **📂 Кнопка «Папка с плагинами» и кроссплатформенное открытие**: В заголовок менеджера плагинов добавлена кнопка быстрого перехода к папке плагинов с поддержкой нативного открытия в проводнике Windows, Finder на macOS и файловом менеджере Linux.
* **🧪 Линтер регламентов разработки (BUG-063)**: Внедрен автоматический тест из 7 проверок (`dev_regulations_lint_test`), контролирующий синхронизацию версий в 5 файлах, изоляцию крейтов, флаги Resource Guard и синхронность глав летописи.

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
