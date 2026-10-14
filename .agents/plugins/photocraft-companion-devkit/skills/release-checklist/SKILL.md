---
name: release-checklist
description: Чек-лист проверки перед подготовкой авторелиза PhotoCraft Companion (синхронизация версий в 4 файлах, валидатор плагинов, отсутствие ошибок в Git).
---

# Pre-Release Checklist Workflow

Используйте этот сценарий перед созданием релиза или пушем тега `v*` в репозиторий.

## Шаг 1. Синхронизация версий во всех манифестах
Убедитесь, что номер версии (например, `1.0.2`) идентичен во всех 4 файлах:
1. `Cargo.toml` -> `[workspace.package] version = "1.0.2"`
2. `scripts/package_release.ps1` -> `$Version = "1.0.2"`
3. `scripts/package_release.sh` -> `VERSION="1.0.2"`
4. `packaging/windows/installer.iss` -> `#define MyAppVersion "1.0.2"`

## Шаг 2. Проверка линтеров и компиляции
```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j 1
```

## Шаг 3. Валидация каталога и сборка реестра плагинов
```powershell
cargo run --package plugin_validator -j 1 -- --validate plugins --build-registry --package-plugins --update-readme README.md --update-readme README.ru.md
```

## Шаг 4. Коммит изменений документации перед тегом
Если README обновились:
```powershell
git add README.md README.ru.md
git commit -m "docs: sync community plugins catalog prior to release"
git push origin main
```

## Шаг 5. Постановка тега и пуш для запуска авторелиза
```powershell
git tag v1.0.2
git push origin v1.0.2
```
GitHub Actions подхватит тег, соберет Windows Installer, Linux DEB, macOS DMG и прикрепит их к GitHub Release.
