# Flexy Studio (Студия Flexy AI)

![Plugin Type](https://img.shields.io/badge/Type-Interactive%20UI-blue.svg)
![Version](https://img.shields.io/badge/Version-1.0.0-green.svg)
![Cloud Engine](https://img.shields.io/badge/Service-Flexy%20AI%20API-purple.svg)

---

**English:**
**Flexy Studio** is the official cloud AI powerhouse plugin for PhotoCraft Companion, directly integrating the state-of-the-art REST API of **[Flexy AI](https://flexy-ai.com/main)** (created by XpucT, creator of *Win 10 Tweaker* and the renowned *Deliberate* models).

Flexy provides an unconstrained, next-generation neural generation architecture with native Cyrillic text rendering, 1-click studio retouching, and powerful image transformations.

---

### 🎨 In-App Features (Flexy Studio Plugin for PhotoCraft)

Tools streamlined and optimized for in-place execution directly over PhotoCraft layers and canvas:

1. **Neural Generation & ControlNet Guided Synthesis:**
   - Text-to-Image generation using Flexy's custom architecture with support for Cyrillic text rendering in `"quotes"`;
   - Active layer **ControlNet** guidance: *Line Art*, *Canny*, *Depth*, *Scribble*, *Pose*;
   - Non-blocking cloud polling with live generation preview and progress bars;
   - Clean layer insertion aligned pixel-to-pixel with your project.
2. **Inpaint, Smart Eraser & Watermark Removal:**
   - **Smart Eraser (Object Removal):** cleanly removes cables, photobombers, and defects;
   - **Generative Fill (Inpainting):** synthesizes new details inside active selections from prompts;
   - **Watermark Remover:** scrubs timestamps, logos, and stock watermarks;
   - **Industrial Context Crop (SPEC-004):** extracts the defect region with 35% context padding. Eliminates server-side 1536 px downscaling on high-res 4K/8K photos, preserving **100% native sensor resolution**;
   - **In-Place Layer Fusion:** optionally bakes the transparent patch directly into the active layer (`layer.mergeDown`) or preserves it as a non-destructive aligned patch layer.
3. **Studio Portrait Retouching & Aging:**
   - **Natural Retouch:** gentle skin tone smoothing while preserving authentic pore microtextures;
   - **Studio Retouch:** glamorous magazine-style skin treatment with soft highlight balance;
   - **Vintage Age (Oldify):** realistic chronological age transformation.
4. **Canvas Expansion, Background Changer & Restoration:**
   - **Outpaint:** seamless panoramic expansion beyond canvas boundaries;
   - **Background Changer:** generates brand-new realistic environments behind foreground subjects;
   - **Colorize B&W:** deep learning color restoration for archival black-and-white photos.
5. **Secure Configuration & Balance:**
   - Secure local storage of your API key in Companion configuration;
   - Masked password input with instant 1-click credit balance verification.

---

### 🌐 Full Web Platform Features (Available on flexy-ai.com)

The web platform offers additional multimodal capabilities that require specialized multi-file uploads and web workflows:

* 👗 **Clothes Swap & Virtual Try-on:** swap outfits, jackets, and fashion pieces on photo subjects in seconds;
* 👤 **Face Swap:** high-fidelity facial transplantation from donor photos to target portraits;
* 🎬 **Video Generation (Txt2Video, Img2Video, Video2Video):** animate static artworks into cinematic video sequences;
* 📐 **Vectorization (Img2Svg):** convert raster graphics and logos into scalable vector curves;
* 🧊 **3D Generation (Img23D):** synthesize textured 3D meshes from single 2D images;
* 🎙️ **Text-to-Speech Audio:** high-quality vocal synthesis and voice generation;
* 🔍 **Multimodal Visual Describe:** reverse-engineer complex AI prompts from uploaded reference images;
* 👥 **Face Similarity Analysis:** precise biometric similarity scoring between subjects;
* ⏳ **Honest API Billing:** API credits never expire and are only deducted upon successful generations.

👉 **Explore the full web platform:** [flexy-ai.com/main](https://flexy-ai.com/main)  
🔑 **Get your API key:** [flexy-ai.com/profile](https://flexy-ai.com/profile)

---

### 💡 Practical Use Cases in PhotoCraft

1. **Concept Art & Guided Character Generation:**
   - Sketch an action pose or mechanical armor silhouette on an active layer in PhotoCraft.
   - Open **Flexy Studio** -> **Generate** tab. Enter a prompt like _"cyberpunk armored operative in rainy neo-tokyo"_.
   - Set **ControlNet Guide** to _Line Art (Active Layer)_ and click **Generate New Layer**.
   - The final render respects your sketch outlines perfectly and lands on a new layer.
2. **Clean In-Place Object Erasure:**
   - Use the Lasso tool to outline an unwanted tourist or power line across a 4000×3000 photo.
   - Switch to **Inpaint & Erase**, leave `Replace in Active Layer (In-Place)` checked, and click **Apply to Active Layer**.
   - Companion executes a 35% Context Crop, dispatches the native crop to Flexy AI, and seamlessly merges the result back. The canvas stays 100% sharp with zero downscaling.
3. **Magazine Portrait Retouching:**
   - Select a portrait layer and open **Portrait & Face**. Select _Studio Retouch Pro_ and adjust intensity.
   - Click **Process Portrait** to obtain skin smoothing that retains lifelike facial skin pores.

---

**Русский:**
**Студия Flexy AI (Flexy Studio)** — официальный флагманский плагин для PhotoCraft Companion, предоставляющий прямой доступ к высокопроизводительному REST API платформы **[Flexy AI](https://flexy-ai.com/main)** (автор — XpucT, создатель *Win 10 Tweaker* и популярных нейросетевых моделей *Deliberate* и *Reliberate*).

Платформа Flexy предлагает независимую генеративную архитектуру нового поколения без цензуры и ограничений, нативную генерацию текста на русском языке, ретушь в 1 клик и продвинутую модификацию фото.

---

### 🎨 Что реализовано прямо в редакторе (Плагин Flexy Studio)

Инструменты, оптимизированные для работы в один клик прямо с активным холстом и слоями PhotoCraft:

1. **Генерация артов и направляющие ControlNet:**
   * Создание артов по текстовому промпту (поддержка кириллических надписей в `"кавычках"`);
   * Направляющие **ControlNet** по активному слою: *Line Art*, *Canny*, *Depth*, *Scribble*, *Pose*;
   * Асинхронный опрос сервера с живым превью генерации и индикацией прогресса;
   * Вставка сгенерированного арта новым слоем, идеально выровненным относительно холста.
2. **Умный ластик, генеративная заливка и удаление водяных знаков:**
   * **Умный ластик (Smart Eraser):** удаление проводов, лишних людей, мусора и дефектов;
   * **Генеративная заливка (Inpaint):** дорисовка любых объектов внутри выделения по описанию;
   * **Удаление водяных знаков:** автоматическая очистка логотипов, подписей и дат;
   * **Промышленный Context Crop (SPEC-004):** вырезание области дефекта с 35% контекстным полем. Исключает серверный даунскейл 4K/8K фото до 1536 px и сохраняет **100% нативную четкость матрицы**;
   * **In-Place слияние:** переключатель «Заменять на активном слое» автоматически запекает результат в исходный слой (`layer.mergeDown`) или создает аккуратную прозрачную заплатку 1:1.
3. **Студийная ретушь портретов и эффекты возраста:**
   * **Естественная ретушь (Natural):** мягкое выравнивание тона кожи с сохранением микротекстуры пор;
   * **Студийный глянец (Studio):** журнальная обработка кожи с балансировкой светотени;
   * **Состаривание (Oldify):** реалистичная возрастная стилизация портрета.
4. **Холст, замена фона и реставрация:**
   * **Расширение полотна (Outpaint):** достройка панорамы за границы холста во все стороны;
   * **Замена фона (Background Changer):** генерация нового окружения за вырезанным объектом;
   * **Колоризация ч/б:** нейросетевое восстановление цвета архивных и старых фотографий.
5. **Безопасное хранение ключей и баланс:**
   * Локальное сохранение API-ключа в конфигурации компаньона;
   * Скрытый ввод пароля и быстрая проверка остатка кредитов в один клик.

---

### 🌐 Что доступно в полной веб-версии на Flexy AI (flexy-ai.com)

Дополнительные мультимодальные инструменты платформы, требующие загрузки множественных файлов и доступные на сайте:

* 👗 **Смена одежды (Clothes Swap):** виртуальная примерка и замена гардероба на фото за считанные секунды;
* 👤 **Перенос лица (Face Swap):** пересадка лица с фото-донора на целевой портрет с адаптацией света и тона;
* 🎬 **Генерация видео (Txt2Video, Img2Video, Video2Video):** превращение статичных изображений в видеоролики;
* 📐 **Векторизация в SVG (Img2Svg):** конвертация растровых логотипов и скетчей в масштабируемый вектор;
* 🧊 **Генерация 3D (Img23D):** создание текстурированных 3D-моделей по плоской картинке;
* 🎙️ **Синтез речи и аудио:** генерация озвучки (Text-to-Speech);
* 🔍 **Мультимодальный анализ (Describe):** распознавание изображения и создание подробного промпта;
* 👥 **Сравнение сходства лиц (Similarity):** биометрический расчет процента совпадения черт лица;
* ⏳ **Честные кредиты:** кредиты API никогда не сгорают в конце месяца и списываются строго за успешные генерации.

👉 **Попробовать все возможности платформы онлайн:** [flexy-ai.com/main](https://flexy-ai.com/main)  
🔑 **Получить API-ключ для плагина:** [flexy-ai.com/profile](https://flexy-ai.com/profile)

---

### 💡 Практическое применение в PhotoCraft

1. **Генерация концепт-артов по наброску (ControlNet):**
   - Нарисуйте контурный силуэт позы персонажа на слое PhotoCraft.
   - Откройте **Студию Flexy AI** -> вкладка **Генерация**, введите промпт (например, _«киберпанк воин в неоновом городе»_).
   - Выберите **ControlNet** -> _Line Art (Активный слой)_ и нажмите **Сгенерировать новый слой**.
   - Готовый детализированный арт появится новым слоем, идеально совпадая с вашим скетчем.
2. **Бесшовное удаление объектов (In-Place + Context Crop):**
   - Выделите лассо лишний провод или объект на снимке 4000×3000 px.
   - Во вкладке **Ластик и Заливка** оставьте чекбокс `Заменять на активном слое (In-Place)` включенным и нажмите **Применить к активному слою**.
   - Компаньон вырежет дефект с 35% полями, обработает в Flexy AI в 100% нативной резкости и запечет обратно в активный слой. Фото не потеряет резкость.
3. **Журнальная ретушь портретов:**
   - Выберите фотопортрет, откройте вкладку **Портрет и Лицо**, выберите _Студийная ретушь Pro_ и нажмите **Обработать портрет**.
   - Кожа выравнивается без эффекта искусственного пластика.
