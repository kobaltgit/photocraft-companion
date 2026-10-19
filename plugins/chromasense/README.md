# ChromaSense A11y — Interactive Color & Accessibility Assistant

`🎨 Interactive UI` `⚡ Rhai Engine` `🌐 WCAG 2.1 Compliant` `🔬 Brettel CVD Model`

---

**English:**
ChromaSense is an interactive accessibility color panel designed for digital painters, concept artists, and graphic designers with color vision deficiencies (CVD) or artists striving for high accessibility compliance in their artwork. Powered by the embedded Rhai scripting engine, a rich 140+ shade perceptual database, and a Vector-First user interface, it provides real-time perceptual color analysis, semantic shade naming, hue compass tracking, live color synchronization with PhotoCraft, and accurate simulation of common color-blindness conditions (Deuteranopia, Protanopia, Tritanopia) using the Brettel-Vienot LMS model.

### Key Features & Use Cases:
1. **Color Selection with Semantic Guidance:** Artists who struggle to differentiate subtle shades can rely on the live semantic color indicator powered by a 140+ shade library (CSS3 standard colors plus classical fine art pigments such as Ultramarine, Burnt Sienna, Malachite, and Carmine) and the vector hue wheel to pick accurate tones without second-guessing.
2. **Side-by-Side Vision Simulation Preview:** A dedicated read-only color swatch displays the exact color perceived by viewers with the selected vision profile (Deuteranopia, Protanopia, or Tritanopia) computed via the Brettel-Vienot colorimetric model in real time.
3. **UI & Graphic Design Accessibility Verification:** Verify color contrast ratios between foreground and background against the WCAG 2.1 threshold (minimum 3:1 for graphic elements, 4.5:1 for body copy), receiving real-time warnings whenever a color collision or contrast loss occurs.
4. **Direct PhotoCraft Color Sync:** Adjusting RGB sliders or picking presets instantly sets the active brush foreground and background colors in PhotoCraft via the native companion bridge.
5. **Target Focus & One-Click Color Swap:** Click directly on either "Active Color" or "Background" slot to switch editing focus (highlighted by an active accent focus ring). Click the vector "Swap Colors" button to instantly swap foreground and background in one click.

---

**Русский:**
ChromaSense — это интерактивная панель доступности и цветовой гармонии, созданная для художников, иллюстраторов и дизайнеров с особенностями цветовосприятия (дальтонизмом), а также для авторов, стремящихся к полному соответствию стандартам доступности (WCAG). Работает на базе встроенного скриптового движка Rhai, колориметрической базы из 140+ оттенков и векторного интерфейса, обеспечивая анализ цветов в реальном времени, семантическое именование оттенков, круговой компас тона, синхронизацию цветов с PhotoCraft и симуляцию основных типов нарушений цветового зрения (дейтеранопия, протанопия, тританопия) по колориметрической модели Бреттла-Вьено.

### Практическое применение и ключевые возможности:
1. **Надежный выбор цвета с семантическими подсказками:** Художники, испытывающие трудности с различением нюансов красно-зеленого или сине-желтого спектра, могут ориентироваться на точные текстовые дескрипторы оттенков из базы 140+ цветов (включая классические художественные пигменты: Ультрамарин, Жжёная сиена, Малахит, Кармин, Кобальт) и угол указателя спектрального круга.
2. **Наглядная симуляция дальтонизма (слот «Как видит зритель»):** Второй нередактируемый цветовой слот в реальном времени отображает оттенок глазами зрителя с выбранным профилем (дейтеранопия, протанопия или тританопия), позволяя художнику сразу увидеть, если важные детали сливаются в неразличимую кашу.
3. **Проверка контраста для полиграфии и интерфейсов:** Мгновенный расчет коэффициента контрастности между цветом переднего плана и фона относительно требований стандарта WCAG 2.1 (не менее 3:1 для графики и 4.5:1 для шрифтов) с выводом предупреждения о коллизиях.
4. **Прямая синхронизация с PhotoCraft:** Перемещение ползунков RGB мгновенно обновляет активный цвет кисти и цвет фона в открытом документе редактора.
5. **Интерактивный выбор фокуса и кнопка Swap:** Клик по плашке «Выбранный цвет» или «Цвет фона» переключает фокус редактирования (активный слот подсвечивается контрастной рамкой). Векторная кнопка «Поменять цвета» мгновенно меняет местами цвета переднего плана и фона в один клик.
