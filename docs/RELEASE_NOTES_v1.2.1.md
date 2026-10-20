# Release Notes: PhotoCraft Companion v1.2.1

**Release Tag:** `v1.2.1`  
**Date:** 2026-10-20  
**Status:** Stable Production Release  
**Author:** kobaltGIT  
**Discord:** [https://discord.gg/2gzn4Un4Bw](https://discord.gg/2gzn4Un4Bw)

---

# 🇬🇧 English Release Notes

PhotoCraft Companion reaches version **1.2.1** — bringing visual polish, vector-first community integration, and theme-adaptive design improvements.

---

### 🌟 Key Highlights & Major Additions

#### 1. Native Vector Discord Community Button
* **Header Quick Access:** Added an official community Discord button directly to the upper-right corner of the `PhotoCraft - Plugins Manager` window, aligning with the horizontal tabs.
* **Vector-First UI (Strict Rule 6 Compliance):** Rendered strictly via pure SVG vector graphics (`usvg`/`tiny_skia`) with zero font glyphs or Unicode emojis, guaranteeing 0% risk of tofu (`□`) rendering artifacts.
* **Theme-Adaptive Visuals:**
  * **Idle:** Subtly matches adjacent tabs in `palette.button_secondary` and active theme text color (light silver in dark mode, dark graphite in light mode), avoiding distraction from settings.
  * **Hover:** Highlights dynamically with Discord's signature Blurple brand color (`#5865F2`).
* **Bilingual Tooltip & One-Click Navigation:** Displays localized tooltip (*"Join Discord Community"* / *"Присоединиться к Discord-сообществу"*) and opens the official invitation link in the default browser.

#### 2. Quality & Architecture Integrity
* **100% Passing Test Suite:** Verified across all 154 unit and integration tests with thread guard (`-j 1`).
* **Strict Localization & Vector Linters:** Passed `vector_first_lint_test`, `i18n_lint_test`, and `dev_regulations_lint_test`.

---

# 🇷🇺 Описание релиза на русском языке

PhotoCraft Companion обновлён до версии **1.2.1** — минорное обновление с акцентом на векторный интерфейс, динамическую тему оформления и удобную интеграцию с Discord-сообществом.

---

### 🌟 Ключевые нововведения

#### 1. Нативная векторная кнопка перехода в Discord в Менеджере плагинов
* **Быстрый доступ в шапке:** В правый верхний угол окна `PhotoCraft - Plugins Manager` добавлена компактная кнопка быстрого перехода на сервер сообщества Discord на одной линии с вкладками табов.
* **Векторный интерфейс (Vector-First UI, Правило 6):** Логотип Discord отрисован через каноничный SVG-контур средствами `usvg`/`tiny_skia`. Полный отказ от эмодзи и юникод-глифов гарантирует отсутствие артефактов и квадратов «тофу» (`□`).
* **Двухуровневая адаптация под темы оформления:**
  * **В покое:** Кнопка окрашена в цвет неактивных вкладок (`palette.button_secondary`), а сама иконка адаптируется под тему (светло-серая в тёмной теме, тёмно-графитовая в светлой).
  * **При наведении (hover):** Подсвечивается фирменным брендовым цветом Discord Blurple (`#5865F2`).
* **Двуязычный тултип:** Аккуратная всплывающая подсказка *«Присоединиться к Discord-сообществу» / «Join Discord Community»* и переход в браузер в один клик.

#### 2. Контроль качества и стабильность
* Все 154 теста Cargo Workspace успешно пройдены со строгим ограничением ресурсов (`-j 1`).
* Подтверждена архитектурная чистота и валидность всех каталогов плагинов.

---

## 📦 Distribution Packages / Файлы дистрибутива

| Platform / ОС | Package Type / Тип пакета | File Name / Имя файла | Description / Назначение |
|---|---|---|---|
| 🪟 **Windows x64** | **Setup Installer** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Рекомендуемый установщик с ярлыками и интеграцией в систему |
| 🪟 **Windows x64** | **Portable Archive** | `PhotoCraft-Companion-v1.2.1-windows-x64-portable.zip` | Портабельная версия: распакуйте и запускайте без установки |
| 🐧 **Linux x64** | **DEB Package** | `photocraft-companion_1.2.1_amd64.deb` | Установочный пакет для Ubuntu, Debian, Mint |
| 🐧 **Linux x64** | **Portable Archive** | `photocraft-companion-v1.2.1-linux-x64-portable.tar.gz` | Портабельный архив для любых Linux-дистрибутивов |
| 🍏 **macOS** | **DMG Installer** | `PhotoCraft-Companion-Setup-macOS.dmg` | Образ диска для macOS (Apple Silicon & Intel) |
| 🍏 **macOS** | **Portable Archive** | `photocraft-companion-v1.2.1-macos-portable.tar.gz` | Портабельный архив для macOS |
