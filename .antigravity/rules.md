# PhotoCraft Plugins — Antigravity Rules

1. **Context7 First:** Всегда запрашивать актуальную документацию для API (`onnxruntime`, PhotoCraft control protocol, Win32 GUI).
2. **Contract-First:** Пакеты не должны напрямую зависеть друг от друга; связь только через `packages/core_contracts`.
3. **Бесшовность пользовательского опыта:** Никаких лишних ручных шагов для пользователя — всё должно применяться прямо в открытом окне PhotoCraft.
4. **Фиксация в доках:**
   - Баги -> `docs/BUGS_AND_ISSUES.md`.
   - Прогресс -> `docs/CHECKLIST.md`.
   - Хроника -> `docs/CHRONICLE.md` и `docs/CHRONICLE.en.md` (синхронно, с метками времени).
5. **Запрет завершения без тестов и актуальных доков (Definition of Done):**
   - **СТРОГО ЗАПРЕЩЕНО** считать любой этап/задачу завершенной без:
     - 100% прохождения всех тестов воркспейса (`cargo test --workspace`);
     - Приведения в актуальное состояние всей проектной документации (`CHECKLIST.md`, `BUGS_AND_ISSUES.md`, `CHRONICLE.md`, `CHRONICLE.en.md`).

