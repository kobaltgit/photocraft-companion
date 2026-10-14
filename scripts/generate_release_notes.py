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
* **🗺️ Depth Anything v2 (`depth_map`)**: State-of-the-art monocular depth estimation plugin with selectable Depth Anything V2 Small and Metric models, producing crisp depth maps and relief normals directly in active layers.
* **📐 Vector Tracing (`vectorize`)**: High-performance algorithmic raster-to-vector tracing plugin converting silhouettes and line art into polygonal vector contours.
* **📄 Interactive Plugin Documentation & Use Cases Modal**: Seamless vector document icon button in each plugin card displaying bilingual practical workflows, layer tips, and model links without restarting PhotoCraft.
* **🛠️ Robust Portable Self-Updater (BUG-056)**: Eliminated terminal crashes and "insufficient memory" errors on Windows by transitioning from fragile batch/xcopy to silent PowerShell (`Wait-Process` + `robocopy` with 32K path support and Defender lock cushions).
* **🤖 Automated Release Notes Publishing**: Integrated bilingual release notes compiler and automated GitHub Releases asset packaging.

### 🇷🇺 Русский
* **🗺️ Depth Anything v2 (`depth_map`)**: Плагин генерации карт глубины на базе моделей Depth Anything V2 (Small и Metric) для рельефа, карт нормалей и 3D-композитинга прямо в активном документе.
* **📐 Векторизация растра (`vectorize`)**: Высокоскоростной алгоритмический плагин трассировки контуров и силуэтов в полигональные векторные пути PhotoCraft.
* **📄 Интерактивное окно документации и сценариев (Use Cases)**: Векторная кнопка в карточке каждого плагина для мгновенного просмотра практических приёмов, рекомендованных слоев и ссылок на веса прямо в редакторе.
* **🛠️ Надежное портабельное самообновление (BUG-056)**: Устранены ошибки «недостаточно памяти» и вылеты терминала на Windows благодаря переходу на тихий PowerShell (`Wait-Process` + `robocopy` с поддержкой длинных путей до 32K символов и задержкой для Windows Defender).
* **🤖 Автопубликация описания релиза**: Полная автоматизация генерации двуязычных заметок и дистрибутивов в GitHub Actions без ручного ввода.

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
