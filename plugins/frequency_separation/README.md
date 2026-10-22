# Frequency Separation (Beauty Retouch)

[![Plugin: Frequency Separation](https://img.shields.io/badge/Plugin-Frequency_Separation-blue.svg)](plugin.json)
[![Type: No-Code Macro](https://img.shields.io/badge/Type-No--Code_Macro-orange.svg)](plugin.json)

**English:**
Professional beauty frequency separation macro for PhotoCraft portrait retouching.
In a single click, it decomposes the active layer into two dedicated non-destructive working layers:
1. **Low Frequency (Color & Tone):** Contains blurry color transitions, shadows, gradients, and skin tones smoothed by Gaussian Blur (radius 4.0 px).
2. **High Frequency (Texture):** Contains fine skin microtexture, pores, hairs, and sharp details isolated via High Pass filter (radius 4.0 px) set to `Linear Light` blend mode.

When stacked together, the mathematical sum of the two layers perfectly reconstructs the original document bit-for-bit without any loss of quality or sharpness, while granting complete independent control over texture and color.

### Practical Use Cases:
- **Flawless Skin Tone Evening:** Smooth blotchy redness, uneven tan lines, or harsh shadows on the "Low Frequency" layer using the Mixer Brush, Soft Brush, or Lasso + Gaussian Blur without turning skin into plastic or blurring pores.
- **Micro-Texture Blemish Cleanup:** Remove blemishes, acne, scars, stray hairs, and dust on the "High Frequency" layer with Clone Stamp or Healing Brush without causing discoloration, muddy spots, or halo rings.
- **Fabric & Product Wrinkle Removal:** Iron out wrinkles and creases in clothing or product backdrops while keeping cloth weave and texture 100% crisp.

### Neural Weights:
- None required. Operates purely through high-speed deterministic native PhotoCraft image processing filters.

---

**Русский:**
Профессиональный макро-плагин частотного разложения для бьюти-ретуши портретов в PhotoCraft.
В один клик раскладывает активный слой на два независимых недеструктивных рабочих слоя:
1. **Низкие частоты (Low Frequency — Цвет и тон):** Содержит плавные цветовые переходы, светотеневой рисунок и тон кожи, сглаженные фильтром Gaussian Blur (радиус 4.0 px).
2. **Высокие частоты (High Frequency — Текстура):** Содержит резкие микродетали, поры кожи, ресницы, волоски и фактуру, выделенные фильтром High Pass (радиус 4.0 px) в режиме наложения `Linear Light`.

Математическое сложение двух слоев в точности до бита воссоздает оригинальное изображение без малейшей потери резкости, но дает ретушеру полный раздельный контроль над текстурой и тоном.

### Практическое применение:
- **Выравнивание тона и цвета кожи:** Устранение пятен, неравномерного загара, синяков под глазами на слое «Low Frequency» мягкой кистью, штампом или размытием по лассо без создания эффекта «замыленного пластика» и потери пор.
- **Удаление дефектов текстуры:** Очистка прыщиков, царапин, волосков и пылинок на слое «High Frequency» штампом без смещения или размазывания подлежащего цвета.
- **Ретушь одежды и предметной съемки:** Быстрое разглаживание складок и помятостей на ткани с сохранением натурального плетения нитей и фактуры материала.

### Нейросетевые веса:
- Не требуются. Работает на базе высокоскоростных встроенных математических фильтров ядра PhotoCraft.
