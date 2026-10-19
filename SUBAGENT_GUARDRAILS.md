# SUBAGENT GUARDRAILS — Свод абсолютных правил разработки PhotoCraft Companion

Данный документ содержит **обязательную выжимку генерального регламента разработки** проекта **PhotoCraft Plugins & Companion**.
Каждый субагент (`ai-specialist`, `ui-specialist`, `bridge-specialist`, `chronicle-writer`, `quality-auditor`, `research`, `self`) **ОБЯЗАН** строго руководствоваться этим документом перед написанием любой строчки кода, запуском тестов или сдачей задачи.

---

## 1. Ресурсный щит при компиляции (Resource Guard) — ПРАВИЛО №1
* **СТРОГО ЗАПРЕЩЕНО** запускать `cargo build`, `cargo check`, `cargo test`, `cargo clippy` без явного флага ограничения потоков:
  ```powershell
  cargo test --workspace -j 1
  cargo test -p ai_engine -j 1
  cargo build --package companion_app --release -j 1
  ```
* Запуск без `-j 1` (максимум `-j 2` только при согласовании с пользователем) перегружает CPU и забивает всю RAM рабочей станции. Нарушение этого правила считается грубейшей ошибкой.

---

## 2. Железный запрет хардкода и keyword-sniffing (Zero-Core Hardcoding)
Ядро компаньона (`crates/ai_engine`, `crates/companion_app`, `crates/photocraft_bridge`) — это **нейтральный универсальный рантайм**. Любой сторонний разработчик должен иметь возможность положить в `plugins/` манифест `plugin.json` и веса — и плагин **обязан заработать без изменения и перекомпиляции программы**.

### Категорически ЗАПРЕЩЕНО в кодовой базе ядра (`crates/*/src/`):
1. **Keyword-sniffing названий моделей или плагинов:**
   ❌ Запрещено писать:
   - `.contains("depth")`, `.contains("normal")`, `.contains("birefnet")`, `.contains("lama")`, `.contains("esrgan")`
   - `.starts_with(...)`, `.ends_with(...)`, `==` по захардкоженным строкам моделей.
   - Любая маршрутизация моделей идет **исключительно через поле `role`** в манифестах `core_contracts` (например, `manifest.model_by_role("depth_estimation")`).
2. **Хардкод расширений весов моделей в строковых литералах:**
   ❌ Запрещены литералы: `".onnx"`, `".bin"`, `".safetensors"`, `".pt"`, `".pth"`.
   - Для формирования путей файлов в кэше моделей используйте безопасный вызов:
     ```rust
     cache_dir.join(&model_hash).with_extension("onnx")
     ```
3. **Хардкод ID плагинов в ядре:**
   ❌ Запрещены префиксы `"ai.photocraft.plugins."`, `"macro.photocraft.plugins."` внутри ядра.
4. **Манипуляция флагом `required: false`:**
   ❌ Запрещено менять `required: true` на `false` в манифестах ради фиктивного прохождения тестов на машинах без весов. Тесты без моделей пишутся через явные Mock-структуры внутри `#[cfg(test)]`.
5. **Вшивание шагов пайплайна внутрь Rust-кода:**
   - Шаги предобработки, постобработки и команды холста (`image.revealAll`, `layer.mergeDown`) объявляются **декларативно в `plugin.json`** (`commands`, `post_commands`).

> **Автоматический контроль:** Нарушения блокируются тестом:
> `cargo test -p ai_engine --test architecture_integrity_test -j 1`

---

## 3. Изоляция крейтов воркспейса (Crate Dependency Isolation)
Монорепозиторий разделен на изолированные пакеты с четким направлением зависимостей:
* `core_contracts` — общий контрактный слой. **НЕ ДОЛЖЕН зависеть ни от одного внутреннего крейта воркспейса** (`photocraft_bridge`, `ai_engine`, `overlay_ui`, `companion_app`, `plugin_validator`).
* `photocraft_bridge` **НЕ ДОЛЖЕН зависеть** от `ai_engine` или `overlay_ui`.
* `ai_engine` **НЕ ДОЛЖЕН зависеть** от `photocraft_bridge` или `overlay_ui`.
* `overlay_ui` **НЕ ДОЛЖЕН зависеть** от `photocraft_bridge` или `ai_engine`.
* `companion_app` — точка сборки DI, зависит от всех остальных крейтов.

> **Автоматический контроль:** Нарушения блокируются тестом:
> `cargo test -p ai_engine --test dev_regulations_lint_test -j 1`

---

## 4. Векторный интерфейс (Vector-First UI — Запрет Emoji и Tofu-глифов)
В шрифтах `egui` нет встроенных цветных Emoji и расширенных Unicode-символов. При их выводе интерфейс ломается уродливыми квадратами `□` (тофу).

### Правила оформления UI:
1. **Категорически ЗАПРЕЩЕНЫ любые Emoji в текстовых строках UI:**
   ❌ `⏳`, `⚠️`, `⬇`, `⬆`, `⬅`, `➡`, `✨`, `🪄`, `🔥`, `🚀`, `💡`.
2. **Категорически ЗАПРЕЩЕНЫ спец-символы Unicode:**
   ❌ `●` (U+25CF), `•` (U+2022), `✓`, `✔`, `✕`, `✖`, `⚙`, `🔍`.
3. **Отрисовка только через векторные примитивы:**
   - Круглые индикаторы статуса (подключение, статус воркера): `ui.painter().circle_filled(center, radius, color)`.
   - Иконки: векторные линии `line_segment(...)`, круги `circle_stroke(...)` или векторные SVG через `SvgIconCache`.
   - Индикация занятости: текстовая строка `"... "` с анимацией или векторный пульсирующий кружок.

> **Автоматический контроль:** Нарушения блокируются тестом:
> `cargo test -p overlay_ui --test vector_first_lint_test -j 1`

---

## 5. Двуязычная локализация (i18n Guard)
* Файлы `icons.rs`, `store.rs`, `theme.rs`, `toolstrip.rs`, `updater.rs`, `win32_window.rs` должны быть **на 100% свободны от кириллицы**.
* В `manager.rs` и `offline_modal.rs` **любая кириллица обязана быть обернута в метод `.t("En", "Ru")`**:
  ```rust
  self.t("Compute & Acceleration", "Ускоритель / Провайдер")
  ```
* Запрещены пустые строки, заглушки `TODO`, `FIXME`, `???`, `untranslated`.

> **Автоматический контроль:** Нарушения блокируются тестом:
> `cargo test -p overlay_ui --test i18n_lint_test -j 1`

---

## 6. Защитные механизмы сетевого взаимодействия (Worker Guardrails)
При реализации сетевого обмена и воркера инференса:
1. **OOM & Overflow Guard:**
   - Размер буфера `num_elements` из `shape` вычисляется строго через `checked_mul`.
   - Лимит размера тензора: `num_elements <= 128_000_000` (макс. ~512 МБ). При переполнении или превышении лимита — немедленный возврат `HTTP 400 Bad Request`.
2. **Атомарная запись моделей в кэш:**
   - Поток `POST /v1/upload_model` пишется во временный файл (`<hash>.tmp`).
   - Инкрементально проверяется SHA-256 хеш.
   - Только при 100% совпадении файл переносится через `std::fs::rename` в `<hash>.onnx`. При ошибке или обрыве временный файл удаляется.
3. **Безопасная декомпрессия тензоров (Zero-Copy & Alignment):**
   - Сжатие на клиенте: `lz4_flex::compress(raw_bytes)` (БЕЗ префикса размера).
   - Распаковка на воркере: предварительное выделение `vec![0.0f32; num_elements]` (гарантия 4-байтного выравнивания) и декомпрессия через `lz4_flex::decompress_into(&compressed, bytemuck::cast_slice_mut(&mut floats))`.

---

## 7. Критерий завершения работ (Definition of Done)
**КАТЕГОРИЧЕСКИ ЗАПРЕЩЕНО** объявлять задачу завершенной без выполнения полного цикла:

1. **Код:**
   - Форматирование: `cargo fmt --all -- --check` (код выхода 0).
   - Тесты всего воркспейса: `cargo test --workspace -j 1` (код выхода 0).
   - Линтер архитектуры: `cargo test -p ai_engine --test architecture_integrity_test -j 1`.
   - Линтер регламентов: `cargo test -p ai_engine --test dev_regulations_lint_test -j 1`.
   - Линтер UI: `cargo test -p overlay_ui --test i18n_lint_test -j 1` и `vector_first_lint_test`.
   - Валидация каталога плагинов: `cargo run --package plugin_validator -j 1 -- --validate plugins`.

2. **Синхронная документация:**
   - `docs/CHECKLIST.md` — отметить выполненные пункты `[x]`.
   - `docs/BUGS_AND_ISSUES.md` — зарегистрировать баг или перевести в `RESOLVED`.
   - `docs/CHRONICLE.md` и `docs/CHRONICLE.en.md` — синхронно добавить запись на русском и английском языках в авторском стиле Shellit с точным таймштампом:
     `*Время: ДД месяца ГГГГ, ЧЧ:ММ (~ХХ минут)*`.

---

## 8. Запрет вызова внешнего MCP shellit
* **СТРОГО ЗАПРЕЩЕНО** вызывать любые инструменты внешнего сервера `shellit` без явного запроса пользователя. Субагенты передают результаты и код родительскому агенту через стандартный канал сообщений (`send_message`).
