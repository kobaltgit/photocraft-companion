# Harmonic Color Studio & Swatch Generator

`🎨 Interactive UI` `⚡ Rhai Engine` `🎯 Classical Color Harmonies` `🖌️ Swatches Export`

---

**English:**
Harmonic Color Studio is a professional interactive color harmony and palette creation studio for digital artists, concept illustrators, and graphic designers in PhotoCraft. Powered by the embedded Rhai scripting runtime, bidirectional session color reactivity (`session.colors`), and the Vector-First UI engine, it allows artists to craft harmonious color palettes based on 6 classical color theory rules (Complementary, Split-Complementary, Triadic, Analogous, Tetradic, and Monochromatic).

Artists can tweak the spread angle and tonal contrast, inspect the spectral hue compass wheel with real-time pointer indication, and click any harmonic swatch to immediately assign it to PhotoCraft's active brush. Palettes can be exported in two distinct workflows: directly into PhotoCraft's native Swatches panel (`swatches.newGroup` + `swatches.add`), or rendered directly as a clean swatch card on a new canvas layer (`layer.new.layer` + `select.rect` + `edit.fill`).

### Key Features & Use Cases:
1. **The 60-30-10 Visual Balance Rule in Illustration:** Digital painters can instantly split an illustration into a 60% dominant base tone, a 30% supporting harmonic secondary color, and a 10% punchy focal accent, guaranteeing strong focal hierarchy and visual cohesion across complex character and environment art.
2. **Dynamic Highlight & Deep Shadow Keying:** When rendering volume and subsurface scattering, artists can use the automatically computed `Highlight Tint` and `Deep Shadow Shade` swatches to paint consistent rim lights and occlusions without guessing perceptual values.
3. **Concept Art & Production Art Bible Generation:** Concept artists delivering turnarounds and prop designs can generate neat horizontal or vertical swatch cards directly on a dedicated canvas layer with one click (`[+] Draw on Canvas`), eliminating manual color picking and speeding up communication with 3D modelers and texture artists.
4. **Native Swatches Panel Synchronization:** Export the complete 5-color harmony group directly into PhotoCraft's native Swatches library (`[+] Add to Swatches`), keeping the canvas pristine while building persistent project palettes.

---

**Русский:**
Harmonic Color Studio (Студия цветовых гармоний) — это интерактивная студия цветовых гармоний и генератор палитр для художников, концепт-иллюстраторов, ретушеров и дизайнеров в PhotoCraft. Работает на базе встроенного скриптового рантайма Rhai, двунаправленной реактивности цвета кисти (`session.colors`) и векторного интерфейса Vector-First, позволяя выстраивать гармоничные палитры по 6 классическим правилам колористики (комплементарная, сплит-комплементарная, триада, аналоговая, тетраида и монохроматическая).

Художник может настраивать угол разброса и тональный контраст, отслеживать спектральный круговой компас с живым векторным указателем и в один клик переключать активную кисть PhotoCraft на любой выбранный оттенок. Созданная палитра экспортируется двумя путями: сохраняется в системную панель образцов PhotoCraft (`swatches.newGroup` + `swatches.add`) или аккуратно вычерчивается прямо на отдельном слое холста (`layer.new.layer` + `select.rect` + `edit.fill`).

### Практическое применение и ключевые возможности:
1. **Правило баланса 60-30-10 в иллюстрации:** Позволяет мгновенно разложить сцену на 60% доминирующего базового окружения, 30% поддерживающего гармонического оттенка и 10% фокусного акцента, гарантируя выверенную композицию персонажей и концепт-артов без цветовой каши.
2. **Точный подбор контрового света и теней:** Слоты `Highlight Tint` (светлый тон/блик) и `Deep Shadow Shade` (глубокая тень) рассчитываются математически от базового тона с учетом заданного контраста, позволяя художникам рендерить реалистичные объемы, падающие тени и световые акценты.
3. **Создание карточек палитры для арт-библий и концепт-арта:** Одной кнопкой `[+] Слой на холсте` плагин создает новый слой с аккуратными цветными плашками (горизонтальная лента или вертикальная колонка), что незаменимо при подготовке референс-листов для 3D-моделлеров и текстурщиков.
4. **Бесшовный экспорт в панель Swatches:** Кнопка `[+] В образцы Swatches` регистрирует новую именованную группу в родной панели образцов PhotoCraft, сохраняя палитру в библиотеку проекта без необходимости захламлять рабочий холст.
