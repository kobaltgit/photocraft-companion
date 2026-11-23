# PhotoCraft Companion v2.2.5

[![GitHub Release](https://img.shields.io/badge/Release-v2.2.5-green.svg?style=flat-square)](https://github.com/kobaltgit/photocraft-companion/releases/tag/v2.2.5)
[![License](https://img.shields.io/badge/License-MIT%20%2F%20Apache--2.0-blue.svg?style=flat-square)](LICENSE)

> [!NOTE]
> **Linux GLIBC 2.35 Compatibility & Stable Runner Baseline / Совместимость Linux с GLIBC 2.35 и базовый сборочный раннер:**
> This release resolves an issue where prebuilt Linux binaries failed to launch on Linux Mint 22.x, Ubuntu 24.04 LTS, and Debian 12 due to requiring `GLIBC_2.43` ([Issue #20](https://github.com/kobaltgit/photocraft-companion/issues/20) / BUG-130). The release build pipeline has been pinned to an `ubuntu-22.04` LTS baseline with `GLIBC 2.35` sysroot, ensuring 100% backward compatibility across all modern Linux distributions.
> 
> *Этот релиз устраняет проблему, из-за которой готовые бинарники Linux не запускались на Linux Mint 22.x, Ubuntu 24.04 LTS и Debian 12 из-за требования `GLIBC_2.43` ([Issue #20](https://github.com/kobaltgit/photocraft-companion/issues/20) / BUG-130). Пайплайн сборки зафиксирован на LTS-раннере `ubuntu-22.04` с базовым sysroot `GLIBC 2.35`, обеспечивая 100% обратную совместимость со всеми современными дистрибутивами Linux.*

---

## 🇬🇧 English: What's New in v2.2.5

### 1. Linux GLIBC 2.35 Baseline Compatibility ([Issue #20](https://github.com/kobaltgit/photocraft-companion/issues/20) / BUG-130)
* **The Problem:** In GitHub Actions, jobs using `runs-on: ubuntu-latest` dynamic environments compile against bleeding-edge toolchains and link against GLIBC 2.43 symbols. Under Linux symbol versioning rules, binaries linked against newer GLIBC versions refuse to launch on stable LTS distributions, failing with `/lib/x86_64-linux-gnu/libm.so.6: version 'GLIBC_2.43' not found` on Linux Mint 22.x (Ubuntu 24.04 LTS base with GLIBC 2.39) and Debian 12 (GLIBC 2.36). Upgrading GLIBC system-wide is dangerous and unsupported for end users.
* **The Solution:**
  * Pinned the Linux packaging job `build-linux` in `.github/workflows/release.yml` from floating `ubuntu-latest` to stable `runs-on: ubuntu-22.04`.
  * In the Ubuntu 22.04 environment, the binary links against a `GLIBC 2.35` sysroot, guaranteeing seamless execution out of the box on Ubuntu 22.04 LTS, Ubuntu 24.04 LTS, Linux Mint 21/22, Debian 12 (Bookworm), Fedora, Arch Linux, and Astra Linux.
  * Synchronized the `test-linux` job in `.github/workflows/ci.yml` to `runs-on: ubuntu-22.04` to maintain consistent verification against the baseline LTS toolchain.

### 2. Explicit Dependencies for DEB Package (`scripts/package_release.sh`)
* Updated `DEBIAN/control` generation in `scripts/package_release.sh` to declare explicit runtime dependencies:
  `Depends: libc6 (>= 2.35), libgtk-3-0, libayatana-appindicator3-1 | libappindicator3-1, libssl3, libx11-6, libwayland-client0`.
* This ensures native package managers (`apt`, `dpkg`, GDebi) properly validate and satisfy dependencies during installation rather than producing silent runtime load failures.

### 3. Automated Regulation & Baseline Guard Test
* Implemented regression test `test_linux_build_workflow_glibc_baseline_guard` in `crates/ai_engine/tests/dev_regulations_lint_test.rs` to enforce that release workflows and packaging scripts maintain the `ubuntu-22.04` runner and `libc6 (>= 2.35)` constraint.

---

## 🇷🇺 Русский: Что нового в версии 2.2.5

### 1. Совместимость Linux с GLIBC 2.35 ([Issue #20](https://github.com/kobaltgit/photocraft-companion/issues/20) / BUG-130)
* **В чём была проблема:** При сборке на раннере `runs-on: ubuntu-latest` в GitHub Actions компилятор линковал вызовы математической библиотеки и рантайма против новейших символов GLIBC 2.43. Из-за механизма версионирования символов в Linux исполняемый файл отказывался запускаться на дистрибутивах со стабильными версиями библиотек, завершаясь ошибкой `/lib/x86_64-linux-gnu/libm.so.6: version 'GLIBC_2.43' not found` на Linux Mint 22.x (база Ubuntu 24.04 c GLIBC 2.39) и Debian 12 (GLIBC 2.36). Обновление системной GLIBC пользователем чревато крахом всей операционной системы.
* **Решение:**
  * В файле `.github/workflows/release.yml` сборочная джоба `build-linux` переведена со скользящего `ubuntu-latest` на стабильный базовый LTS-раннер `runs-on: ubuntu-22.04`.
  * При сборке на Ubuntu 22.04 бинарник линкуется против базового sysroot с `GLIBC 2.35`, что гарантирует безупречную работу из коробки на Ubuntu 22.04 LTS, Ubuntu 24.04 LTS, Linux Mint 21/22, Debian 12 (Bookworm), Fedora, Arch Linux и Astra Linux.
  * Тестовая джоба `test-linux` в `.github/workflows/ci.yml` также синхронизирована на `runs-on: ubuntu-22.04` для постоянного контроля совместимости.

### 2. Явное объявление зависимостей в DEB-пакете (`scripts/package_release.sh`)
* В манифест `DEBIAN/control` добавлена декларация необходимых библиотек:
  `Depends: libc6 (>= 2.35), libgtk-3-0, libayatana-appindicator3-1 | libappindicator3-1, libssl3, libx11-6, libwayland-client0`.
* Системные менеджеры пакетов (`apt`, `dpkg`) теперь корректно проверяют окружение при установке, исключая тихие сбои запуска.

### 3. Автоматический тест регламента сборки
* В `crates/ai_engine/tests/dev_regulations_lint_test.rs` добавлен тест `test_linux_build_workflow_glibc_baseline_guard`, защищающий пайплайн от случайного отката раннеров на `ubuntu-latest` и гарантирующий сохранение базовой совместимости.

---

## 📦 Packages & Downloads / Файлы дистрибутива

| Platform / ОС | Package Type / Тип пакета | File Name / Имя файла | Description / Назначение |
|---|---|---|---|
| 🪟 **Windows x64** | **Setup Installer** | `PhotoCraft-Companion-Setup-Windows-x64.exe` | Рекомендуемый установщик с ярлыками и интеграцией в систему |
| 🪟 **Windows x64** | **Portable Archive** | `PhotoCraft-Companion-v2.2.5-windows-x64-portable.zip` | Портабельная версия: распакуйте и запускайте без установки |
| 🐧 **Linux x64** | **DEB Package** | `PhotoCraft-Companion-v2.2.5-linux-amd64.deb` | Установочный пакет для Ubuntu 22.04+, Mint 21/22, Debian 12+ |
| 🐧 **Linux x64** | **Portable Archive** | `photocraft-companion-v2.2.5-linux-x86_64-portable.tar.gz` | Универсальный портабельный архив для любых дистрибутивов Linux |
| 🍏 **macOS** | **DMG Installer** | `PhotoCraft-Companion-v2.2.5-macos.dmg` | Образ диска для macOS (Apple Silicon & Intel) |
| 🍏 **macOS** | **Portable Archive** | `PhotoCraft-Companion-v2.2.5-macos-portable.tar.gz` | Портабельный архив для macOS |
