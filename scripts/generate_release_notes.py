#!/usr/bin/env python3
"""
PhotoCraft Companion - Automated Release Notes Generator
Generates clean, bilingual (English & Russian) GitHub Release Notes with download tables and features list.
Prioritizes dedicated docs/RELEASE_NOTES_v{version}.md if present.
"""

import argparse
import os
import sys

def parse_args():
    parser = argparse.ArgumentParser(description="Generate bilingual release notes for PhotoCraft Companion")
    parser.add_argument("--version", required=True, help="Release version string, e.g. 1.2.2")
    parser.add_argument("--output", default="release_notes.md", help="Output Markdown file path")
    return parser.parse_args()

def find_dedicated_release_notes(version):
    script_dir = os.path.dirname(os.path.abspath(__file__))
    repo_root = os.path.abspath(os.path.join(script_dir, ".."))
    
    candidates = [
        os.path.join(repo_root, "docs", f"RELEASE_NOTES_v{version}.md"),
        os.path.join(os.getcwd(), "docs", f"RELEASE_NOTES_v{version}.md"),
        os.path.join(os.getcwd(), f"RELEASE_NOTES_v{version}.md"),
    ]
    for c in candidates:
        if os.path.isfile(c):
            return c
    return None

def generate_release_notes(version):
    dedicated = find_dedicated_release_notes(version)
    if dedicated:
        print(f"Using dedicated release notes from: {dedicated}")
        with open(dedicated, "r", encoding="utf-8") as f:
            return f.read().strip() + "\n"

    return f"""# PhotoCraft Companion v{version}

[![GitHub Release](https://img.shields.io/badge/Release-v{version}-007acc.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v{version})
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)
[![Companion For](https://img.shields.io/badge/Companion%20For-PhotoCraft-orange.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion)
[![Discord](https://img.shields.io/badge/Discord-Companion_Server-5865F2?logo=discord&logoColor=white)](https://discord.gg/2gzn4Un4Bw)

---

## 🌟 What's New in v{version} / Что нового в версии {version}

### 🇬🇧 English
* **🍏 Native macOS Bundle Execution & CodeSeal Protection (BUG-076)**: Routed `.app` bundle execution through Apple LaunchServices (`/usr/bin/open -n -a "<bundle.app>" --args ...`), solving WindowServer initialization issues and Gatekeeper terminations (`Killed: 9`). Added semantic CodeSeal protection in `is_token_dir_writable` to avoid saving `control.token` inside `.app` bundles, which invalidates code signatures.
* **🪟 Win32 Process Identification & Zero Handle Leaks (BUG-077)**: Upgraded host process detection to verified Win32 PID query via `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, ...)` and `QueryFullProcessImageNameW`, guaranteeing `CloseHandle(h_process)` and zero descriptor leaks.
* **🖥️ DPI-Aware Screen Centering & Boundary Guard (BUG-077)**: Introduced `calculate_primary_screen_center(w, h)` factoring `(GetDpiForSystem() / 96.0)` for centered placement on 125%/150% DPI screens. Added active display boundary check via `MonitorFromRect` to automatically snap off-screen windows back to the primary display.
* **📐 Persistent Plugin Manager Geometry (BUG-077)**: Persisted `manager_window_position` and `manager_window_size` in `CompanionConfig`. Window dimensions track strictly through `inner_rect.size()`, eliminating progressive window expansion on each reopen.
* **💡 Enhanced First-Run UX & Offline Onboarding (BUG-076, BUG-077)**: Expanded offline modal to 300×280 px without clipping, added contextual hints for macOS Gatekeeper and Windows SmartScreen, clear status when `photocraft.exe` is detected without socket, and settings auto-saving on launch.

### 🇷🇺 Русский
* **🍏 Нативный запуск бандлов macOS и защита CodeSeal (BUG-076)**: Запуск бандлов `.app` переведён на системный сервис `/usr/bin/open -n -a "<bundle.app>" --args ...`, что устранило сбои WindowServer и аварийные завершения Gatekeeper (`Killed: 9`). Внедрена семантическая защита `is_token_dir_writable`, исключающая запись токена внутрь бандлов и повреждение подписи.
* **🪟 Строгая Win32-идентификация и предотвращение утечек дескрипторов (BUG-077)**: Отказ от ненадежного перебора заголовков: внедрена строгая верификация PID, исполняемого файла через `OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, ...)` и гарантированное закрытие `CloseHandle(h_process)`, устраняющее утечку дескрипторов Windows.
* **🖥️ DPI-Aware центрирование и защита от улета окон (BUG-077)**: Реализована функция `calculate_primary_screen_center(w, h)` с учетом системного масштаба `(GetDpiForSystem() / 96.0)`. Добавлен валидатор границ `MonitorFromRect`, возвращающий окна в центр при отключении внешних мониторов.
* **📐 Персистентность геометрии Plugin Manager с защитой от раздувания (BUG-077)**: Сохранение позиции и размеров окна менеджера в `CompanionConfig` с отслеживанием размеров строго по клиентской области `inner_rect.size()`, исключая прогрессирующее увеличение окна.
* **💡 Улучшение оффлайн-онбординга и подсказки первого запуска (BUG-076, BUG-077)**: Размер оффлайн-окна увеличен до 300×280 px во всех точках входа, внедрены платформенные инструкции первого запуска (Gatekeeper, SmartScreen), отображение статуса запущенного редактора и автосохранение настроек при запуске.

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
