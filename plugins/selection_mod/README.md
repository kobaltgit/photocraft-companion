# Selection Modifiers — Non-Destructive Selection Modifier Stack

![Plugin Type: Interactive UI](https://img.shields.io/badge/Plugin%20Type-Interactive%20UI-blue.svg)
![Target: PhotoCraft Selection Engine](https://img.shields.io/badge/Target-PhotoCraft%20Selection%20Engine-purple.svg)
![Version: 1.0.0](https://img.shields.io/badge/Version-1.0.0-emerald.svg)

---

**English:**

### Description
**Selection Modifiers** is an interactive companion studio tool for PhotoCraft designed to provide a completely non-destructive, reorderable selection modification workflow.

In raster editors, standard selection transformations (Expand, Contract, Smooth, Border, Feather) are traditionally destructive: repeatedly applying modifications degrades contour fidelity and accumulates geometric artifacts. Furthermore, selection operations are strictly non-commutative ($A \circ B \neq B \circ A$): running `Contract -> Smooth` yields a noticeably different contour than `Smooth -> Contract`.

With **Selection Modifiers**, artists can freely arrange modifier operations in any sequence using vector up/down chevrons, toggle individual operations on or off, and fine-tune radii in real time. Using a two-phase commit architecture with background alpha channel caching, applying the entire composite stack generates **exactly one atomic Undo step** in PhotoCraft history.

### Lifecycle & Non-Destructive Mechanics
- **Zero Data-Loss Reset:** The *Reset* button clears slider values and unchecks all operations in the UI while keeping the canvas selection contour 100% intact. The current contour is committed as a clean working base for fresh operations.
- **Commit-on-Uncheck (Bake):** Unchecking an active operation zeroes its radius and bakes the current altered canvas shape as the new working base contour.
- **Bi-Directional Selection Sync:** When selection boundaries change in PhotoCraft (e.g., using Lasso, Marquee, or Wand), the companion window immediately detects the updated bounds via `document.selection` subscription and re-synchronizes the base snapshot.

*Contributed by CharlieGoesMax & kobaltGIT.*

### Practical Use Cases

1. **Die-Cut Stickers & White Border Silhouette:**
   - *Workflow:* Use Quick Selection or Wand in PhotoCraft to select a subject cutout. Open Selection Modifiers. Enable `Smooth` (4 px) to clean noisy edge pixels, followed by `Expand` (16 px) to create the sticker padding, and finish with `Smooth` (8 px) for curved sticker corners. Click *Apply*.
   - *Result:* In PhotoCraft, fill the resulting selection on a background layer with pure white to create a crisp print-ready sticker contour in 1 click.
2. **Smooth Cutout Refinement & Fringe Cleanup (Defringe):**
   - *Workflow:* When extracting hair or complex objects from noisy backgrounds, selection edges often carry harsh pixel aliasing. In Selection Modifiers, stack `Contract` (1 px) to trim the background color fringe, followed by `Smooth` (2 px) and `Feather` (0.5 px).
   - *Result:* Seamlessly blends the cutout onto any new background without manually brushing or erasing matte edges.
3. **Vignette & Focused Glow Masks:**
   - *Workflow:* Create a rectangular or elliptical canvas selection. Open Selection Modifiers, enable `Contract` (60 px), followed by `Feather` (45 px). Click *Apply*.
   - *Result:* Creates an ultra-smooth organic gradient mask ready for Curves adjustment layers, cinematic lens vignettes, or localized exposure burns.

---

**Русский:**

### Описание
**Модификаторы выделения (Selection Modifiers)** — интерактивный инструмент PhotoCraft Companion, реализующий неразрушающий стек трансформации контуров выделения с возможностью произвольной настройки порядка операций.

В растровых графических редакторах стандартные команды изменения выделения носят разрушающий характер: повторное сжатие, расширение, сглаживание и растушевка накапливают геометрические искажения контура. Кроме того, эти операции некоммутативны: порядок выполнения кардинально меняет результат (`Сжатие -> Сглаживание` дает совершенно иную форму, чем `Сглаживание -> Сжатие`).

Инструмент позволяет выстраивать цепочку модификаторов (`Border`, `Smooth`, `Expand`, `Contract`, `Feather`) в любом порядке векторными стрелками перемещения, включать/выключать отдельные шаги чекбоксами и настраивать радиусы в пикселях. Благодаря двухфазной архитектуре коммита с кэшированием базового выделения в альфа-канале, применение всего стека фиксируется в PhotoCraft **ровно за один чистый шаг Undo**.

### Жизненный цикл и неразрушающие механизмы
- **Безопасный сброс (Zero Data-Loss Reset):** Кнопка *«Сбросить»* обнуляет радиусы и выключает чекбоксы только в интерфейсе Companion, гарантируя 100% сохранность контура на холсте PhotoCraft. Текущий контур фиксируется как свежая чистая база.
- **Фиксация при отключении (Commit-on-Uncheck):** Снятие чекбокса активного модификатора обнуляет его радиус и фиксирует (запекает) текущую форму холста как новый основной рабочий контур.
- **Двусторонняя реактивность выделения:** При изменении или перевыделении контура инструментами PhotoCraft (лассо, прямоугольное выделение, волшебная палочка) окно Companion мгновенно отслеживает обновление границ через подписку `document.selection` и сбрасывает устаревший слепок базы для бесшовного подхвата нового контура.

*Авторская спецификация: CharlieGoesMax & kobaltGIT.*

### Практическое применение

1. **Контур для наклеек и стикеров (Die-Cut Sticker Border):**
   - *Сценарий:* Выделите персонажа или объект на холсте. Откройте панель «Модификаторы выделения». Включите `Сглаживание` (4 px) для устранения мелких шероховатостей, затем `Расширить` (16 px) для белого отступа стикера и финальное `Сглаживание` (8 px) для скругления углов. Нажмите *«Применить»*.
   - *Результат:* Залейте полученный контур белым цветом на новом слое под объектом — идеальная заготовка для типографской плоттерной резки за одно нажатие.
2. **Очистка бахромы и каймы вырезки (Defringe & Smooth Cutout):**
   - *Сценарий:* При вырезании объектов с контрастного фона по краю часто остается пиксельная кайма. В стеке модификаторов расположите `Сжать` (1 px) для срезания каймы, затем `Сглаживание` (2 px) и лёгкую `Растушевку` (0.5 px).
   - *Результат:* Контур вырезанного объекта органично вписывается в новый фон без необходимости ручной зачистки ластиком.
3. **Виньетки и мягкие маски для цветокоррекции:**
   - *Сценарий:* Выделите рабочую область холста. Включите `Сжать` (60 px) и `Растушевку` (45 px). Нажмите *«Применить»*.
   - *Результат:* Создается мягкая, кинематографичная градиентная маска для корректирующего слоя «Кривые» или фильтра виньетирования.
